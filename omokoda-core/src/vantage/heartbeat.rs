//! Periodic heartbeat reporter — POSTs node health to Vantage /api/nodes/heartbeat.
//! Migrated from sovereign-node. Belongs in Omo-Koda2 (agent OS layer).

use std::time::Duration;
use tracing::{info, warn};
use serde_json::{json, Value};

/// Minimal config for the heartbeat task.
#[derive(Clone)]
pub struct HeartbeatConfig {
    pub vantage_base_url: String,
    pub vantage_api_token: String,
    pub node_did: String,
    pub osovm_url: Option<String>,
    pub interval_secs: u64,
}

/// Spawn heartbeat background task. No-ops if interval_secs == 0.
pub fn spawn_heartbeat(config: HeartbeatConfig, http: reqwest::Client) {
    if config.interval_secs == 0 {
        return;
    }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(config.interval_secs));
        loop {
            interval.tick().await;
            report_once(&config, &http).await;
        }
    });
}

async fn report_once(config: &HeartbeatConfig, http: &reqwest::Client) {
    let body = json!({
        "node_did":   config.node_did,
        "osovm_url":  config.osovm_url.as_deref().unwrap_or(""),
    });

    let url = format!("{}/api/nodes/heartbeat", config.vantage_base_url);
    let result = http
        .post(&url)
        .bearer_auth(&config.vantage_api_token)
        .json(&body)
        .send()
        .await;

    match result {
        Ok(r) if r.status().is_success() =>
            info!(node_did = %config.node_did, "heartbeat sent"),
        Ok(r) =>
            warn!(status = %r.status(), node_did = %config.node_did, "heartbeat non-2xx"),
        Err(e) =>
            warn!(error = %e, node_did = %config.node_did, "heartbeat failed"),
    }
}
