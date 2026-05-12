use crate::identity::dna::generate_dna_fingerprint;
use crate::parser::Statement;
use crate::receipt::{Receipt, ReceiptStore};
use crate::reputation::{reputation_gain, tier_for, tool_allowed, ACT_TIER_0_BASE};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

const ODU_SEED_BYTES: usize = 32;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub receipt: Option<Receipt>,
    pub private_mode: bool,
}

#[derive(Debug, Clone)]
pub struct AgentState {
    id: String,
    name: String,
    birth_timestamp: u64,
    odu_seed: Vec<u8>,
    dna_fingerprint: String,
    reputation: f64,
}

impl AgentState {
    fn birth(name: String) -> Self {
        let birth_timestamp = current_unix_timestamp();
        let mut odu_seed = vec![0u8; ODU_SEED_BYTES];
        rand::thread_rng().fill(&mut odu_seed[..]);
        let dna_fingerprint = generate_dna_fingerprint(&name, birth_timestamp, &odu_seed);
        let id = format!("agent-{}", &dna_fingerprint[..16]);

        Self {
            id,
            name,
            birth_timestamp,
            odu_seed,
            dna_fingerprint,
            reputation: 0.0,
        }
    }

    pub fn id(&self) -> &str {
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

    pub fn odu_seed_len(&self) -> usize {
        self.odu_seed.len()
    }

    pub fn reputation(&self) -> f64 {
        self.reputation
    }

    pub fn tier(&self) -> u8 {
        tier_for(self.reputation)
    }
}

#[derive(Debug, Default)]
pub struct Steward {
    agent: Option<AgentState>,
    receipts: ReceiptStore,
}

impl Steward {
    pub fn new() -> Self {
        Self {
            agent: None,
            receipts: ReceiptStore::new(),
        }
    }

    pub fn dispatch(&mut self, stmt: Statement) -> Result<ExecutionResult, String> {
        match stmt {
            Statement::Birth { name, .. } => {
                self.agent = Some(AgentState::birth(name));
                Ok(ExecutionResult {
                    receipt: None,
                    private_mode: false,
                })
            }
            Statement::Think { private, .. } => {
                self.ensure_born()?;
                Ok(ExecutionResult {
                    receipt: None,
                    private_mode: private,
                })
            }
            Statement::Act { tool, params, .. } => {
                if !tool_allowed(self.tier(), &tool) {
                    return Err(format!("tool requires higher tier: {tool}"));
                }

                let agent_id = self.ensure_born()?.id().to_string();
                let receipt = Receipt::new(&agent_id, &tool, &params);
                self.receipts.record(receipt.clone());
                let agent = self.ensure_born_mut()?;
                agent.reputation = (agent.reputation
                    + reputation_gain(ACT_TIER_0_BASE, agent.reputation))
                .min(100.0);

                Ok(ExecutionResult {
                    receipt: Some(receipt),
                    private_mode: false,
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
            agent.reputation = (agent.reputation - penalty).max(0.0);
        }
    }

    pub fn set_reputation_for_test(&mut self, reputation: f64) {
        if let Some(agent) = self.agent.as_mut() {
            agent.reputation = reputation.clamp(0.0, 100.0);
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
