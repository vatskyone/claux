use anyhow::{Context, Result};
use comfy_table::{Cell, Color, Table};
use std::fs;

use crate::skills::load_skills;
use crate::models::SkillSource;

#[derive(clap::Subcommand)]
pub enum SkillsAction {
    /// List all available skills with usage stats.
    List,
    /// Export a custom skill directory to the current folder (or --out DIR).
    Export {
        /// Name of the skill to export.
        name: String,
        /// Destination directory (default: current dir).
        #[arg(long, short = 'o')]
        out: Option<String>,
    },
    /// Import a skill directory into ~/.claude/skills/.
    Import {
        /// Path to a skill directory (must contain SKILL.md).
        path: String,
    },
    /// Scaffold a new custom skill with a SKILL.md template.
    New {
        /// Name for the new skill.
        name: String,
    },
}

pub fn run(action: &SkillsAction) -> Result<()> {
    match action {
        SkillsAction::List    => list(),
        SkillsAction::Export { name, out } => export(name, out.as_deref()),
        SkillsAction::Import { path } => import(path),
        SkillsAction::New { name } => new_skill(name),
    }
}

fn list() -> Result<()> {
    let skills = load_skills();
    if skills.is_empty() {
        println!("No skills found.");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("").fg(Color::Grey),
        Cell::new("Skill").fg(Color::Grey),
        Cell::new("Type").fg(Color::Grey),
        Cell::new("Uses").fg(Color::Grey),
        Cell::new("Last used").fg(Color::Grey),
        Cell::new("Rating").fg(Color::Grey),
    ]);

    for skill in &skills {
        let dot     = if skill.source == SkillSource::Custom { "●" } else { "○" };
        let kind    = if skill.source == SkillSource::Custom { "custom" } else { "builtin" };
        let last    = skill.last_used_ms
            .map(|ms| relative_ms(ms))
            .unwrap_or_else(|| "never".to_string());
        let rating  = stars(skill.rating);
        table.add_row(vec![
            Cell::new(dot),
            Cell::new(&skill.name),
            Cell::new(kind),
            Cell::new(skill.usage_count.to_string()),
            Cell::new(last),
            Cell::new(rating),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn export(name: &str, out: Option<&str>) -> Result<()> {
    let skills_dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("skills"))
        .context("cannot find home directory")?;

    let src = skills_dir.join(name);
    if !src.is_dir() {
        anyhow::bail!("custom skill '{}' not found in ~/.claude/skills/", name);
    }

    let dest_base = out.map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("."));
    let dest = dest_base.join(name);
    fs::create_dir_all(&dest)?;

    for entry in fs::read_dir(&src)?.flatten() {
        let path = entry.path();
        if path.is_file() {
            let fname = entry.file_name();
            fs::copy(&path, dest.join(&fname))?;
        }
    }
    eprintln!("Exported '{}' → {}", name, dest.display());
    Ok(())
}

fn import(path: &str) -> Result<()> {
    let src = std::path::Path::new(path);
    anyhow::ensure!(src.is_dir(), "path '{}' is not a directory", path);
    anyhow::ensure!(src.join("SKILL.md").exists(), "directory must contain a SKILL.md file");

    let name = src.file_name()
        .and_then(|n| n.to_str())
        .context("invalid directory name")?;

    let dest = dirs::home_dir()
        .context("cannot find home directory")?
        .join(".claude").join("skills").join(name);

    fs::create_dir_all(&dest)?;
    for entry in fs::read_dir(src)?.flatten() {
        let p = entry.path();
        if p.is_file() {
            fs::copy(&p, dest.join(entry.file_name()))?;
        }
    }
    eprintln!("Imported '{}' → {}", name, dest.display());
    Ok(())
}

fn new_skill(name: &str) -> Result<()> {
    let dest = dirs::home_dir()
        .context("cannot find home directory")?
        .join(".claude").join("skills").join(name);

    anyhow::ensure!(!dest.exists(), "skill '{}' already exists", name);

    fs::create_dir_all(&dest)?;
    let template = format!(
        "# {}\n\n## Description\n\n<!-- What this skill does -->\n\n## When to use\n\n<!-- Trigger conditions -->\n\n## Instructions\n\n<!-- Step-by-step instructions -->\n",
        name
    );
    fs::write(dest.join("SKILL.md"), template)?;
    eprintln!("Created: ~/.claude/skills/{}/SKILL.md", name);
    eprintln!("Edit the file to add your skill's description and instructions.");
    Ok(())
}

fn relative_ms(ms: u64) -> String {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let secs = (now_ms.saturating_sub(ms)) / 1_000;
    if secs < 3_600       { format!("{}m ago", secs / 60) }
    else if secs < 86_400 { format!("{}h ago", secs / 3_600) }
    else                  { format!("{}d ago", secs / 86_400) }
}

fn stars(rating: u8) -> String {
    let n = rating.min(5) as usize;
    format!("{}{}", "★".repeat(n), "☆".repeat(5 - n))
}
