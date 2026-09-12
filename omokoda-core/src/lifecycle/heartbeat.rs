//! Tamper-evident agent heartbeat — each beat chains to the previous via SHA-256.
//!
//! The hash chain proves *continuity of life*: if any beat is dropped or altered,
//! the chain breaks. Zàngbétò can audit the chain as a receipts provenance trace.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// One heartbeat record. Serialises to canonical JSON for hashing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    pub agent_id:               String,
    pub boot_id:                String,
    pub sequence:               u64,
    pub timestamp:              u64,
    pub state:                  HeartbeatState,
    pub tier:                   String,
    pub active_daemons:         Vec<String>,
    pub current_work:           Option<String>,
    pub previous_heartbeat_hash: Option<String>,
    pub signature:              Option<String>,  // ed25519, future
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeartbeatState {
    Alive,
    Thinking,
    Working,
    Resting,  // Sabbath
    Offline,
}

impl AgentHeartbeat {
    /// Compute the canonical SHA-256 hash of this beat (excluding the signature field).
    pub fn hash(&self) -> String {
        let canonical = serde_json::json!({
            "agent_id":               self.agent_id,
            "boot_id":                self.boot_id,
            "sequence":               self.sequence,
            "timestamp":              self.timestamp,
            "state":                  self.state,
            "tier":                   self.tier,
            "active_daemons":         self.active_daemons,
            "current_work":           self.current_work,
            "previous_heartbeat_hash": self.previous_heartbeat_hash,
        });
        let bytes = canonical.to_string();
        let digest = Sha256::digest(bytes.as_bytes());
        hex::encode(digest)
    }

    /// Build the first beat in a new chain (no predecessor).
    pub fn genesis(agent_id: impl Into<String>, tier: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            boot_id:  Uuid::new_v4().to_string(),
            sequence: 0,
            timestamp: now_secs(),
            state:    HeartbeatState::Alive,
            tier:     tier.into(),
            active_daemons: vec![],
            current_work: None,
            previous_heartbeat_hash: None,
            signature: None,
        }
    }

    /// Produce the next beat, chaining from `prev`.
    pub fn next_from(
        prev: &AgentHeartbeat,
        state: HeartbeatState,
        active_daemons: Vec<String>,
        current_work: Option<String>,
    ) -> Self {
        Self {
            agent_id:  prev.agent_id.clone(),
            boot_id:   prev.boot_id.clone(),
            sequence:  prev.sequence + 1,
            timestamp: now_secs(),
            state,
            tier:      prev.tier.clone(),
            active_daemons,
            current_work,
            previous_heartbeat_hash: Some(prev.hash()),
            signature: None,
        }
    }

    /// Verify the chain link: does `candidate.previous_heartbeat_hash == expected_prev.hash()`?
    pub fn verify_chain(expected_prev: &AgentHeartbeat, candidate: &AgentHeartbeat) -> bool {
        candidate.previous_heartbeat_hash.as_deref() == Some(&expected_prev.hash())
            && candidate.sequence == expected_prev.sequence + 1
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_has_no_predecessor() {
        let beat = AgentHeartbeat::genesis("agent:1", "resident");
        assert_eq!(beat.sequence, 0);
        assert!(beat.previous_heartbeat_hash.is_none());
    }

    #[test]
    fn next_chains_correctly() {
        let g = AgentHeartbeat::genesis("agent:1", "resident");
        let g_hash = g.hash();
        let next = AgentHeartbeat::next_from(&g, HeartbeatState::Thinking, vec![], None);
        assert_eq!(next.sequence, 1);
        assert_eq!(next.previous_heartbeat_hash.as_deref(), Some(g_hash.as_str()));
        assert!(AgentHeartbeat::verify_chain(&g, &next));
    }

    #[test]
    fn chain_breaks_on_tamper() {
        let g = AgentHeartbeat::genesis("agent:1", "resident");
        let mut next = AgentHeartbeat::next_from(&g, HeartbeatState::Alive, vec![], None);
        next.sequence = 999; // tamper
        assert!(!AgentHeartbeat::verify_chain(&g, &next));
    }

    #[test]
    fn hash_is_deterministic() {
        let g = AgentHeartbeat::genesis("agent:X", "citizen");
        assert_eq!(g.hash(), g.hash());
    }

    #[test]
    fn different_beats_have_different_hashes() {
        let g = AgentHeartbeat::genesis("agent:1", "resident");
        let n = AgentHeartbeat::next_from(&g, HeartbeatState::Working, vec![], None);
        assert_ne!(g.hash(), n.hash());
    }
}
