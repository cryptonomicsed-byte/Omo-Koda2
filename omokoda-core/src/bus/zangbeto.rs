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

#[cfg(test)]
mod tests {
    use super::*;

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
