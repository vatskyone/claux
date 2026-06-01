use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── Token usage for one session ───────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub thinking_tokens: u64,
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
        if total == 0 {
            0.0
        } else {
            self.cache_read_tokens as f64 / total as f64
        }
    }
}

// ── Core session model ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ClaudeSession {
    pub id: String,
    pub project_path: String,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub total_cost: f64,
    pub token_usage: TokenUsage,
    pub model: String,
    pub is_active: bool,
    pub title: Option<String>,
    pub entrypoint: Option<String>,
    pub claudemd_score: Option<u8>,
    /// Per-calendar-day cost attribution (local midnight as key).
    #[serde(skip)]
    pub daily_costs: HashMap<NaiveDate, f64>,
    /// Absolute path to the source JSONL file (used for agent parsing).
    #[serde(skip)]
    pub jsonl_path: PathBuf,
    /// User-assigned tag label (loaded from ~/.claude/claux/tags.json).
    #[serde(skip)]
    pub tag: Option<String>,
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
        if secs < 60 {
            return 0.0;
        }
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
    pub tool_use_id: String,
    /// The short agentId from `sourceToolAssistantUUID` (used to find sub-agent JSONL).
    pub agent_id: Option<String>,
    /// e.g. "Explore", "Plan", "general-purpose", "claude-code-guide"
    pub subagent_type: String,
    /// One-line task summary from `input.description`.
    pub description: String,
    /// Full prompt text from `input.prompt`.
    pub prompt: String,
    /// Timestamp of the assistant turn that spawned the agent.
    pub start_time: DateTime<Local>,
    /// Timestamp of the user turn that delivered the tool_result.
    pub end_time: Option<DateTime<Local>>,
    /// Whether a matching tool_result was received.
    pub completed: bool,
    /// First 250 chars of the tool_result content.
    pub output_preview: String,
    /// Accumulated from the sub-agent's JSONL file; zeroed if file not found.
    pub token_usage: TokenUsage,
    pub total_cost: f64,
    pub model: Option<String>,
    /// Quality score 1–5 (computed from completion status and output richness).
    pub quality_score: u8,
}

/// Compute a 1–5 quality score for an agent run.
///
/// - 1 = did not complete
/// - 2 = completed but output too short or contains error keywords
/// - 3 = moderate output (50–199 chars)
/// - 4 = good output (200–499 chars)
/// - 5 = rich output (≥ 500 chars)
pub fn compute_quality_score(completed: bool, output: &str) -> u8 {
    if !completed {
        return 1;
    }
    let low = output.to_lowercase();
    let has_error = low.contains("error:") || low.contains("failed") || low.contains("unable to");
    if has_error {
        return 2;
    }
    let len = output.len();
    if len < 50 {
        return 2;
    }
    if len < 200 {
        return 3;
    }
    if len < 500 {
        return 4;
    }
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
        0 => (0, 0.0),
        1..=4 => (1, (total_tasks - 1) as f64 / 4.0),
        5..=14 => (2, (total_tasks - 5) as f64 / 10.0),
        15..=29 => (3, (total_tasks - 15) as f64 / 15.0),
        30..=59 => (4, (total_tasks - 30) as f64 / 30.0),
        _ => (5, 1.0),
    }
}

// ── Account / plan info (from ~/.claude.json) ─────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub display_name: String,
    pub email: String,
    pub plan_type: String,
    pub org_name: String,
    pub org_role: String,
    pub billing_type: String,
    pub account_created: String,
    pub sub_created: Option<String>,
    pub rate_limit_tier: String,
    pub has_extra_usage: bool,
}

// ── CLAUDE.md detailed analysis ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClaudemdAnalysis {
    pub score: u8,
    pub word_count: usize,
    pub heading_count: usize,
    pub has_build: bool,
    pub has_tests: bool,
    pub has_run: bool,
    pub has_structure: bool,
    pub has_conventions: bool,
    pub has_workflow: bool,
    pub has_commands: bool,
    pub has_important: bool,
    pub suggestions: Vec<&'static str>,
}

// ── Skill info ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SkillSource {
    Builtin,
    Custom,
}

#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub source: SkillSource,
    pub description: Option<String>,
    pub usage_count: usize,
    pub last_used_ms: Option<u64>,
    pub rating: u8,
    pub content: Option<String>,
}

// ── User-configurable budget limits ──────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClauxConfig {
    pub projects_root: Option<String>,
    pub sessions_root: Option<String>,
    pub weekly_budget_usd: Option<f64>,
    pub plan_5h_limit_usd: Option<f64>,
    pub monthly_credit_usd: Option<f64>,
}

// ── Session checkpoint (save/load context) ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// 8-char random hex ID.
    pub id: String,
    /// User-provided label.
    pub name: String,
    /// RFC3339 creation timestamp.
    pub created_at: String,
    pub project_path: String,
    pub session_id: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    /// Lifetime project cost (sum of all sessions) at save time.
    pub cost_total_usd: f64,
    /// Active session cost at save time.
    pub session_cost_usd: f64,
    pub total_sessions: usize,
    /// Files changed since the prior checkpoint's commit (via git diff).
    pub files_changed: Vec<String>,
    pub claudemd_score: Option<u8>,
    pub summary: String,
}

// ── Aggregate / summary structs ───────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct SpendSummary {
    pub today: f64,
    pub yesterday: f64,
    pub this_week: f64,
    pub prev_week: f64,
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
    pub path: String,
    pub display_path: String,
    pub total_cost: f64,
    pub session_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelSpend {
    pub model: String,
    pub display_name: String,
    pub total_cost: f64,
    pub session_count: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MonthlyForecast {
    /// Average daily spend over the last 7 days.
    pub avg_per_day_7d: f64,
    /// Cumulative spend since the 1st of the current calendar month.
    pub month_to_date: f64,
    /// Calendar days elapsed this month (including today).
    pub days_elapsed: u32,
    /// Calendar days remaining this month (excluding today).
    pub days_remaining: u32,
    /// Projected total for the current month at the 7-day pace.
    pub projected_eom: f64,
    /// Projected spend over 365 days at the 7-day pace.
    pub projected_annual: f64,
}
