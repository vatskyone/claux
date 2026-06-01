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
            rating: skill_rating(*count),
            content: None,
        });
    }

    // Walk ~/.claude/skills/ for custom (user-defined) skills
    if let Some(skills_dir) = dirs::home_dir().map(|h| h.join(".claude").join("skills")) {
        if skills_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&skills_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    let skill_md = path.join("SKILL.md");
                    if !skill_md.exists() {
                        continue;
                    }

                    let content = fs::read_to_string(&skill_md).ok();
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
                        rating: skill_rating(count),
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

/// 1–5 star rating based on cumulative usage count.
pub fn skill_rating(count: usize) -> u8 {
    match count {
        0 => 1,
        1..=2 => 2,
        3..=9 => 3,
        10..=29 => 4,
        _ => 5,
    }
}
