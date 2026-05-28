mod account;
mod commands;
mod config;
mod format;
mod models;
mod monitor;
mod parser;
mod render;
mod skills;
mod spend;
mod tags;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

use commands::export::ExportFormat;
use commands::skills::SkillsAction;
use commands::config::ConfigAction;
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

    /// Show analytics: daily spend, by project, by model.
    Analytics {
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
    ///
    /// Examples:
    ///   claux tag abc123          # show current tag
    ///   claux tag abc123 "work"   # set tag
    ///   claux tag abc123 -r       # remove tag
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
    ///
    /// Keys: weekly-budget (USD), monthly-credit (USD)
    /// Examples:
    ///   claux config set weekly-budget 50
    ///   claux config get weekly-budget
    ///   claux config unset weekly-budget
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Launch the live TUI dashboard (press q to quit, r to refresh).
    Tui,

    /// Print shell completion script (zsh, bash, fish).
    ///
    /// Usage: claux completions zsh >> ~/.zshrc
    Completions {
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Status { json: false });

    // Shell completions — just print and exit.
    if let Commands::Completions { shell } = &command {
        let mut cmd = Cli::command();
        generate(*shell, &mut cmd, "claux", &mut std::io::stdout());
        return Ok(());
    }

    // The TUI manages its own session loading loop.
    if let Commands::Tui = &command {
        return commands::tui::run();
    }

    // Account and skills don't need session loading.
    if let Commands::Account = &command {
        return commands::account::run();
    }
    if let Commands::Skills { action } = &command {
        return commands::skills::run(action);
    }
    if let Commands::Config { action } = &command {
        return commands::config::run(action);
    }

    // For all other commands, load sessions once.
    let mut cache = SessionCache::new();
    let sessions  = load_sessions(&mut cache);

    match command {
        Commands::Status { json } => {
            commands::status::run(&sessions, json)?;
        }
        Commands::Sessions { limit, json } => {
            commands::sessions::run(&sessions, limit, json)?;
        }
        Commands::Spend { json } => {
            commands::spend::run(&sessions, json)?;
        }
        Commands::Analytics { days, json } => {
            commands::analytics::run(&sessions, days, json)?;
        }
        Commands::Export { format, output, limit } => {
            commands::export::run(&sessions, limit, format, output.as_deref())?;
        }
        Commands::Tag { session, label, remove } => {
            commands::tag::run(&sessions, &session, label.as_deref(), remove)?;
        }
        Commands::Tui | Commands::Completions { .. } |
        Commands::Account | Commands::Skills { .. } | Commands::Config { .. } => unreachable!(),
    }

    Ok(())
}
