use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike};

use crate::models::ClaudeSession;

#[derive(Debug, Clone, PartialEq)]
pub enum ProgressReason {
    NoActiveSession,
    NoDataYet,
    LimitUnset,
    SourceUnavailable,
}

#[derive(Debug, Clone)]
pub struct ProgressBarState {
    pub label: String,
    pub fraction: f64,
    pub current: f64,
    pub limit: Option<f64>,
    pub reason: Option<ProgressReason>,
    pub reset_at: Option<DateTime<Local>>,
}

pub fn five_hour_state(
    sessions: &[ClaudeSession],
    now: DateTime<Local>,
    limit: Option<f64>,
    has_active_session: bool,
) -> ProgressBarState {
    let current = compute_rolling_five_hour_spend(sessions, now);
    let reason = if sessions.is_empty() {
        Some(ProgressReason::SourceUnavailable)
    } else if !has_active_session {
        Some(ProgressReason::NoActiveSession)
    } else if current == 0.0 {
        Some(ProgressReason::NoDataYet)
    } else {
        None
    };

    match limit.filter(|l| *l > 0.0) {
        Some(cap) => ProgressBarState {
            label: "Last 5h".to_string(),
            fraction: safe_fraction(current, cap),
            current,
            limit: Some(cap),
            reason,
            reset_at: Some(next_five_hour_reset(now)),
        },
        None => ProgressBarState {
            label: "Last 5h".to_string(),
            fraction: 0.0,
            current,
            limit: None,
            reason: Some(ProgressReason::LimitUnset),
            reset_at: Some(next_five_hour_reset(now)),
        },
    }
}

pub fn weekly_state(
    sessions: &[ClaudeSession],
    now: DateTime<Local>,
    limit: Option<f64>,
) -> ProgressBarState {
    let today = now.date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let current: f64 = sessions
        .iter()
        .flat_map(|s| s.daily_costs.iter())
        .filter(|(d, _)| **d >= monday && **d <= today)
        .map(|(_, c)| c)
        .sum();

    let base_reason = if sessions.is_empty() {
        Some(ProgressReason::SourceUnavailable)
    } else if current == 0.0 {
        Some(ProgressReason::NoDataYet)
    } else {
        None
    };

    match limit.filter(|l| *l > 0.0) {
        Some(cap) => ProgressBarState {
            label: "This week".to_string(),
            fraction: safe_fraction(current, cap),
            current,
            limit: Some(cap),
            reason: base_reason,
            reset_at: Some(next_weekly_reset(now)),
        },
        None => ProgressBarState {
            label: "This week".to_string(),
            fraction: 0.0,
            current,
            limit: None,
            reason: Some(ProgressReason::LimitUnset),
            reset_at: Some(next_weekly_reset(now)),
        },
    }
}

pub fn safe_fraction(current: f64, limit: f64) -> f64 {
    if !current.is_finite() || !limit.is_finite() || limit <= 0.0 {
        return 0.0;
    }
    (current / limit).clamp(0.0, 1.0)
}

pub fn compute_rolling_five_hour_spend(sessions: &[ClaudeSession], now: DateTime<Local>) -> f64 {
    let window_start = now - Duration::hours(5);
    sessions
        .iter()
        .map(|s| {
            let start = s.start_time;
            let end = s.end_time.unwrap_or(now);
            if end <= window_start || start >= now {
                return 0.0;
            }
            let overlap_start = if start > window_start {
                start
            } else {
                window_start
            };
            let overlap_end = if end < now { end } else { now };
            let overlap_secs = (overlap_end - overlap_start).num_seconds().max(0) as f64;
            let session_secs = (end - start).num_seconds().max(1) as f64;
            (overlap_secs / session_secs) * s.total_cost
        })
        .sum()
}

pub fn next_five_hour_reset(now: DateTime<Local>) -> DateTime<Local> {
    let hour = now.hour();
    let next_bucket = ((hour / 5) + 1) * 5;
    if next_bucket < 24 {
        now.with_hour(next_bucket)
            .and_then(|dt| dt.with_minute(0))
            .and_then(|dt| dt.with_second(0))
            .and_then(|dt| dt.with_nanosecond(0))
            .unwrap_or(now + Duration::hours(1))
    } else {
        let tomorrow = (now + Duration::days(1)).date_naive();
        Local
            .with_ymd_and_hms(tomorrow.year(), tomorrow.month(), tomorrow.day(), 0, 0, 0)
            .single()
            .unwrap_or(now + Duration::hours(1))
    }
}

pub fn next_weekly_reset(now: DateTime<Local>) -> DateTime<Local> {
    let today = now.date_naive();
    let days_since_mon = today.weekday().num_days_from_monday() as i64;
    let next_monday = today - Duration::days(days_since_mon) + Duration::days(7);
    Local
        .with_ymd_and_hms(
            next_monday.year(),
            next_monday.month(),
            next_monday.day(),
            0,
            0,
            0,
        )
        .single()
        .unwrap_or(now + Duration::days(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ClaudeSession, TokenUsage};
    use chrono::TimeZone;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn mk_session(start: DateTime<Local>, end: DateTime<Local>, total_cost: f64) -> ClaudeSession {
        ClaudeSession {
            id: "s1".to_string(),
            project_path: "/tmp/p".to_string(),
            start_time: start,
            end_time: Some(end),
            total_cost,
            token_usage: TokenUsage::default(),
            model: "claude-sonnet-4-6".to_string(),
            is_active: false,
            title: None,
            entrypoint: None,
            claudemd_score: None,
            daily_costs: HashMap::new(),
            jsonl_path: PathBuf::from("/tmp/s1.jsonl"),
            tag: None,
        }
    }

    #[test]
    fn safe_fraction_clamps_values() {
        assert_eq!(safe_fraction(2.0, 4.0), 0.5);
        assert_eq!(safe_fraction(8.0, 4.0), 1.0);
        assert_eq!(safe_fraction(f64::NAN, 4.0), 0.0);
        assert_eq!(safe_fraction(2.0, 0.0), 0.0);
    }

    #[test]
    fn five_hour_spend_uses_overlap() {
        let now = Local.with_ymd_and_hms(2026, 6, 1, 10, 0, 0).unwrap();
        let start = Local.with_ymd_and_hms(2026, 6, 1, 6, 0, 0).unwrap();
        let end = Local.with_ymd_and_hms(2026, 6, 1, 10, 0, 0).unwrap();
        let s = mk_session(start, end, 10.0);
        let spend = compute_rolling_five_hour_spend(&[s], now);
        assert!((spend - 10.0).abs() < 1e-6);
    }

    #[test]
    fn next_reset_is_5_hour_boundary() {
        let now = Local.with_ymd_and_hms(2026, 6, 1, 13, 20, 0).unwrap();
        let next = next_five_hour_reset(now);
        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 0);
    }
}
