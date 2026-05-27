use chrono::{Duration, Local, NaiveDate};
use std::collections::HashMap;

use crate::models::{ClaudeSession, DailySpend, ModelSpend, ProjectSpend, SpendSummary};

/// Compute today / week / month spend buckets from per-turn daily cost attribution.
pub fn compute_spend(sessions: &[ClaudeSession]) -> SpendSummary {
    let today = Local::now().date_naive();

    let start_today      = today;
    let start_yesterday  = today - Duration::days(1);
    let start_this_week  = today - Duration::days(6);
    let start_prev_week  = today - Duration::days(13);
    let start_this_month = today - Duration::days(29);
    let start_prev_month = today - Duration::days(59);

    let mut s = SpendSummary::default();

    for session in sessions {
        for (&day, &cost) in &session.daily_costs {
            if day >= start_today {
                s.today      += cost;
            }
            if day == start_yesterday {
                s.yesterday  += cost;
            }
            if day >= start_this_week && day < start_today {
                // "this_week" = last 7 calendar days excluding today
                // The macOS app uses >= start_of_today - 7 days which includes today
                // Mirror that: this_week = sum of last 7 days including today
            }
            if day >= start_prev_week && day < start_this_week {
                s.prev_week  += cost;
            }
            if day >= start_prev_month && day < start_this_month {
                s.prev_month += cost;
            }
        }
    }

    // Re-compute week/month to match Swift logic (>= cutoff, no upper bound)
    s.this_week  = 0.0;
    s.this_month = 0.0;
    for session in sessions {
        for (&day, &cost) in &session.daily_costs {
            if day >= start_this_week  { s.this_week  += cost; }
            if day >= start_this_month { s.this_month += cost; }
        }
    }
    // Yesterday: full previous calendar day
    s.yesterday  = sessions.iter()
        .flat_map(|s| s.daily_costs.iter())
        .filter(|(&d, _)| d == start_yesterday)
        .map(|(_, &c)| c)
        .sum();
    // prev_week: 7 days before the this_week window
    s.prev_week = sessions.iter()
        .flat_map(|s| s.daily_costs.iter())
        .filter(|(&d, _)| d >= start_prev_week && d < start_this_week)
        .map(|(_, &c)| c)
        .sum();
    // prev_month: 30 days before the this_month window
    s.prev_month = sessions.iter()
        .flat_map(|s| s.daily_costs.iter())
        .filter(|(&d, _)| d >= start_prev_month && d < start_this_month)
        .map(|(_, &c)| c)
        .sum();

    s
}

/// Build a 30-day daily spend array (including zero-cost days).
pub fn compute_daily_spend(sessions: &[ClaudeSession]) -> Vec<DailySpend> {
    let today  = Local::now().date_naive();
    let cutoff = today - Duration::days(29);

    let mut map: HashMap<NaiveDate, f64> = HashMap::new();
    for session in sessions {
        for (&day, &cost) in &session.daily_costs {
            if day >= cutoff {
                *map.entry(day).or_insert(0.0) += cost;
            }
        }
    }

    let mut result: Vec<DailySpend> = (0..30)
        .map(|i| {
            let day = cutoff + Duration::days(i);
            DailySpend { date: day, cost: *map.get(&day).unwrap_or(&0.0) }
        })
        .collect();

    result.sort_by_key(|d| d.date);
    result
}

/// Project breakdown sorted by total cost desc.
pub fn compute_project_breakdown(sessions: &[ClaudeSession]) -> Vec<ProjectSpend> {
    let mut map: HashMap<String, (f64, usize)> = HashMap::new();
    for s in sessions {
        let e = map.entry(s.project_path.clone()).or_insert((0.0, 0));
        e.0 += s.total_cost;
        e.1 += 1;
    }
    let mut result: Vec<ProjectSpend> = map.into_iter()
        .map(|(path, (cost, count))| ProjectSpend {
            display_path:  crate::format::project_path(&path),
            path,
            total_cost:    cost,
            session_count: count,
        })
        .collect();
    result.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap());
    result
}

/// Model breakdown sorted by total cost desc.
pub fn compute_model_breakdown(sessions: &[ClaudeSession]) -> Vec<ModelSpend> {
    let mut map: HashMap<String, (f64, usize)> = HashMap::new();
    for s in sessions {
        let e = map.entry(s.model.clone()).or_insert((0.0, 0));
        e.0 += s.total_cost;
        e.1 += 1;
    }
    let mut result: Vec<ModelSpend> = map.into_iter()
        .map(|(model, (cost, count))| ModelSpend {
            display_name:  crate::format::model_short_name(&model),
            model,
            total_cost:    cost,
            session_count: count,
        })
        .collect();
    result.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap());
    result
}
