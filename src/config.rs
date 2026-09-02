use crate::user_config::Theme;
use penrose::Color;

/// Layout settings
pub struct LayoutConfig {
    pub max_main: u32,
    pub ratio: f32,
    pub ratio_step: f32,
    pub outer_px: u32,
    pub inner_px: u32,
    pub top_px: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            max_main: 1,
            ratio: 0.6,
            ratio_step: 0.1,
            outer_px: 5,
            inner_px: 5,
            top_px: 18,
        }
    }
}

/// Workspace settings
pub const WORKSPACES: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// Scratchpad settings
pub struct ScratchPadConfig {
    pub name: &'static str,
    pub command: &'static str,
    pub class_name: &'static str,
    pub width_ratio: f64,
    pub height_ratio: f64,
}

impl Default for ScratchPadConfig {
    fn default() -> Self {
        Self {
            name: "terminal",
            command: "st -c StScratchpad",
            class_name: "StScratchpad",
            width_ratio: 0.8,
            height_ratio: 0.8,
        }
    }
}

/// Main configuration structure
#[allow(dead_code)]
pub struct PwmConfig {
    pub focused_border: Color,
    pub normal_border: Color,
    pub layout: LayoutConfig,
    pub scratchpad: ScratchPadConfig,
}

impl PwmConfig {
    pub fn new(theme: &Theme) -> Self {
        Self {
            focused_border: theme.accent.into(),
            normal_border: theme.grey.into(),
            layout: LayoutConfig::default(),
            scratchpad: ScratchPadConfig::default(),
        }
    }
}
