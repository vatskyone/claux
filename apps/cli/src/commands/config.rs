use anyhow::{bail, Result};

use crate::config::{load_claux_config, save_claux_config};

#[derive(clap::Subcommand)]
pub enum ConfigAction {
    /// Print the current value of a config key.
    Get {
        /// Config key: weekly-budget or monthly-credit
        key: String,
    },
    /// Set a config value.
    Set {
        /// Config key: weekly-budget or monthly-credit
        key: String,
        /// Numeric value in USD.
        value: f64,
    },
    /// Unset (remove) a config value.
    Unset {
        /// Config key: weekly-budget or monthly-credit
        key: String,
    },
}

pub fn run(action: &ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Get { key } => {
            let cfg = load_claux_config();
            match key.as_str() {
                "weekly-budget"  => println!("{}", fmt_opt(cfg.weekly_budget_usd)),
                "monthly-credit" => println!("{}", fmt_opt(cfg.monthly_credit_usd)),
                other => bail!("unknown key '{}'. Valid keys: weekly-budget, monthly-credit", other),
            }
        }
        ConfigAction::Set { key, value } => {
            let mut cfg = load_claux_config();
            match key.as_str() {
                "weekly-budget"  => cfg.weekly_budget_usd  = Some(*value),
                "monthly-credit" => cfg.monthly_credit_usd = Some(*value),
                other => bail!("unknown key '{}'. Valid keys: weekly-budget, monthly-credit", other),
            }
            save_claux_config(&cfg)?;
            eprintln!("Set {} = ${:.2}", key, value);
        }
        ConfigAction::Unset { key } => {
            let mut cfg = load_claux_config();
            match key.as_str() {
                "weekly-budget"  => cfg.weekly_budget_usd  = None,
                "monthly-credit" => cfg.monthly_credit_usd = None,
                other => bail!("unknown key '{}'. Valid keys: weekly-budget, monthly-credit", other),
            }
            save_claux_config(&cfg)?;
            eprintln!("Unset {}", key);
        }
    }
    Ok(())
}

fn fmt_opt(v: Option<f64>) -> String {
    match v {
        Some(n) => format!("${:.2}", n),
        None    => "not set".to_string(),
    }
}
