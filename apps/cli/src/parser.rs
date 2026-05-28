use anyhow::Result;
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::models::{AgentRun, ClaudeSession, ClaudemdAnalysis, TokenUsage, compute_quality_score};

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
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Local));
    }
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

    let length_score: u32 = if words >= 300 { 30 }
        else if words >= 150 { 23 }
        else if words >= 80  { 16 }
        else if words >= 30  {  8 }
        else                 {  0 };

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

/// Detailed CLAUDE.md analysis — same scoring as `score_claudemd` but returns a struct.
pub fn score_claudemd_detailed(content: &str) -> ClaudemdAnalysis {
    let word_count: usize = content.split_whitespace().count();

    let mut heading_count = 0usize;
    let mut code_delimiters = 0u32;
    let mut bullets = 0u32;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with('#')   { heading_count += 1; }
        if t.starts_with("```") { code_delimiters += 1; }
        if t.starts_with("- ") || t.starts_with("* ") { bullets += 1; }
    }

    let lower = content.to_lowercase();
    let has_build       = ["build", "compile", "swift build", "npm run", "yarn", "make ", "gradle", "cmake"]
        .iter().any(|kw| lower.contains(kw));
    let has_tests       = ["test", "pytest", "jest ", "xcode test", "unit test", "spec"]
        .iter().any(|kw| lower.contains(kw));
    let has_run         = ["run ", "start ", "launch", "execute", "serve"]
        .iter().any(|kw| lower.contains(kw));
    let has_structure   = ["structure", "architecture", "layout", "directory", "folder", "project"]
        .iter().any(|kw| lower.contains(kw));
    let has_conventions = ["convention", "style guide", "pattern", "naming", "format", "lint"]
        .iter().any(|kw| lower.contains(kw));
    let has_important   = ["important", "note:", "warning", "do not", "never ", "always ", "avoid"]
        .iter().any(|kw| lower.contains(kw));
    let has_commands    = ["command", "script", "bash", "shell", "cli"]
        .iter().any(|kw| lower.contains(kw));
    let has_workflow    = ["workflow", "process", "step", "instruction", "guideline"]
        .iter().any(|kw| lower.contains(kw));

    let length_score: u32 = if word_count >= 300 { 30 }
        else if word_count >= 150 { 23 }
        else if word_count >= 80  { 16 }
        else if word_count >= 30  {  8 }
        else                      {  0 };

    let structure_score = (heading_count as u32 * 5).min(15)
        + ((code_delimiters / 2) * 5).min(10)
        + (bullets / 4).min(5);

    let content_score: u32 = [
        has_build, has_tests, has_run, has_structure,
        has_conventions, has_important, has_commands, has_workflow,
    ].iter().map(|&b| if b { 5u32 } else { 0 }).sum();

    let score = (length_score + structure_score + content_score).min(100) as u8;

    // Build suggestions (most impactful first, cap at 4)
    let mut suggestions: Vec<&'static str> = Vec::new();
    if !has_build       { suggestions.push("Add build/compile commands"); }
    if !has_tests       { suggestions.push("Add test instructions"); }
    if !has_run         { suggestions.push("Add run/launch instructions"); }
    if !has_commands    { suggestions.push("Add common command examples"); }
    if !has_structure   { suggestions.push("Describe project structure"); }
    if !has_conventions { suggestions.push("Document coding conventions"); }
    if word_count < 80  { suggestions.push("Expand content (too brief)"); }
    if heading_count == 0 { suggestions.push("Add section headings (#)"); }
    suggestions.truncate(4);

    ClaudemdAnalysis {
        score,
        word_count,
        heading_count,
        has_build,
        has_tests,
        has_run,
        has_structure,
        has_conventions,
        has_workflow,
        has_commands,
        has_important,
        suggestions,
    }
}

