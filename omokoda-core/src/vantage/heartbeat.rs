//! Periodic heartbeat reporter — POSTs canonical AgentHeartbeat to Vantage.
//! Gap #65: unified to use lifecycle::AgentHeartbeat instead of custom JSON.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{info, warn};
use crate::lifecycle::{AgentHeartbeat, HeartbeatState};

/// Config for the heartbeat task.
#[derive(Clone)]
pub struct HeartbeatConfig {
    pub vantage_base_url:  String,
    pub vantage_api_token: String,
    pub agent_id:          String,
    pub tier:              String,
    pub interval_secs:     u64,
}

/// Shared mutable state for the heartbeat chain.
#[derive(Clone, Default)]
pub struct HeartbeatChain {
    pub inner: Arc<Mutex<HeartbeatChainState>>,
}

#[derive(Default)]
pub struct HeartbeatChainState {
    pub sequence:              u64,
    pub previous_hash:         Option<String>,
    pub boot_id:               String,
    pub active_daemons:        Vec<String>,
    pub current_work:          Option<String>,
    pub message_count:         u64,
}

impl HeartbeatChain {
    pub fn new(boot_id: impl Into<String>) -> Self {
        let chain = Self::default();
        chain.inner.lock().unwrap().boot_id = boot_id.into();
        chain
    }
}

/// Spawn heartbeat background task. No-ops if interval_secs == 0.
pub fn spawn_heartbeat(
    config: HeartbeatConfig,
    chain: HeartbeatChain,
    http: reqwest::Client,
) {
    if config.interval_secs == 0 { return; }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(config.interval_secs));
        loop {
            interval.tick().await;
            report_once(&config, &chain, &http).await;
        }
    });
}

async fn report_once(
    config: &HeartbeatConfig,
    chain:  &HeartbeatChain,
    http:   &reqwest::Client,
) {
    let beat = {
        let mut state = chain.inner.lock().unwrap();
        let beat = AgentHeartbeat {
            agent_id:                config.agent_id.clone(),
            boot_id:                 state.boot_id.clone(),
            sequence:                state.sequence,
            timestamp:               now_secs(),
            state:                   HeartbeatState::Alive,
            tier:                    config.tier.clone(),
            active_daemons:          state.active_daemons.clone(),
            current_work:            state.current_work.clone(),
            previous_heartbeat_hash: state.previous_hash.clone(),
            signature:               None,
            soma_vector:             None,
            message_count:           state.message_count,
        };
        let hash = beat.hash();
        state.previous_hash = Some(hash);
        state.sequence += 1;
        beat
    };

    let url = format!("{}/api/agents/heartbeat", config.vantage_base_url);
    let result = http
        .post(&url)
        .bearer_auth(&config.vantage_api_token)
        .json(&beat)
        .send()
        .await;

    match result {
        Ok(r) if r.status().is_success() =>
            info!(agent_id = %beat.agent_id, seq = beat.sequence, "heartbeat sent"),
        Ok(r) =>
            warn!(status = %r.status(), agent_id = %beat.agent_id, "heartbeat non-2xx"),
        Err(e) =>
            warn!(error = %e, agent_id = %beat.agent_id, "heartbeat failed"),
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
