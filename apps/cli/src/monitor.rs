use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::models::ClaudeSession;
use crate::parser::parse_session;

const ACTIVE_MTIME_THRESHOLD: Duration = Duration::from_secs(90);

// ── mtime cache ───────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct SessionCache {
    entries: HashMap<PathBuf, (SystemTime, ClaudeSession)>,
}

impl SessionCache {
    pub fn new() -> Self { Self::default() }

    /// Load (or reload if changed) the session at `path`.
    pub fn get_or_parse(
        &mut self,
        path: &Path,
        active_ids: &HashSet<String>,
        mtime: SystemTime,
        is_recent: bool,
    ) -> Option<ClaudeSession> {
        if let Some((cached_mtime, cached_session)) = self.entries.get(path) {
            if *cached_mtime == mtime {
                // Cache hit: just refresh is_active flag
                let mut s = cached_session.clone();
                let id_active = active_ids.contains(&s.id);
                s.is_active = id_active || is_recent;
                s.end_time  = if s.is_active { None } else { cached_session.end_time };
                return Some(s);
            }
        }
        // Cache miss or stale — re-parse
        match parse_session(path, active_ids, is_recent) {
            Ok(s) => {
                self.entries.insert(path.to_path_buf(), (mtime, s.clone()));
                Some(s)
            }
            Err(e) => {
                eprintln!("warn: failed to parse {:?}: {}", path, e);
                None
            }
        }
    }

    /// Remove stale entries for files no longer on disk.
    pub fn evict(&mut self, seen: &HashSet<PathBuf>) {
        self.entries.retain(|k, _| seen.contains(k));
    }
}

// ── Active session IDs ────────────────────────────────────────────────────────

/// Read `~/.claude/sessions/<x>.json` files and collect the `sessionId` fields.
pub fn load_active_ids() -> HashSet<String> {
    let mut ids = HashSet::new();
    let dir = match dirs::home_dir() {
        Some(h) => h.join(".claude").join("sessions"),
        None    => return ids,
    };
    let Ok(entries) = fs::read_dir(&dir) else { return ids };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
        let Ok(text) = fs::read_to_string(&path) else { continue };
        let Ok(val): Result<serde_json::Value, _> = serde_json::from_str(&text) else { continue };
        if let Some(id) = val.get("sessionId").and_then(|v| v.as_str()) {
            ids.insert(id.to_string());
        }
    }
    ids
}

// ── JSONL file discovery ──────────────────────────────────────────────────────

/// Find all `*.jsonl` files under `~/.claude/projects/`.
pub fn find_jsonl_files() -> Vec<PathBuf> {
    let root = match dirs::home_dir() {
        Some(h) => h.join(".claude").join("projects"),
        None    => return vec![],
    };
    let mut files = Vec::new();
    collect_jsonl(&root, &mut files);
    files
}

fn collect_jsonl(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            out.push(path);
        }
    }
}

// ── Load all sessions ─────────────────────────────────────────────────────────

/// Scan disk, parse or cache-hit sessions, and return them sorted
/// (active first, then by start_time descending).
pub fn load_sessions(cache: &mut SessionCache) -> Vec<ClaudeSession> {
    let active_ids = load_active_ids();
    let files      = find_jsonl_files();
    let now        = SystemTime::now();

    let mut seen = HashSet::new();
    let mut sessions = Vec::new();

    for path in &files {
        let Ok(meta) = fs::metadata(path) else { continue };
        let Ok(mtime) = meta.modified() else { continue };

        // Is the file "recently modified" (active fallback)?
        let is_recent = now.duration_since(mtime)
            .map(|d| d < ACTIVE_MTIME_THRESHOLD)
            .unwrap_or(false);

        seen.insert(path.clone());
        if let Some(s) = cache.get_or_parse(path, &active_ids, mtime, is_recent) {
            sessions.push(s);
        }
    }

    cache.evict(&seen);

    sessions.sort_by(|a, b| {
        b.is_active.cmp(&a.is_active)
            .then(b.start_time.cmp(&a.start_time))
    });

    sessions
}