/// Walk up from `project_path` (same logic as `find_claudemd`) but return the path.
pub fn find_claudemd_path(project_path: &str) -> Option<std::path::PathBuf> {
    let start = Path::new(project_path);
    let home = dirs::home_dir();

    let mut dir = start.to_path_buf();
    for _ in 0..8 {
        let candidate = dir.join("CLAUDE.md");
        if candidate.exists() {
            if let Ok(meta) = fs::metadata(&candidate) {
                if meta.len() >= 10 { return Some(candidate); }
            }
        }
        if home.as_deref() == Some(&dir) { break; }
        if !dir.pop() { break; }
    }

    let mut queue: Vec<(std::path::PathBuf, u32)> = vec![(start.to_path_buf(), 0)];
    while let Some((cur, depth)) = queue.first().cloned() {
        queue.remove(0);
        if depth > 4 { continue; }
        let candidate = cur.join("CLAUDE.md");
        if candidate.exists() {
            if let Ok(meta) = fs::metadata(&candidate) {
                if meta.len() >= 10 { return Some(candidate); }
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

fn find_claudemd(project_path: &str) -> Option<u8> {
    let start = Path::new(project_path);

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

        let turn_time = val.get("timestamp")
            .and_then(Value::as_str)
            .and_then(parse_timestamp);

        if let Some(t) = turn_time {
            if first_time.is_none() { first_time = Some(t); }
            last_time = Some(t);
        }

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

        if entry_type == "ai-title" {
            if let Some(t) = val.get("aiTitle").and_then(Value::as_str) {
                title = Some(t.to_string());
            }
        }

        if entry_type == "assistant" {
            if let Some(msg) = val.get("message") {
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

                    last_context_window = inp + cr + cw;

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

                    let p = pricing_for(&model);
                    let turn_cost = (inp  as f64 * p.input
                                   + out  as f64 * p.output
                                   + cr   as f64 * p.cache_read
                                   + cw   as f64 * p.cache_write)
                                  / 1_000_000.0;
                    total_cost += turn_cost;

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
        jsonl_path: path.to_path_buf(),
        tag: None,
    })
}

// ── Agent parsing ─────────────────────────────────────────────────────────────

/// Extract text from a polymorphic `tool_result` content field.
/// Claude Code writes it as either a bare String or [{type:"text", text:"..."}].
fn extract_tool_result_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr.iter()
            .filter(|i| i.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|i| i.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// Internal accumulator while scanning JSONL for a pending (not-yet-completed) agent.
struct PendingAgent {
    subagent_type: String,
    description:   String,
    prompt:        String,
    start_time:    DateTime<Local>,
}

/// Parse all sub-agent tool calls from a session's JSONL file.
///
/// For each Agent tool_use + matching tool_result pair, builds an `AgentRun`.
/// Also attempts to read the sub-agent's own JSONL file for token usage.
pub fn parse_agents(session_path: &Path) -> Vec<AgentRun> {
    let Ok(content) = fs::read_to_string(session_path) else { return vec![] };

    // Directory containing the session file — used to find subagents/
    let project_dir = session_path.parent().unwrap_or(Path::new("."));

    let mut pending: HashMap<String, PendingAgent> = HashMap::new();
    let mut completed: Vec<AgentRun> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let Ok(val): Result<Value, _> = serde_json::from_str(line) else { continue };

        let turn_time = val.get("timestamp")
            .and_then(Value::as_str)
            .and_then(parse_timestamp);

        let entry_type = val.get("type").and_then(Value::as_str).unwrap_or("");

        // ── Assistant turn: collect Agent tool_use calls ──────────────────
        if entry_type == "assistant" {
            let Some(msg) = val.get("message") else { continue };
            let Some(blocks) = msg.get("content").and_then(Value::as_array) else { continue };

            for block in blocks {
                let block_type = block.get("type").and_then(Value::as_str).unwrap_or("");
                let block_name = block.get("name").and_then(Value::as_str).unwrap_or("");
                if block_type != "tool_use" || block_name != "Agent" { continue; }

                let tool_use_id = block.get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if tool_use_id.is_empty() { continue; }

                let input = block.get("input").cloned().unwrap_or(Value::Null);
                let subagent_type = input.get("subagent_type")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();
                let description = input.get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let prompt = input.get("prompt")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();

                pending.insert(tool_use_id, PendingAgent {
                    subagent_type,
                    description,
                    prompt,
                    start_time: turn_time.unwrap_or_else(Local::now),
                });
            }
        }

        // ── User turn: collect tool_result completions ────────────────────
        if entry_type == "user" {
            // sourceToolAssistantUUID links back to the sub-agent file
            let source_uuid = val.get("sourceToolAssistantUUID")
                .and_then(Value::as_str)
                .map(|s| s.to_string());

            let Some(msg) = val.get("message") else { continue };
            let Some(blocks) = msg.get("content").and_then(Value::as_array) else { continue };

            for block in blocks {
                let block_type = block.get("type").and_then(Value::as_str).unwrap_or("");
                if block_type != "tool_result" { continue; }

                let tool_use_id = block.get("tool_use_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();

                let Some(pending_agent) = pending.remove(&tool_use_id) else { continue };

                let raw_content = block.get("content").cloned().unwrap_or(Value::Null);
                let full_text   = extract_tool_result_text(&raw_content);
                let preview     = full_text.chars().take(250).collect::<String>();

                let quality = compute_quality_score(true, &full_text);

                completed.push(AgentRun {
                    tool_use_id,
                    agent_id:       source_uuid.clone(),
                    subagent_type:  pending_agent.subagent_type,
                    description:    pending_agent.description,
                    prompt:         pending_agent.prompt,
                    start_time:     pending_agent.start_time,
                    end_time:       Some(turn_time.unwrap_or_else(Local::now)),
                    completed:      true,
                    output_preview: preview,
                    token_usage:    TokenUsage::default(),
                    total_cost:     0.0,
                    model:          None,
                    quality_score:  quality,
                });
            }
        }
    }

    // Flush still-pending (running) agents as incomplete
    for (tool_use_id, pa) in pending {
        completed.push(AgentRun {
            tool_use_id,
            agent_id:       None,
            subagent_type:  pa.subagent_type,
            description:    pa.description,
            prompt:         pa.prompt,
            start_time:     pa.start_time,
            end_time:       None,
            completed:      false,
            output_preview: String::new(),
            token_usage:    TokenUsage::default(),
            total_cost:     0.0,
            model:          None,
            quality_score:  1,
        });
    }

    // Enrich with token usage from sub-agent JSONL files
    for agent in &mut completed {
        let Some(ref agent_id) = agent.agent_id else { continue };
        let sub_path = project_dir
            .join("subagents")
            .join(format!("agent-{}.jsonl", agent_id));
        if !sub_path.exists() { continue; }

        if let Some((usage, cost, m)) = parse_subagent_usage(&sub_path) {
            agent.token_usage = usage;
            agent.total_cost  = cost;
            agent.model       = m;
        }
    }

    // Sort chronologically
    completed.sort_by_key(|a| a.start_time);
    completed
}

/// Read a sub-agent JSONL and return `(TokenUsage, total_cost, Option<model>)`.
fn parse_subagent_usage(path: &Path) -> Option<(TokenUsage, f64, Option<String>)> {
    let content = fs::read_to_string(path).ok()?;

    let mut model = String::from("claude-sonnet-4-6");
    let mut input_tokens:       u64 = 0;
    let mut output_tokens:      u64 = 0;
    let mut cache_read_tokens:  u64 = 0;
    let mut cache_write_tokens: u64 = 0;
    let mut thinking_tokens:    u64 = 0;
    let mut total_cost = 0.0f64;
    let mut found_any = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let Ok(val): Result<Value, _> = serde_json::from_str(line) else { continue };

        if val.get("type").and_then(Value::as_str) != Some("assistant") { continue; }
        let Some(msg) = val.get("message") else { continue };

        if let Some(m) = msg.get("model").and_then(Value::as_str) {
            if !m.is_empty() { model = m.to_string(); }
        }

        if let Some(usage) = msg.get("usage") {
            let inp = usage.get("input_tokens")               .and_then(Value::as_u64).unwrap_or(0);
            let out = usage.get("output_tokens")              .and_then(Value::as_u64).unwrap_or(0);
            let cr  = usage.get("cache_read_input_tokens")    .and_then(Value::as_u64).unwrap_or(0);
            let cw  = usage.get("cache_creation_input_tokens").and_then(Value::as_u64).unwrap_or(0);

            input_tokens       += inp;
            output_tokens      += out;
            cache_read_tokens  += cr;
            cache_write_tokens += cw;

            let think: u64 = msg.get("content")
                .and_then(Value::as_array)
                .map(|blocks| {
                    blocks.iter()
                        .filter(|b| b.get("type").and_then(Value::as_str) == Some("thinking"))
                        .filter_map(|b| b.get("thinking").and_then(Value::as_str))
                        .map(|t| (t.len() as u64 / 4).max(1))
                        .sum()
                })
                .unwrap_or(0);
            thinking_tokens += think;

            let p = pricing_for(&model);
            total_cost += (inp as f64 * p.input
                         + out as f64 * p.output
                         + cr  as f64 * p.cache_read
                         + cw  as f64 * p.cache_write)
                        / 1_000_000.0;
            found_any = true;
        }
    }

    if !found_any { return None; }

    Some((
        TokenUsage {
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_write_tokens,
            thinking_tokens,
            context_window_tokens: 0,
        },
        total_cost,
        Some(model),
    ))
}
