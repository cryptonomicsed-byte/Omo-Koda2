use crate::types::{AgentId, BlockId, GeoCoord, MeshMembership, MeshRole, ResourceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshState {
    pub block_id: BlockId,
    pub role: MeshRole,
    pub location: Option<GeoCoord>,
    pub neighbors: HashMap<AgentId, NeighborProfile>,
    pub trust_scores: HashMap<AgentId, f32>,
    pub shared_resources: Vec<ResourceEntry>,
    pub capability_card: CapabilityCard,
    pub membership: MeshMembership,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub commitments_made: u32,
    pub commitments_kept: u32,
}

impl MeshState {
    pub fn new(block_id: BlockId, role: MeshRole, agent_id: AgentId) -> Self {
        Self {
            block_id: block_id.clone(),
            role: role.clone(),
            location: None,
            neighbors: HashMap::new(),
            trust_scores: HashMap::new(),
            shared_resources: Vec::new(),
            capability_card: CapabilityCard {
                agent_id,
                tools: Vec::new(),
                resources: Vec::new(),
                schedule: Vec::new(),
                version: 1,
                signed_hash: [0u8; 32],
            },
            membership: MeshMembership::Probation,
            joined_at: chrono::Utc::now(),
            commitments_made: 0,
            commitments_kept: 0,
        }
    }

    pub fn trust_score_for(&self, agent_id: &str) -> f32 {
        self.trust_scores.get(agent_id).copied().unwrap_or(0.0)
    }

    pub fn update_trust(&mut self, agent_id: &str, delta: f32) {
        let score = self.trust_scores.entry(agent_id.to_string()).or_insert(0.5);
        *score = (*score + delta).clamp(0.0, 1.0);
    }

    pub fn can_initiate_proposals(&self) -> bool {
        self.membership == MeshMembership::Active
    }

    pub fn can_reserve_resources(&self) -> bool {
        self.membership == MeshMembership::Active
    }

    pub fn graduation_progress(&self) -> f32 {
        if self.commitments_made == 0 {
            return 0.0;
        }
        self.commitments_kept as f32 / self.commitments_made as f32
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeighborProfile {
    pub agent_id: AgentId,
    pub public_key: Vec<u8>,
    pub capability_card: CapabilityCard,
    pub role: MeshRole,
    pub trust_score: f32,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceEntry {
    pub resource_id: ResourceId,
    pub name: String,
    pub available: bool,
    pub reserved_by: Option<AgentId>,
    pub reserved_until: Option<chrono::DateTime<chrono::Utc>>,
    pub access_conditions: Vec<String>,
}

impl ResourceEntry {
    pub fn is_available_now(&self) -> bool {
        if !self.available {
            return false;
        }
        if let Some(until) = self.reserved_until {
            return chrono::Utc::now() > until;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityCard {
    pub agent_id: AgentId,
    pub tools: Vec<ToolDescriptor>,
    pub resources: Vec<ResourceDescriptor>,
    pub schedule: Vec<AvailabilityWindow>,
    pub version: u32,
    pub signed_hash: [u8; 32],
}

impl CapabilityCard {
    pub fn compute_hash(&self) -> [u8; 32] {
        let data = serde_json::to_string(self).unwrap_or_default();
        *blake3::hash(data.as_bytes()).as_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub required_tier: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceDescriptor {
    pub resource_id: ResourceId,
    pub name: String,
    pub description: String,
    pub availability: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AvailabilityWindow {
    pub days: Vec<String>,
    pub start_utc: String,
    pub end_utc: String,
}
