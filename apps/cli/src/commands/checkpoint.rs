use anyhow::Result;
use comfy_table::Cell;
use owo_colors::OwoColorize;

use crate::checkpoints::{
    delete_checkpoint, find_checkpoint, generate_context_md, infer_project_path, load_checkpoints,
    save_checkpoint, write_context_md,
};
use crate::models::ClaudeSession;
use crate::monitor::{load_sessions, SessionCache};
use crate::render::{kv, make_table, section, success, warning};

#[derive(clap::Subcommand)]
pub enum CheckpointAction {
    /// Save a named checkpoint for the current project.
    Save {
        /// Short name/label (prompted if omitted).
        name: Option<String>,
    },
    /// List checkpoints for the current project.
    List,
    /// Print checkpoint context to stdout (use --write to also create .claux/CONTEXT.md).
    Load {
        /// Checkpoint ID prefix.
        id: String,
        /// Write .claux/CONTEXT.md into the project directory.
        #[arg(long)]
        write: bool,
    },
    /// Delete a checkpoint by ID prefix.
    Delete {
        /// Checkpoint ID prefix.
        id: String,
    },
}

pub fn run(action: &CheckpointAction) -> Result<()> {
    let mut cache = SessionCache::new();
    let sessions = load_sessions(&mut cache);
    let project_path = infer_project_path(&sessions);

    match action {
        CheckpointAction::Save { name } => cmd_save(&sessions, &project_path, name.as_deref()),
        CheckpointAction::List => cmd_list(&project_path),
        CheckpointAction::Load { id, write } => cmd_load(&project_path, id, *write),
        CheckpointAction::Delete { id } => cmd_delete(&project_path, id),
    }
}

fn cmd_save(sessions: &[ClaudeSession], project_path: &str, name: Option<&str>) -> Result<()> {
    let label = match name {
        Some(n) => n.to_string(),
        None => {
            eprint!("Checkpoint name: ");
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf)?;
            let trimmed = buf.trim().to_string();
            if trimmed.is_empty() {
                anyhow::bail!("checkpoint name cannot be empty");
            }
            trimmed
        }
    };

    let cp = save_checkpoint(project_path, sessions, &label)?;
    println!("{}", section("Checkpoint"));
    println!(
        "{}",
        success(format!("Saved checkpoint {} \"{}\"", cp.id, cp.name))
    );
    if let Some(b) = &cp.git_branch {
        let commit = cp
            .git_commit
            .as_deref()
            .map(|c| &c[..c.len().min(8)])
            .unwrap_or("");
        println!("{}", kv("branch", format!("{} {}", b, commit)));
    }
    println!(
        "{}",
        kv(
            "cost to date",
            format!("${:.2} ({} sessions)", cp.cost_total_usd, cp.total_sessions)
        )
    );
    if !cp.files_changed.is_empty() {
        println!(
            "{}",
            kv("changed files", cp.files_changed.len().to_string())
        );
    }
    Ok(())
}

fn cmd_list(project_path: &str) -> Result<()> {
    let checkpoints = load_checkpoints(project_path);
    println!("{}", section("Checkpoint"));
    if checkpoints.is_empty() {
        println!("{}", warning("No checkpoints for this project"));
        println!(
            "{}",
            "  Run `claux checkpoint save` to create one.".dimmed()
        );
        return Ok(());
    }

    let mut table = make_table(&["ID", "Name", "Saved", "Branch", "Cost", "Files"]);

    for cp in &checkpoints {
        let date = cp
            .created_at
            .split('T')
            .next()
            .unwrap_or(&cp.created_at)
            .to_string();
        let branch = cp.git_branch.clone().unwrap_or_else(|| "—".to_string());
        let cost = format!("${:.2}", cp.cost_total_usd);
        let files = cp.files_changed.len().to_string();
        table.add_row(vec![
            Cell::new(&cp.id),
            Cell::new(&cp.name),
            Cell::new(date),
            Cell::new(branch),
            Cell::new(cost),
            Cell::new(files),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn cmd_load(project_path: &str, id_prefix: &str, do_write: bool) -> Result<()> {
    let checkpoints = load_checkpoints(project_path);
    let cp = find_checkpoint(&checkpoints, id_prefix)
        .ok_or_else(|| anyhow::anyhow!("no checkpoint found with ID prefix '{}'", id_prefix))?;

    println!("{}", generate_context_md(cp));

    if do_write {
        write_context_md(project_path, cp)?;
        eprintln!("{}", success("Wrote .claux/CONTEXT.md"));
    }
    Ok(())
}

fn cmd_delete(project_path: &str, id_prefix: &str) -> Result<()> {
    delete_checkpoint(project_path, id_prefix)?;
    println!("{}", section("Checkpoint"));
    println!("{}", success(format!("Deleted checkpoint {}", id_prefix)));
    Ok(())
}
