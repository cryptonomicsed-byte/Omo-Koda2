use crate::identity::bipon39::Bipon39;
use crate::identity::dna::generate_dna_fingerprint;
use crate::identity::odu::{OduIdentity, OduSeed};
use crate::identity::pet::PetIdentity;
use crate::justice::JusticeEngine;
use crate::parser::{MetadataPair, Statement};
use crate::providers::ProviderRegistry;
use crate::receipt::{Receipt, ReceiptStore};
use crate::reputation::{reputation_gain, tier_for, ACT_TIER_0_BASE};
use crate::session::{ConversationMessage, PrivateSessionData, Session};
use crate::tools::ToolRegistry;
use ed25519_dalek::SigningKey;
use hkdf::Hkdf;
use omokoda_hermetic::HermeticState;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;
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
    session: Session,
    receipts: ReceiptStore,
    hermetic_state: HermeticState,
    public_key: [u8; 32],
    #[serde(skip)]
    private_data: Option<PrivateSessionData>,
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
            id,
            name,
            birth_timestamp,
            odu_seed,
            odu_identity,
            pet_identity,
            dna_fingerprint,
            reputation: 0.0,
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

    pub fn update_reputation(&mut self, new_rep: f64) {
        self.reputation = new_rep.clamp(0.0, 100.0);
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

    pub async fn dispatch(&mut self, stmt: Statement) -> Result<ExecutionResult, String> {
        match stmt {
            Statement::Birth { name, metadata } => {
                self.agent = Some(AgentState::birth(name, metadata));
                self.auto_save();
                Ok(ExecutionResult {
                    receipt: None,
                    private_mode: false,
                    tool_output: None,
                })
            }
            Statement::Think {
                prompt, private, ..
            } => {
                self.think(prompt, private).await
            }
            Statement::Act { tool, params, sandbox } => {
                self.act(tool, params, sandbox).await
            }
            Statement::SlashCmd { command, arg } => {
                match command.as_str() {
                    "configure" => self.slash_configure(arg),
                    "status" => self.slash_status(),
                    "tools" => self.slash_tools(),
                    "help" => self.slash_help(),
                    "unlock" => self.slash_unlock(arg),
                    "seal" => self.slash_seal(arg),
                    _ => Err(format!("unhandled slash command: /{command}")),
                }
            }
        }
    }

    fn slash_unlock(&mut self, arg: Option<String>) -> Result<ExecutionResult, String> {
        let passphrase = arg.ok_or_else(|| "unlock requires a passphrase".to_string())?;
        let agent = self.ensure_born_mut()?;
        
        let private_data = agent.session.decrypt_private(&passphrase)
            .map_err(|e| format!("failed to unlock private session: {e}"))?;
            
        agent.private_data = Some(private_data);
        
        // Cache the derived key for auto-saving
        let salt = crate::session::generate_salt(&agent.id, agent.birth_timestamp);
        let key = crate::session::derive_key(&passphrase, &salt)
            .map_err(|_| "failed to derive key".to_string())?;
        self.unlock_key = Some(key);
        
        Ok(ExecutionResult {
            receipt: None,
            private_mode: false,
            tool_output: Some("Session unlocked. Private memory is now accessible.".to_string()),
        })
    }

    fn slash_seal(&mut self, arg: Option<String>) -> Result<ExecutionResult, String> {
        if let Some(passphrase) = arg {
            let agent = self.ensure_born_mut()?;
            let salt = crate::session::generate_salt(&agent.id, agent.birth_timestamp);
            let key = crate::session::derive_key(&passphrase, &salt)
                .map_err(|_| "failed to derive key".to_string())?;
            self.unlock_key = Some(key);
        }

        self.seal_private()?;
        Ok(ExecutionResult {
            receipt: None,
            private_mode: false,
            tool_output: Some("Session sealed. Private memory cleared from memory.".to_string()),
        })
    }

    fn seal_private(&mut self) -> Result<(), String> {
        if let (Some(agent), Some(key)) = (&mut self.agent, self.unlock_key) {
            if let Some(private_data) = &agent.private_data {
                agent.session.encrypt_private_with_key(private_data, &key)
                    .map_err(|e| format!("failed to seal private data: {e}"))?;
            }
        }
        if let Some(agent) = &mut self.agent {
            agent.private_data = None;
        }
        self.unlock_key = None;
        self.auto_save();
        Ok(())
    }

    fn slash_configure(&mut self, arg: Option<String>) -> Result<ExecutionResult, String> {
        let arg = arg.ok_or_else(|| "configure requires an argument (e.g., provider:ollama)".to_string())?;
        let parts: Vec<&str> = arg.split(':').collect();
        if parts.len() != 2 {
            return Err("invalid configure format. Use key:value".to_string());
        }
        
        let key = parts[0];
        let value = parts[1];
        
        let agent = self.ensure_born_mut()?;
        agent.session.apply_metadata(key, value);
        self.auto_save();
        
        Ok(ExecutionResult {
            receipt: None,
            private_mode: false,
            tool_output: Some(format!("Configured {key} to {value}")),
        })
    }

    fn slash_status(&self) -> Result<ExecutionResult, String> {
        self.ensure_born()?;
        let agent = self.agent_state().unwrap();
        let status = format!(
            "Agent ID: {}\nName: {}\nReputation: {:.3}\nTier: {}\nMood: {}\nMask: {}",
            agent.id(),
            agent.name(),
            agent.reputation(),
            agent.tier(),
            agent.pet_identity().mood,
            agent.pet_identity().mask
        );
        
        Ok(ExecutionResult {
            receipt: None,
            private_mode: false,
            tool_output: Some(status),
        })
    }

    fn slash_tools(&self) -> Result<ExecutionResult, String> {
        self.ensure_born()?;
        let tier = self.tier();
        let tools = crate::reputation::tools_for_tier(tier);
        let output = format!("Allowed tools for Tier {}:\n- {}", tier, tools.join("\n- "));
        
        Ok(ExecutionResult {
            receipt: None,
            private_mode: false,
            tool_output: Some(output),
        })
    }

    fn slash_help(&self) -> Result<ExecutionResult, String> {
        let help_text = r#"Omokoda CLI Help:
birth "<name>" [metadata...] - Create a new agent
think "<prompt>" [/publish]   - Reasoning turn
act "<tool>" "<params>" [/sandbox] - Execute a tool
/status                      - Show agent status
/tools                       - List allowed tools
/configure <key>:<value>      - Update session config
/help                        - Show this help"#;

        Ok(ExecutionResult {
            receipt: None,
            private_mode: false,
            tool_output: Some(help_text.to_string()),
        })
    }

    pub async fn think(&mut self, prompt: String, private: bool) -> Result<ExecutionResult, String> {
        self.ensure_born()?;
        
        let history = self.agent_state().unwrap().session().public_messages.clone();
        
        // Execute the real reasoning turn via ProviderRouter
        let thought_output = self.providers.route_think(&prompt, &history, private).await?;

        let agent = self.ensure_born_mut()?;
        
        if private {
            let pdata = agent.private_data.as_mut()
                .ok_or_else(|| "session is locked. Use /unlock <passphrase> to access private memory.".to_string())?;
            pdata.push_private(ConversationMessage::user_text(prompt));
            pdata.push_private(ConversationMessage::assistant_text(&thought_output));
        } else {
            // Always push to session for persistent memory (User prompt)
            agent.session.push_public(ConversationMessage::user_text(prompt));
            
            // Push Assistant response (the thought output)
            agent.session.push_public(ConversationMessage::assistant_text(&thought_output));
        }
        
        self.auto_save();
        
        Ok(ExecutionResult {
            receipt: None,
            private_mode: private,
            tool_output: Some(thought_output),
        })
    }

    pub async fn act(&mut self, tool: String, params: String, sandbox: bool) -> Result<ExecutionResult, String> {
        let current_tier = self.tier();

        // If tool is denied or not found, return Err immediately
        let tool_output = self.tools.execute(&tool, &params, sandbox, current_tier).await?;

        let quality = self.justice.evaluate_act(&tool_output, false);
        let multiplier = quality.multiplier();

        let agent = self.ensure_born_mut()?;
        let agent_id = agent.id().to_string();
        let last_hash = agent.receipts.last_hash().to_string();
        let merkle_root = agent.receipts.current_merkle_root();
        let signing_key = agent.signing_key();

        let receipt = Receipt::new_merkle(&agent_id, &tool, &params, &last_hash, &merkle_root, &signing_key);
        agent.receipts.record(receipt.clone());

        let gain = reputation_gain(ACT_TIER_0_BASE, agent.reputation, multiplier);
        let new_rep = agent.reputation + gain;
        agent.update_reputation(new_rep);

        let msg = ConversationMessage {
            role: crate::session::MessageRole::Assistant,
            blocks: vec![crate::session::ContentBlock::ToolUse {
                id: receipt.receipt_id.clone(),
                name: tool,
                input: params,
            }],
        };
        
        let result_msg = ConversationMessage {
            role: crate::session::MessageRole::Assistant,
            blocks: vec![crate::session::ContentBlock::ToolResult {
                tool_use_id: receipt.receipt_id.clone(),
                tool_name: receipt.action.clone(),
                output: tool_output.clone(),
                is_error: false,
            }],
        };

        if sandbox {
            let pdata = agent.private_data.as_mut()
                .ok_or_else(|| "session is locked. Unlock to use /sandbox mode.".to_string())?;
            pdata.push_private(msg);
            pdata.push_private(result_msg);
        } else {
            agent.session.push_public(msg);
            agent.session.push_public(result_msg);
        }

        self.auto_save();

        Ok(ExecutionResult {
            receipt: Some(receipt),
            private_mode: false,
            tool_output: Some(tool_output),
        })
    }

    pub fn slash_ethics(&mut self) -> Result<(), String> {
        let reputation = self.reputation();
        let new_rep = self.justice.check_ethics_violation(reputation);
        let agent = self.ensure_born_mut()?;
        agent.update_reputation(new_rep);
        self.auto_save();
        Ok(())
    }

    pub fn slash_budget(&mut self) -> Result<(), String> {
        let reputation = self.reputation();
        let new_rep = self.justice.check_budget_overrun(reputation);
        let agent = self.ensure_born_mut()?;
        agent.update_reputation(new_rep);
        self.auto_save();
        Ok(())
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
        
        // Sandbox penalty: 0.010 per day
        let sandbox_penalty = days as f64 * 0.010;

        if let Some(agent) = self.agent.as_mut() {
            let new_rep = agent.reputation - penalty - sandbox_penalty;
            agent.update_reputation(new_rep);
        }
        self.auto_save();
    }

    pub fn set_reputation_for_test(&mut self, reputation: f64) {
        if let Some(agent) = self.agent.as_mut() {
            agent.update_reputation(reputation);
        }
        self.auto_save();
    }

    pub fn load_agent(&mut self, agent_id: &str) -> Result<(), String> {
        let path = PathBuf::from("sessions").join(format!("{}.json", agent_id));
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read agent file: {e}"))?;
        let agent: AgentState = serde_json::from_str(&content)
            .map_err(|e| format!("failed to deserialize agent: {e}"))?;
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

    fn auto_save(&self) {
        if let Some(agent) = &self.agent {
            let mut agent_to_save = agent.clone();
            
            // Re-encrypt private data if we have the key and data
            if let (Some(key), Some(pdata)) = (self.unlock_key, &agent.private_data) {
                let _ = agent_to_save.session.encrypt_private_with_key(pdata, &key);
            }

            let path = self.persistence_path.clone().unwrap_or_else(|| {
                let dir = PathBuf::from("sessions");
                if !dir.exists() {
                    let _ = std::fs::create_dir_all(&dir);
                }
                dir.join(format!("{}.json", agent.id()))
            });
            
            if let Ok(encoded) = serde_json::to_string_pretty(&agent_to_save) {
                let _ = std::fs::write(path, encoded);
            }
        }
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}
