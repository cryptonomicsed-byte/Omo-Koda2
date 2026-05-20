use crate::identity::AgentId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActReceipt {
    pub agent_id: AgentId,
    pub action_tool: String,
    pub action_params: String,
    pub hermetic_scores: String,
    pub decision: String,
    pub overall_score: f32,
    pub timestamp: u64,
}
