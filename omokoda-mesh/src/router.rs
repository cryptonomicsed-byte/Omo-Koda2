use crate::{
    negotiation::{NegotiationThread, Proposal},
    resource::ResourceRegistry,
    state::MeshState,
    trust::MeshTrustModel,
    types::{AgentId, BlockId, NegotiationId},
};
use std::collections::HashMap;

pub struct MeshRouter {
    pub block_members: HashMap<BlockId, Vec<AgentId>>,
    pub negotiation_threads: HashMap<NegotiationId, NegotiationThread>,
    pub resource_registry: ResourceRegistry,
    pub trust_model: MeshTrustModel,
}

#[derive(Debug)]
pub enum RouterError {
    AgentNotInBlock,
    InsufficientMembership(String),
    NegotiationNotFound(NegotiationId),
    NegotiationExpired(NegotiationId),
    ResourceError(crate::resource::ResourceError),
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::AgentNotInBlock => write!(f, "Agent is not a member of this block"),
            RouterError::InsufficientMembership(m) => write!(f, "Insufficient membership: {m}"),
            RouterError::NegotiationNotFound(id) => write!(f, "Negotiation '{id}' not found"),
            RouterError::NegotiationExpired(id) => write!(f, "Negotiation '{id}' has expired"),
            RouterError::ResourceError(e) => write!(f, "Resource error: {e}"),
        }
    }
}

impl MeshRouter {
    pub fn new() -> Self {
        Self {
            block_members: HashMap::new(),
            negotiation_threads: HashMap::new(),
            resource_registry: ResourceRegistry::new(),
            trust_model: MeshTrustModel::new(),
        }
    }

    pub fn register_member(&mut self, block_id: BlockId, agent_id: AgentId) {
        self.block_members
            .entry(block_id)
            .or_default()
            .push(agent_id);
    }

    pub fn propose(
        &mut self,
        proposer: AgentId,
        respondent: AgentId,
        proposal: Proposal,
        mesh_state: &MeshState,
    ) -> Result<NegotiationId, RouterError> {
        if !mesh_state.can_initiate_proposals() {
            return Err(RouterError::InsufficientMembership(
                "Active membership required to initiate proposals".to_string(),
            ));
        }
        let thread = NegotiationThread::new(proposer, respondent, proposal);
        let id = thread.id.clone();
        self.negotiation_threads.insert(id.clone(), thread);
        Ok(id)
    }

    pub fn respond(
        &mut self,
        negotiation_id: &str,
        respondent: &str,
        decision: &str,
    ) -> Result<(), RouterError> {
        let thread = self
            .negotiation_threads
            .get_mut(negotiation_id)
            .ok_or_else(|| RouterError::NegotiationNotFound(negotiation_id.to_string()))?;

        if thread.is_expired() {
            return Err(RouterError::NegotiationExpired(negotiation_id.to_string()));
        }

        match decision {
            "accept" => {
                thread.accept();
                self.trust_model.commitment_completed(respondent);
                self.trust_model
                    .commitment_completed(&thread.proposer.clone());
            }
            "reject" => thread.reject(),
            _ => thread.reject(),
        }
        Ok(())
    }

    pub fn neighbors_for_block(&self, block_id: &str) -> Vec<&AgentId> {
        self.block_members
            .get(block_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn trust_score(&self, agent_id: &str) -> f32 {
        self.trust_model.score(agent_id)
    }

    pub fn get_negotiation(&self, id: &str) -> Option<&NegotiationThread> {
        self.negotiation_threads.get(id)
    }
}

impl Default for MeshRouter {
    fn default() -> Self {
        Self::new()
    }
}
