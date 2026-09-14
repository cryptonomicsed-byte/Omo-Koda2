use serde::{Deserialize, Serialize};

/// Discriminated-union body for each Tier-2 memory category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum PrivateMemoryBody {
    Identity(IdentityBody),
    Birth(BirthBody),
    Thought(ThoughtBody),
    Relation(RelationBody),
    Preference(PreferenceBody),
    Decision(DecisionBody),
    Capability(CapabilityBody),
    Receipt(ReceiptBody),
    Note(NoteBody),
}

/// A single private memory entry stored in MemoryVault.entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateMemoryEntry {
    /// UUID v4 — stable across edits.
    pub id: String,
    /// Unix seconds.
    pub created_ts: u64,
    /// Last update timestamp (None = never updated).
    pub updated_ts: Option<u64>,
    /// Structured body.
    pub body: PrivateMemoryBody,
    /// Optional free-form tags for retrieval.
    #[serde(default)]
    pub tags: Vec<String>,
}

// ── Body types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityBody {
    pub agent_id: String,
    pub name: String,
    pub odu_index: u8,
    pub odu_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthBody {
    pub genesis_receipt_id: String,
    pub birth_ts: u64,
    pub ip_root_kind: u16,
    pub nostr_pubkey: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtBody {
    pub content: String,
    /// Short-form summary for fast retrieval.
    pub summary: Option<String>,
    /// Confidence 0.0–1.0.
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationBody {
    pub peer_id: String,
    pub peer_name: Option<String>,
    pub relation_kind: String,
    pub trust_score: f32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceBody {
    pub key: String,
    pub value: serde_json::Value,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionBody {
    pub description: String,
    pub rationale: String,
    pub outcome: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityBody {
    pub skill: String,
    pub level: String,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptBody {
    /// ARP receipt ID or Vantage receipt reference.
    pub receipt_id: String,
    pub action: String,
    pub amount_cents: Option<u64>,
    pub currency: Option<String>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteBody {
    pub content: String,
}
