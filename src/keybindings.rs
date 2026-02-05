use penrose::x11rb::RustConn;
use penrose::{
    builtin::actions::{
        exit, modify_with, send_layout_message, spawn as spawn_action,
        floating::{MouseDragHandler, MouseResizeHandler, sink_focused},
        log_current_state,
	key_handler,
    },
    builtin::layout::messages::{ExpandMain, IncMain, ShrinkMain},
    core::bindings::{
        KeyEventHandler, MouseEventHandler, MouseState, click_handler,
    },
    extensions::actions::toggle_fullscreen,
    extensions::hooks::ToggleNamedScratchPad,
    Color,
    util::spawn as spawn_cmd,
    map,
};
use penrose::extensions::util::dmenu::{DMenu, DMenuConfig, MenuMatch};
use std::collections::HashMap;

use crate::config::{WORKSPACES, BLACK, WHITE, LAVENDER};

type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

/// Cria os keybindings principais
pub fn raw_key_bindings(
    toggle_scratchpad: ToggleNamedScratchPad,
) -> HashMap<String, KeyHandler> {
    let mut raw_bindings = map! {
        map_keys: |k: &str| k.to_string();

        // Navegação de janelas
        "M-j" => modify_with(|cs| cs.focus_down()),
        "M-k" => modify_with(|cs| cs.focus_up()),
        "M-S-j" => modify_with(|cs| cs.swap_down()),
        "M-S-k" => modify_with(|cs| cs.swap_up()),
        
        // Gestão de janelas
        "M-q" => modify_with(|cs| cs.kill_focused()),
        "M-S-f" => toggle_fullscreen(),
        
        // Navegação de workspaces/tags
        "M-Tab" => modify_with(|cs| cs.toggle_tag()),
        
        // Navegação de ecrãs
        "M-bracketright" => modify_with(|cs| cs.next_screen()),
        "M-bracketleft" => modify_with(|cs| cs.previous_screen()),
        
        // Layouts
        "M-m" => modify_with(|cs| cs.next_layout()),
        "M-S-m" => modify_with(|cs| cs.previous_layout()),
        "M-Up" => send_layout_message(|| IncMain(1)),
        "M-Down" => send_layout_message(|| IncMain(-1)),
        "M-Right" => send_layout_message(|| ExpandMain),
        "M-Left" => send_layout_message(|| ShrinkMain),
        
        // Aplicações
        "M-Return" => spawn_action("st"),
        "M-d" => spawn_action("dmenu_run"),
        "M-t" => spawn_action("slock"),
        
        // Scratchpad
        "M-s" => Box::new(toggle_scratchpad),
        
        // Sistema
        "M-x" => logout_menu(),
        "M-S-s" => log_current_state(),
        "M-S-q" => exit(),
    };

    // Adiciona bindings para workspaces
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

/// Cria os bindings do rato
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

/// Menu de logout com dmenu
pub fn logout_menu() -> KeyHandler {

    key_handler(|state, _x| {
        let choices = vec![
            "󰒲  suspend",
            "󰍃  logout",
            "󱞳  reboot",
            "󰤆  shutdown",
        ];

        let config = DMenuConfig {
            ignore_case: true,
            show_line_numbers: false,
            show_on_bottom: false,
            password_input: false,
            custom_prompt: Some("Power Menu".to_string()),
            bg_color: Color::new_from_hex(BLACK),
            fg_color: Color::new_from_hex(WHITE),
            selected_color: Color::new_from_hex(LAVENDER),
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
