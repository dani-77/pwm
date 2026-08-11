use penrose::{Color, x::XConn};
use penrose_ui::{
    Result, TextStyle,
    bar::{
        Position, StatusBar,
        widgets::{
            ActiveWindowName, CurrentLayout, IntervalText, RefreshText, Widget, Workspaces,
            sys::{
                helpers::battery_file_search,
                interval::{amixer_volume, current_date_and_time},
                refresh::wifi_network,
            },
        },
    },
};
use std::{fs, sync::Mutex, time::Duration};

const BLACK: u32 = 0x282828ff;
const WHITE: u32 = 0xebdbb2ff;
const GREY: u32 = 0x3c3836ff;
const LAVENDER: u32 = 0xAA96DA;
const FONT: &str = "JetBrainsMono Nerd Font";
const BAR_HEIGHT_PX: u32 = 22;
const BAR_POINT_SIZE: u8 = 12;
const MAX_ACTIVE_WINDOW_CHARS: usize = 50;

// penrose_ui's battery widget only reads charge_now/charge_full, which some
// laptops (e.g. this one) don't expose, reporting energy_now/energy_full
// instead. `capacity` is always present and already a 0-100 percentage,
// regardless of which units the battery reports in.
fn battery_icon(charge: u32, status: &str) -> &'static str {
    if status == "Charging" {
        "\u{f0084}" // nf-md-battery_charging
    } else if charge >= 90 || status == "Full" {
        "\u{f240}" // nf-fa-battery_full
    } else if charge >= 70 {
        "\u{f241}" // nf-fa-battery_three_quarters
    } else if charge >= 50 {
        "\u{f242}" // nf-fa-battery_half
    } else if charge >= 20 {
        "\u{f243}" // nf-fa-battery_quarter
    } else {
        "\u{f244}" // nf-fa-battery_empty
    }
}

fn battery_percent(bat: &str) -> Option<String> {
    let status = fs::read_to_string(format!("/sys/class/power_supply/{bat}/status")).ok()?;
    let status = status.trim();
    let capacity = fs::read_to_string(format!("/sys/class/power_supply/{bat}/capacity")).ok()?;
    let charge: u32 = capacity.trim().parse().ok()?;

    Some(format!("{} {charge}%", battery_icon(charge, status)))
}

// Reads the aggregate "cpu" line from /proc/stat and returns (idle_ticks, total_ticks).
// CPU usage isn't an instantaneous value in Linux, only a running counter of time spent
// per state since boot, so a percentage requires diffing two samples over an interval.
fn read_cpu_ticks() -> Option<(u64, u64)> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?;
    let mut fields = line.split_whitespace();
    if fields.next()? != "cpu" {
        return None;
    }

    let values: Vec<u64> = fields.filter_map(|v| v.parse().ok()).collect();
    if values.len() < 4 {
        return None;
    }

    // user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice
    let idle = values[3] + values.get(4).copied().unwrap_or(0); // idle + iowait
    let total: u64 = values.iter().sum();

    Some((idle, total))
}

// Builds a CPU usage widget that samples /proc/stat on each refresh, keeping the previous
// sample in a `Mutex` (the closure only implements `Fn`, so interior mutability is needed
// to remember state between calls) to compute the percentage of busy ticks since last time.
fn cpu_usage(style: TextStyle, interval: Duration) -> IntervalText {
    let previous: Mutex<Option<(u64, u64)>> = Mutex::new(None);

    IntervalText::new(
        style,
        move || {
            let (idle, total) = read_cpu_ticks()?;
            let mut previous = previous.lock().ok()?;

            let pct = match *previous {
                Some((prev_idle, prev_total)) => {
                    let idle_delta = idle.saturating_sub(prev_idle);
                    let total_delta = total.saturating_sub(prev_total);
                    match (idle_delta * 100).checked_div(total_delta) {
                        Some(idle_pct) => 100u64.saturating_sub(idle_pct),
                        None => 0,
                    }
                }
                None => 0,
            };

            *previous = Some((idle, total));

            // U+F2DB = nf-fa-microchip
            Some(format!("\u{f2db} {pct}%"))
        },
        interval,
        false,
        true,
    )
}

fn ram_percent() -> Option<String> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;

    let mut total = None;
    let mut available = None;

    for line in meminfo.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = v.split_whitespace().next()?.parse::<u64>().ok();
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            available = v.split_whitespace().next()?.parse::<u64>().ok();
        }

        if total.is_some() && available.is_some() {
            break;
        }
    }

    let total = total?;
    let available = available?;
    if total == 0 {
        return None;
    }

    let pct = (total - available) * 100 / total;

    // U+E266 = nf-fae-memory
    Some(format!("\u{e266} {pct}%"))
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
        Box::new(cpu_usage(pstyle, ms(2000))),
        Box::new(RefreshText::new(pstyle, || {
            ram_percent().unwrap_or_default()
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
