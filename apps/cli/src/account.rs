use std::collections::HashMap;
use std::fs;

use serde_json::Value;

use crate::models::AccountInfo;

pub fn load_account_info() -> Option<AccountInfo> {
    let path = dirs::home_dir()?.join(".claude.json");
    let content = fs::read_to_string(path).ok()?;
    let v: Value = serde_json::from_str(&content).ok()?;
    let oa = v.get("oauthAccount")?;

    Some(AccountInfo {
        display_name: oa
            .get("displayName")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        email: oa
            .get("emailAddress")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        plan_type: oa
            .get("organizationType")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        org_name: oa
            .get("organizationName")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        org_role: oa
            .get("organizationRole")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        billing_type: oa
            .get("billingType")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        account_created: oa
            .get("accountCreatedAt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        sub_created: oa
            .get("subscriptionCreatedAt")
            .and_then(Value::as_str)
            .map(|s| s.to_string()),
        rate_limit_tier: oa
            .get("organizationRateLimitTier")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        has_extra_usage: oa
            .get("hasExtraUsageEnabled")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

/// Returns `{skill_name → (usage_count, last_used_ms)}` from `skillUsage` in `~/.claude.json`.
pub fn load_skill_usage() -> HashMap<String, (usize, Option<u64>)> {
    let path = match dirs::home_dir() {
        Some(h) => h.join(".claude.json"),
        None => return HashMap::new(),
    };
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let v: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let mut map = HashMap::new();
    if let Some(obj) = v.get("skillUsage").and_then(Value::as_object) {
        for (name, stats) in obj {
            let count = stats.get("usageCount").and_then(Value::as_u64).unwrap_or(0) as usize;
            let last = stats.get("lastUsedAt").and_then(Value::as_u64);
            map.insert(name.clone(), (count, last));
        }
    }
    map
}
