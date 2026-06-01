use chrono::{DateTime, Local};

/// "$1.24"
pub fn cost(v: f64) -> String {
    format!("${:.2}", v)
}

/// "1.2M", "42.3K", or bare integer.
pub fn tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// "2h 15m" or "45m".
pub fn duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{}h {:02}m", h, m)
    } else {
        format!("{}m", m)
    }
}

/// "5m ago", "3h ago", "5d ago".
pub fn relative_time(dt: &DateTime<Local>) -> String {
    let s = (Local::now() - *dt).num_seconds().max(0);
    if s < 3600 {
        format!("{}m ago", s / 60)
    } else if s < 86400 {
        format!("{}h ago", s / 3600)
    } else {
        format!("{}d ago", s / 86400)
    }
}

/// "Sonnet 4.6", "Opus 3", "Haiku 4.5", etc.
///
/// Handles both naming schemes Claude has used:
///   New: `claude-sonnet-4-6`         → "Sonnet 4.6"
///   New: `claude-haiku-4-5-20251001` → "Haiku 4.5"
///   Old: `claude-3-5-sonnet-20241022`→ "Sonnet 3.5"
///   Old: `claude-3-opus-20240229`    → "Opus 3"
pub fn model_short_name(model: &str) -> String {
    let lower = model.to_lowercase();
    let family = if lower.contains("opus") {
        "Opus"
    } else if lower.contains("sonnet") {
        "Sonnet"
    } else if lower.contains("haiku") {
        "Haiku"
    } else {
        return model.to_string();
    };

    let parts: Vec<&str> = lower.split('-').collect();
    let family_lower = family.to_lowercase();
    let Some(family_idx) = parts.iter().position(|&p| p == family_lower) else {
        return family.to_string();
    };

    // Version numbers are 1–2 digit integers; dates like 20241022 are > 99.
    let is_version =
        |s: &str| -> Option<u32> { s.parse::<u32>().ok().filter(|&n| n > 0 && n <= 99) };

    // New format: version comes AFTER family (claude-sonnet-4-6)
    let after: Vec<u32> = parts[family_idx + 1..]
        .iter()
        .take(3)
        .filter_map(|s| is_version(s))
        .collect();
    if !after.is_empty() {
        return if after.len() >= 2 {
            format!("{} {}.{}", family, after[0], after[1])
        } else {
            format!("{} {}", family, after[0])
        };
    }

    // Old format: version comes BEFORE family (claude-3-5-sonnet-20241022)
    let before: Vec<u32> = parts[1..family_idx]
        .iter()
        .filter_map(|s| is_version(s))
        .collect();
    if !before.is_empty() {
        return if before.len() >= 2 {
            format!("{} {}.{}", family, before[0], before[1])
        } else {
            format!("{} {}", family, before[0])
        };
    }

    family.to_string()
}

/// /Users/foo/bar → ~/bar
pub fn project_path(raw: &str) -> String {
    let parts: Vec<&str> = raw.splitn(5, '/').collect();
    // ["", "Users", "<name>", "rest"]  →  "~/rest"
    if parts.len() >= 4 && parts[1] == "Users" {
        let rest = parts[3..].join("/");
        if rest.is_empty() {
            "~".to_string()
        } else {
            format!("~/{}", rest)
        }
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_short_name() {
        assert_eq!(model_short_name("claude-sonnet-4-6"), "Sonnet 4.6");
        assert_eq!(model_short_name("claude-haiku-4-5-20251001"), "Haiku 4.5");
        assert_eq!(model_short_name("claude-3-5-sonnet-20241022"), "Sonnet 3.5");
        assert_eq!(model_short_name("claude-3-opus-20240229"), "Opus 3");
    }

    #[test]
    fn test_tokens() {
        assert_eq!(tokens(0), "0");
        assert_eq!(tokens(999), "999");
        assert_eq!(tokens(1_000), "1.0K");
        assert_eq!(tokens(42_300), "42.3K");
        assert_eq!(tokens(1_200_000), "1.2M");
    }

    #[test]
    fn test_duration() {
        assert_eq!(duration(45 * 60), "45m");
        assert_eq!(duration(2 * 3600 + 15 * 60), "2h 15m");
    }

    #[test]
    fn test_project_path() {
        assert_eq!(project_path("/Users/snow/Desktop/CLAUX"), "~/Desktop/CLAUX");
        assert_eq!(project_path("/tmp/foo"), "/tmp/foo");
    }
}
