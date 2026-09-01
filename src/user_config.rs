//! User-editable configuration, loaded from `~/.config/pwm/config.toml`.
//!
//! This file is read once at startup (see [`load_theme`]); there is no hot
//! reload yet, so changes only take effect after logging out and back in.
//! Anything missing, malformed, or simply absent falls back to sane
//! built-in defaults rather than failing to start the window manager.
use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::PathBuf};
use tracing::warn;

/// A resolved color + font theme, ready to hand to the bar and to
/// `PwmConfig`. Colors are `0xRRGGBBAA` values, matching the format
/// `penrose::Color` expects (the alpha byte is accepted for consistency
/// with the rest of the codebase, but is ignored by the actual X11/Xft
/// drawing paths).
#[derive(Debug, Clone)]
pub struct Theme {
    pub black: u32,
    pub white: u32,
    pub grey: u32,
    pub accent: u32,
    pub font: String,
}

const DEFAULT_FONT: &str = "JetBrainsMono Nerd Font";
const DEFAULT_THEME_NAME: &str = "d77";

/// The themes pwm ships with. `d77` is the original hand-picked palette
/// (Gruvbox Dark + a custom lavender accent); the others are sourced from
/// each project's own published palette.
fn builtin_themes() -> HashMap<String, Theme> {
    HashMap::from([
        (
            DEFAULT_THEME_NAME.to_string(),
            Theme {
                black: 0x282828ff,
                white: 0xebdbb2ff,
                grey: 0x3c3836ff,
                accent: 0xaa96daff,
                font: DEFAULT_FONT.to_string(),
            },
        ),
        (
            "gruvbox_dark".to_string(),
            Theme {
                black: 0x282828ff,
                white: 0xebdbb2ff,
                grey: 0x3c3836ff,
                accent: 0xfe8019ff, // canonical Gruvbox bright orange
                font: DEFAULT_FONT.to_string(),
            },
        ),
        (
            "arc_dark".to_string(),
            Theme {
                black: 0x2f343fff,
                white: 0xd3dae3ff,
                grey: 0x353945ff,
                accent: 0x5294e2ff,
                font: DEFAULT_FONT.to_string(),
            },
        ),
        (
            "dracula".to_string(),
            Theme {
                black: 0x282a36ff,
                white: 0xf8f8f2ff,
                grey: 0x44475aff,
                accent: 0xbd93f9ff,
                font: DEFAULT_FONT.to_string(),
            },
        ),
        (
            "tokyo_night".to_string(),
            Theme {
                black: 0x1a1b26ff,
                white: 0xc0caf5ff,
                grey: 0x414868ff,
                accent: 0xbb9af7ff,
                font: DEFAULT_FONT.to_string(),
            },
        ),
    ])
}

#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    #[serde(default)]
    theme: RawThemeSection,
}

#[derive(Debug, Default, Deserialize)]
struct RawThemeSection {
    active: Option<String>,
    // Any table under [theme.*] other than the `active` key above is
    // captured here as a user-defined (or overridden) palette.
    #[serde(flatten)]
    palettes: HashMap<String, RawTheme>,
}

#[derive(Debug, Deserialize)]
struct RawTheme {
    black: String,
    white: String,
    grey: String,
    accent: String,
    #[serde(default = "default_font")]
    font: String,
}

fn default_font() -> String {
    DEFAULT_FONT.to_string()
}

/// Parses a color string in `0xRRGGBB[AA]` or `#RRGGBB[AA]` form (the `0x`/`#`
/// prefix is optional) into the `0xRRGGBBAA` form `penrose::Color` expects.
fn parse_hex_color(raw: &str) -> Option<u32> {
    let cleaned = raw.trim().trim_start_matches("0x").trim_start_matches('#');

    match cleaned.len() {
        6 => u32::from_str_radix(cleaned, 16)
            .ok()
            .map(|rgb| (rgb << 8) | 0xff),
        8 => u32::from_str_radix(cleaned, 16).ok(),
        _ => None,
    }
}

impl RawTheme {
    /// Converts to a resolved [`Theme`], falling back to `fallback`'s fields
    /// one at a time for anything that fails to parse, so a single typo
    /// doesn't take out the whole palette.
    fn resolve(&self, name: &str, fallback: &Theme) -> Theme {
        let field = |raw: &str, field_name: &str, default: u32| match parse_hex_color(raw) {
            Some(v) => v,
            None => {
                warn!(
                    theme = name,
                    field = field_name,
                    value = raw,
                    "invalid color, using fallback"
                );
                default
            }
        };

        Theme {
            black: field(&self.black, "black", fallback.black),
            white: field(&self.white, "white", fallback.white),
            grey: field(&self.grey, "grey", fallback.grey),
            accent: field(&self.accent, "accent", fallback.accent),
            font: self.font.clone(),
        }
    }
}

/// Resolves the `~/.config/pwm/config.toml` path, honoring `$XDG_CONFIG_HOME`.
fn config_path() -> Option<PathBuf> {
    let base = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(env::var_os("HOME")?).join(".config"),
    };

    Some(base.join("pwm").join("config.toml"))
}

fn read_raw_config() -> RawConfig {
    let Some(path) = config_path() else {
        return RawConfig::default();
    };

    let Ok(contents) = fs::read_to_string(&path) else {
        // Not finding a config file is the common case (nobody has written
        // one yet) - nothing to warn about.
        return RawConfig::default();
    };

    match toml::from_str(&contents) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!(path = %path.display(), error = %e, "failed to parse config.toml, using defaults");
            RawConfig::default()
        }
    }
}

/// Loads the active theme for this session: built-in themes, overlaid with
/// any palettes defined in `config.toml`, selecting whichever one
/// `[theme].active` names (defaulting to `"d77"`, and falling back to it if
/// the requested name doesn't exist).
pub fn load_theme() -> Theme {
    let raw = read_raw_config();
    let mut themes = builtin_themes();
    let default_theme = themes[DEFAULT_THEME_NAME].clone();

    for (name, raw_theme) in &raw.theme.palettes {
        let fallback = themes.get(name).unwrap_or(&default_theme).clone();
        themes.insert(name.clone(), raw_theme.resolve(name, &fallback));
    }

    let active = raw.theme.active.as_deref().unwrap_or(DEFAULT_THEME_NAME);

    match themes.get(active) {
        Some(theme) => theme.clone(),
        None => {
            warn!(
                active,
                "unknown theme, falling back to '{DEFAULT_THEME_NAME}'"
            );
            default_theme
        }
    }
}
