use anyhow::Result;
use chrono::{DateTime, Local};
use comfy_table::{Cell, Color, Table};
use owo_colors::OwoColorize;

use crate::account::{load_account_info, load_skill_usage};
use crate::skills::skill_rating;

pub fn run() -> Result<()> {
    let Some(info) = load_account_info() else {
        eprintln!("Could not read account info from ~/.claude.json");
        return Ok(());
    };

    // ── Account info card ──────────────────────────────────────────────────────
    println!();
    println!("{}", "  Account".bold());
    println!();

    let plan_label = plan_display(&info.plan_type);
    let rows: &[(&str, &str)] = &[
        ("Name",         &info.display_name),
        ("Email",        &info.email),
        ("Plan",         &plan_label),
        ("Organization", &info.org_name),
        ("Role",         &info.org_role),
        ("Billing",      &info.billing_type),
        ("Rate tier",    &info.rate_limit_tier),
    ];
    for (label, value) in rows {
        println!("  {:<16} {}", format!("{}:", label).dimmed(), value);
    }

    if let Some(created) = parse_date(&info.account_created) {
        println!("  {:<16} {}", "Account since:".dimmed(), created);
    }
    if let Some(sub) = &info.sub_created {
        if let Some(subdate) = parse_date(sub) {
            println!("  {:<16} {}", "Subscribed since:".dimmed(), subdate);
        }
    }
    println!(
        "  {:<16} {}",
        "Extra usage:".dimmed(),
        if info.has_extra_usage { "enabled".green().to_string() } else { "disabled".dimmed().to_string() }
    );

    // ── Skill usage table ──────────────────────────────────────────────────────
    let usage = load_skill_usage();
    if !usage.is_empty() {
        println!();
        println!("{}", "  Skills".bold());
        println!();

        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Skill").fg(Color::Grey),
            Cell::new("Uses").fg(Color::Grey),
            Cell::new("Last used").fg(Color::Grey),
            Cell::new("Rating").fg(Color::Grey),
        ]);

        let mut rows: Vec<(String, usize, Option<u64>)> = usage
            .into_iter()
            .map(|(n, (c, l))| (n, c, l))
            .collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        for (name, count, last_ms) in &rows {
            let last_str = last_ms
                .map(|ms| relative_ms(ms))
                .unwrap_or_else(|| "never".to_string());
            let rating = stars(skill_rating(*count));
            table.add_row(vec![
                Cell::new(&name),
                Cell::new(count.to_string()),
                Cell::new(last_str),
                Cell::new(rating),
            ]);
        }
        println!("{table}");
    }

    Ok(())
}

fn plan_display(plan_type: &str) -> String {
    match plan_type {
        "claude_pro"     => "Claude Pro".to_string(),
        "claude_max"     => "Claude Max".to_string(),
        "claude_team"    => "Claude Team".to_string(),
        "claude_free"    => "Claude Free".to_string(),
        "free"           => "Free".to_string(),
        other            => other.to_string(),
    }
}

fn parse_date(iso: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(iso)
        .ok()
        .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d").to_string())
}

fn relative_ms(ms: u64) -> String {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let secs = (now_ms.saturating_sub(ms)) / 1_000;
    if secs < 3_600      { format!("{}m ago", secs / 60) }
    else if secs < 86_400 { format!("{}h ago", secs / 3_600) }
    else                  { format!("{}d ago", secs / 86_400) }
}

fn stars(rating: u8) -> String {
    let n = rating.min(5) as usize;
    format!("{}{}", "★".repeat(n), "☆".repeat(5 - n))
}
