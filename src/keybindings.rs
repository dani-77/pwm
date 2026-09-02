use penrose::extensions::util::dmenu::{DMenu, DMenuConfig, MenuMatch};
use penrose::x11rb::RustConn;
use penrose::{
    Color,
    builtin::actions::{
        exit,
        floating::{MouseDragHandler, MouseResizeHandler, sink_focused},
        key_handler, log_current_state, modify_with, send_layout_message, spawn as spawn_action,
    },
    builtin::layout::messages::{ExpandMain, IncMain, ShrinkMain},
    core::bindings::{KeyEventHandler, MouseEventHandler, MouseState, click_handler},
    extensions::actions::toggle_fullscreen,
    extensions::hooks::ToggleNamedScratchPad,
    map,
    util::spawn as spawn_cmd,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::warn;

use crate::config::WORKSPACES;
use crate::user_config::{Apps, BindSpec, Theme};

type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

// spawn_action (penrose::builtin::actions::spawn) takes a &'static str, but
// the program names come from config.toml as owned Strings loaded once at
// startup - leaking them is the same "runtime String -> &'static str"
// tradeoff already used for the battery name in bar.rs and for window rule
// class names/tags, and is fine for values that live for the process' life.
fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// Set by [`restart`] right before it stops the window manager loop the
/// same way `exit()` does; `main` checks it once `wm.run()` returns and
/// re-execs the pwm binary instead of letting the process end, so
/// `config.toml` (and everything derived from it - theme, keybindings,
/// window/float rules, bar widgets) gets re-read from scratch. This is a
/// "fake" hot reload rather than a true one: existing windows survive
/// (`manage_existing_clients` re-adopts them into the new process, exactly
/// as it already does for anything alive when pwm starts up normally), but
/// the bar and borders do flash briefly while the fresh process spins back
/// up. `pub(crate)` since only `main` needs to read it.
pub(crate) static RESTART_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Stops the window manager - see [`RESTART_REQUESTED`] - so `main` can
/// restart pwm in place instead of quitting for good.
fn restart() -> KeyHandler {
    let mut do_exit = exit::<RustConn>();
    key_handler(move |state, x| {
        RESTART_REQUESTED.store(true, Ordering::SeqCst);
        do_exit.call(state, x)
    })
}

/// The default keymap, always used as the base that `config.toml`'s
/// `[[bind]]` entries (if any) get overlaid onto by [`raw_key_bindings`].
/// Kept as plain code (rather than expressed as a list of [`BindSpec`]s run
/// through [`action_for`]) so pwm always has a keymap that works even if
/// config.toml parsing/dispatch itself had a bug.
///
/// `toggle_scratchpad` is `None` when a `[[bind]]` entry claims the
/// `toggle_scratchpad` action for a different key - see
/// [`raw_key_bindings`] - in which case the default `M-s` binding for it is
/// simply omitted rather than binding an object that's already been moved.
fn default_bindings(
    toggle_scratchpad: Option<ToggleNamedScratchPad>,
    theme: &Theme,
    apps: &Apps,
) -> HashMap<String, KeyHandler> {
    let terminal = leak(apps.terminal.clone());
    let launcher = leak(apps.launcher.clone());
    let locker = leak(apps.locker.clone());

    let mut bindings = map! {
        map_keys: |k: &str| k.to_string();

        // Window navigation
        "M-j" => modify_with(|cs| cs.focus_down()),
        "M-k" => modify_with(|cs| cs.focus_up()),
        "M-S-j" => modify_with(|cs| cs.swap_down()),
        "M-S-k" => modify_with(|cs| cs.swap_up()),

        // Window management
        "M-q" => modify_with(|cs| cs.kill_focused()),
        "M-S-f" => toggle_fullscreen(),

        // Workspace/tag navigation
        "M-Tab" => modify_with(|cs| cs.toggle_tag()),

        // Screen navigation
        "M-period" => modify_with(|cs| cs.next_screen()),
        "M-comma" => modify_with(|cs| cs.previous_screen()),

        // Layouts
        "M-m" => modify_with(|cs| cs.next_layout()),
        "M-S-m" => modify_with(|cs| cs.previous_layout()),
        "M-Up" => send_layout_message(|| IncMain(1)),
        "M-Down" => send_layout_message(|| IncMain(-1)),
        "M-Right" => send_layout_message(|| ExpandMain),
        "M-Left" => send_layout_message(|| ShrinkMain),

        // Applications
        "M-Return" => spawn_action(terminal),
        "M-d" => spawn_action(launcher),
        "M-t" => spawn_action(locker),

        // System
        "M-x" => logout_menu(theme),
        "M-S-s" => log_current_state(),
        "M-S-q" => exit(),
        "M-S-r" => restart(),

        // Volume (PulseAudio)
        "XF86AudioRaiseVolume" => spawn_action("pactl set-sink-volume @DEFAULT_SINK@ +5%"),
        "XF86AudioLowerVolume" => spawn_action("pactl set-sink-volume @DEFAULT_SINK@ -5%"),
        "XF86AudioMute" => spawn_action("pactl set-sink-mute @DEFAULT_SINK@ toggle"),
    };

    // Scratchpad - see the toggle_scratchpad param's doc comment above
    if let Some(toggle_scratchpad) = toggle_scratchpad {
        bindings.insert("M-s".to_string(), Box::new(toggle_scratchpad));
    }

    bindings
}

// The named actions [[bind]] entries in config.toml can refer to. Mirrors
// exactly what default_bindings() above wires up by hand, so the two stay
// interchangeable. `toggle_scratchpad` can only ever be bound once (the
// underlying ToggleNamedScratchPad isn't Copy/Clone), so a second [[bind]]
// naming it - or an unrecognized action name - both fall through to the
// warning in raw_key_bindings() below rather than failing to start.
fn action_for(
    action: &str,
    arg: Option<&str>,
    theme: &Theme,
    apps: &Apps,
    toggle_scratchpad: &mut Option<ToggleNamedScratchPad>,
) -> Option<KeyHandler> {
    Some(match action {
        "focus_down" => modify_with(|cs| cs.focus_down()),
        "focus_up" => modify_with(|cs| cs.focus_up()),
        "swap_down" => modify_with(|cs| cs.swap_down()),
        "swap_up" => modify_with(|cs| cs.swap_up()),
        "kill_focused" => modify_with(|cs| cs.kill_focused()),
        "toggle_fullscreen" => toggle_fullscreen(),
        "toggle_tag" => modify_with(|cs| cs.toggle_tag()),
        "next_screen" => modify_with(|cs| cs.next_screen()),
        "previous_screen" => modify_with(|cs| cs.previous_screen()),
        "next_layout" => modify_with(|cs| cs.next_layout()),
        "previous_layout" => modify_with(|cs| cs.previous_layout()),
        "expand_main" => send_layout_message(|| ExpandMain),
        "shrink_main" => send_layout_message(|| ShrinkMain),
        "inc_main" => {
            let amount: i8 = arg.and_then(|a| a.parse().ok()).unwrap_or(1);
            send_layout_message(move || IncMain(amount))
        }
        "spawn_terminal" => spawn_action(leak(apps.terminal.clone())),
        "spawn_launcher" => spawn_action(leak(apps.launcher.clone())),
        "spawn_locker" => spawn_action(leak(apps.locker.clone())),
        "spawn" => spawn_action(leak(arg?.to_string())),
        "toggle_scratchpad" => Box::new(toggle_scratchpad.take()?),
        "logout_menu" => logout_menu(theme),
        "log_state" => log_current_state(),
        "exit" => exit(),
        "restart" => restart(),
        _ => return None,
    })
}

/// Creates the main keybindings: the built-in defaults, with `config.toml`'s
/// `[[bind]]` entries (if any) overlaid on top key by key - a `[[bind]]` for
/// a key that already does something replaces just that one binding, every
/// other default stays intact. Unrecognized action names, and any use of
/// `toggle_scratchpad` beyond the first, are skipped with a warning rather
/// than refusing to start - a typo in one bind shouldn't cost you every
/// other one. Per-workspace M-<tag>/M-S-<tag> bindings are always added on
/// top of everything else - they're generated from `WORKSPACES` rather than
/// something to hand-write per binding.
pub fn raw_key_bindings(
    toggle_scratchpad: ToggleNamedScratchPad,
    theme: &Theme,
    apps: &Apps,
    binds: Option<&[BindSpec]>,
) -> HashMap<String, KeyHandler> {
    // If a [[bind]] wants toggle_scratchpad on some other key, the default
    // M-s binding for it needs to not claim the object first - it can only
    // be bound once.
    let moves_scratchpad =
        binds.is_some_and(|specs| specs.iter().any(|s| s.action == "toggle_scratchpad"));
    let mut toggle_scratchpad = Some(toggle_scratchpad);
    let default_scratchpad = if moves_scratchpad {
        None
    } else {
        toggle_scratchpad.take()
    };

    let mut raw_bindings = default_bindings(default_scratchpad, theme, apps);

    if let Some(specs) = binds {
        for spec in specs {
            match action_for(
                &spec.action,
                spec.arg.as_deref(),
                theme,
                apps,
                &mut toggle_scratchpad,
            ) {
                Some(handler) => {
                    raw_bindings.insert(spec.key.clone(), handler);
                }
                None => warn!(
                    key = spec.key,
                    action = spec.action,
                    "unknown action or action already bound elsewhere, skipping"
                ),
            }
        }
    }

    // Add bindings for workspaces
    for tag in WORKSPACES.iter() {
        raw_bindings.extend([
            (
                format!("M-{tag}"),
                modify_with(move |client_set| client_set.focus_tag(tag)),
            ),
            (
                format!("M-S-{tag}"),
                modify_with(move |client_set| client_set.move_focused_to_tag(tag)),
            ),
        ]);
    }

    raw_bindings
}

/// Creates the mouse bindings
pub fn mouse_bindings() -> HashMap<MouseState, Box<dyn MouseEventHandler<RustConn>>> {
    use penrose::core::bindings::{
        ModifierKey::{Meta, Shift},
        MouseButton::{Left, Middle, Right},
    };

    map! {
        map_keys: |(button, modifiers)| MouseState { button, modifiers };

        (Left, vec![Shift, Meta]) => MouseDragHandler::boxed_default(),
        (Right, vec![Shift, Meta]) => MouseResizeHandler::boxed_default(),
        (Middle, vec![Shift, Meta]) => click_handler(sink_focused()),
    }
}

/// Logout menu with dmenu
pub fn logout_menu(theme: &Theme) -> KeyHandler {
    let (black, white, accent) = (theme.black, theme.white, theme.accent);

    key_handler(move |state, _x| {
        let choices = vec!["󰒲  suspend", "󰍃  logout", "󱞳  reboot", "󰤆  shutdown"];

        let config = DMenuConfig {
            ignore_case: true,
            show_line_numbers: false,
            show_on_bottom: false,
            password_input: false,
            custom_prompt: Some("Power Menu".to_string()),
            bg_color: Color::new_from_hex(black),
            fg_color: Color::new_from_hex(white),
            selected_color: Color::new_from_hex(accent),
            ..DMenuConfig::default()
        };

        let screen_index = state.client_set.current_screen().index();
        let dmenu = DMenu::new(&config, screen_index);

        if let Ok(MenuMatch::Line(_, choice)) = dmenu.build_menu(choices) {
            match choice.as_str() {
                "󰒲  suspend" => {
                    let _ = spawn_cmd("loginctl suspend");
                }
                "󰍃  logout" => {
                    let _ = spawn_cmd("kill -9 -1");
                }
                "󰤆  shutdown" => {
                    let _ = spawn_cmd("loginctl poweroff");
                }
                "󱞳  reboot" => {
                    let _ = spawn_cmd("loginctl reboot");
                }
                _ => {}
            }
        }

        Ok(())
    })
}
