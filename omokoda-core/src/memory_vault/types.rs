use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    #[default]
    Private,
    Followers,
    Federated,
    Public,
}

impl std::fmt::Display for AccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessLevel::Private => write!(f, "private"),
            AccessLevel::Followers => write!(f, "followers"),
            AccessLevel::Federated => write!(f, "federated"),
            AccessLevel::Public => write!(f, "public"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub access: AccessLevel,
    pub federation_peers: Vec<String>,
    pub auto_export: bool,
    pub last_synced: Option<String>,
}

impl Default for VaultConfig {
    fn default() -> Self {
        VaultConfig {
            access: AccessLevel::Private,
            federation_peers: vec![],
            auto_export: true,
            last_synced: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Star {
    pub id: String,
    pub title: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub size: f64,
    pub color: String,
    pub constellation: String,
    pub tags: Vec<String>,
    pub content_type: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub source: [f64; 3],
    pub target: [f64; 3],
    pub weight: f64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nebula {
    pub id: String,
    pub trace_type: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub opacity: f64,
    pub size: f64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyData {
    pub agent_name: String,
    pub agent_id: String,
    pub stars: Vec<Star>,
    pub edges: Vec<Edge>,
    pub nebulae: Vec<Nebula>,
    pub clusters: HashMap<String, Vec<Star>>,
    pub bounds: GalaxyBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatus {
    pub enabled: bool,
    pub config: VaultConfig,
    pub note_counts: HashMap<String, usize>,
    pub vault_path: String,
}

#[derive(Deserialize)]
pub struct UpdateConfigBody {
    pub access: AccessLevel,
    pub federation_peers: Option<Vec<String>>,
    pub auto_export: Option<bool>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    pub timestamp: String,
    pub resource: String,
    pub access_type: String,
    pub accessor: String,
}

#[derive(Deserialize)]
pub struct CreateKnowledgeBody {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: Option<f64>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct AccessLogQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDirEntry {
    pub name: String,
    pub path: String,
}

#[derive(Deserialize)]
pub struct ListDirQuery {
    pub dir: Option<String>,
}
