use crate::types::{AgentId, ResourceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationReceipt {
    pub resource_id: ResourceId,
    pub reserved_by: AgentId,
    pub reserved_from: u64,
    pub reserved_until: u64,
    pub purpose: String,
    pub previous_hash: [u8; 32],
    pub hash: [u8; 32],
}

impl ReservationReceipt {
    pub fn new(
        resource_id: ResourceId,
        reserved_by: AgentId,
        reserved_from: u64,
        reserved_until: u64,
        purpose: String,
        previous_hash: [u8; 32],
    ) -> Self {
        let mut receipt = Self {
            resource_id,
            reserved_by,
            reserved_from,
            reserved_until,
            purpose,
            previous_hash,
            hash: [0u8; 32],
        };
        receipt.hash = receipt.compute_hash();
        receipt
    }

    fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.resource_id.as_bytes());
        hasher.update(self.reserved_by.as_bytes());
        hasher.update(&self.reserved_from.to_le_bytes());
        hasher.update(&self.reserved_until.to_le_bytes());
        hasher.update(self.purpose.as_bytes());
        hasher.update(&self.previous_hash);
        *hasher.finalize().as_bytes()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Resource '{0}' not found")]
    NotFound(ResourceId),
    #[error("Resource '{0}' already reserved by {1} until {2}")]
    AlreadyReserved(ResourceId, AgentId, u64),
    #[error("Insufficient trust score: need {needed}, have {actual}")]
    InsufficientTrust { needed: f32, actual: f32 },
}

pub struct ResourceRegistry {
    resources: HashMap<ResourceId, ResourceState>,
    reservation_log: Vec<ReservationReceipt>,
    last_hash: [u8; 32],
}

struct ResourceState {
    name: String,
    _owner: AgentId,
    reserved_by: Option<AgentId>,
    reserved_until: Option<u64>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            reservation_log: Vec::new(),
            last_hash: [0u8; 32],
        }
    }

    pub fn register(&mut self, resource_id: ResourceId, name: String, owner: AgentId) {
        self.resources.insert(
            resource_id,
            ResourceState {
                name,
                _owner: owner,
                reserved_by: None,
                reserved_until: None,
            },
        );
    }

    pub fn reserve(
        &mut self,
        resource_id: &str,
        agent_id: &str,
        duration_secs: u64,
        purpose: &str,
        _agent_trust: f32,
    ) -> Result<ReservationReceipt, ResourceError> {
        let now = current_unix_ts();

        let state = self
            .resources
            .get_mut(resource_id)
            .ok_or_else(|| ResourceError::NotFound(resource_id.to_string()))?;

        if let (Some(reserved_by), Some(until)) = (&state.reserved_by, state.reserved_until) {
            if now < until {
                return Err(ResourceError::AlreadyReserved(
                    resource_id.to_string(),
                    reserved_by.clone(),
                    until,
                ));
            }
        }

        state.reserved_by = Some(agent_id.to_string());
        state.reserved_until = Some(now + duration_secs);

        let receipt = ReservationReceipt::new(
            resource_id.to_string(),
            agent_id.to_string(),
            now,
            now + duration_secs,
            purpose.to_string(),
            self.last_hash,
        );
        self.last_hash = receipt.hash;
        self.reservation_log.push(receipt.clone());
        Ok(receipt)
    }

    pub fn release(&mut self, resource_id: &str, agent_id: &str) -> bool {
        if let Some(state) = self.resources.get_mut(resource_id) {
            if state.reserved_by.as_deref() == Some(agent_id) {
                state.reserved_by = None;
                state.reserved_until = None;
                return true;
            }
        }
        false
    }

    pub fn list_available(&self) -> Vec<(ResourceId, String)> {
        let now = current_unix_ts();
        self.resources
            .iter()
            .filter(|(_, s)| s.reserved_until.map(|u| now > u).unwrap_or(true))
            .map(|(id, s)| (id.clone(), s.name.clone()))
            .collect()
    }

    pub fn reservation_log(&self) -> &[ReservationReceipt] {
        &self.reservation_log
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn current_unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
