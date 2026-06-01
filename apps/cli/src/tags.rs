use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn tags_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("claux").join("tags.json"))
}

/// Load all session tags from disk. Returns an empty map on any error.
pub fn load_tags() -> HashMap<String, String> {
    let path = match tags_path() {
        Some(p) => p,
        None => return HashMap::new(),
    };
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// Set a tag for a session ID. Pass an empty string to clear.
pub fn save_tag(session_id: &str, tag: &str) -> Result<()> {
    let path = tags_path().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut tags = load_tags();
    if tag.trim().is_empty() {
        tags.remove(session_id);
    } else {
        tags.insert(session_id.to_string(), tag.trim().to_string());
    }
    fs::write(&path, serde_json::to_string_pretty(&tags)?)?;
    Ok(())
}
