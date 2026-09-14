use serde::{Deserialize, Serialize};

use crate::session::ConversationMessage;

/// Tier 2 identity vault: private agent-authored knowledge.
/// Contains thoughts, relationships, preferences — the agent's inner world.
/// Logically separate from IdentityVault (keys) — thoughts vs. credentials.
/// Never transmitted; lives only inside the sealed session blob.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tier2Vault {
    /// Conversation history the agent has authored (private messages).
    pub private_messages: Vec<ConversationMessage>,
    /// Tier 2 structured memory entries (thoughts, relations, preferences…).
    pub entries: Vec<crate::memory::private_schema::PrivateMemoryEntry>,
}

impl Tier2Vault {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.private_messages.is_empty() && self.entries.is_empty()
    }
}
