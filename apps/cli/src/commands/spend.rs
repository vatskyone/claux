use anyhow::Result;
use owo_colors::OwoColorize;

use crate::format;
use crate::models::ClaudeSession;
use crate::render::trend;
use crate::spend::compute_spend;

pub fn run(sessions: &[ClaudeSession], json: bool) -> Result<()> {
    let s = compute_spend(sessions);

    if json {
        println!("{}", serde_json::to_string_pretty(&s)?);
        return Ok(());
    }

    let col1 = 12usize;
    let col2 = 10usize;

    println!("{}", "─── Spend Summary ───────────────────────".dimmed());

    // Today
    println!(
        "{:<col1$}  {:>col2$}   {}",
        "Today".bold(),
        format::cost(s.today),
        trend(s.today, s.yesterday)
    );

    // This week
    println!(
        "{:<col1$}  {:>col2$}   {}",
        "This week".bold(),
        format::cost(s.this_week),
        trend(s.this_week, s.prev_week)
    );

    // This month
    println!(
        "{:<col1$}  {:>col2$}   {}",
        "This month".bold(),
        format::cost(s.this_month),
        trend(s.this_month, s.prev_month)
    );

    println!("{}", "─────────────────────────────────────────".dimmed());

    Ok(())
}
