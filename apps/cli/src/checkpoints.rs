use anyhow::Result;
use chrono::Local;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;

use crate::models::{Checkpoint, ClaudeSession};

// ── Path helpers ──────────────────────────────────────────────────────────────

fn project_hash(project_path: &str) -> String {
    let mut h = DefaultHasher::new();
    project_path.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn local_index_path(project_path: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".claude")
            .join("claux")
            .join("checkpoints")
            .join(format!("{}.json", project_hash(project_path)))
    })
}

fn per_project_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".claux").join("checkpoints.json")
}

// ── Load / save ───────────────────────────────────────────────────────────────

pub fn load_checkpoints(project_path: &str) -> Vec<Checkpoint> {
    let path = match local_index_path(project_path) {
        Some(p) => p,
        None    => return vec![],
    };
    let data = match std::fs::read_to_string(&path) {
        Ok(d)  => d,
        Err(_) => return vec![],
    };
    let mut list: Vec<Checkpoint> = serde_json::from_str(&data).unwrap_or_default();
    // Newest first
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    list
}

fn write_checkpoints(project_path: &str, checkpoints: &[Checkpoint]) -> Result<()> {
    // Local index
    if let Some(path) = local_index_path(project_path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(checkpoints)?)?;
    }

    // Per-project copy
    let pp = per_project_path(project_path);
    if let Some(parent) = pp.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&pp, serde_json::to_string_pretty(checkpoints)?)?;

    Ok(())
}

pub fn save_checkpoint(
    project_path: &str,
    sessions:     &[ClaudeSession],
    name:         &str,
) -> Result<Checkpoint> {
    // Cost and session stats for this project
    let project_sessions: Vec<&ClaudeSession> = sessions
        .iter()
        .filter(|s| s.project_path == project_path)
        .collect();
    let cost_total_usd = project_sessions.iter().map(|s| s.total_cost).sum();
    let total_sessions = project_sessions.len();

    let active = project_sessions.iter().find(|s| s.is_active);
    let session_id       = active.map(|s| s.id.clone());
    let session_cost_usd = active.map(|s| s.total_cost).unwrap_or(0.0);
    let claudemd_score   = active
        .and_then(|s| s.claudemd_score)
        .or_else(|| project_sessions.last().and_then(|s| s.claudemd_score));

    // Git info
    let git_commit = git_rev(project_path);
    let git_branch = git_branch(project_path);

    // Files changed since the last checkpoint
    let existing = load_checkpoints(project_path);
    let prior_commit = existing.iter()
        .find(|c| c.git_commit.is_some())
        .and_then(|c| c.git_commit.clone());
    let files_changed = prior_commit
        .as_deref()
        .and_then(|prev| git_diff_files(project_path, prev, git_commit.as_deref()))
        .unwrap_or_default();

    let id = random_hex8();
    let cp = Checkpoint {
        id:               id.clone(),
        name:             name.to_string(),
        created_at:       Local::now().to_rfc3339(),
        project_path:     project_path.to_string(),
        session_id,
        git_branch,
        git_commit,
        cost_total_usd,
        session_cost_usd,
        total_sessions,
        files_changed,
        claudemd_score,
        summary:          String::new(),
    };

    let mut all = load_checkpoints(project_path);
    all.insert(0, cp.clone());
    write_checkpoints(project_path, &all)?;

    Ok(cp)
}

pub fn delete_checkpoint(project_path: &str, id_prefix: &str) -> Result<()> {
    let mut all = load_checkpoints(project_path);
    let before = all.len();
    all.retain(|c| !c.id.starts_with(id_prefix));
    if all.len() == before {
        anyhow::bail!("no checkpoint found with ID prefix '{}'", id_prefix);
    }
    write_checkpoints(project_path, &all)
}

pub fn find_checkpoint<'a>(checkpoints: &'a [Checkpoint], id_prefix: &str) -> Option<&'a Checkpoint> {
    checkpoints.iter().find(|c| c.id.starts_with(id_prefix))
}

// ── Context markdown generation ───────────────────────────────────────────────

pub fn generate_context_md(cp: &Checkpoint) -> String {
    let date = cp.created_at
        .split('T')
        .next()
        .unwrap_or(&cp.created_at)
        .to_string();

    let branch_line = match (&cp.git_branch, &cp.git_commit) {
        (Some(b), Some(c)) => format!("**Branch:** {}  ·  **Commit:** {}", b, &c[..c.len().min(8)]),
        (Some(b), None)    => format!("**Branch:** {}", b),
        (None, Some(c))    => format!("**Commit:** {}", &c[..c.len().min(8)]),
        (None, None)       => String::new(),
    };

    let claudemd_line = cp.claudemd_score
        .map(|s| format!("\n**CLAUDE.md score:** {}/100", s))
        .unwrap_or_default();

    let files_section = if cp.files_changed.is_empty() {
        String::new()
    } else {
        let list: String = cp.files_changed.iter()
            .map(|f| format!("- {}\n", f))
            .collect();
        format!("\n## Files changed since last checkpoint\n{}", list)
    };

    let summary_section = if cp.summary.is_empty() {
        String::new()
    } else {
        format!("\n## Summary\n{}\n", cp.summary)
    };

    format!(
        "# Checkpoint: {}\n\n\
         **Saved:** {}  \n\
         {}  \n\
         **Cost to date:** ${:.2} across {} sessions  \n\
         **Session cost:** ${:.2}  \n\
         {}\
         {}\
         {}\n\
         ---\n\
         *Load this context: `claux checkpoint load {}`  \
         or read `.claux/CONTEXT.md`*\n",
        cp.name,
        date,
        branch_line,
        cp.cost_total_usd,
        cp.total_sessions,
        cp.session_cost_usd,
        claudemd_line,
        summary_section,
        files_section,
        cp.id,
    )
}

pub fn write_context_md(project_path: &str, cp: &Checkpoint) -> Result<()> {
    let dir = PathBuf::from(project_path).join(".claux");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("CONTEXT.md");
    std::fs::write(&path, generate_context_md(cp))?;
    Ok(())
}

// ── Git helpers ───────────────────────────────────────────────────────────────

fn git_rev(project_path: &str) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn git_branch(project_path: &str) -> Option<String> {
    Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
        .flatten()
}

fn git_diff_files(project_path: &str, from: &str, to: Option<&str>) -> Option<Vec<String>> {
    let to_ref = to.unwrap_or("HEAD");
    let range  = format!("{}..{}", from, to_ref);
    let out = Command::new("git")
        .args(["diff", "--name-only", &range])
        .current_dir(project_path)
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    let files: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Some(files)
}

// ── Misc ──────────────────────────────────────────────────────────────────────

fn random_hex8() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // XOR with address-based noise for uniqueness within the same second
    let noise = &t as *const u32 as u64;
    format!("{:08x}", (t as u64) ^ noise)
}

/// Infer the most relevant project path from a session list.
pub fn infer_project_path(sessions: &[ClaudeSession]) -> String {
    sessions.iter()
        .find(|s| s.is_active)
        .or_else(|| sessions.first())
        .map(|s| s.project_path.clone())
        .unwrap_or_else(|| ".".to_string())
}
