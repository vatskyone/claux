use anyhow::Result;
use owo_colors::OwoColorize;

use crate::models::ClaudeSession;
use crate::render::{section, success, warning};
use crate::tags;

pub fn run(
    sessions: &[ClaudeSession],
    session_prefix: &str,
    label: Option<&str>,
    remove: bool,
) -> Result<()> {
    let prefix_lower = session_prefix.to_lowercase();

    let matching: Vec<&ClaudeSession> = sessions
        .iter()
        .filter(|s| s.id.to_lowercase().starts_with(&prefix_lower))
        .collect();

    match matching.len() {
        0 => {
            eprintln!("{}", section("Tag"));
            eprintln!(
                "{}",
                warning(format!(
                    "No session found with ID starting with '{}'",
                    session_prefix
                ))
            );
            return Ok(());
        }
        n if n > 1 => {
            eprintln!("{}", section("Tag"));
            eprintln!(
                "{}",
                warning(format!(
                    "Ambiguous: {} sessions start with '{}'. Use more characters.",
                    n, session_prefix
                ))
            );
            for s in &matching {
                eprintln!(
                    "  {}  {}",
                    (&s.id[..s.id.len().min(16)]).to_string().dimmed(),
                    s.display_path()
                );
            }
            return Ok(());
        }
        _ => {}
    }

    let s = matching[0];
    let id_short = &s.id[..s.id.len().min(12)];

    if remove {
        tags::save_tag(&s.id, "")?;
        println!("{}", section("Tag"));
        println!(
            "{}",
            success(format!(
                "Removed tag from session {}… ({})",
                id_short,
                s.display_path()
            ))
        );
        return Ok(());
    }

    if let Some(lbl) = label {
        tags::save_tag(&s.id, lbl)?;
        println!("{}", section("Tag"));
        println!(
            "{}",
            success(format!("Tagged session {}… -> [{}]", id_short, lbl.trim()))
        );
    } else {
        println!("{}", section("Tag"));
        // Show current tag
        let current = s.tag.as_deref().unwrap_or("");
        if current.is_empty() {
            println!("{}", warning(format!("Session {}… has no tag", id_short)));
            println!(
                "{}",
                format!("  Use: claux tag {} <label>", session_prefix).dimmed()
            );
        } else {
            println!("Session {}… [{}]", id_short, current);
        }
    }

    Ok(())
}
