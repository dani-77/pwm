use penrose::builtin::layout::{
    Grid, MainAndStack, Monocle,
    transformers::{Gaps, ReflectHorizontal, ReserveTop},
};
use penrose::core::layout::LayoutStack;
use penrose::extensions::layout::{Fibonacci, Tatami};
use penrose::stack;

use crate::config::LayoutConfig;

/// Creates the stack of available layouts
pub fn layouts() -> LayoutStack {
    let config = LayoutConfig::default();
    create_layouts_with_config(&config)
}

/// Creates layouts with custom configuration
pub fn create_layouts_with_config(config: &LayoutConfig) -> LayoutStack {
    stack!(
        // MainAndStack left side (default)
        MainAndStack::side(config.max_main, config.ratio, config.ratio_step),
        // MainAndStack right side (reflected)
        ReflectHorizontal::wrap(MainAndStack::side(
            config.max_main,
            config.ratio,
            config.ratio_step
        )),
        // MainAndStack at the bottom
        MainAndStack::bottom(config.max_main, config.ratio, config.ratio_step),
        // Monocle (fullscreen for focused window)
        Monocle::boxed(),
        // Grid (all windows in a grid)
        Grid::boxed(),
        // Fibonacci spiral
        Fibonacci::boxed_default(),
        // Tatami (Japanese style)
        Tatami::boxed_default()
    )
    .map(|layout| {
        ReserveTop::wrap(
            Gaps::wrap(layout, config.outer_px, config.inner_px),
            config.top_px,
        )
    })
}

/// Minimalist layout stack (fewer options)
#[allow(dead_code)]
pub fn minimal_layouts(config: &LayoutConfig) -> LayoutStack {
    stack!(
        MainAndStack::side(config.max_main, config.ratio, config.ratio_step),
        Monocle::boxed(),
        Grid::boxed()
    )
    .map(|layout| {
        ReserveTop::wrap(
            Gaps::wrap(layout, config.outer_px, config.inner_px),
            config.top_px,
        )
    })
}

/// Layout stack without gaps (maximum space)
#[allow(dead_code)]
pub fn no_gaps_layouts(config: &LayoutConfig) -> LayoutStack {
    stack!(
        MainAndStack::side(config.max_main, config.ratio, config.ratio_step),
        ReflectHorizontal::wrap(MainAndStack::side(
            config.max_main,
            config.ratio,
            config.ratio_step
        )),
        Monocle::boxed(),
        Grid::boxed()
    )
    .map(|layout| ReserveTop::wrap(layout, config.top_px))
}
