use comfy_table::{Cell, CellAlignment, Color, Table, presets};
use owo_colors::OwoColorize;

/// Build a pre-styled table with CLAUX's column header style.
pub fn make_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_BORDERS_ONLY);
    table.set_header(
        headers.iter().map(|h| {
            Cell::new(h)
                .fg(Color::DarkGrey)
                .set_alignment(CellAlignment::Left)
        })
    );
    table
}

/// Green dot for active, dim dot for inactive.
pub fn active_dot(active: bool) -> String {
    if active {
        "●".green().to_string()
    } else {
        "○".dimmed().to_string()
    }
}

/// Coloured cost string: green if low, yellow if medium, red if high.
pub fn cost_colored(c: f64) -> String {
    let s = crate::format::cost(c);
    if c < 1.0      { s.green().to_string()  }
    else if c < 5.0 { s.yellow().to_string() }
    else            { s.red().to_string()     }
}

/// Context bar: "████░░░░ 34%"
pub fn context_bar(fraction: f64, width: usize) -> String {
    let filled = ((fraction * width as f64).round() as usize).min(width);
    let empty  = width - filled;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    let pct = format!("{:.0}%", fraction * 100.0);
    let colored = if fraction < 0.70 {
        bar.blue().to_string()
    } else if fraction < 0.90 {
        bar.yellow().to_string()
    } else {
        bar.red().to_string()
    };
    format!("{} {}", colored, pct)
}

/// Mini ASCII sparkline for daily spend (bar chart scaled to `height` rows).
/// Returns a single line of block chars.
pub fn spend_sparkline(values: &[f64], width: usize) -> String {
    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max = values.iter().cloned().fold(0.0f64, f64::max);
    if max == 0.0 {
        return " ".repeat(width);
    }
    let step = values.len() as f64 / width as f64;
    (0..width)
        .map(|i| {
            let idx = (i as f64 * step) as usize;
            let v = values.get(idx).cloned().unwrap_or(0.0);
            let level = ((v / max) * (bars.len() - 1) as f64).round() as usize;
            bars[level.min(bars.len() - 1)]
        })
        .collect()
}

/// Trend indicator: "↑ $0.44" or "↓ $0.12" with color.
pub fn trend(current: f64, previous: f64) -> String {
    if previous == 0.0 {
        return String::new();
    }
    let diff = current - previous;
    if diff >= 0.0 {
        format!("↑ {}", crate::format::cost(diff)).red().to_string()
    } else {
        format!("↓ {}", crate::format::cost(-diff)).green().to_string()
    }
}

/// Model name colored by family.
pub fn model_colored(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("opus")   { name.purple().to_string() }
    else if lower.contains("haiku") { name.green().to_string()  }
    else                        { name.blue().to_string()    }
}
