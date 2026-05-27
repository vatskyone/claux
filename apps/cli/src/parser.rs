use anyhow::Result;
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::models::{ClaudeSession, TokenUsage};

// ── Pricing table ($ per million tokens) ─────────────────────────────────────

struct Pricing {
    input:       f64,
    output:      f64,
    cache_read:  f64,
    cache_write: f64,
}

fn pricing_for(model: &str) -> Pricing {
    let lower = model.to_lowercase();
    if lower.contains("opus") {
        Pricing { input: 15.0, output: 75.0, cache_read: 1.50, cache_write: 18.75 }
    } else if lower.contains("haiku") {
        Pricing { input: 0.80, output: 4.0,  cache_read: 0.08, cache_write: 1.00 }
    } else {
        // Sonnet (default)
        Pricing { input: 3.0,  output: 15.0, cache_read: 0.30, cache_write: 3.75 }
    }
}

// ── Timestamp parsing ─────────────────────────────────────────────────────────

fn parse_timestamp(s: &str) -> Option<DateTime<Local>> {
    // Try RFC3339 / ISO 8601 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Local));
    }
    // Fallback: basic format without colons in offset (e.g. "2024-01-15T10:30:00.000Z")
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return Some(Local.from_utc_datetime(&dt.naive_utc()));
    }
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
        return Some(Local.from_utc_datetime(&dt.naive_utc()));
    }
    None
}

// ── CLAUDE.md scoring ─────────────────────────────────────────────────────────

const JUNK_DIRS: &[&str] = &[
    ".git", "node_modules", ".build", "DerivedData", "Pods",
    "vendor", ".swiftpm", "dist", "build", ".next", "__pycache__",
];

fn score_claudemd(content: &str) -> u8 {
    let words: usize = content.split_whitespace().count();

    // Length score (0–30)
    let length_score: u32 = if words >= 300 { 30 }
        else if words >= 150 { 23 }
        else if words >= 80  { 16 }
        else if words >= 30  {  8 }
        else                 {  0 };

    // Structure score (0–30)
    let mut headings = 0u32;
    let mut code_delimiters = 0u32;
    let mut bullets = 0u32;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with('#') { headings += 1; }
        if t.starts_with("```") { code_delimiters += 1; }
        if t.starts_with("- ") || t.starts_with("* ") { bullets += 1; }
    }
    let structure_score = (headings * 5).min(15)
        + ((code_delimiters / 2) * 5).min(10)
        + (bullets / 4).min(5);

    // Content coverage (0–40) — 8 categories × 5 pts
    let lower = content.to_lowercase();
    let categories: &[&[&str]] = &[
        &["build", "compile", "swift build", "npm run", "yarn", "make ", "gradle", "cmake"],
        &["test", "pytest", "jest ", "xcode test", "unit test", "spec"],
        &["run ", "start ", "launch", "execute", "serve"],
        &["structure", "architecture", "layout", "directory", "folder", "project"],
        &["convention", "style guide", "pattern", "naming", "format", "lint"],
        &["important", "note:", "warning", "do not", "never ", "always ", "avoid"],
        &["command", "script", "bash", "shell", "cli"],
        &["workflow", "process", "step", "instruction", "guideline"],
    ];
    let content_score: u32 = categories.iter()
        .map(|kws| if kws.iter().any(|kw| lower.contains(kw)) { 5 } else { 0 })
        .sum();

    let total = (length_score + structure_score + content_score).min(100);
    total as u8
}

fn find_claudemd(project_path: &str) -> Option<u8> {
    let start = Path::new(project_path);

    // Pass 1: walk up (max 8 steps)
    let mut dir = start.to_path_buf();
    let home = dirs::home_dir();
    for _ in 0..8 {
        let candidate = dir.join("CLAUDE.md");
        if candidate.exists() {
            if let Ok(content) = fs::read_to_string(&candidate) {
                if content.len() >= 10 {
                    return Some(score_claudemd(&content));
                }
            }
        }
        if home.as_deref() == Some(&dir) { break; }
        if !dir.pop() { break; }
    }

    // Pass 2: walk down (BFS, depth ≤ 4)
    let mut queue: Vec<(std::path::PathBuf, u32)> = vec![(start.to_path_buf(), 0)];
    while let Some((cur, depth)) = queue.first().cloned() {
        queue.remove(0);
        if depth > 4 { continue; }
        let candidate = cur.join("CLAUDE.md");
        if candidate.exists() {
            if let Ok(content) = fs::read_to_string(&candidate) {
                if content.len() >= 10 {
                    return Some(score_claudemd(&content));
                }
            }
        }
        if depth < 4 {
            if let Ok(entries) = fs::read_dir(&cur) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if !name_str.starts_with('.') && !JUNK_DIRS.contains(&name_str.as_ref()) {
                            queue.push((path, depth + 1));
                        }
                    }
                }
            }
        }
    }
    None
}

// ── Main JSONL parser ─────────────────────────────────────────────────────────

