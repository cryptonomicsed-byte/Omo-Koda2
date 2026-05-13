use crate::identity::bipon39::Bipon39;
use crate::identity::dna::generate_dna_fingerprint;
use crate::identity::odu::{OduIdentity, OduSeed};
use crate::identity::pet::PetIdentity;
use crate::identity::AgentId;
use crate::justice::JusticeEngine;
use crate::parser::{MetadataPair, Statement};
use crate::providers::ProviderRegistry;
use crate::receipt::{Receipt, ReceiptStore};
use crate::reputation::{tier_for, ReputationChangeReason, ReputationEntry, ReputationLedger};
use crate::session::{ContentBlock, ConversationMessage, MessageRole, PrivateSessionData, Session};
use crate::tools::ToolRegistry;
use ed25519_dalek::SigningKey;
use hkdf::Hkdf;
use omokoda_hermetic::HermeticState;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub receipt: Option<Receipt>,
    pub private_mode: bool,
    pub tool_output: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TurnEvent {
    Started,
    Token(String),
    ToolRequestDetected(String),
    ReceiptGenerated(Receipt),
    Warning(String),
    Error(String),
    Finished,
}

pub type TurnEventSender = mpsc::Sender<TurnEvent>;

pub const AGENT_STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentState {
    pub version: u32,
    id: AgentId,
    name: String,
    birth_timestamp: u64,
    odu_seed: OduSeed,
    odu_identity: OduIdentity,
    pet_identity: PetIdentity,
    dna_fingerprint: String,
    reputation: f64,
    reputation_ledger: ReputationLedger,
    session: Session,
    receipts: ReceiptStore,
    hermetic_state: HermeticState,
    public_key: [u8; 32],
    #[serde(skip)]
    pub private_data: Option<PrivateSessionData>,
}

