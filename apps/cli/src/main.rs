mod commands;
mod format;
mod models;
mod monitor;
mod parser;
mod render;
mod spend;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

    /// Launch the live TUI dashboard (press q to quit, r to refresh).
    Tui,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Status { json: false });

    // The TUI manages its own session loading loop.
    if let Commands::Tui = &command {
        return commands::tui::run();
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
        Commands::Tui => unreachable!(),
    }

    Ok(())
}
