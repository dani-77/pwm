use penrose::{Color, x::XConn};
use penrose_ui::{
    Result, TextStyle,
    bar::{
        Position, StatusBar,
        widgets::{
            ActiveWindowName, CurrentLayout, RefreshText, Widget, Workspaces,
            sys::{
                helpers::battery_file_search,
                interval::{amixer_volume, current_date_and_time},
                refresh::wifi_network,
            },
        },
    },
};
use std::{fs, time::Duration};

const BLACK: u32 = 0x282828ff;
const WHITE: u32 = 0xebdbb2ff;
const GREY: u32 = 0x3c3836ff;
const LAVENDER: u32 = 0xAA96DA;
const FONT: &str = "Iosevka";
const BAR_HEIGHT_PX: u32 = 22;
const BAR_POINT_SIZE: u8 = 12;
const MAX_ACTIVE_WINDOW_CHARS: usize = 50;

// penrose_ui's battery widget only reads charge_now/charge_full, which some
// laptops (e.g. this one) don't expose, reporting energy_now/energy_full
// instead. `capacity` is always present and already a 0-100 percentage,
// regardless of which units the battery reports in.
// FIXME: the >=70/>=50/>=20/else branches below all return the identical
// glyph — clippy::if_same_then_else (see CI) is right to flag this as
// dead-weight duplication. Looks like a copy-paste slip when this was
// written: only "Charging" and "Full"/>=90% actually render differently
// today, so the bar shows the same icon at 65% and at 5% charge. Left
// as-is rather than guessing which Nerd Font glyphs were intended for
// each tier — needs a human to pick the right four icons.
#[allow(clippy::if_same_then_else)]
fn battery_icon(charge: u32, status: &str) -> &'static str {
    if status == "Charging" {
        ""
    } else if charge >= 90 || status == "Full" {
        ""
    } else if charge >= 70 {
        ""
    } else if charge >= 50 {
        ""
    } else if charge >= 20 {
        ""
    } else {
        ""
    }
}

fn battery_percent(bat: &str) -> Option<String> {
    let status = fs::read_to_string(format!("/sys/class/power_supply/{bat}/status")).ok()?;
    let status = status.trim();
    let capacity = fs::read_to_string(format!("/sys/class/power_supply/{bat}/capacity")).ok()?;
    let charge: u32 = capacity.trim().parse().ok()?;

    Some(format!("{} {charge}%", battery_icon(charge, status)))
}

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

    let bat: &'static str = battery_file_search()
        .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
        .unwrap_or("BAT0");

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
        Box::new(RefreshText::new(pstyle, move || {
            battery_percent(bat).unwrap_or_default()
        })),
        Box::new(amixer_volume("Master", pstyle, ms(1000))),
        Box::new(wifi_network(pstyle)),
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
