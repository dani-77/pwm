mod bar;
mod config;
mod keybindings;
mod layouts;

use penrose::x11rb::RustConn;
use penrose::{
    Result,
    core::{Config, WindowManager, bindings::parse_keybindings_with_xmodmap},
    extensions::hooks::{
        NamedScratchPad, SpawnOnStartup, add_ewmh_hooks, add_named_scratchpads,
        manage::{FloatingCentered, SetWorkspace},
    },
    manage_hooks,
    x::query::ClassName,
};
use tracing_subscriber::{self, prelude::*};

use config::PwmConfig;
use keybindings::{mouse_bindings, raw_key_bindings};
use layouts::layouts;

fn main() -> Result<()> {
    // Inicializa logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .finish()
        .init();

    // Carrega configuração
    let pwm_config = PwmConfig::new();

    // Hooks de gestão de janelas
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

    // Configuração do Penrose
    let config = add_ewmh_hooks(Config {
        focused_border: pwm_config.focused_border,
        normal_border: pwm_config.normal_border,
        default_layouts: layouts(),
        manage_hook: Some(my_manage_hook),
        startup_hook: Some(SpawnOnStartup::boxed(pwm_config.apps.startup_script)),
        ..Config::default()
    });

    // Scratchpad
    let (nsp, toggle_nsp) = NamedScratchPad::new(
        pwm_config.scratchpad.name,
        pwm_config.scratchpad.command,
        ClassName(pwm_config.scratchpad.class_name),
        FloatingCentered::new(
            pwm_config.scratchpad.width_ratio,
            pwm_config.scratchpad.height_ratio,
        ),
        true,
    );

    // Inicializa conexão X11
    let conn = RustConn::new()?;

    // Parse keybindings
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings(toggle_nsp))?;

    // Cria status bar
    let bar = bar::status_bar().expect("failed to create status bar");

    // Cria window manager
    let wm = bar.add_to(WindowManager::new(
        config,
        key_bindings,
        mouse_bindings(),
        conn,
    )?);

    // Adiciona scratchpads
    let wm = add_named_scratchpads(wm, vec![nsp]);

    // Run!
    wm.run()
}