impl AgentState {
    pub fn birth(name: String, metadata: Vec<MetadataPair>) -> Self {
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
        let receipts = ReceiptStore::new();
        let hermetic_state = HermeticState::from_odu_seed(odu_seed.as_bytes());
        let pet_identity = PetIdentity::derive(&odu_identity, &hermetic_state, 0);

        let dna_fingerprint = generate_dna_fingerprint(&name, birth_timestamp, odu_seed.as_bytes());
        let id = AgentId::new(&dna_fingerprint);

        let mut session = Session::new(id.clone(), name.clone(), birth_timestamp);
        for pair in metadata {
            session.apply_metadata(&pair.key, &pair.value);
        }

        let private_data = PrivateSessionData {
            odu_seed: odu_seed.clone(),
            odu_identity: odu_identity.clone(),
            private_messages: Vec::new(),
        };

        // Derive signing key
        let signing_key = derive_signing_key(&odu_seed);
        let public_key = signing_key.verifying_key().to_bytes();

        Self {
            version: AGENT_STATE_VERSION,
            id,
            name,
            birth_timestamp,
            odu_seed,
            odu_identity,
            pet_identity,
            dna_fingerprint,
            reputation: 0.0,
            reputation_ledger: ReputationLedger::new(),
            session,
            receipts,
            hermetic_state,
            public_key,
            private_data: Some(private_data),
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

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn receipts(&self) -> &ReceiptStore {
        &self.receipts
    }

    pub fn hermetic_state(&self) -> &HermeticState {
        &self.hermetic_state
    }

    pub fn public_key(&self) -> &[u8; 32] {
        &self.public_key
    }

    pub fn private_data(&self) -> Option<&PrivateSessionData> {
        self.private_data.as_ref()
    }

    pub fn signing_key(&self) -> SigningKey {
        derive_signing_key(&self.odu_seed)
    }

    pub fn add_message(&mut self, message: ConversationMessage) {
        if message.is_private {
            if let Some(pd) = &mut self.private_data {
                pd.push_private(message);
            }
        } else {
            self.session.add_message(message);
        }
    }

    pub fn update_reputation(&mut self, new_rep: f64, reason: ReputationChangeReason) {
        let old_rep = self.reputation;
        self.reputation = new_rep.clamp(0.0, 100.0);
        let amount = self.reputation - old_rep;

        self.reputation_ledger.record(ReputationEntry {
            timestamp: current_unix_timestamp(),
            amount,
            reason,
            previous_reputation: old_rep,
            new_reputation: self.reputation,
        });

        self.pet_identity = PetIdentity::derive(&self.odu_identity, &self.hermetic_state, self.tier());
        self.session.reputation = self.reputation;
    }
}

fn derive_signing_key(odu_seed: &OduSeed) -> SigningKey {
    let hk = Hkdf::<Sha256>::new(None, odu_seed.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(b"omokoda-ed25519-v1", &mut okm)
        .expect("HKDF expansion failed");
    SigningKey::from_bytes(&okm)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Steward {
    agent: Option<AgentState>,
    #[serde(skip, default = "ToolRegistry::new")]
    tools: ToolRegistry,
    #[serde(skip, default = "ProviderRegistry::new")]
    providers: ProviderRegistry,
    #[serde(skip, default = "JusticeEngine::new")]
    justice: JusticeEngine,
    #[serde(skip)]
    persistence_path: Option<PathBuf>,
    #[serde(skip)]
    unlock_key: Option<[u8; 32]>,
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
            tools: ToolRegistry::new(),
            providers: ProviderRegistry::new(),
            justice: JusticeEngine::new(),
            persistence_path: None,
            unlock_key: None,
        }
    }

    pub fn set_persistence_path(&mut self, path: PathBuf) {
        self.persistence_path = Some(path);
    }

    pub fn set_mock_provider(&mut self, response: String) {
        self.providers = ProviderRegistry::with_mock(response);
    }

    pub fn register_provider(&mut self, provider: Box<dyn crate::providers::LlmProvider>) {
        self.providers.register(provider);
    }

    pub async fn dispatch(&mut self, stmt: Statement) -> Result<ExecutionResult, String> {
        match stmt {
            Statement::Birth { name, metadata } => {
                let mut agent = AgentState::birth(name, metadata);
                let provider = agent.session.config.default_provider.clone();
                if !provider.is_empty() && !provider.eq_ignore_ascii_case("default") {
                    if !self.providers.is_known_provider(&provider) {
                        return Err(format!("unknown provider '{}' in birth metadata", provider));
                    }
                }
                self.agent = Some(agent);
                self.auto_save();
                Ok(ExecutionResult {
                    receipt: None,
                    private_mode: false,
                    tool_output: None,
                })
            }
            Statement::Think { prompt, private } => {
                if private {
                    let agent = self.ensure_born()?;
                    if agent.private_data.is_none() {
                        return Err("Agent is locked. Unlock first with /unlock <password>".to_string());
                    }
                }

                let agent = self.ensure_born()?;
                let config = &agent.session.config;
                let provider = &config.default_provider;

                let response = self
                    .providers
                    .think(provider, &prompt, &[], private)
                    .await
                    .map_err(|e| format!("Provider error: {}", e))?;

                let current_rep = self.reputation();
                let (new_rep, _) = self.justice.evaluate_think(current_rep, false);

                let agent_mut = self.ensure_born_mut()?;
                agent_mut.add_message(ConversationMessage::new_user(
                    prompt,
                    private,
                ));
                agent_mut.add_message(ConversationMessage::new_assistant(response.clone(), private));

                agent_mut.update_reputation(new_rep, ReputationChangeReason::Think);

                self.auto_save();

                Ok(ExecutionResult {
                    receipt: None,
                    private_mode: private,
                    tool_output: Some(response),
                })
            }
            Statement::Act {
                tool,
                params,
                sandbox,
            } => {
                let agent = self.ensure_born()?;
                let tier = agent.tier();

                if !self.tools.is_allowed(&tool, tier) {
                    return Err(format!(
                        "Tool '{}' requires higher reputation (current tier: {})",
                        tool, tier
                    ));
                }

                // If sandbox requested, verify it's enabled in session config or force it
                let force_sandbox = sandbox || agent.session.config.default_sandbox;

                let output = self
                    .tools
                    .execute(&tool, &params, force_sandbox, tier)
                    .await
                    .map_err(|e| format!("Tool execution failed: {}", e))?;

                // Justice module: Reputation update
                let current_rep = agent.reputation();
                let (new_rep, _) = self.justice.evaluate_action(
                    current_rep,
                    &tool,
                    &params,
                    &output,
                    true,
                );

                let agent_mut = self.ensure_born_mut()?;
                agent_mut.update_reputation(new_rep, ReputationChangeReason::Act);

                // Receipt generation
                let last_hash = agent_mut.receipts.last_hash().to_string();
                let merkle_root = agent_mut.receipts.current_merkle_root();
                let signing_key = agent_mut.signing_key();
                let agent_id = agent_mut.id().clone();
                let receipt = Receipt::new_merkle(&agent_id, &tool, &params, &last_hash, &merkle_root, &signing_key);

                agent_mut.receipts.record(receipt.clone());

                // Session history
                agent_mut.add_message(ConversationMessage {
                    role: MessageRole::Assistant,
                    blocks: vec![ContentBlock::ToolUse {
                        id: receipt.receipt_id.clone(),
                        name: tool.clone(),
                        input: params.clone(),
                    }],
                    is_private: force_sandbox,
                    timestamp: current_unix_timestamp(),
                });

                agent_mut.add_message(ConversationMessage {
                    role: MessageRole::Tool,
                    blocks: vec![ContentBlock::ToolResult {
                        tool_use_id: receipt.receipt_id.clone(),
                        output: output.clone(),
                        is_error: false,
                    }],
                    is_private: force_sandbox,
                    timestamp: current_unix_timestamp(),
                });

                self.auto_save();
                Ok(ExecutionResult {
                    receipt: Some(receipt),
                    private_mode: false,
                    tool_output: Some(output),
                })
            }
            Statement::SlashCmd { command, arg } => {
                match command.as_str() {
                    "status" => {
                        let agent = self.ensure_born()?;
                        let status = format!(
                            "Agent Name: {}\nAgent ID: {}\nTier: {}\nReputation: {:.3}\nDNA: {}\nPet: {}\nReceipts: {}\n",
                            agent.name,
                            agent.id,
                            agent.tier(),
                            agent.reputation,
                            agent.dna_fingerprint,
                            agent.pet_identity.pet(),
                            agent.receipts.count()
                        );
                        Ok(ExecutionResult {
                            receipt: None,
                            private_mode: false,
                            tool_output: Some(status),
                        })
                    }
                    "help" => {
                        let help = "Omokoda CLI Help:\nAvailable commands: birth, think, act, /status, /help, /tools, /private, /publish, /sandbox, /transfer, /configure, /unlock, /seal";
                        Ok(ExecutionResult {
                            receipt: None,
                            private_mode: false,
                            tool_output: Some(help.to_string()),
                        })
                    }
                    "tools" => {
                        let agent = self.ensure_born()?;
                        let tier = agent.tier();
                        let tools = self.tools.list_available(tier);
                        let tools_list = tools.iter().map(|t| format!("- {}", t)).collect::<Vec<_>>().join("\n");
                        let output = format!("Allowed tools for Tier {}:\n{}", tier, tools_list);
                        Ok(ExecutionResult {
                            receipt: None,
                            private_mode: false,
                            tool_output: Some(output),
                        })
                    }
                    "configure" => {
                        let agent = self.ensure_born_mut()?;
                        let arg_str = arg.ok_or_else(|| "configure requires an argument (e.g. provider:mock)".to_string())?;
                        if let Some((key, value)) = arg_str.split_once(':') {
                            match key {
                                "provider" => {
                                    if !self.providers.is_known_provider(value) && !value.eq_ignore_ascii_case("default") {
                                        let available = self.providers.provider_names().join(", ");
                                        return Err(format!("unknown provider '{}'. available: {}", value, available));
                                    }
                                    agent.session.config.default_provider = value.to_string();
                                    self.auto_save();
                                    Ok(ExecutionResult {
                                        receipt: None,
                                        private_mode: false,
                                        tool_output: Some(format!("Configured provider to {}", value)),
                                    })
                                }
                                "privacy" => {
                                    let parsed = match value {
                                        "true" | "on" | "yes" => true,
                                        "false" | "off" | "no" => false,
                                        _ => {
                                            return Err("privacy must be true/on/yes or false/off/no".to_string())
                                        }
                                    };
                                    agent.session.config.default_privacy = parsed;
                                    self.auto_save();
                                    Ok(ExecutionResult {
                                        receipt: None,
                                        private_mode: false,
                                        tool_output: Some(format!("Configured privacy to {}", parsed)),
                                    })
                                }
                                "sandbox" => {
                                    let parsed = match value {
                                        "true" | "on" | "yes" => true,
                                        "false" | "off" | "no" => false,
                                        _ => {
                                            return Err("sandbox must be true/on/yes or false/off/no".to_string())
                                        }
                                    };
                                    agent.session.config.default_sandbox = parsed;
                                    self.auto_save();
                                    Ok(ExecutionResult {
                                        receipt: None,
                                        private_mode: false,
                                        tool_output: Some(format!("Configured sandbox to {}", parsed)),
                                    })
                                }
                                _ => Err(format!("Unknown configuration key: {}", key)),
                            }
                        } else {
                            Err("Invalid configuration format. Use key:value".to_string())
                        }
                    }
                    "seal" => {
                        let password = arg.ok_or_else(|| "seal requires a password".to_string())?;
                        let agent = self.ensure_born_mut()?;
                        
                        let private_data = agent.private_data.take().ok_or_else(|| "agent already sealed".to_string())?;
                        
                        // Deriving a "unlock key" from password for this session
                        let mut key = [0u8; 32];
                        let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
                            password.as_bytes(),
                            &agent.public_key, // using public key as salt
                            100,
                            &mut key,
                        );
                        
                        agent.session.seal_private(&private_data, &agent.odu_seed, &key)?;
                        self.unlock_key = None;
                        self.auto_save();
                        
                        Ok(ExecutionResult {
                            receipt: None,
                            private_mode: false,
                            tool_output: Some("Agent private memory sealed.".to_string()),
                        })
                    }
                    "unlock" => {
                        let password = arg.ok_or_else(|| "unlock requires a password".to_string())?;
                        let agent = self.ensure_born_mut()?;
                        
                        if agent.private_data.is_some() {
                            return Err("agent already unlocked".to_string());
                        }

                        let mut key = [0u8; 32];
                        let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
                            password.as_bytes(),
                            &agent.public_key,
                            100,
                            &mut key,
                        );

                        // Try to unseal
                        let private_data = agent.session.unseal_private(&agent.odu_seed, &key)?;
                        agent.private_data = Some(private_data);
                        self.unlock_key = Some(key);
                        
                        Ok(ExecutionResult {
                            receipt: None,
                            private_mode: false,
                            tool_output: Some("Agent private memory unlocked.".to_string()),
                        })
                    }
                    _ => Err(format!("Slash command '/{}' not yet implemented in Steward", command)),
                }
            }
        }
    }