/// Parse a single JSONL session file into a `ClaudeSession`.
/// `active_ids` is the set of session IDs currently listed in `~/.claude/sessions/`.
pub fn parse_session(path: &Path, active_ids: &HashSet<String>, is_recent: bool) -> Result<ClaudeSession> {
    let content = fs::read_to_string(path)?;
    let session_id = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let mut project_path = String::new();
    let mut entrypoint: Option<String> = None;
    let mut title: Option<String> = None;
    let mut model = "claude-sonnet-4-6".to_string();

    let mut input_tokens:       u64 = 0;
    let mut output_tokens:      u64 = 0;
    let mut cache_read_tokens:  u64 = 0;
    let mut cache_write_tokens: u64 = 0;
    let mut thinking_tokens:    u64 = 0;
    let mut last_context_window: u64 = 0;

    let mut total_cost = 0.0f64;
    let mut daily_costs: HashMap<NaiveDate, f64> = HashMap::new();

    let mut first_time: Option<DateTime<Local>> = None;
    let mut last_time:  Option<DateTime<Local>> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let Ok(val): Result<Value, _> = serde_json::from_str(line) else { continue };

        // Timestamp
        let turn_time = val.get("timestamp")
            .and_then(Value::as_str)
            .and_then(parse_timestamp);

        if let Some(t) = turn_time {
            if first_time.is_none() { first_time = Some(t); }
            last_time = Some(t);
        }

        // cwd + entrypoint (once)
        if project_path.is_empty() {
            if let Some(cwd) = val.get("cwd").and_then(Value::as_str) {
                project_path = cwd.to_string();
            }
        }
        if entrypoint.is_none() {
            if let Some(ep) = val.get("entrypoint").and_then(Value::as_str) {
                entrypoint = Some(ep.to_string());
            }
        }

        let entry_type = val.get("type").and_then(Value::as_str).unwrap_or("");

        // AI title
        if entry_type == "ai-title" {
            if let Some(t) = val.get("aiTitle").and_then(Value::as_str) {
                title = Some(t.to_string());
            }
        }

        // Assistant turns → tokens + cost
        if entry_type == "assistant" {
            if let Some(msg) = val.get("message") {
                // Model
                if let Some(m) = msg.get("model").and_then(Value::as_str) {
                    if !m.is_empty() { model = m.to_string(); }
                }

                if let Some(usage) = msg.get("usage") {
                    let inp  = usage.get("input_tokens")               .and_then(Value::as_u64).unwrap_or(0);
                    let out  = usage.get("output_tokens")              .and_then(Value::as_u64).unwrap_or(0);
                    let cr   = usage.get("cache_read_input_tokens")    .and_then(Value::as_u64).unwrap_or(0);
                    let cw   = usage.get("cache_creation_input_tokens").and_then(Value::as_u64).unwrap_or(0);

                    input_tokens       += inp;
                    output_tokens      += out;
                    cache_read_tokens  += cr;
                    cache_write_tokens += cw;

                    // Context window: last turn only
                    last_context_window = inp + cr + cw;

                    // Thinking tokens from content blocks
                    let turn_thinking: u64 = msg.get("content")
                        .and_then(Value::as_array)
                        .map(|blocks| {
                            blocks.iter()
                                .filter(|b| b.get("type").and_then(Value::as_str) == Some("thinking"))
                                .filter_map(|b| b.get("thinking").and_then(Value::as_str))
                                .map(|t| (t.len() as u64 / 4).max(1))
                                .sum()
                        })
                        .unwrap_or(0);
                    thinking_tokens += turn_thinking;

                    // Per-turn cost
                    let p = pricing_for(&model);
                    let turn_cost = (inp  as f64 * p.input
                                   + out  as f64 * p.output
                                   + cr   as f64 * p.cache_read
                                   + cw   as f64 * p.cache_write)
                                  / 1_000_000.0;
                    total_cost += turn_cost;

                    // Daily attribution
                    let day: NaiveDate = turn_time
                        .unwrap_or_else(Local::now)
                        .date_naive();
                    *daily_costs.entry(day).or_insert(0.0) += turn_cost;
                }
            }
        }
    }

    let start_time = first_time.unwrap_or_else(Local::now);
    let is_active_by_id = active_ids.contains(&session_id);
    let is_active = is_active_by_id || is_recent;
    let end_time = if is_active { None } else { last_time };

    // CLAUDE.md score (best-effort; ignore errors)
    let claudemd_score = if project_path.is_empty() {
        None
    } else {
        find_claudemd(&project_path)
    };

    Ok(ClaudeSession {
        id: session_id,
        project_path,
        start_time,
        end_time,
        total_cost,
        token_usage: TokenUsage {
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_write_tokens,
            thinking_tokens,
            context_window_tokens: last_context_window,
        },
        model,
        is_active,
        title,
        entrypoint,
        claudemd_score,
        daily_costs,
    })
}
