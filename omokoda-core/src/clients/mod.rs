pub mod stubs;
pub mod http;

use serde::{Deserialize, Serialize};

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ClientError {
    Unavailable(String),
    InvalidInput(String),
    Timeout,
    Unauthorized,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(s) => write!(f, "Service unavailable: {s}"),
            Self::InvalidInput(s) => write!(f, "Invalid input: {s}"),
            Self::Timeout => write!(f, "Request timed out"),
            Self::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

// ─── Result Types ─────────────────────────────────────────────────────────────

/// From Bipon39-Rust-
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MnemonicResult {
    pub phrase: String,
    pub word_count: u8,
}

/// From Bipon39-Rust-: 7-principle personality distribution + 5-element vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityResult {
    /// Index maps to HermeticPrinciple (0=Mentalism … 6=Gender)
    pub distribution: [u8; 7],
    /// 5-element energy: [fire, water, earth, air, ether]
    pub elemental: [f32; 5],
    /// Index of the dominant principle (0–6)
    pub dominant: u8,
}

/// From vanity2: Ed25519 signing key + Sui-compatible address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletResult {
    pub signing_key: [u8; 32],
    pub address: String,
}

/// From vanity2: display-safe cloaked word list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloakResult {
    pub cloaked_words: Vec<String>,
}

/// From vanity2: address poisoning scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoisonScanResult {
    pub is_safe: bool,
    pub similar_to: Option<String>,
}

/// From Ritual-codex-Julia: Proof-of-Cognitive-Work verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocwResult {
    pub verified: bool,
    pub floor: u64,
}

/// From Ritual-codex-Julia: Busy Beaver Unit complexity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BbuResult {
    pub score: f64,
}

/// From Ritual-codex-Julia: Augury memory branch prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuguryResult {
    pub predicted_branch: String,
    pub confidence: f64,
}

/// Input to Augury prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPattern {
    pub branch_id: String,
    pub timestamp: f64,
    pub weight: f64,
}

/// From ifascript: Odu lookup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OduResult {
    pub id: u8,
    pub name: String,
    pub prescription: String,
}

/// From ifascript: Ebo exception level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EboLevel {
    Advisory,
    Caution,
    Critical,
}

/// From ifascript: Ebo exception evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EboResult {
    pub level: EboLevel,
    pub message: String,
}

/// From ifascript: LARQL query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LarqlResult {
    pub steps: Vec<String>,
    pub confidence: f64,
    pub human_override: bool,
}

/// Input node for Nex- swarm graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub tool: String,
    pub params: serde_json::Value,
    pub depends_on: Vec<String>,
}

/// From Nex-: graph execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResult {
    pub graph_id: String,
    pub nodes_executed: u32,
}

/// From Nex-: graph execution state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphState {
    Pending,
    Running,
    Complete,
    Failed(String),
}

/// From Nex-: graph status query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatus {
    pub graph_id: String,
    pub state: GraphState,
}

// ─── Client Traits ────────────────────────────────────────────────────────────

/// Bipon39-Rust-: mnemonic encoding + personality profiling
pub trait BiponClient: Send + Sync {
    fn entropy_to_mnemonic(&self, entropy: &[u8]) -> Result<MnemonicResult, ClientError>;
    fn mnemonic_to_seed(&self, phrase: &str, passphrase: &str) -> Result<[u8; 64], ClientError>;
    fn personality_profile(&self, mnemonic: &str) -> Result<PersonalityResult, ClientError>;
}

/// vanity2: wallet derivation + display cloaking + poison detection
pub trait VanityClient: Send + Sync {
    fn derive_wallet(&self, mnemonic: &str, passphrase: &str) -> Result<WalletResult, ClientError>;
    fn cloak_display(&self, words: &[String], offset: u8) -> Result<CloakResult, ClientError>;
    fn scan_poison(&self, candidate: &str, known: &[String]) -> Result<PoisonScanResult, ClientError>;
}

/// Ritual-codex-Julia: PoCW verification + BBU scoring + Augury prediction
pub trait RitualClient: Send + Sync {
    fn verify_pocw(&self, tier: u8, steps: u64) -> Result<PocwResult, ClientError>;
    fn score_bbu(&self, code: &str) -> Result<BbuResult, ClientError>;
    fn augury_predict(&self, patterns: &[MemoryPattern]) -> Result<AuguryResult, ClientError>;
}

/// ifascript: LARQL queries + Odu lookup + Ebo exception + cowrie entropy
pub trait IfascriptClient: Send + Sync {
    fn lookup_odu(&self, index: u8) -> Result<OduResult, ClientError>;
    fn cast_ebo(&self, odu: u8) -> Result<EboResult, ClientError>;
    fn generate_entropy(&self, seed: &[u8]) -> Result<Vec<u8>, ClientError>;
    fn larql_query(&self, query: &str, tier: u8) -> Result<LarqlResult, ClientError>;
}

/// Nex-: swarm graph execution + status
pub trait NexClient: Send + Sync {
    fn submit_graph(&self, nodes: Vec<GraphNode>) -> Result<GraphResult, ClientError>;
    fn graph_status(&self, graph_id: &str) -> Result<GraphStatus, ClientError>;
}

// ─── Convenience Bundle ───────────────────────────────────────────────────────

/// All 5 clients bundled for injection into the Steward
pub struct ExternalClients {
    pub bipon: std::sync::Arc<dyn BiponClient>,
    pub vanity: std::sync::Arc<dyn VanityClient>,
    pub ritual: std::sync::Arc<dyn RitualClient>,
    pub ifascript: std::sync::Arc<dyn IfascriptClient>,
    pub nex: std::sync::Arc<dyn NexClient>,
}

impl ExternalClients {
    /// Build with all stubs — safe default for testing and fallback
    pub fn all_stubs() -> Self {
        use stubs::*;
        Self {
            bipon: std::sync::Arc::new(StubBiponClient),
            vanity: std::sync::Arc::new(StubVanityClient),
            ritual: std::sync::Arc::new(StubRitualClient),
            ifascript: std::sync::Arc::new(StubIfascriptClient),
            nex: std::sync::Arc::new(StubNexClient),
        }
    }
}
