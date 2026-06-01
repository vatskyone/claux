use anyhow::Result;
use owo_colors::OwoColorize;

use crate::format;
use crate::metrics::record_empty_state;
use crate::models::ClaudeSession;
use crate::render::{active_dot, cost_colored, make_table, model_colored, section, warning};

pub fn run(sessions: &[ClaudeSession], limit: usize, json: bool) -> Result<()> {
    let items: Vec<&ClaudeSession> = sessions.iter().take(limit).collect();

    if json {
        let val: Vec<serde_json::Value> = items
            .iter()
            .map(|s| serde_json::to_value(s))
            .collect::<Result<Vec<_>, _>>()?;
        println!("{}", serde_json::to_string_pretty(&val)?);
        return Ok(());
    }

    if items.is_empty() {
        record_empty_state("no_data_yet");
        println!("{}", section("Sessions"));
        println!("{}", warning("No sessions found"));
        println!("{}", "  Start a session to populate history.".dimmed());
        return Ok(());
    }

    println!("{}", section("Sessions"));
    let mut table = make_table(&["", "When", "Duration", "Model", "Project", "Cost"]);

    for s in &items {
        let dot = active_dot(s.is_active);
        let when = format::relative_time(&s.start_time);
        let dur = format::duration(s.duration_secs());
        let model = model_colored(&format::model_short_name(&s.model));
        let path = s.display_path();
        let cost = cost_colored(s.total_cost);
        let title = s.title.as_deref().unwrap_or(&path);
        let label = if title.len() > 40 {
            format!("{}…", &title[..39])
        } else {
            title.to_string()
        };

        table.add_row(vec![dot, when, dur, model, label, cost]);
    }

    println!("{table}");
    Ok(())
}
