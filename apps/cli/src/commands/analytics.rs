use anyhow::Result;
use owo_colors::OwoColorize;
use serde_json::json;

use crate::format;
use crate::metrics::{load_local_metrics, record_empty_state, reset_local_metrics};
use crate::models::ClaudeSession;
use crate::render::{make_table, model_colored, spend_sparkline};
use crate::spend::{compute_daily_spend, compute_model_breakdown, compute_project_breakdown};

pub fn run(sessions: &[ClaudeSession], days: usize, json: bool) -> Result<()> {
    let daily = compute_daily_spend(sessions);
    let projects = compute_project_breakdown(sessions);
    let models = compute_model_breakdown(sessions);

    if json {
        let val = json!({
            "daily": daily,
            "projects": projects,
            "models": models,
        });
        println!("{}", serde_json::to_string_pretty(&val)?);
        return Ok(());
    }

    let visible: Vec<_> = daily.iter().rev().take(days).rev().collect();
    let costs: Vec<f64> = visible.iter().map(|d| d.cost).collect();
    let total: f64 = costs.iter().sum();

    println!("{}", "─── Daily Spend ──────────────────────────".dimmed());
    println!(
        "  {}  (last {} days, total {})",
        spend_sparkline(&costs, 30).blue().to_string(),
        days,
        format::cost(total).bold().to_string()
    );
    println!();

    if costs.iter().all(|c| *c == 0.0) {
        record_empty_state("no_data_yet");
    }

    let mut dtable = make_table(&["Date", "Cost", "Sparkline"]);
    let max_cost = costs.iter().cloned().fold(0.0f64, f64::max).max(0.01);
    for d in visible.iter().rev().take(14) {
        let bar = "█".repeat(((d.cost / max_cost * 20.0) as usize).min(20));
        dtable.add_row(vec![
            d.date.format("%b %d").to_string(),
            format::cost(d.cost),
            bar,
        ]);
    }
    println!("{dtable}");
    println!();

    println!("{}", "─── By Project ───────────────────────────".dimmed());
    if projects.is_empty() {
        record_empty_state("no_data_yet");
        println!("  No data yet (no parsed sessions in current tracking window).",);
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

    println!("{}", "─── By Model ─────────────────────────────".dimmed());
    if models.is_empty() {
        record_empty_state("source_unavailable");
        println!("  Source unavailable or no model usage parsed yet.");
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

pub fn run_local_metrics(reset: bool, as_json: bool) -> Result<()> {
    let metrics = load_local_metrics();

    if as_json {
        println!("{}", serde_json::to_string_pretty(&metrics)?);
    } else {
        println!("Local CLI metrics (stored on-device)");
        println!();

        println!("Command usage:");
        if metrics.command_counts.is_empty() {
            println!("  (no command data yet)");
        } else {
            let mut items: Vec<_> = metrics.command_counts.iter().collect();
            items.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            for (name, count) in items {
                println!("  {:<16} {}", name, count);
            }
        }

        println!();
        println!("Failure classes:");
        if metrics.failure_counts.is_empty() {
            println!("  (none)");
        } else {
            let mut items: Vec<_> = metrics.failure_counts.iter().collect();
            items.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            for (name, count) in items {
                println!("  {:<16} {}", name, count);
            }
        }

        println!();
        println!("Empty-state reasons:");
        if metrics.empty_state_counts.is_empty() {
            println!("  (none)");
        } else {
            let mut items: Vec<_> = metrics.empty_state_counts.iter().collect();
            items.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            for (name, count) in items {
                println!("  {:<16} {}", name, count);
            }
        }

        println!();
        println!("Refresh latency buckets:");
        if metrics.refresh_latency_buckets.is_empty() {
            println!("  (none)");
        } else {
            let mut items: Vec<_> = metrics.refresh_latency_buckets.iter().collect();
            items.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            for (name, count) in items {
                println!("  {:<16} {}", name, count);
            }
        }

        if let Some(ts) = &metrics.updated_at {
            println!();
            println!("Last updated: {}", ts);
        }
    }

    if reset {
        reset_local_metrics()?;
        eprintln!("Local metrics reset.");
    }

    Ok(())
}
