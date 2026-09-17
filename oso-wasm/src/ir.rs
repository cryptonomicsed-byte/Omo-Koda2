use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsoIR {
    pub oso_ir_version: String,
    pub contract_class: String,
    pub contract_name: String,
    #[serde(default)]
    pub assets: Vec<AssetSpec>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub minimum_tier: u8,
    pub settlement: SettlementSpec,
    pub policy: PolicySpec,
    #[serde(default)]
    pub lifecycle: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSpec {
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettlementSpec {
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub amount: Option<u64>,
    #[serde(default)]
    pub tithe_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicySpec {
    #[serde(default)]
    pub require_evidence: bool,
    #[serde(default)]
    pub min_witnesses: u8,
    #[serde(default)]
    pub dispute_window_s: Option<u64>,
}
