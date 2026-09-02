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

use crate::config::WORKSPACES;
use crate::user_config::{Apps, Theme};

type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

// spawn_action (penrose::builtin::actions::spawn) takes a &'static str, but
// the program names come from config.toml as owned Strings loaded once at
// startup - leaking them is the same "runtime String -> &'static str"
// tradeoff already used for the battery name in bar.rs and for window rule
// class names/tags, and is fine for values that live for the process' life.
fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// Creates the main keybindings
pub fn raw_key_bindings(
    toggle_scratchpad: ToggleNamedScratchPad,
    theme: &Theme,
    apps: &Apps,
) -> HashMap<String, KeyHandler> {
    let terminal = leak(apps.terminal.clone());
    let launcher = leak(apps.launcher.clone());
    let locker = leak(apps.locker.clone());

    let mut raw_bindings = map! {
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

        // Scratchpad
        "M-s" => Box::new(toggle_scratchpad),

        // System
        "M-x" => logout_menu(theme),
        "M-S-s" => log_current_state(),
        "M-S-q" => exit(),

        // Volume (PulseAudio)
        "XF86AudioRaiseVolume" => spawn_action("pactl set-sink-volume @DEFAULT_SINK@ +5%"),
        "XF86AudioLowerVolume" => spawn_action("pactl set-sink-volume @DEFAULT_SINK@ -5%"),
        "XF86AudioMute" => spawn_action("pactl set-sink-mute @DEFAULT_SINK@ toggle"),
    };

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
