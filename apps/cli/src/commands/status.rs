use anyhow::Result;
use owo_colors::OwoColorize;

use crate::metrics::record_empty_state;
use crate::models::ClaudeSession;
use crate::render::{context_bar, cost_colored, model_colored};
use crate::{format, render};

pub fn run(sessions: &[ClaudeSession], json: bool) -> Result<()> {
    let active = sessions.iter().find(|s| s.is_active);

    if json {
        let val = match active {
            Some(s) => serde_json::to_value(s)?,
            None => serde_json::Value::Null,
        };
        println!("{}", serde_json::to_string_pretty(&val)?);
        return Ok(());
    }

    match active {
        None => {
            record_empty_state("no_active_session");
            println!("{}", "○  No active session".dimmed());
        }
        Some(s) => {
            let dot = render::active_dot(true);
            let dur = format::duration(s.duration_secs());
            let path = s.display_path();
            let model = format::model_short_name(&s.model);

            // Header line
            println!(
                "{}  {}  {}  {}",
                dot,
                "Active Session".bold(),
                "·".dimmed(),
                dur.dimmed()
            );

            // Path + model
            println!(
                "   {}  {}  {}",
                path.cyan().bold(),
                "·".dimmed(),
                model_colored(&model)
            );

            // Cost row
            let cost = cost_colored(s.total_cost);
            let burn = format!("{}/hr", format::cost(s.burn_rate_per_hour()));
            let ctx = context_bar(s.context_health_fraction(), 10);
            let cache = format!("{:.0}% cache", s.token_usage.cache_hit_rate() * 100.0);

            println!(
                "   Cost {}   Burn {}   Context {}   {}",
                cost,
                burn.dimmed(),
                ctx,
                cache.dimmed()
            );

            // Tokens
            let total_tok = s.token_usage.input_tokens
                + s.token_usage.output_tokens
                + s.token_usage.cache_read_tokens
                + s.token_usage.cache_write_tokens;
            println!("   {} tokens", format::tokens(total_tok).dimmed());

            if let Some(score) = s.claudemd_score {
                let label = if score >= 70 {
                    "Good".green().to_string()
                } else if score >= 40 {
                    "Basic".yellow().to_string()
                } else {
                    "Poor".red().to_string()
                };
                println!("   CLAUDE.md quality: {} ({})", score, label);
            }
        }
    }

    Ok(())
}
