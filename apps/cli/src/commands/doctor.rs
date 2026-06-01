use anyhow::Result;
use serde::Serialize;
use serde_json::json;
use std::fs;

use crate::config::{active_sessions_dir, projects_root_dir};
use crate::metrics::record_empty_state;
use crate::monitor::{find_jsonl_files, load_active_ids};
use crate::parser::parse_session;

#[derive(Debug, Serialize)]
struct ParseHealth {
    discovered_jsonl: usize,
    parsed_ok: usize,
    parsed_failed: usize,
}

pub fn run(json_output: bool) -> Result<()> {
    let projects_dir = projects_root_dir();
    let sessions_dir = active_sessions_dir();

    let projects_exists = projects_dir.exists();
    let sessions_exists = sessions_dir.exists();

    let files = find_jsonl_files();
    let active_ids = load_active_ids();

    let mut parsed_ok = 0usize;
    let mut parsed_failed = 0usize;

    for file in &files {
        let meta = match fs::metadata(file) {
            Ok(m) => m,
            Err(_) => {
                parsed_failed += 1;
                continue;
            }
        };
        let mtime = match meta.modified() {
            Ok(m) => m,
            Err(_) => {
                parsed_failed += 1;
                continue;
            }
        };

        let is_recent = std::time::SystemTime::now()
            .duration_since(mtime)
            .map(|d| d < std::time::Duration::from_secs(90))
            .unwrap_or(false);

        match parse_session(file, &active_ids, is_recent) {
            Ok(_) => parsed_ok += 1,
            Err(_) => parsed_failed += 1,
        }
    }

    let parse_health = ParseHealth {
        discovered_jsonl: files.len(),
        parsed_ok,
        parsed_failed,
    };

    if files.is_empty() {
        if !projects_exists {
            record_empty_state("source_unavailable");
        } else {
            record_empty_state("no_data_yet");
        }
    }

    if json_output {
        let out = json!({
            "projects_dir": projects_dir,
            "projects_dir_exists": projects_exists,
            "sessions_dir": sessions_dir,
            "sessions_dir_exists": sessions_exists,
            "active_session_ids": active_ids.len(),
            "parse_health": parse_health,
            "recommendations": recommendations(projects_exists, sessions_exists, &parse_health),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("CLAUX doctor");
    println!("  projects dir : {}", projects_dir.display());
    println!("  sessions dir : {}", sessions_dir.display());
    println!(
        "  projects dir : {}",
        if projects_exists { "ok" } else { "missing" }
    );
    println!(
        "  sessions dir : {}",
        if sessions_exists { "ok" } else { "missing" }
    );
    println!("  active ids   : {}", active_ids.len());
    println!(
        "  parse health : {} ok / {} failed / {} files",
        parse_health.parsed_ok, parse_health.parsed_failed, parse_health.discovered_jsonl
    );

    let recs = recommendations(projects_exists, sessions_exists, &parse_health);
    if recs.is_empty() {
        println!("  status       : healthy");
    } else {
        println!("  status       : needs attention");
        for rec in recs {
            println!("  - {}", rec);
        }
    }

    Ok(())
}

fn recommendations(
    projects_exists: bool,
    sessions_exists: bool,
    parse: &ParseHealth,
) -> Vec<&'static str> {
    let mut recs = Vec::new();
    if !projects_exists {
        recs.push("Session source unavailable. Run: claux config init (or set projects-root).");
    }
    if !sessions_exists {
        recs.push(
            "Active session source unavailable. Run: claux config init (or set sessions-root).",
        );
    }
    if parse.discovered_jsonl == 0 {
        recs.push(
            "No session logs found yet. Start a Claude/Codex session, then rerun claux doctor.",
        );
    }
    if parse.parsed_failed > 0 {
        recs.push(
            "Some log files could not be parsed. Check malformed JSONL files under projects-root.",
        );
    }
    recs
}
