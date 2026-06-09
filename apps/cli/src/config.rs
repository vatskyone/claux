use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::models::{ClauxConfig, RateLimitsSnapshot};

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("claux").join("config.json"))
}

fn default_projects_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("projects"))
}

fn default_sessions_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("sessions"))
}

pub fn projects_root_dir() -> PathBuf {
    let cfg = load_claux_config();
    cfg.projects_root
        .map(PathBuf::from)
        .or_else(default_projects_root)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn active_sessions_dir() -> PathBuf {
    let cfg = load_claux_config();
    cfg.sessions_root
        .map(PathBuf::from)
        .or_else(default_sessions_root)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn load_claux_config() -> ClauxConfig {
    let path = match config_path() {
        Some(p) => p,
        None => return ClauxConfig::default(),
    };
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return ClauxConfig::default(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn load_rate_limits() -> Option<RateLimitsSnapshot> {
    let path = dirs::home_dir()?.join(".claude").join("claux").join("rate_limits.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_claux_config(cfg: &ClauxConfig) -> Result<()> {
    let path = config_path().ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}