    pub fn agent_state(&self) -> Option<&AgentState> {
        self.agent.as_ref()
    }

    pub fn reputation(&self) -> f64 {
        self.agent.as_ref().map_or(0.0, |a| a.reputation())
    }

    pub fn tier(&self) -> u8 {
        self.agent.as_ref().map_or(0, |a| a.tier())
    }

    pub fn set_reputation_for_test(&mut self, rep: f64) {
        if let Some(agent) = &mut self.agent {
            agent.update_reputation(rep, ReputationChangeReason::ManualAudit);
            self.auto_save();
        }
    }

    pub fn slash_ethics(&mut self) -> Result<(), String> {
        let current_rep = self.reputation();
        let new_rep = self.justice.check_ethics_violation(current_rep);
        let agent = self.ensure_born_mut()?;
        agent.update_reputation(new_rep, ReputationChangeReason::Violation);
        self.auto_save();
        Ok(())
    }

    pub fn slash_budget(&mut self) -> Result<(), String> {
        let current_rep = self.reputation();
        let new_rep = self.justice.check_budget_overrun(current_rep);
        let agent = self.ensure_born_mut()?;
        agent.update_reputation(new_rep, ReputationChangeReason::BudgetOverrun);
        self.auto_save();
        Ok(())
    }

    pub async fn dispatch_with_event_sink(
        &mut self,
        stmt: Statement,
        mut sink: TurnEventSender,
    ) -> Result<ExecutionResult, String> {
        let _ = sink.send(TurnEvent::Started).await;
        if let Statement::Act { tool, .. } = &stmt {
            let _ = sink.send(TurnEvent::ToolRequestDetected(tool.clone())).await;
        }

        let result = self.dispatch(stmt).await;

        match &result {
            Ok(exec) => {
                if let Some(receipt) = exec.receipt.clone() {
                    let _ = sink.send(TurnEvent::ReceiptGenerated(receipt)).await;
                }
                let _ = sink.send(TurnEvent::Finished).await;
            }
            Err(err) => {
                let _ = sink.send(TurnEvent::Error(err.clone())).await;
                let _ = sink.send(TurnEvent::Finished).await;
            }
        }

        result
    }

