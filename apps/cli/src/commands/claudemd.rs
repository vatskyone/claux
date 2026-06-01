use anyhow::Result;
use owo_colors::OwoColorize;
use serde_json::json;
use std::fs;

use crate::checkpoints::infer_project_path;
use crate::claudemd::{
    claudemd_path, generate_for_project, improve_for_project, read_claudemd, write_claudemd,
};
use crate::monitor::{load_sessions, SessionCache};
use crate::parser::score_claudemd_detailed;
use crate::render::{kv, section, success};

#[derive(clap::Subcommand)]
pub enum ClaudeMdAction {
    /// Generate a CLAUDE.md starter from repository structure.
    Generate {
        /// Project path. Defaults to active session project or cwd.
        #[arg(long)]
        project: Option<String>,
        /// Write to <project>/CLAUDE.md instead of printing.
        #[arg(long)]
        write: bool,
        /// Overwrite existing CLAUDE.md when used with --write.
        #[arg(long)]
        force: bool,
        /// Output metadata as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Improve an existing CLAUDE.md by filling missing high-signal sections.
    Improve {
        /// Project path. Defaults to active session project or cwd.
        #[arg(long)]
        project: Option<String>,
        /// Write improvements to <project>/CLAUDE.md.
        #[arg(long)]
        write: bool,
        /// Create <project>/CLAUDE.md.bak before writing.
        #[arg(long)]
        backup: bool,
        /// Output metadata as JSON.
        #[arg(long)]
        json: bool,
    },
}

pub fn run(action: &ClaudeMdAction) -> Result<()> {
    match action {
        ClaudeMdAction::Generate {
            project,
            write,
            force,
            json,
        } => {
            let project_path = resolve_project_path(project.as_deref())?;
            let generated = generate_for_project(&project_path)?;
            let analysis = score_claudemd_detailed(&generated.content);

            if *write {
                let out = write_claudemd(&project_path, &generated.content, *force)?;
                if *json {
                    let val = json!({
                        "project_path": generated.project_path,
                        "output_path": out,
                        "score": analysis.score,
                        "suggestions": analysis.suggestions,
                        "language": generated.language,
                        "framework": generated.framework,
                        "key_dirs": generated.key_dirs,
                    });
                    println!("{}", serde_json::to_string_pretty(&val)?);
                } else {
                    println!("{}", section("CLAUDE.md"));
                    println!("{}", success(format!("Wrote {}", out.display())));
                    println!(
                        "{}",
                        kv("estimated score", format!("{}/100", analysis.score))
                    );
                    if !analysis.suggestions.is_empty() {
                        println!("{}", "Next improvements:".bold());
                        for s in &analysis.suggestions {
                            println!("  - {}", s);
                        }
                    }
                }
                return Ok(());
            }

            if *json {
                let val = json!({
                    "project_path": generated.project_path,
                    "score": analysis.score,
                    "suggestions": analysis.suggestions,
                    "language": generated.language,
                    "framework": generated.framework,
                    "key_dirs": generated.key_dirs,
                    "content": generated.content,
                });
                println!("{}", serde_json::to_string_pretty(&val)?);
            } else {
                println!("{}", section("CLAUDE.md Preview"));
                println!("{}", generated.content);
                eprintln!(
                    "\n{}",
                    kv("estimated score", format!("{}/100", analysis.score))
                );
                if !analysis.suggestions.is_empty() {
                    eprintln!("Potential improvements:");
                    for s in &analysis.suggestions {
                        eprintln!("  - {}", s);
                    }
                }
                eprintln!("\nWrite it with: claux claudemd generate --write");
            }
        }
        ClaudeMdAction::Improve {
            project,
            write,
            backup,
            json,
        } => {
            let project_path = resolve_project_path(project.as_deref())?;
            let before = read_claudemd(&project_path)?;
            let before_score = score_claudemd_detailed(&before).score;

            let improved = improve_for_project(&project_path)?;
            let after_analysis = score_claudemd_detailed(&improved.content);
            let after_score = after_analysis.score;

            let mut output_path = None;
            if *write {
                let path = claudemd_path(&project_path);
                if *backup {
                    let backup_path = path.with_extension("md.bak");
                    fs::copy(&path, &backup_path)?;
                }
                fs::write(&path, &improved.content)?;
                output_path = Some(path.display().to_string());
            }

            if *json {
                let val = json!({
                    "project_path": improved.project_path,
                    "output_path": output_path,
                    "before_score": before_score,
                    "after_score": after_score,
                    "delta": (after_score as i32 - before_score as i32),
                    "suggestions": after_analysis.suggestions,
                    "language": improved.language,
                    "framework": improved.framework,
                    "key_dirs": improved.key_dirs,
                    "content": if *write { serde_json::Value::Null } else { serde_json::Value::String(improved.content.clone()) },
                });
                println!("{}", serde_json::to_string_pretty(&val)?);
            } else if *write {
                println!("{}", section("CLAUDE.md"));
                println!(
                    "{}",
                    success(format!(
                        "Updated {}",
                        claudemd_path(&project_path).display()
                    ))
                );
                println!(
                    "{}",
                    kv(
                        "score",
                        format!(
                            "{} -> {} (delta: {:+})",
                            before_score,
                            after_score,
                            after_score as i32 - before_score as i32
                        )
                    )
                );
                if *backup {
                    println!(
                        "{}",
                        kv(
                            "backup",
                            claudemd_path(&project_path)
                                .with_extension("md.bak")
                                .display()
                                .to_string()
                        )
                    );
                }
                if !after_analysis.suggestions.is_empty() {
                    println!("{}", "Remaining improvements:".bold());
                    for s in &after_analysis.suggestions {
                        println!("  - {}", s);
                    }
                }
            } else {
                println!("{}", section("CLAUDE.md Preview"));
                println!("{}", improved.content);
                eprintln!(
                    "\n{}",
                    kv(
                        "score",
                        format!(
                            "{} -> {} (delta: {:+})",
                            before_score,
                            after_score,
                            after_score as i32 - before_score as i32
                        )
                    )
                );
                if !after_analysis.suggestions.is_empty() {
                    eprintln!("Remaining improvements:");
                    for s in &after_analysis.suggestions {
                        eprintln!("  - {}", s);
                    }
                }
                eprintln!("\nWrite it with: claux claudemd improve --write [--backup]");
            }
        }
    }

    Ok(())
}

fn resolve_project_path(explicit: Option<&str>) -> Result<String> {
    if let Some(p) = explicit {
        return Ok(p.to_string());
    }

    let mut cache = SessionCache::new();
    let sessions = load_sessions(&mut cache);
    let inferred = infer_project_path(&sessions);
    if inferred != "." {
        return Ok(inferred);
    }

    let cwd = std::env::current_dir()?;
    Ok(cwd.display().to_string())
}
