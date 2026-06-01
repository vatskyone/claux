mod account;
mod checkpoints;
mod commands;
mod config;
mod format;
mod metrics;
mod models;
mod monitor;
mod parser;
mod render;
mod skills;
mod spend;
mod tags;
mod usage;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};

use commands::checkpoint::CheckpointAction;
use commands::config::ConfigAction;
use commands::export::ExportFormat;
use commands::skills::SkillsAction;
use metrics::record_command;
use monitor::{load_sessions, SessionCache};

#[derive(Parser)]
#[command(
    name    = "claux",
    version = env!("CARGO_PKG_VERSION"),
    about   = "CLAUX — Claude Code session tracker",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum AnalyticsAction {
    /// Show local-only CLI usage metrics.
    Local {
        /// Reset local metrics after printing.
        #[arg(long)]
        reset: bool,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Show the currently active session (default).
    Status {
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// List recent sessions.
    Sessions {
        /// Maximum number of sessions to show.
        #[arg(long, short = 'n', default_value_t = 20)]
        limit: usize,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Show today / week / month spend summary.
    Spend {
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Show analytics or local metrics.
    Analytics {
        /// Analytics subcommands.
        #[command(subcommand)]
        action: Option<AnalyticsAction>,
        /// Number of days to include in the daily chart.
        #[arg(long, default_value_t = 30)]
        days: usize,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Export session history as JSON or CSV.
    Export {
        /// Output format (json or csv).
        #[arg(long, value_enum, default_value = "json")]
        format: ExportFormat,
        /// Write to this file instead of stdout.
        #[arg(long, short = 'o')]
        output: Option<String>,
        /// Maximum number of sessions to export.
        #[arg(long, short = 'n', default_value_t = 10000)]
        limit: usize,
    },

    /// Get or set a label on a session. Use a unique ID prefix to identify the session.
    Tag {
        /// Session ID prefix (unique prefix of the session ID shown in `claux sessions`).
        session: String,
        /// Label to assign (omit to show current tag).
        label: Option<String>,
        /// Remove the tag.
        #[arg(long, short = 'r')]
        remove: bool,
    },

    /// Show account, plan, and skill usage information.
    Account,

    /// Manage CLAUX skills (list, export, import, new).
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
    },

    /// Get or set CLAUX configuration values.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Inspect local environment, paths, and session parse health.
    Doctor {
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Save, list, load, or delete project checkpoints.
    Checkpoint {
        #[command(subcommand)]
        action: CheckpointAction,
    },

    /// Launch the live TUI dashboard (press q to quit, r to refresh).
    Tui,

    /// Print shell completion script (zsh, bash, fish).
    Completions { shell: Shell },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Status { json: false });
    let metric_name = metric_name(&command);

    let result = run_command(command);
    match &result {
        Ok(_) => record_command(metric_name, true, None),
        Err(_) => record_command(metric_name, false, Some("command_error")),
    }

    result
}

fn run_command(command: Commands) -> Result<()> {
    if let Commands::Completions { shell } = &command {
        let mut cmd = Cli::command();
        generate(*shell, &mut cmd, "claux", &mut std::io::stdout());
        return Ok(());
    }

    if let Commands::Tui = &command {
        return commands::tui::run();
    }

    if let Commands::Account = &command {
        return commands::account::run();
    }
    if let Commands::Skills { action } = &command {
        return commands::skills::run(action);
    }
    if let Commands::Config { action } = &command {
        return commands::config::run(action);
    }
    if let Commands::Checkpoint { action } = &command {
        return commands::checkpoint::run(action);
    }
    if let Commands::Doctor { json } = &command {
        return commands::doctor::run(*json);
    }

    let mut cache = SessionCache::new();
    let sessions = load_sessions(&mut cache);

    match command {
        Commands::Status { json } => commands::status::run(&sessions, json)?,
        Commands::Sessions { limit, json } => commands::sessions::run(&sessions, limit, json)?,
        Commands::Spend { json } => commands::spend::run(&sessions, json)?,
        Commands::Analytics { action, days, json } => match action {
            Some(AnalyticsAction::Local { reset, json }) => {
                commands::analytics::run_local_metrics(reset, json)?
            }
            None => commands::analytics::run(&sessions, days, json)?,
        },
        Commands::Export {
            format,
            output,
            limit,
        } => commands::export::run(&sessions, limit, format, output.as_deref())?,
        Commands::Tag {
            session,
            label,
            remove,
        } => commands::tag::run(&sessions, &session, label.as_deref(), remove)?,
        Commands::Completions { .. }
        | Commands::Tui
        | Commands::Account
        | Commands::Skills { .. }
        | Commands::Config { .. }
        | Commands::Checkpoint { .. }
        | Commands::Doctor { .. } => unreachable!(),
    }

    Ok(())
}

fn metric_name(command: &Commands) -> &'static str {
    match command {
        Commands::Status { .. } => "status",
        Commands::Sessions { .. } => "sessions",
        Commands::Spend { .. } => "spend",
        Commands::Analytics { .. } => "analytics",
        Commands::Export { .. } => "export",
        Commands::Tag { .. } => "tag",
        Commands::Account => "account",
        Commands::Skills { .. } => "skills",
        Commands::Config { .. } => "config",
        Commands::Doctor { .. } => "doctor",
        Commands::Checkpoint { .. } => "checkpoint",
        Commands::Tui => "tui",
        Commands::Completions { .. } => "completions",
    }
}