    pub fn apply_daily_decay(&mut self, days: u32) {
        if let Some(agent) = &mut self.agent {
            let mut rep = agent.reputation();
            for _ in 0..days {
                rep -= 0.008 + (rep * 0.001); // simplistic decay
            }
            agent.update_reputation(rep, ReputationChangeReason::Decay);
            self.auto_save();
        }
    }

    fn auto_save(&self) {
        if let Some(agent) = &self.agent {
            if let Some(path) = &self.persistence_path {
                if let Ok(content) = serde_json::to_string_pretty(agent) {
                    let _ = std::fs::write(path, content);
                }
            } else {
                // Default save to sessions/<id>.json if no path set
                let dir = std::path::Path::new("sessions");
                if !dir.exists() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let path = dir.join(format!("{}.json", agent.id()));
                if let Ok(content) = serde_json::to_string_pretty(agent) {
                    let _ = std::fs::write(path, content);
                }
            }
        }
    }

    pub fn load_agent(&mut self, agent_id: &AgentId) -> Result<(), String> {
        let path = PathBuf::from("sessions").join(format!("{}.json", agent_id));
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read agent file: {e}"))?;
        let agent: AgentState = serde_json::from_str(&content)
            .map_err(|e| format!("failed to deserialize agent: {e}"))?;

        if agent.version != AGENT_STATE_VERSION {
            return Err(format!(
                "Unsupported agent version: {}. Expected: {}",
                agent.version, AGENT_STATE_VERSION
            ));
        }

        self.agent = Some(agent);
        self.persistence_path = Some(path);
        Ok(())
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
        .expect("Time went backwards")
        .as_secs()
}
