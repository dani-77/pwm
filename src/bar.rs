use penrose::{
    Color,
    x::XConn,
};
use penrose_ui::{
    Result, TextStyle,
    bar::{
        Position, StatusBar,
        widgets::{
            ActiveWindowName, CurrentLayout, Widget, Workspaces,
            sys::interval::{current_date_and_time},
        },
    },
};
use std::time::Duration;

const BLACK: u32 = 0x282828ff;
const WHITE: u32 = 0xebdbb2ff;
const GREY: u32 = 0x3c3836ff;
const LAVENDER: u32 = 0xAA96DA;
const FONT: &str = "Iosevka";
const BAR_HEIGHT_PX: u32 = 20;
const BAR_POINT_SIZE: u8 = 12;
const MAX_ACTIVE_WINDOW_CHARS: usize = 50;

fn widgets<X: XConn>() -> Vec<Box<dyn Widget<X>>> {
    let highlight: Color = LAVENDER.into();
    let empty_ws: Color = GREY.into();
    
    let style = TextStyle {
        fg: WHITE.into(),
        bg: Some(BLACK.into()),
        padding: (2, 2),
    };

    let pstyle = TextStyle {
        padding: (5, 5),
        ..style
    };

    let ms = |n: u64| Duration::from_millis(n);

    vec![
        Box::new(Workspaces::new(style, highlight, empty_ws)),
        Box::new(CurrentLayout::new(style)),
        Box::new(ActiveWindowName::new(
            MAX_ACTIVE_WINDOW_CHARS,
            TextStyle {
                bg: Some(highlight),
                padding: (6, 4),
                ..style
            },
            true,
            false,
        )),
        Box::new(current_date_and_time(pstyle, ms(10_000))),
    ]
}

pub fn status_bar<X: XConn>() -> Result<StatusBar<X>> {
    StatusBar::try_new(
        Position::Top,
        BAR_HEIGHT_PX,
        Color::new_from_hex(BLACK),
        FONT,
        BAR_POINT_SIZE,
        widgets(),
    )
}
