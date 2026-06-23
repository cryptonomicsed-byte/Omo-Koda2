use crate::types::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeOutcome {
    ResolvedAmicably,
    ResolvedByOrchestrator,
    EscalatedToHuman,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborTrust {
    pub agent_id: AgentId,
    pub score: f32,
    pub commitments_completed: u32,
    pub commitments_broken: u32,
    pub last_interaction: u64,
    pub dispute_outcomes: Vec<DisputeOutcome>,
}

impl NeighborTrust {
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            score: 0.5,
            commitments_completed: 0,
            commitments_broken: 0,
            last_interaction: 0,
            dispute_outcomes: Vec::new(),
        }
    }

    pub fn record_commitment_completed(&mut self) {
        self.commitments_completed += 1;
        self.score = (self.score + 0.05).min(1.0);
        self.last_interaction = current_unix_ts();
    }

    pub fn record_commitment_broken(&mut self) {
        self.commitments_broken += 1;
        self.score = (self.score - 0.15).max(0.0);
        self.last_interaction = current_unix_ts();
    }

    pub fn reliability_ratio(&self) -> f32 {
        let total = self.commitments_completed + self.commitments_broken;
        if total == 0 {
            return 0.5;
        }
        self.commitments_completed as f32 / total as f32
    }
}

pub struct MeshTrustModel {
    neighbor_trust: HashMap<AgentId, NeighborTrust>,
    probation_threshold: u32,
    _trust_decay_rate: f64,
    _dispute_penalty: f64,
}

impl MeshTrustModel {
    pub fn new() -> Self {
        Self {
            neighbor_trust: HashMap::new(),
            probation_threshold: 5,
            _trust_decay_rate: 0.005,
            _dispute_penalty: 0.15,
        }
    }

    pub fn get_or_init(&mut self, agent_id: &str) -> &mut NeighborTrust {
        self.neighbor_trust
            .entry(agent_id.to_string())
            .or_insert_with(|| NeighborTrust::new(agent_id.to_string()))
    }

    pub fn score(&self, agent_id: &str) -> f32 {
        self.neighbor_trust
            .get(agent_id)
            .map(|t| t.score)
            .unwrap_or(0.5)
    }

    pub fn commitment_completed(&mut self, agent_id: &str) {
        self.get_or_init(agent_id).record_commitment_completed();
    }

    pub fn commitment_broken(&mut self, agent_id: &str) {
        self.get_or_init(agent_id).record_commitment_broken();
    }

    pub fn should_graduate(&self, agent_id: &str, commitments_kept: u32) -> bool {
        commitments_kept >= self.probation_threshold && self.score(agent_id) >= 0.6
    }

    pub fn all_scores(&self) -> Vec<(AgentId, f32)> {
        self.neighbor_trust
            .iter()
            .map(|(id, t)| (id.clone(), t.score))
            .collect()
    }
}

impl Default for MeshTrustModel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProbationManager {
    pub threshold: u32,
}

impl ProbationManager {
    pub fn new(threshold: u32) -> Self {
        Self { threshold }
    }

    pub fn should_graduate(&self, commitments_kept: u32, trust_score: f32) -> bool {
        commitments_kept >= self.threshold && trust_score >= 0.6
    }
}

fn current_unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
