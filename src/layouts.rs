use penrose::builtin::layout::{
    Grid, MainAndStack, Monocle,
    transformers::{Gaps, ReflectHorizontal, ReserveTop},
};
use penrose::core::layout::LayoutStack;
use penrose::extensions::layout::{Fibonacci, Tatami};
use penrose::stack;

use crate::config::LayoutConfig;

/// Cria o stack de layouts disponíveis
pub fn layouts() -> LayoutStack {
    let config = LayoutConfig::default();
    create_layouts_with_config(&config)
}

/// Cria layouts com configuração personalizada
pub fn create_layouts_with_config(config: &LayoutConfig) -> LayoutStack {
    stack!(
        // MainAndStack lado esquerdo (padrão)
        MainAndStack::side(config.max_main, config.ratio, config.ratio_step),
        // MainAndStack lado direito (refletido)
        ReflectHorizontal::wrap(MainAndStack::side(
            config.max_main,
            config.ratio,
            config.ratio_step
        )),
        // MainAndStack em baixo
        MainAndStack::bottom(config.max_main, config.ratio, config.ratio_step),
        // Monocle (fullscreen para janela focada)
        Monocle::boxed(),
        // Grid (todas as janelas em grelha)
        Grid::boxed(),
        // Fibonacci spiral
        Fibonacci::boxed_default(),
        // Tatami (estilo japonês)
        Tatami::boxed_default()
    )
    .map(|layout| {
        ReserveTop::wrap(
            Gaps::wrap(layout, config.outer_px, config.inner_px),
            config.top_px,
        )
    })
}

/// Layout stack minimalista (menos opções)
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

/// Layout stack sem gaps (máximo espaço)
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
