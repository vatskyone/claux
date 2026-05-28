use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::models::ClauxConfig;

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("claux").join("config.json"))
}

pub fn load_claux_config() -> ClauxConfig {
    let path = match config_path() {
        Some(p) => p,
        None    => return ClauxConfig::default(),
    };
    let content = match fs::read_to_string(path) {
        Ok(c)  => c,
        Err(_) => return ClauxConfig::default(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_claux_config(cfg: &ClauxConfig) -> Result<()> {
    let path = config_path()
        .ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}
