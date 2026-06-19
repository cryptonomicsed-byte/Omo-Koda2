use crate::types::{AgentId, NegotiationId, ResourceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitmentKind {
    ResourceShare,
    ServicePerform,
    DataExchange,
    AccessGrant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NegotiationStatus {
    Proposed,
    Accepted,
    Rejected,
    CounterProposed,
    Expired,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub kind: CommitmentKind,
    pub resource_id: Option<ResourceId>,
    pub description: String,
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub give: Vec<Commitment>,
    pub take: Vec<Commitment>,
    pub duration_secs: u64,
    pub conditions: Vec<String>,
}

impl Proposal {
    pub fn give_summary(&self) -> String {
        self.give
            .iter()
            .map(|c| c.description.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn take_summary(&self) -> String {
        self.take
            .iter()
            .map(|c| c.description.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    pub sender: AgentId,
    pub content: String,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationThread {
    pub id: NegotiationId,
    pub proposer: AgentId,
    pub respondent: AgentId,
    pub proposal: Proposal,
    pub status: NegotiationStatus,
    pub messages: Vec<SignedMessage>,
    pub created_at: u64,
    pub ttl_secs: u64,
}

impl NegotiationThread {
    pub fn new(proposer: AgentId, respondent: AgentId, proposal: Proposal) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            proposer,
            respondent,
            proposal,
            status: NegotiationStatus::Proposed,
            messages: Vec::new(),
            created_at: current_unix_ts(),
            ttl_secs: 86_400,
        }
    }

    pub fn is_expired(&self) -> bool {
        current_unix_ts() > self.created_at + self.ttl_secs
    }

    pub fn accept(&mut self) {
        self.status = NegotiationStatus::Accepted;
    }

    pub fn reject(&mut self) {
        self.status = NegotiationStatus::Rejected;
    }
}

fn current_unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
