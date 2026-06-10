use std::fs;

use crate::account;
use crate::models::{SkillInfo, SkillSource};

pub fn load_skills() -> Vec<SkillInfo> {
    let usage = account::load_skill_usage();
    let mut skills: Vec<SkillInfo> = Vec::new();

    // Seed from skillUsage (covers all known skills including builtins)
    for (name, (count, last)) in &usage {
        skills.push(SkillInfo {
            name: name.clone(),
            source: SkillSource::Builtin,
            description: None,
            usage_count: *count,
            last_used_ms: *last,
            score: skill_score(*count, *last),
            rating: skill_rating(*count, *last),
            content: None,
        });
    }

    // Walk ~/.claude/skills/ for custom (user-defined) skills.
    // Supports both flat files (<name>.md) and subdirectories (<name>/SKILL.md).
    if let Some(skills_dir) = dirs::home_dir().map(|h| h.join(".claude").join("skills")) {
        if skills_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&skills_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let (name, content) = if path.is_dir() {
                        let skill_md = path.join("SKILL.md");
                        if !skill_md.exists() {
                            continue;
                        }
                        let name = entry.file_name().to_string_lossy().to_string();
                        (name, fs::read_to_string(&skill_md).ok())
                    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        let name = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if name.is_empty() {
                            continue;
                        }
                        (name, fs::read_to_string(&path).ok())
                    } else {
                        continue;
                    };

                    let description = content
                        .as_deref()
                        .and_then(|c| {
                            c.lines()
                                .find(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                        })
                        .or_else(|| {
                            content
                                .as_deref()
                                .and_then(|c| c.lines().find(|l| !l.trim().is_empty()))
                                .map(|l| l.trim_start_matches('#').trim())
                        })
                        .map(|s| s.to_string());

                    let (count, last) = usage.get(&name).copied().unwrap_or((0, None));

                    // Custom entry supersedes any builtin seed with the same name
                    skills.retain(|s| s.name != name);
                    skills.push(SkillInfo {
                        name,
                        source: SkillSource::Custom,
                        description,
                        usage_count: count,
                        last_used_ms: last,
                        score: skill_score(count, last),
                        rating: skill_rating(count, last),
                        content,
                    });
                }
            }
        }
    }

    // Sort by usage desc, then alphabetical
    skills.sort_by(|a, b| b.usage_count.cmp(&a.usage_count).then(a.name.cmp(&b.name)));
    skills
}

/// 0–100 composite score: 60 pts for usage tier + 40 pts for recency.
pub fn skill_score(count: usize, last_used_ms: Option<u64>) -> u8 {
    if count == 0 {
        return 0;
    }
    let usage_pts: u32 = match count {
        1..=2 => 15,
        3..=9 => 30,
        10..=29 => 45,
        30..=99 => 55,
        _ => 60,
    };
    let recency_pts: u32 = if let Some(last_ms) = last_used_ms {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let days = now_ms.saturating_sub(last_ms) / 86_400_000;
        match days {
            0..=6 => 40,
            7..=29 => 30,
            30..=89 => 20,
            90..=179 => 10,
            180..=364 => 5,
            _ => 0,
        }
    } else {
        0
    };
    (usage_pts + recency_pts).min(100) as u8
}

/// 0–5 star rating derived from the composite score.
pub fn skill_rating(count: usize, last_used_ms: Option<u64>) -> u8 {
    let score = skill_score(count, last_used_ms);
    if score == 0 {
        return 0;
    }
    ((score as f64 / 100.0 * 5.0).round() as u8).max(1).min(5)
}
