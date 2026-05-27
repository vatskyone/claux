use chrono::{DateTime, Local, NaiveDate};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

// ── Token usage for one session ───────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize)]
pub struct TokenUsage {
    pub input_tokens:          u64,
    pub output_tokens:         u64,
    pub cache_read_tokens:     u64,
    pub cache_write_tokens:    u64,
    pub thinking_tokens:       u64,
    /// Tokens from the **most recent** assistant turn only (input + cache_read + cache_write).
    /// Used for context-window fill percentage.
    pub context_window_tokens: u64,
}

impl TokenUsage {
    /// Sum of all input-side tokens across the full session.
    pub fn total_context_tokens(&self) -> u64 {
        self.input_tokens + self.cache_read_tokens + self.cache_write_tokens
    }

    /// Cache-hit rate: cache_read / (input + cache_read + cache_write).
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.input_tokens + self.cache_read_tokens + self.cache_write_tokens;
        if total == 0 { 0.0 } else { self.cache_read_tokens as f64 / total as f64 }
    }
}

// ── Core session model ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ClaudeSession {
    pub id:             String,
    pub project_path:   String,
    pub start_time:     DateTime<Local>,
    pub end_time:       Option<DateTime<Local>>,
    pub total_cost:     f64,
    pub token_usage:    TokenUsage,
    pub model:          String,
    pub is_active:      bool,
    pub title:          Option<String>,
    pub entrypoint:     Option<String>,
    pub claudemd_score: Option<u8>,
    /// Per-calendar-day cost attribution (local midnight as key).
    #[serde(skip)]
    pub daily_costs:    HashMap<NaiveDate, f64>,
    /// Absolute path to the source JSONL file (used for agent parsing).
    #[serde(skip)]
    pub jsonl_path:     PathBuf,
}

impl ClaudeSession {
    /// Duration in seconds from start to end (or now if active).
    pub fn duration_secs(&self) -> i64 {
        let end = self.end_time.unwrap_or_else(Local::now);
        (end - self.start_time).num_seconds().max(0)
    }

    /// Burn rate in dollars per hour. Zero if session is < 60 s.
    pub fn burn_rate_per_hour(&self) -> f64 {
        let secs = self.duration_secs();
        if secs < 60 { return 0.0; }
        self.total_cost / (secs as f64 / 3600.0)
    }

    /// Rough 1-hour forward projection.
    #[allow(dead_code)]
    pub fn projected_cost(&self) -> f64 {
        self.total_cost + self.burn_rate_per_hour()
    }

    /// Context-window fill fraction (0.0–1.0). Denom is 200 000 tokens.
    pub fn context_health_fraction(&self) -> f64 {
        const MAX: u64 = 200_000;
        let fill = if self.token_usage.context_window_tokens > 0 {
            self.token_usage.context_window_tokens
        } else {
            self.token_usage.total_context_tokens()
        };
        (fill as f64 / MAX as f64).min(1.0)
    }

    /// Display path: /Users/foo/... → ~/...
    pub fn display_path(&self) -> String {
        crate::format::project_path(&self.project_path)
    }
}

// ── Sub-agent run ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AgentRun {
    /// The `tool_use.id` from the parent session's assistant turn.
    pub tool_use_id:    String,
    /// The short agentId from `sourceToolAssistantUUID` (used to find sub-agent JSONL).
    pub agent_id:       Option<String>,
    /// e.g. "Explore", "Plan", "general-purpose", "claude-code-guide"
    pub subagent_type:  String,
    /// One-line task summary from `input.description`.
    pub description:    String,
    /// Full prompt text from `input.prompt`.
    pub prompt:         String,
    /// Timestamp of the assistant turn that spawned the agent.
    pub start_time:     DateTime<Local>,
    /// Timestamp of the user turn that delivered the tool_result.
    pub end_time:       Option<DateTime<Local>>,
    /// Whether a matching tool_result was received.
    pub completed:      bool,
    /// First 250 chars of the tool_result content.
    pub output_preview: String,
    /// Accumulated from the sub-agent's JSONL file; zeroed if file not found.
    pub token_usage:    TokenUsage,
    pub total_cost:     f64,
    pub model:          Option<String>,
    /// Quality score 1–5 (computed from completion status and output richness).
    pub quality_score:  u8,
}

/// Compute a 1–5 quality score for an agent run.
///
/// - 1 = did not complete
/// - 2 = completed but output too short or contains error keywords
/// - 3 = moderate output (50–199 chars)
/// - 4 = good output (200–499 chars)
/// - 5 = rich output (≥ 500 chars)
pub fn compute_quality_score(completed: bool, output: &str) -> u8 {
    if !completed { return 1; }
    let low = output.to_lowercase();
    let has_error = low.contains("error:") || low.contains("failed") || low.contains("unable to");
    if has_error { return 2; }
    let len = output.len();
    if len < 50        { return 2; }
    if len < 200       { return 3; }
    if len < 500       { return 4; }
    5
}

/// Returns `(level, progress)` for display as an XP bar.
///
/// Level thresholds (total tasks of that type across all sessions):
/// - Lv.0 : 0 tasks
/// - Lv.1 : 1–4
/// - Lv.2 : 5–14
/// - Lv.3 : 15–29
/// - Lv.4 : 30–59
/// - Lv.5 : 60+
pub fn agent_level(total_tasks: usize) -> (u8, f64) {
    match total_tasks {
        0       => (0, 0.0),
        1..=4   => (1, (total_tasks - 1) as f64 / 4.0),
        5..=14  => (2, (total_tasks - 5) as f64 / 10.0),
        15..=29 => (3, (total_tasks - 15) as f64 / 15.0),
        30..=59 => (4, (total_tasks - 30) as f64 / 30.0),
        _       => (5, 1.0),
    }
}

// ── Aggregate / summary structs ───────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct SpendSummary {
    pub today:      f64,
    pub yesterday:  f64,
    pub this_week:  f64,
    pub prev_week:  f64,
    pub this_month: f64,
    pub prev_month: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailySpend {
    pub date: NaiveDate,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSpend {
    pub path:          String,
    pub display_path:  String,
    pub total_cost:    f64,
    pub session_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelSpend {
    pub model:         String,
    pub display_name:  String,
    pub total_cost:    f64,
    pub session_count: usize,
}
