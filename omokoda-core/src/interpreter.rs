use crate::identity::bipon39::Bipon39;
use crate::identity::dna::generate_dna_fingerprint;
use crate::identity::odu::{OduIdentity, OduSeed};
use crate::identity::pet::PetIdentity;
use crate::parser::Statement;
use crate::receipt::{Receipt, ReceiptStore};
use crate::reputation::{reputation_gain, tier_for, tool_allowed, ACT_TIER_0_BASE};
use crate::tools::ToolRegistry;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub receipt: Option<Receipt>,
    pub private_mode: bool,
    pub tool_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentId(String);

impl AgentId {
    pub fn new(dna_fingerprint: &str) -> Self {
        Self(format!("agent-{}", &dna_fingerprint[..16]))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentState {
    id: AgentId,
    name: String,
    birth_timestamp: u64,
    odu_seed: OduSeed,
    odu_identity: OduIdentity,
    pet_identity: PetIdentity,
    dna_fingerprint: String,
    reputation: f64,
}

impl AgentState {
    pub fn birth(name: String) -> Self {
        let birth_timestamp = current_unix_timestamp();
        let mut entropy = [0u8; 32];
        rand::thread_rng().fill(&mut entropy);

        let mnemonic = Bipon39::entropy_to_mnemonic(&entropy);
        let indices = Bipon39::mnemonic_to_indices(&mnemonic).unwrap();
        let primary_index = Bipon39::get_odu_index(&indices);

        let odu_seed = OduSeed::new(entropy);
        let odu_identity = OduIdentity {
            primary_index,
            mnemonic,
        };
        let pet_identity = PetIdentity::derive(&odu_identity, 0);

        let dna_fingerprint = generate_dna_fingerprint(&name, birth_timestamp, odu_seed.as_bytes());
        let id = AgentId::new(&dna_fingerprint);

        Self {
            id,
            name,
            birth_timestamp,
            odu_seed,
            odu_identity,
            pet_identity,
            dna_fingerprint,
            reputation: 0.0,
        }
    }

    pub fn id(&self) -> &AgentId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn birth_timestamp(&self) -> u64 {
        self.birth_timestamp
    }

    pub fn dna_fingerprint(&self) -> &str {
        &self.dna_fingerprint
    }

    pub fn odu_seed(&self) -> &OduSeed {
        &self.odu_seed
    }

    pub fn odu_identity(&self) -> &OduIdentity {
        &self.odu_identity
    }

    pub fn pet_identity(&self) -> &PetIdentity {
        &self.pet_identity
    }

    pub fn reputation(&self) -> f64 {
        self.reputation
    }

    pub fn tier(&self) -> u8 {
        tier_for(self.reputation)
    }

    pub fn update_reputation(&mut self, new_rep: f64) {
        self.reputation = new_rep.clamp(0.0, 100.0);
        self.pet_identity = PetIdentity::derive(&self.odu_identity, self.tier());
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Steward {
    agent: Option<AgentState>,
    receipts: ReceiptStore,
    #[serde(skip, default = "ToolRegistry::new")]
    tools: ToolRegistry,
}

impl Default for Steward {
    fn default() -> Self {
        Self::new()
    }
}

impl Steward {
    pub fn new() -> Self {
        Self {
            agent: None,
            receipts: ReceiptStore::new(),
            tools: ToolRegistry::new(),
        }
    }

    pub fn dispatch(&mut self, stmt: Statement) -> Result<ExecutionResult, String> {
        match stmt {
            Statement::Birth { name, .. } => {
                self.agent = Some(AgentState::birth(name));
                Ok(ExecutionResult {
                    receipt: None,
                    private_mode: false,
                    tool_output: None,
                })
            }
            Statement::Think { private, .. } => {
                self.ensure_born()?;
                Ok(ExecutionResult {
                    receipt: None,
                    private_mode: private,
                    tool_output: None,
                })
            }
            Statement::Act { tool, params, .. } => {
                if !tool_allowed(self.tier(), &tool) {
                    return Err(format!("tool requires higher tier: {tool}"));
                }

                let agent_id = self.ensure_born()?.id().to_string();
                let last_hash = self.receipts.last_hash().to_string();
                let receipt = Receipt::new(&agent_id, &tool, &params, &last_hash);
                self.receipts.record(receipt.clone());

                let tool_output = match self.tools.execute(&tool, &params) {
                    Ok(output) => Some(output),
                    Err(e) => Some(format!("Error: {}", e)),
                };

                let agent = self.ensure_born_mut()?;
                let new_rep = agent.reputation + reputation_gain(ACT_TIER_0_BASE, agent.reputation);
                agent.update_reputation(new_rep);

                Ok(ExecutionResult {
                    receipt: Some(receipt),
                    private_mode: false,
                    tool_output,
                })
            }
            Statement::SlashCmd { .. } => {
                Err("slash commands are not executable by the Steward".into())
            }
        }
    }

    pub fn agent_state(&self) -> Option<&AgentState> {
        self.agent.as_ref()
    }

    pub fn reputation(&self) -> f64 {
        self.agent_state()
            .map(AgentState::reputation)
            .unwrap_or(0.0)
    }

    pub fn tier(&self) -> u8 {
        self.agent_state().map(AgentState::tier).unwrap_or(0)
    }

    pub fn apply_daily_decay(&mut self, days: u64) {
        if days == 0 {
            return;
        }

        let early_days = days.min(7) as f64;
        let late_days = days.saturating_sub(7) as f64;
        let penalty = (early_days * 0.008) + (late_days * 0.015);
        if let Some(agent) = self.agent.as_mut() {
            let new_rep = agent.reputation - penalty;
            agent.update_reputation(new_rep);
        }
    }

    pub fn set_reputation_for_test(&mut self, reputation: f64) {
        if let Some(agent) = self.agent.as_mut() {
            agent.update_reputation(reputation);
        }
    }

    fn ensure_born(&self) -> Result<&AgentState, String> {
        self.agent
            .as_ref()
            .ok_or_else(|| "agent must be born first".to_string())
    }

    fn ensure_born_mut(&mut self) -> Result<&mut AgentState, String> {
        self.agent
            .as_mut()
            .ok_or_else(|| "agent must be born first".to_string())
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}
