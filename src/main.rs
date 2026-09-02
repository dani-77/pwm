mod bar;
mod config;
mod keybindings;
mod layouts;
mod user_config;

use penrose::x11rb::RustConn;
use penrose::{
    Result,
    core::{Config, WindowManager, bindings::parse_keybindings_with_xmodmap, hooks::ManageHook},
    extensions::hooks::{
        NamedScratchPad, SpawnOnStartup, add_ewmh_hooks, add_named_scratchpads,
        manage::{FloatingCentered, SetWorkspace},
    },
    x::query::ClassName,
};
use tracing_subscriber::{self, prelude::*};

use config::PwmConfig;
use keybindings::{mouse_bindings, raw_key_bindings};
use layouts::layouts;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .finish()
        .init();

    // Load configuration
    let user_config = user_config::load();
    let theme = &user_config.theme;
    let apps = &user_config.apps;
    let pwm_config = PwmConfig::new(theme);

    // Window management hooks: which window class opens on which tag.
    // ClassName/SetWorkspace both need a &'static str; window_rules is
    // loaded once at startup, so leaking it (same tradeoff as the app
    // names in keybindings.rs) is fine for the life of the process.
    let my_manage_hook: Box<dyn ManageHook<RustConn>> = {
        let hooks: Vec<Box<dyn ManageHook<RustConn>>> = user_config
            .window_rules
            .iter()
            .map(|rule| {
                let class: &'static str = Box::leak(rule.class.clone().into_boxed_str());
                let workspace: &'static str = Box::leak(rule.workspace.clone().into_boxed_str());
                ManageHook::boxed((ClassName(class), SetWorkspace(workspace)))
            })
            .collect();

        Box::new(hooks)
    };

    // Penrose configuration
    let config = add_ewmh_hooks(Config {
        focused_border: pwm_config.focused_border,
        normal_border: pwm_config.normal_border,
        default_layouts: layouts(),
        manage_hook: Some(my_manage_hook),
        startup_hook: Some(SpawnOnStartup::boxed(apps.startup_script.clone())),
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

    // Initialize X11 connection
    let conn = RustConn::new()?;

    // Parse keybindings
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings(
        toggle_nsp,
        theme,
        apps,
        user_config.binds.as_deref(),
    ))?;

    // Create status bar
    let bar =
        bar::status_bar(theme, &user_config.bar_widgets).expect("failed to create status bar");

    // Create window manager
    let wm = bar.add_to(WindowManager::new(
        config,
        key_bindings,
        mouse_bindings(),
        conn,
    )?);

    // Add scratchpads
    let wm = add_named_scratchpads(wm, vec![nsp]);

    // Run!
    wm.run()
}
