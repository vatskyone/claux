use anyhow::Result;
use std::fs;

use crate::models::ClaudeSession;

#[derive(Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
}

pub fn run(
    sessions: &[ClaudeSession],
    limit:    usize,
    fmt:      ExportFormat,
    output:   Option<&str>,
) -> Result<()> {
    let items: Vec<&ClaudeSession> = sessions.iter().take(limit).collect();

    let content = match fmt {
        ExportFormat::Json => {
            let val: Vec<serde_json::Value> = items.iter()
                .map(|s| serde_json::to_value(s).unwrap_or_default())
                .collect();
            serde_json::to_string_pretty(&val)?
        }
        ExportFormat::Csv => build_csv(&items),
    };

    match output {
        Some(path) => {
            fs::write(path, &content)?;
            eprintln!("Exported {} sessions to {}", items.len(), path);
        }
        None => print!("{}", content),
    }

    Ok(())
}

fn build_csv(sessions: &[&ClaudeSession]) -> String {
    let mut out = String::new();
    out.push_str("id,project_path,start_time,end_time,duration_secs,cost_usd,model,\
                  input_tokens,output_tokens,cache_read_tokens,cache_write_tokens,\
                  thinking_tokens,is_active,title,tag\n");

    for s in sessions {
        let end = s.end_time.map(|t| t.to_rfc3339()).unwrap_or_default();
        let title = csv_escape(s.title.as_deref().unwrap_or(""));
        let path  = csv_escape(&s.project_path);
        let tag   = csv_escape(s.tag.as_deref().unwrap_or(""));

        out.push_str(&format!(
            "{},{},{},{},{},{:.6},{},{},{},{},{},{},{},{},{}\n",
            s.id,
            path,
            s.start_time.to_rfc3339(),
            end,
            s.duration_secs(),
            s.total_cost,
            s.model,
            s.token_usage.input_tokens,
            s.token_usage.output_tokens,
            s.token_usage.cache_read_tokens,
            s.token_usage.cache_write_tokens,
            s.token_usage.thinking_tokens,
            s.is_active,
            title,
            tag,
        ));
    }
    out
}

/// Minimal CSV escaping: wrap in quotes if the value contains comma, quote, or newline.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

