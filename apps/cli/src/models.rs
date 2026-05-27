use chrono::{DateTime, Local, NaiveDate};
use serde::Serialize;
use std::collections::HashMap;

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
