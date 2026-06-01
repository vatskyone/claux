use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalMetrics {
    pub command_counts: HashMap<String, u64>,
    pub failure_counts: HashMap<String, u64>,
    pub empty_state_counts: HashMap<String, u64>,
    pub refresh_latency_buckets: HashMap<String, u64>,
    pub updated_at: Option<String>,
}

fn metrics_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("claux").join("local_metrics.json"))
}

fn save_local_metrics(metrics: &LocalMetrics) -> Result<()> {
    let path = metrics_path().ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(metrics)?)?;
    Ok(())
}

pub fn load_local_metrics() -> LocalMetrics {
    let path = match metrics_path() {
        Some(p) => p,
        None => return LocalMetrics::default(),
    };

    let Ok(content) = fs::read_to_string(path) else {
        return LocalMetrics::default();
    };

    serde_json::from_str(&content).unwrap_or_default()
}

pub fn reset_local_metrics() -> Result<()> {
    save_local_metrics(&LocalMetrics::default())
}

pub fn record_command(name: &str, success: bool, failure_class: Option<&str>) {
    let mut metrics = load_local_metrics();
    *metrics.command_counts.entry(name.to_string()).or_insert(0) += 1;
    if !success {
        let class = failure_class.unwrap_or("command_error");
        *metrics.failure_counts.entry(class.to_string()).or_insert(0) += 1;
    }
    metrics.updated_at = Some(Local::now().to_rfc3339());
    let _ = save_local_metrics(&metrics);
}

pub fn record_empty_state(reason: &str) {
    let mut metrics = load_local_metrics();
    *metrics
        .empty_state_counts
        .entry(reason.to_string())
        .or_insert(0) += 1;
    metrics.updated_at = Some(Local::now().to_rfc3339());
    let _ = save_local_metrics(&metrics);
}

pub fn record_refresh_latency(elapsed: Duration) {
    let mut metrics = load_local_metrics();
    let bucket = if elapsed.as_millis() < 50 {
        "lt_50ms"
    } else if elapsed.as_millis() < 150 {
        "50_149ms"
    } else if elapsed.as_millis() < 500 {
        "150_499ms"
    } else {
        "ge_500ms"
    };

    *metrics
        .refresh_latency_buckets
        .entry(bucket.to_string())
        .or_insert(0) += 1;
    metrics.updated_at = Some(Local::now().to_rfc3339());
    let _ = save_local_metrics(&metrics);
}
