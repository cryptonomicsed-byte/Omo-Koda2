//! ZÀNGBÉTÒ enforcement client.
//!
//! When `ZANGBETO_URL` is set, the runtime reports anomalies (e.g. a denied
//! capability) to the ZÀNGBÉTÒ enforcement bridge and receives the enforcement
//! action keyed on the same `agent_id` the agent registers on Vantage.
//!
//! Fail-open: a no-op returning `None` when `ZANGBETO_URL` is unset, so runtimes
//! without an enforcer are unaffected. Best-effort — transport errors are
//! swallowed rather than failing the act.

use std::sync::OnceLock;

use reqwest::Client;
use serde_json::json;

static HTTP: OnceLock<Client> = OnceLock::new();
fn http() -> &'static Client {
    HTTP.get_or_init(Client::new)
}

fn normalize_base(raw: &str) -> Option<String> {
    let url = raw.trim().trim_end_matches('/');
    if url.is_empty() {
        None
    } else {
        Some(url.to_string())
    }
}

fn base_url() -> Option<String> {
    normalize_base(&std::env::var("ZANGBETO_URL").ok()?)
}

/// Report an anomaly for `agent_id` to the enforcement bridge and return the
/// enforcement action ZÀNGBÉTÒ decided (the parsed JSON response), or `None`
/// when no enforcer is configured or the call fails.
///
/// `severity` ∈ observational | warning | critical | catastrophic.
/// `classification` ∈ schema_drift | economic_anomaly | temporal_inconsistency |
///   capability_escape | concurrency_conflict.
pub async fn report_anomaly(
    agent_id: &str,
    severity: &str,
    classification: &str,
    detail: &str,
) -> Option<serde_json::Value> {
    let base = base_url()?;
    let resp = http()
        .post(format!("{base}/enforce"))
        .json(&json!({
            "agent_id": agent_id,
            "severity": severity,
            "classification": classification,
            "detail": detail,
        }))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<serde_json::Value>().await.ok()
}

/// Ask ZÀNGBÉTÒ to review a *proposed* act before it runs and return its verdict
/// (parsed JSON), or `None` when no enforcer is configured or the call fails.
/// This is the pre-act enforcement gate: a blocking verdict (see
/// [`verdict_blocks`]) denies an otherwise-allowed act. Fail-open.
pub async fn review_act(agent_id: &str, tool: &str, detail: &str) -> Option<serde_json::Value> {
    let base = base_url()?;
    let resp = http()
        .post(format!("{base}/review"))
        .json(&json!({
            "agent_id": agent_id,
            "tool": tool,
            "detail": detail,
        }))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<serde_json::Value>().await.ok()
}

/// Interpret a ZÀNGBÉTÒ verdict: does it order the act blocked?
///
/// Conservative / fail-open by design — only an *explicit* blocking signal
/// returns `true`; an empty, malformed, or unrecognized verdict does not block.
/// Recognized: a string `action`/`decision`/`enforcement`/`status`/`verdict`
/// naming a blocking action, or `{"block": true}` / `{"allowed": false}`.
pub fn verdict_blocks(verdict: &serde_json::Value) -> bool {
    const BLOCKING: &[&str] = &[
        "block",
        "deny",
        "quarantine",
        "suspend",
        "halt",
        "reject",
        "jail",
    ];
    for key in ["action", "decision", "enforcement", "status", "verdict"] {
        if let Some(s) = verdict.get(key).and_then(|v| v.as_str()) {
            if BLOCKING.iter().any(|b| s.eq_ignore_ascii_case(b)) {
                return true;
            }
        }
    }
    if verdict.get("block").and_then(|v| v.as_bool()) == Some(true) {
        return true;
    }
    if verdict.get("allowed").and_then(|v| v.as_bool()) == Some(false) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn verdict_blocks_on_explicit_signals() {
        assert!(verdict_blocks(&json!({"action": "quarantine"})));
        assert!(verdict_blocks(&json!({"decision": "DENY"})));
        assert!(verdict_blocks(&json!({"enforcement": "block"})));
        assert!(verdict_blocks(&json!({"status": "suspend"})));
        assert!(verdict_blocks(&json!({"block": true})));
        assert!(verdict_blocks(&json!({"allowed": false})));
    }

    #[test]
    fn verdict_allows_by_default() {
        // Fail-open: anything not an explicit block must not gate the act.
        assert!(!verdict_blocks(&json!({})));
        assert!(!verdict_blocks(&json!({"action": "observe"})));
        assert!(!verdict_blocks(&json!({"decision": "allow"})));
        assert!(!verdict_blocks(&json!({"block": false})));
        assert!(!verdict_blocks(&json!({"allowed": true})));
        assert!(!verdict_blocks(&json!("garbage")));
        assert!(!verdict_blocks(&json!(42)));
    }

    #[test]
    fn normalize_base_trims_and_rejects_empty() {
        assert_eq!(
            normalize_base("http://enforcer:8787/"),
            Some("http://enforcer:8787".to_string())
        );
        assert_eq!(
            normalize_base("  http://x:1/  "),
            Some("http://x:1".to_string())
        );
        assert_eq!(normalize_base(""), None);
        assert_eq!(normalize_base("   "), None);
        assert_eq!(normalize_base("/"), None);
    }
}
