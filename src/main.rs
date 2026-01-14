mod bar;
use penrose::Color;
use penrose::x11rb::RustConn;
use penrose::{
    Result, manage_hooks, 
    builtin::{
        actions::{
            exit, key_handler,
            floating::{MouseDragHandler, MouseResizeHandler, sink_focused},
            log_current_state, modify_with, send_layout_message, spawn as spawn_action,
        },
        layout::{
            MainAndStack, Monocle, Grid,
            messages::{ExpandMain, IncMain, ShrinkMain},
            transformers::{Gaps, ReflectHorizontal, ReserveTop},
        },
    },
    core::{
        Config, WindowManager,
        bindings::{
            KeyEventHandler, MouseEventHandler, MouseState, click_handler,
            parse_keybindings_with_xmodmap,
        },
        layout::LayoutStack,
    },
    extensions::{
    	util::dmenu::{DMenu, DMenuConfig, MenuMatch},
    	layout::{Fibonacci, Tatami},
        actions::toggle_fullscreen, 
        hooks::{NamedScratchPad, ToggleNamedScratchPad, add_named_scratchpads, manage::{SetWorkspace, FloatingCentered}, SpawnOnStartup, add_ewmh_hooks},
    },
    util::spawn as spawn_cmd,
    map, stack,
    x::query::ClassName,
};

use std::collections::HashMap;
use tracing_subscriber::{self, prelude::*};

type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

const GREY: u32 = 0x3c3836ff;
const LAVENDER: u32 = 0xAA96DA;

fn raw_key_bindings(
    toggle_1: ToggleNamedScratchPad,
) -> HashMap<String, KeyHandler>{
  let mut raw_bindings = map! {
        map_keys: |k: &str| k.to_string();

        "M-j" => modify_with(|cs| cs.focus_down()),
        "M-k" => modify_with(|cs| cs.focus_up()),
        "M-S-j" => modify_with(|cs| cs.swap_down()),
        "M-S-k" => modify_with(|cs| cs.swap_up()),
        "M-q" => modify_with(|cs| cs.kill_focused()),
        "M-Tab" => modify_with(|cs| cs.toggle_tag()),
        "M-bracketright" => modify_with(|cs| cs.next_screen()),
        "M-bracketleft" => modify_with(|cs| cs.previous_screen()),
        "M-m" => modify_with(|cs| cs.next_layout()),
        "M-S-m" => modify_with(|cs| cs.previous_layout()),
        "M-Up" => send_layout_message(|| IncMain(1)),
        "M-Down" => send_layout_message(|| IncMain(-1)),
        "M-Right" => send_layout_message(|| ExpandMain),
        "M-Left" => send_layout_message(|| ShrinkMain),
	"M-d" => spawn_action("dmenu_run"), 
	"M-t" => spawn_action("slock"),
        "M-S-s" => log_current_state(),
        "M-Return" => spawn_action("st"),
        "M-S-f" => toggle_fullscreen(),

        "M-S-q" => exit(),
	    "M-x" => logout_menu(),
	    "M-s" => Box::new(toggle_1),
    };

    for tag in &["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
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

fn mouse_bindings() -> HashMap<MouseState, Box<dyn MouseEventHandler<RustConn>>> {
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

pub fn logout_menu() -> KeyHandler {
    key_handler(|state, _x| {
        let choices = vec!["󰒲  suspend", "󰍃  logout", "󱞳  reboot", "󰤆  shutdown"];

        let config = DMenuConfig {
            ignore_case: true,
            show_line_numbers: false,
            show_on_bottom: false,
            password_input: false,
            custom_prompt: Some("Power Menu".to_string()),
            bg_color: Color::new_from_hex(0x282828),      // BLACK
            fg_color: Color::new_from_hex(0xebdbb2),      // WHITE
            selected_color: Color::new_from_hex(0xAA96DA), // LAVENDER
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

pub fn layouts() -> LayoutStack {
    let max_main = 1;
    let ratio = 0.6;
    let ratio_step = 0.1;
    let outer_px = 5;
    let inner_px = 5;
    let top_px = 18;

    stack!(
        MainAndStack::side(max_main, ratio, ratio_step),
        ReflectHorizontal::wrap(MainAndStack::side(max_main, ratio, ratio_step)),
        MainAndStack::bottom(max_main, ratio, ratio_step),
    	Monocle::boxed(),
    	Grid::boxed(),
    	Fibonacci::boxed_default(),
	    Tatami::boxed_default()
    )
    .map(|layout| ReserveTop::wrap(Gaps::wrap(layout, outer_px, inner_px), top_px))
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("trace")
        .finish()
        .init();

    let my_manage_hook = manage_hooks! {
        ClassName("gimp") => SetWorkspace("3"),
        ClassName("deadbeef") => SetWorkspace("5"),
        ClassName("mpv") => SetWorkspace("5"),
        ClassName("ncspot") => SetWorkspace("5"),
        ClassName("spotify") => SetWorkspace("5"),
        ClassName("thunderbird") => SetWorkspace("9"),
        ClassName("chromium") => SetWorkspace("9"),
        ClassName("firefox") => SetWorkspace("9"),
        ClassName("qutebrowser") => SetWorkspace("9"),
    };

    let config = add_ewmh_hooks(Config {
        focused_border: LAVENDER.into(),
        normal_border: GREY.into(),
        default_layouts: layouts(),
        manage_hook: Some(my_manage_hook),
        startup_hook: Some(SpawnOnStartup::boxed("/etc/xdg/pwm/startup.sh")),
        ..Config::default()
    });

    let (nsp_1, toggle_1) = NamedScratchPad::new(
        "terminal",
        "st -c StScratchpad",
        ClassName("StScratchpad"),
        FloatingCentered::new(0.8, 0.8),
        true,
    );

    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings(toggle_1))?;
    
    let bar = bar::status_bar().expect("failed to create status bar");
    let wm = bar.add_to(WindowManager::new(
        config,
        key_bindings,
        mouse_bindings(),
        conn,
    )?);

    let wm = add_named_scratchpads(wm, vec![nsp_1]);

    wm.run()
}
