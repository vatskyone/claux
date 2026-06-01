use crate::config::{active_sessions_dir, load_claux_config, projects_root_dir, save_claux_config};
use crate::render::{kv, section, success};
use anyhow::{bail, Result};

#[derive(clap::Subcommand)]
pub enum ConfigAction {
    /// Print the current value of a config key.
    Get {
        /// Config key.
        key: String,
    },
    /// Set a config value.
    Set {
        /// Config key.
        key: String,
        /// Numeric value in USD for budget keys.
        value: String,
    },
    /// Unset (remove) a config value.
    Unset {
        /// Config key.
        key: String,
    },
    /// Initialize first-run config and defaults.
    Init {
        /// Optional weekly budget in USD.
        #[arg(long)]
        weekly_budget: Option<f64>,
        /// Optional 5-hour plan limit in USD.
        #[arg(long)]
        plan_5h_limit: Option<f64>,
        /// Optional monthly credit cap in USD.
        #[arg(long)]
        monthly_credit: Option<f64>,
        /// Override projects root path.
        #[arg(long)]
        projects_root: Option<String>,
        /// Override active sessions path.
        #[arg(long)]
        sessions_root: Option<String>,
    },
}

pub fn run(action: &ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Get { key } => {
            let cfg = load_claux_config();
            println!("{}", section("Config"));
            match key.as_str() {
                "weekly-budget" => println!("{}", kv("weekly-budget", fmt_opt_usd(cfg.weekly_budget_usd))),
                "plan-5h-limit" => println!("{}", kv("plan-5h-limit", fmt_opt_usd(cfg.plan_5h_limit_usd))),
                "monthly-credit" => println!("{}", kv("monthly-credit", fmt_opt_usd(cfg.monthly_credit_usd))),
                "projects-root" => println!("{}", kv("projects-root", cfg.projects_root.unwrap_or_else(|| projects_root_dir().display().to_string()))),
                "sessions-root" => println!("{}", kv("sessions-root", cfg.sessions_root.unwrap_or_else(|| active_sessions_dir().display().to_string()))),
                other => bail!(
                    "unknown key '{}'. Valid keys: weekly-budget, plan-5h-limit, monthly-credit, projects-root, sessions-root",
                    other
                ),
            }
        }
        ConfigAction::Set { key, value } => {
            let mut cfg = load_claux_config();
            match key.as_str() {
                "weekly-budget" => cfg.weekly_budget_usd = Some(parse_usd(value, key)?),
                "plan-5h-limit" => cfg.plan_5h_limit_usd = Some(parse_usd(value, key)?),
                "monthly-credit" => cfg.monthly_credit_usd = Some(parse_usd(value, key)?),
                "projects-root" => cfg.projects_root = Some(value.clone()),
                "sessions-root" => cfg.sessions_root = Some(value.clone()),
                other => bail!(
                    "unknown key '{}'. Valid keys: weekly-budget, plan-5h-limit, monthly-credit, projects-root, sessions-root",
                    other
                ),
            }
            save_claux_config(&cfg)?;
            eprintln!("{}", success(format!("Set {} = {}", key, value)));
        }
        ConfigAction::Unset { key } => {
            let mut cfg = load_claux_config();
            match key.as_str() {
                "weekly-budget" => cfg.weekly_budget_usd = None,
                "plan-5h-limit" => cfg.plan_5h_limit_usd = None,
                "monthly-credit" => cfg.monthly_credit_usd = None,
                "projects-root" => cfg.projects_root = None,
                "sessions-root" => cfg.sessions_root = None,
                other => bail!(
                    "unknown key '{}'. Valid keys: weekly-budget, plan-5h-limit, monthly-credit, projects-root, sessions-root",
                    other
                ),
            }
            save_claux_config(&cfg)?;
            eprintln!("{}", success(format!("Unset {}", key)));
        }
        ConfigAction::Init {
            weekly_budget,
            plan_5h_limit,
            monthly_credit,
            projects_root,
            sessions_root,
        } => {
            let mut cfg = load_claux_config();

            if cfg.projects_root.is_none() {
                cfg.projects_root = Some(projects_root_dir().display().to_string());
            }
            if cfg.sessions_root.is_none() {
                cfg.sessions_root = Some(active_sessions_dir().display().to_string());
            }

            if let Some(v) = weekly_budget {
                cfg.weekly_budget_usd = Some(*v);
            }
            if let Some(v) = plan_5h_limit {
                cfg.plan_5h_limit_usd = Some(*v);
            }
            if let Some(v) = monthly_credit {
                cfg.monthly_credit_usd = Some(*v);
            }
            if let Some(path) = projects_root {
                cfg.projects_root = Some(path.clone());
            }
            if let Some(path) = sessions_root {
                cfg.sessions_root = Some(path.clone());
            }

            save_claux_config(&cfg)?;
            println!("{}", section("Config"));
            println!("{}", success("Initialized ~/.claude/claux/config.json"));
            println!(
                "{}",
                kv(
                    "projects-root",
                    cfg.projects_root.as_deref().unwrap_or("(not set)")
                )
            );
            println!(
                "{}",
                kv(
                    "sessions-root",
                    cfg.sessions_root.as_deref().unwrap_or("(not set)")
                )
            );
            println!(
                "{}",
                kv("weekly-budget", fmt_opt_usd(cfg.weekly_budget_usd))
            );
            println!(
                "{}",
                kv("plan-5h-limit", fmt_opt_usd(cfg.plan_5h_limit_usd))
            );
            println!(
                "{}",
                kv("monthly-credit", fmt_opt_usd(cfg.monthly_credit_usd))
            );
        }
    }
    Ok(())
}

fn fmt_opt_usd(v: Option<f64>) -> String {
    match v {
        Some(n) => format!("${:.2}", n),
        None => "not set".to_string(),
    }
}

fn parse_usd(value: &str, key: &str) -> Result<f64> {
    let parsed: f64 = value
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid numeric value '{}' for {}", value, key))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        bail!("{} must be > 0", key);
    }
    Ok(parsed)
}
