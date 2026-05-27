use anyhow::Result;
use owo_colors::OwoColorize;
use serde_json::json;

use crate::models::ClaudeSession;
use crate::spend::{compute_daily_spend, compute_model_breakdown, compute_project_breakdown};
use crate::render::{make_table, model_colored, spend_sparkline};
use crate::format;

pub fn run(sessions: &[ClaudeSession], days: usize, json: bool) -> Result<()> {
    let daily    = compute_daily_spend(sessions);
    let projects = compute_project_breakdown(sessions);
    let models   = compute_model_breakdown(sessions);

    if json {
        let val = json!({
            "daily": daily,
            "projects": projects,
            "models": models,
        });
        println!("{}", serde_json::to_string_pretty(&val)?);
        return Ok(());
    }

    // ── Daily spend chart ─────────────────────────────────────────────────────
    let visible: Vec<_> = daily.iter().rev().take(days).rev().collect();
    let costs: Vec<f64> = visible.iter().map(|d| d.cost).collect();
    let total: f64      = costs.iter().sum();

    println!("{}", "─── Daily Spend ──────────────────────────".dimmed());
    println!("  {}  (last {} days, total {})",
        spend_sparkline(&costs, 30).blue().to_string(),
        days,
        format::cost(total).bold().to_string()
    );
    println!();

    // Daily table (last 14 days)
    let mut dtable = make_table(&["Date", "Cost", "Sparkline"]);
    for d in visible.iter().rev().take(14) {
        let bar = "█".repeat(((d.cost / costs.iter().cloned().fold(0.01f64, f64::max) * 20.0) as usize).min(20));
        dtable.add_row(vec![
            d.date.format("%b %d").to_string(),
            format::cost(d.cost),
            bar,
        ]);
    }
    println!("{dtable}");
    println!();

    // ── By project ────────────────────────────────────────────────────────────
    println!("{}", "─── By Project ───────────────────────────".dimmed());
    if projects.is_empty() {
        println!("  No data.");
    } else {
        let mut ptable = make_table(&["Project", "Sessions", "Cost"]);
        for p in projects.iter().take(10) {
            ptable.add_row(vec![
                p.display_path.clone(),
                p.session_count.to_string(),
                format::cost(p.total_cost),
            ]);
        }
        println!("{ptable}");
    }
    println!();

    // ── By model ──────────────────────────────────────────────────────────────
    println!("{}", "─── By Model ─────────────────────────────".dimmed());
    if models.is_empty() {
        println!("  No data.");
    } else {
        let mut mtable = make_table(&["Model", "Sessions", "Cost"]);
        for m in &models {
            mtable.add_row(vec![
                model_colored(&m.display_name),
                m.session_count.to_string(),
                format::cost(m.total_cost),
            ]);
        }
        println!("{mtable}");
    }

    Ok(())
}
