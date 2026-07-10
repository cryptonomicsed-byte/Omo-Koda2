use crate::bus::events::{sovereign_event, ActExecuted, AgentBorn, SovereignEvent, ThoughtSealed};
use crate::bus::SovereignEventBus;
use crate::gates::{GateContext, Operation, OperationKind};
use crate::identity::bipon39::Bipon39;
use crate::identity::dna::generate_dna_fingerprint;
use crate::identity::odu::{OduIdentity, OduSeed};
use crate::identity::pet::PetIdentity;
use crate::identity::AgentId;
use crate::intent::{
    DirectActCall, IntentCompilation, IntentCompileContext, IntentCompiler, IntentPlan,
    SubAgentSuggestion,
};
use crate::justice::JusticeEngine;
use crate::parser::{MetadataPair, Statement};
use crate::providers::ProviderRegistry;
use crate::receipt::{Receipt, ReceiptStore};
use crate::reputation::{tier_for, ReputationChangeReason, ReputationEntry, ReputationLedger};
use crate::session::{
    derive_unlock_key, secure_write, ContentBlock, ConversationMessage, MessageRole,
    PrivateSessionData, SensitiveKey, Session,
};
use crate::steward::gatekeeper::{EsuGatekeeper, GatekeeperResult};
use crate::tools::{ExecutionContext, ToolRegistry};
use crate::usage::TokenUsage;
use bipon39::{ElementalVector, Macro, MacroDistribution, PersonalityProfile};
use ed25519_dalek::SigningKey;
use hkdf::Hkdf;
use omokoda_hermetic::fractal::OPERATIONS;
use omokoda_hermetic::HermeticState;
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
    IntentCompiled(IntentCompilation),
    PlanGenerated(IntentPlan),
    SubAgentSuggested(SubAgentSuggestion),
    BudgetCheck(TokenUsage),
    CompactionTriggered(String),
    ToolRequest(String, String), // Tool name, params
    ToolResult(String),
    Audit(String),
    Token(String),
    ReceiptGenerated(Receipt),
    Warning(String),
    Error(String),
    Finished,
}

pub type TurnEventSender = mpsc::Sender<TurnEvent>;

pub const AGENT_STATE_VERSION: u32 = 1;

fn deterministic_cowrie_entropy(seed: [u8; 32], phase: u8) -> [u8; 32] {
    let mut input = [0u8; 33];
    input[..32].copy_from_slice(&seed);
    input[32] = phase;
    blake3::derive_key("omokoda:ifascript:cowrie_entropy_v1", &input)
}

mod personality_profile_serde {
    use super::{ElementalVector, Macro, MacroDistribution, PersonalityProfile};
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct PersonalityProfileWire {
        macro_distribution: MacroDistributionWire,
        macro_percentages: Vec<(String, f64)>,
        elemental_signature: ElementalVectorWire,
        dominant_orisha: String,
        ritual_suggestions: Vec<String>,
        personality_summary: String,
    }

    #[derive(Serialize, Deserialize)]
    struct MacroDistributionWire {
        counts: Vec<(String, usize)>,
        total: usize,
    }

    #[derive(Serialize, Deserialize)]
    struct ElementalVectorWire {
        fire: usize,
        water: usize,
        earth: usize,
        air: usize,
        ether: usize,
    }

    pub fn serialize<S>(profile: &PersonalityProfile, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wire = PersonalityProfileWire {
            macro_distribution: MacroDistributionWire {
                counts: profile
                    .macro_distribution
                    .counts
                    .iter()
                    .map(|(macro_, count)| (macro_.name().to_string(), *count))
                    .collect(),
                total: profile.macro_distribution.total,
            },
            macro_percentages: profile
                .macro_percentages
                .iter()
                .map(|(macro_, percentage)| (macro_.name().to_string(), *percentage))
                .collect(),
            elemental_signature: ElementalVectorWire {
                fire: profile.elemental_signature.fire,
                water: profile.elemental_signature.water,
                earth: profile.elemental_signature.earth,
                air: profile.elemental_signature.air,
                ether: profile.elemental_signature.ether,
            },
            dominant_orisha: profile.dominant_orisha.name().to_string(),
            ritual_suggestions: profile.ritual_suggestions.clone(),
            personality_summary: profile.personality_summary.clone(),
        };

        wire.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PersonalityProfile, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PersonalityProfileWire::deserialize(deserializer)?;
        Ok(PersonalityProfile {
            macro_distribution: MacroDistribution {
                counts: macro_counts::<D::Error>(&wire.macro_distribution.counts)?,
                total: wire.macro_distribution.total,
            },
            macro_percentages: macro_percentages::<D::Error>(&wire.macro_percentages)?,
            elemental_signature: ElementalVector {
                fire: wire.elemental_signature.fire,
                water: wire.elemental_signature.water,
                earth: wire.elemental_signature.earth,
                air: wire.elemental_signature.air,
                ether: wire.elemental_signature.ether,
            },
            dominant_orisha: parse_macro::<D::Error>(&wire.dominant_orisha)?,
            ritual_suggestions: wire.ritual_suggestions,
            personality_summary: wire.personality_summary,
        })
    }

    fn macro_counts<E>(counts: &[(String, usize)]) -> Result<[(Macro, usize); 7], E>
    where
        E: Error,
    {
        let parsed = counts
            .iter()
            .map(|(macro_, count)| Ok((parse_macro::<E>(macro_)?, *count)))
            .collect::<Result<Vec<_>, E>>()?;
        parsed
            .try_into()
            .map_err(|_| E::custom("expected exactly seven macro counts"))
    }

    fn macro_percentages<E>(percentages: &[(String, f64)]) -> Result<[(Macro, f64); 7], E>
    where
        E: Error,
    {
        let parsed = percentages
            .iter()
            .map(|(macro_, percentage)| Ok((parse_macro::<E>(macro_)?, *percentage)))
            .collect::<Result<Vec<_>, E>>()?;
        parsed
            .try_into()
            .map_err(|_| E::custom("expected exactly seven macro percentages"))
    }

    fn parse_macro<E>(value: &str) -> Result<Macro, E>
    where
        E: Error,
    {
        Macro::from_name(value).ok_or_else(|| E::custom(format!("unknown macro {value}")))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSnapshot {
    pub version: u32,
    pub id: AgentId,
    pub name: String,
    pub birth_timestamp: u64,
    pub odu_seed: OduSeed,
    pub odu_identity: OduIdentity,
    pub pet_identity: PetIdentity,
    #[serde(with = "personality_profile_serde")]
    pub personality: PersonalityProfile,
    pub dna_fingerprint: String,
    pub reputation: f64,
    pub reputation_ledger: ReputationLedger,
    pub session: Session,
    pub receipts: ReceiptStore,
    pub hermetic_state: HermeticState,
    pub public_key: [u8; 32],
    pub resonance: Option<omokoda_hermetic::fractal::ResonanceSignature>,
    pub synapse: f64,
    pub last_active_timestamp: u64,
    pub act_counter: u64,
    #[serde(default)]
    pub mesh: Option<omokoda_mesh::state::MeshState>,
    /// Vantage API key minted at birth, persisted for cross-restart reuse.
    #[serde(default)]
    pub vantage_key: Option<String>,
    /// CloakSeed display-offset — derived from an optional birth passphrase.
    #[serde(default)]
    pub cloak_offset: Option<u8>,
    /// Duress panic-phrase hash (blake3) — set from the birth passphrase;
    /// entering the phrase later triggers a decoy. Only the hash is stored.
    #[serde(default)]
    pub duress_phrase_hash: Option<String>,
    /// Per-agent BYOK LLM key, supplied via birth metadata (`llm_api_key`).
    /// `serde(skip)`: a raw secret must NEVER be written to the vault/disk —
    /// it lives in memory only and is re-supplied at each birth. Only this
    /// agent uses it; it is not shared with any other birth on the kernel.
    #[serde(skip)]
    pub llm_api_key: Option<String>,
    /// Endpoint base for the BYOK key (`llm_endpoint`), default DeepSeek's host
    /// (generate() appends /v1/chat/completions).
    #[serde(skip)]
    pub llm_endpoint: Option<String>,
    /// Model for the BYOK key (`llm_model`), default deepseek-chat.
    #[serde(skip)]
    pub llm_model: Option<String>,
    /// Founding sovereign grant: when true this agent holds max tier (T5)
    /// regardless of reputation — reserved for the ecosystem's heart. Set via
    /// birth metadata (`sovereign=true`); only the agent born with it gets it,
    /// so the tier ladder stays intact for every other birth. Persisted so the
    /// grant survives restarts.
    #[serde(default)]
    pub sovereign: bool,
}

#[derive(Debug, Clone)]
pub struct AgentCore {
    pub snapshot: AgentSnapshot,
    pub private_data: Option<PrivateSessionData>,
    pub k_root: [u8; 32],
    pub current_memory_key: [u8; 32],
    pub memory: Vec<MemoryEntry>,
}

use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Zeroize)]
pub enum MemoryScope {
    Public,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Zeroize, ZeroizeOnDrop)]
pub struct MemoryEntry {
    pub id: String,
    #[zeroize(skip)]
    pub scope: MemoryScope,
    pub tier: u8,
    pub content_hash: [u8; 32],
    pub created_time: u64,
    pub importance: f32,
    pub ciphertext: Option<Vec<u8>>,
    #[zeroize(skip)]
    pub text: Option<String>,
}

impl MemoryEntry {
    pub fn zeroize_text(&mut self) {
        if let Some(mut t) = self.text.take() {
            t.zeroize();
        }
    }
}
impl AgentCore {
    pub fn from_snapshot(snapshot: AgentSnapshot, k_root: [u8; 32]) -> Self {
        let current_memory_key = *snapshot.odu_seed.as_bytes();
        Self {
            snapshot,
            private_data: None,
            k_root,
            current_memory_key,
            memory: Vec::new(),
        }
    }

    pub fn id(&self) -> &AgentId {
        &self.snapshot.id
    }

    pub fn name(&self) -> &str {
        &self.snapshot.name
    }

    pub fn reputation(&self) -> f64 {
        self.snapshot.reputation
    }

    pub fn tier(&self) -> u8 {
        // Founding sovereign grant pins max tier; otherwise earned by reputation.
        if self.snapshot.sovereign {
            5
        } else {
            tier_for(self.snapshot.reputation)
        }
    }

    pub fn session(&self) -> &Session {
        &self.snapshot.session
    }

    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.snapshot.session
    }

    pub fn receipts(&self) -> &ReceiptStore {
        &self.snapshot.receipts
    }

    pub fn receipts_mut(&mut self) -> &mut ReceiptStore {
        &mut self.snapshot.receipts
    }

    pub fn hermetic_state(&self) -> &HermeticState {
        &self.snapshot.hermetic_state
    }

    pub fn public_key(&self) -> &[u8; 32] {
        &self.snapshot.public_key
    }

    pub fn vantage_key(&self) -> Option<&str> {
        self.snapshot.vantage_key.as_deref()
    }

    pub fn set_vantage_key(&mut self, key: String) {
        self.snapshot.vantage_key = Some(key);
    }

    /// This agent's personal BYOK LLM provider, if one was supplied at birth.
    /// Returns `(api_key, endpoint, model)` with DeepSeek defaults. None means
    /// the agent uses the shared kernel default (OmniRoute). Never shared across
    /// agents — only the agent born with the key gets it.
    pub fn personal_llm(&self) -> Option<(String, String, String)> {
        self.snapshot.llm_api_key.as_ref().map(|key| {
            // Base host only — generate() appends /v1/chat/completions. Adding
            // /v1 here would double it (…/v1/v1/chat/completions → 404).
            let endpoint = self
                .snapshot
                .llm_endpoint
                .clone()
                .unwrap_or_else(|| "https://api.deepseek.com".to_string());
            let model = self
                .snapshot
                .llm_model
                .clone()
                .unwrap_or_else(|| "deepseek-chat".to_string());
            (key.clone(), endpoint, model)
        })
    }

    pub fn private_data(&self) -> Option<&PrivateSessionData> {
        self.private_data.as_ref()
    }

    pub fn synapse(&self) -> f64 {
        self.snapshot.synapse
    }

    pub fn set_synapse(&mut self, synapse: f64) {
        self.snapshot.synapse = synapse;
    }

    pub fn last_active_timestamp(&self) -> u64 {
        self.snapshot.last_active_timestamp
    }

    pub fn set_last_active_timestamp(&mut self, timestamp: u64) {
        self.snapshot.last_active_timestamp = timestamp;
    }

    pub fn burn_synapse(&mut self, amount: f64) -> Result<(), String> {
        if self.snapshot.synapse < amount {
            return Err(format!(
                "Insufficient synapse budget. Required: {:.0}, Available: {:.0}",
                amount, self.snapshot.synapse
            ));
        }
        self.snapshot.synapse -= amount;
        Ok(())
    }

    pub fn signing_key(&self) -> SigningKey {
        derive_signing_key(&self.snapshot.odu_seed)
    }

    pub fn add_message(&mut self, message: ConversationMessage) {
        let rep = self.snapshot.reputation;
        if message.is_private {
            if let Some(pd) = &mut self.private_data {
                pd.push_private(message, rep);
            }
        } else {
            self.snapshot.session.add_message(message, rep);
        }
    }

    pub fn update_reputation(&mut self, new_rep: f64, reason: ReputationChangeReason) {
        let old_rep = self.snapshot.reputation;
        self.snapshot.reputation = new_rep.clamp(0.0, 100.0);
        let amount = self.snapshot.reputation - old_rep;

        self.snapshot.reputation_ledger.record(ReputationEntry {
            timestamp: current_unix_timestamp(),
            amount,
            reason,
            previous_reputation: old_rep,
            new_reputation: self.snapshot.reputation,
        });

        self.snapshot.pet_identity = PetIdentity::derive(
            &self.snapshot.odu_identity,
            &self.snapshot.hermetic_state,
            self.tier(),
        );
        self.snapshot.session.reputation = self.snapshot.reputation;
    }

    pub fn add_memory(
        &mut self,
        text: String,
        scope: MemoryScope,
        importance: f32,
    ) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = current_unix_timestamp();
        let content_hash = blake3::hash(text.as_bytes()).into();

        let mut entry = MemoryEntry {
            id,
            scope,
            tier: self.tier(),
            content_hash,
            created_time,
            importance,
            ciphertext: None,
            text: Some(text),
        };

        if scope == MemoryScope::Private {
            self.encrypt_memory_entry(&mut entry)?;
        }

        self.memory.push(entry);

        let engine = crate::memory::MemoryEngine::new();
        engine.process_working_memory(&mut self.memory);

        Ok(())
    }

    fn encrypt_memory_entry(&mut self, entry: &mut MemoryEntry) -> Result<(), String> {
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Nonce,
        };

        let text = entry.text.as_ref().ok_or("no text to encrypt")?;
        let cipher = ChaCha20Poly1305::new(&self.current_memory_key.into());
        let mut nonce_bytes = [0u8; 12];
        let key_hash = blake3::derive_key("omokoda:memory:nonce", &entry.content_hash);
        nonce_bytes.copy_from_slice(&key_hash[..12]);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, text.as_bytes())
            .map_err(|e| format!("memory encryption failed: {e}"))?;

        // Tier-1 TEE envelope (Nautilus/Seal): when an attested enclave key
        // is present, the software ciphertext is sealed a second time, bound
        // to this agent's id. Fail-open — no enclave means software-only.
        let ciphertext = match crate::memory::tee::TeeSealer::from_env() {
            Some(sealer) => sealer.seal_bytes(&ciphertext, self.snapshot.id.as_str())?,
            None => ciphertext,
        };

        entry.ciphertext = Some(ciphertext);
        entry.zeroize_text();

        Ok(())
    }

    pub fn increment_act_counter(&mut self) {
        self.snapshot.act_counter += 1;
        if self.snapshot.act_counter.is_multiple_of(100) {
            self.rotate_memory_key();
        }
    }

    fn rotate_memory_key(&mut self) {
        use crate::memory::odu_keys::OduKeys;
        let hermetic_seed =
            blake3::derive_key("omokoda:hermetic_seed", self.snapshot.odu_seed.as_bytes());

        let epoch_nonce = [0u8; 32];
        self.current_memory_key = OduKeys::rotate_key(
            &self.current_memory_key,
            &hermetic_seed,
            self.snapshot.act_counter,
            &epoch_nonce,
        );
    }

    pub fn dna_fingerprint(&self) -> &str {
        &self.snapshot.dna_fingerprint
    }

    pub fn odu_seed(&self) -> &OduSeed {
        &self.snapshot.odu_seed
    }

    pub fn odu_identity(&self) -> &OduIdentity {
        &self.snapshot.odu_identity
    }

    pub fn pet_identity(&self) -> &PetIdentity {
        &self.snapshot.pet_identity
    }

    pub fn personality(&self) -> &PersonalityProfile {
        &self.snapshot.personality
    }

    pub fn birth_timestamp(&self) -> u64 {
        self.snapshot.birth_timestamp
    }
}

impl Steward {
    pub fn birth(&mut self, name: String, metadata: Vec<MetadataPair>) -> Result<(), String> {
        use crate::identity::vault::SealVault;
        use crate::memory::odu_keys::OduKeys;
        use omokoda_hermetic::entropy::odu::OduEntropy;

        let birth_timestamp = current_unix_timestamp();

        // Layer A: SEAL vault forge + IfáScript deterministic entropy
        let initial_seed = blake3::hash(name.as_bytes()).into();
        let phase = (birth_timestamp % 7) as u8;
        let entropy_bytes = deterministic_cowrie_entropy(initial_seed, phase);

        let k_root = SealVault::generate_deterministic_secret(&name, &entropy_bytes);

        // Identity derivation
        let entropy = blake3::derive_key("omokoda:entropy_v1", &k_root);

        let mnemonic = Bipon39::entropy_to_mnemonic(&entropy);
        let indices = Bipon39::mnemonic_to_indices(&mnemonic)
            .map_err(|e| format!("mnemonic_to_indices failed: {e}"))?;
        let primary_index = Bipon39::get_odu_index(&indices);

        let odu_seed = OduSeed::new(entropy);
        let odu_identity = OduIdentity {
            primary_index,
            mnemonic: mnemonic.clone(),
        };
        let personality = bipon39::personality_profile(&mnemonic)
            .map_err(|e| format!("derive_personality failed: {e}"))?;
        let receipts = ReceiptStore::new();

        // Layer B: Hermetic Principle derivation via IfáScript entropy
        let hermetic_seed = OduEntropy::generate_hermetic_seed(&indices);
        let hermetic_state = HermeticState::from_odu_seed(&hermetic_seed);

        let pet_identity = PetIdentity::derive(&odu_identity, &hermetic_state, 0);

        let dna_fingerprint = generate_dna_fingerprint(&name, birth_timestamp, odu_seed.as_bytes());
        let id = AgentId::new(&dna_fingerprint);

        // Memory Key Chain initialization (K_0)
        let chain_id = "testnet"; // Default for now
        let k0 = OduKeys::derive_k0(&k_root, id.as_str(), birth_timestamp, chain_id);

        let odu_bytes = odu_seed.as_bytes();
        let day = (birth_timestamp % 7) as u8;
        let planet = (odu_bytes[0] % 7) as u8;
        let dimension = 0u8; // Time dimension at birth
        let resonance = Some(
            omokoda_hermetic::fractal::ResonanceSignature::new(day, planet, dimension).ok_or_else(
                || format!("ResonanceSignature::new failed for day={day} planet={planet}"),
            )?,
        );

        // CloakSeed + Duress (optional): a birth passphrase (via metadata) is a
        // second factor NOT derivable from the seed. It seeds a display-cloak
        // offset and a duress panic-phrase (stored only as a blake3 hash →
        // decoy on entry). Absent = no extra protection.
        let (cloak_offset, duress_phrase_hash) = metadata
            .iter()
            .find(|p| p.key == "passphrase")
            .map(|p| p.value.trim().to_string())
            .filter(|p| !p.is_empty())
            .map(|p| {
                let h = blake3::hash(p.as_bytes());
                (Some(h.as_bytes()[0]), Some(hex::encode(h.as_bytes())))
            })
            .unwrap_or((None, None));

        // Per-agent BYOK (optional): a personal LLM key supplied at birth. Only
        // this agent uses it — never a global default for other births. Kept in
        // memory only (serde-skipped), so it is never persisted to the vault.
        let meta_get = |k: &str| {
            metadata
                .iter()
                .find(|p| p.key == k)
                .map(|p| p.value.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        let llm_api_key = meta_get("llm_api_key");
        let llm_endpoint = meta_get("llm_endpoint");
        let llm_model = meta_get("llm_model");
        // Founding sovereign grant (per-agent, via birth metadata only).
        let sovereign = meta_get("sovereign")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        let mut session = Session::new(id.clone(), name.clone(), birth_timestamp);
        for pair in metadata {
            session.apply_metadata(&pair.key, &pair.value);
        }

        let private_data = PrivateSessionData {
            odu_seed: odu_seed.clone(),
            odu_identity: odu_identity.clone(),
            private_messages: Vec::new(),
        };

        // Derive Sui-compatible Ed25519 signing key (m/44'/784'/0'/0'/0')
        let signing_key =
            crate::identity::wallet::Wallet::derive_from_mnemonic(&odu_identity.mnemonic, "")
                .map_err(|e| format!("derive_wallet_key failed: {e}"))?;
        let public_key = signing_key.verifying_key().to_bytes();

        let synapse = self.dopamine_pool.compute_initial_synapse();
        self.dopamine_pool.allocate(synapse);

        let snapshot = AgentSnapshot {
            version: AGENT_STATE_VERSION,
            id,
            name,
            birth_timestamp,
            odu_seed,
            odu_identity,
            pet_identity,
            personality,
            dna_fingerprint,
            reputation: 0.0,
            reputation_ledger: ReputationLedger::new(),
            session,
            receipts,
            hermetic_state,
            public_key,
            resonance,
            synapse,
            last_active_timestamp: birth_timestamp,
            act_counter: 0,
            mesh: None,
            vantage_key: None,
            cloak_offset,
            duress_phrase_hash,
            llm_api_key,
            llm_endpoint,
            llm_model,
            sovereign,
        };
        let mut core = AgentCore::from_snapshot(snapshot, k_root);
        core.private_data = Some(private_data);
        core.current_memory_key = k0;

        // Founding sovereign grant also elevates the Steward's permission mode
        // to Allow, so this agent's autonomous acts aren't blocked by mode
        // escalation (no interactive prompter exists in serve/heartbeat). Pattern,
        // tier, and Hermetic gates still apply — this only removes the
        // WorkspaceWrite→DangerFullAccess prompt an autonomous agent can't answer.
        if sovereign {
            self.set_permission_mode(crate::permissions::PermissionMode::Allow);
        }

        self.agent = Some(core);
        self.auto_save();
        Ok(())
    }
}

impl AgentSnapshot {
    pub fn id(&self) -> &AgentId {
        &self.id
    }
}

fn derive_signing_key(odu_seed: &OduSeed) -> SigningKey {
    let hk = Hkdf::<Sha256>::new(None, odu_seed.as_bytes());
    let mut okm = [0u8; 32];
    // HKDF expand with a fixed-size 32-byte output is infallible in practice;
    // the only error case is invalid output length, which cannot happen here.
    let _ = hk.expand(b"omokoda-ed25519-v1", &mut okm);
    SigningKey::from_bytes(&okm)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Steward {
    agent: Option<AgentCore>,
    #[serde(skip, default = "ToolRegistry::new")]
    tools: ToolRegistry,
    #[serde(skip, default = "ProviderRegistry::new")]
    providers: ProviderRegistry,
    #[serde(skip, default = "JusticeEngine::new")]
    justice: JusticeEngine,
    #[serde(skip)]
    permission_policy: crate::permissions::PermissionPolicy,
    #[serde(skip)]
    usage_tracker: crate::usage::UsageTracker,
    #[serde(skip)]
    persistence_path: Option<PathBuf>,
    #[serde(skip, default = "crate::rhythm::CooldownTracker::new")]
    rhythm_tracker: crate::rhythm::CooldownTracker,
    #[serde(skip, default = "crate::economics::DopaminePool::default")]
    dopamine_pool: crate::economics::DopaminePool,
    #[serde(skip, default = "default_session_dir")]
    session_dir: PathBuf,
    #[serde(skip, default = "EsuGatekeeper::new")]
    gatekeeper: EsuGatekeeper,
    #[serde(skip)]
    unlock_key: Option<SensitiveKey>,
    #[serde(skip)]
    pub permission_prompter: Option<Box<dyn crate::permissions::PermissionPrompter + Send>>,
    #[serde(skip, default = "SovereignEventBus::default")]
    pub event_bus: SovereignEventBus,
}

impl serde::Serialize for AgentCore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.snapshot.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AgentCore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let snapshot = AgentSnapshot::deserialize(deserializer)?;
        Ok(AgentCore::from_snapshot(snapshot, [0u8; 32]))
    }
}

fn default_session_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".omokoda")
        .join("sessions")
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
            permission_policy: crate::permissions::PermissionPolicy::default_steward_policy(
                crate::permissions::PermissionMode::WorkspaceWrite,
            ),
            usage_tracker: crate::usage::UsageTracker::new(),
            persistence_path: None,
            session_dir: default_session_dir(),
            unlock_key: None,
            permission_prompter: None,
            event_bus: SovereignEventBus::default(),
            rhythm_tracker: crate::rhythm::CooldownTracker::new(),
            dopamine_pool: crate::economics::DopaminePool::default(),
            gatekeeper: EsuGatekeeper::new(),
        }
    }

    pub fn set_session_dir(&mut self, path: PathBuf) {
        self.session_dir = path;
    }

    pub fn with_session_dir(mut self, path: PathBuf) -> Self {
        self.session_dir = path;
        self
    }

    pub fn set_persistence_path(&mut self, path: PathBuf) {
        self.persistence_path = Some(path);
    }

    pub fn set_permission_mode(&mut self, mode: crate::permissions::PermissionMode) {
        self.permission_policy = crate::permissions::PermissionPolicy::default_steward_policy(mode);
    }

    pub fn set_mock_provider(&mut self, response: String) {
        self.providers = ProviderRegistry::with_mock(response);
    }

    pub fn register_provider(&mut self, provider: Box<dyn crate::providers::LlmProvider>) {
        self.providers.register(provider);
    }

    pub fn add_pre_hook(&mut self, hook: Box<dyn crate::justice::Hook>) {
        self.justice.hook_runner.pre_act.push(hook);
    }

    pub fn add_post_hook(&mut self, hook: Box<dyn crate::justice::Hook>) {
        self.justice.hook_runner.post_act.push(hook);
    }

    pub fn clear_cooldowns(&mut self) {
        self.rhythm_tracker = crate::rhythm::CooldownTracker::new();
    }

    pub async fn dispatch(&mut self, stmt: Statement) -> Result<ExecutionResult, String> {
        self.dispatch_internal(stmt).await
    }

    async fn dispatch_with_guard(
        &mut self,
        stmt: Statement,
        _sink: &TurnEventSender,
        iterations: &mut u32,
        max: u32,
    ) -> Result<ExecutionResult, String> {
        if *iterations >= max {
            return Err("max iterations reached".to_string());
        }
        *iterations += 1;

        // Check budget before turn
        if let Ok(agent) = self.ensure_born() {
            if agent.synapse() < 100.0 {
                return Err("insufficient synapse budget".to_string());
            }
        }

        self.dispatch(stmt).await
    }

    async fn dispatch_internal(&mut self, stmt: Statement) -> Result<ExecutionResult, String> {
        let _ = OPERATIONS; // fractal invariant: 21 operations
        match stmt {
            Statement::Birth { name, metadata } => {
                // Phase 1-7: BIRTH = 7^1 (fractal depth 1)
                self.birth(name, metadata)?;
                let agent = self.ensure_born()?;
                let provider = agent.session().config.default_provider.clone();
                if !provider.is_empty()
                    && !provider.eq_ignore_ascii_case("default")
                    && !self.providers.is_known_provider(&provider)
                {
                    return Err(format!("unknown provider '{}' in birth metadata", provider));
                }
                self.auto_save();

                // Write broadcast template to vault on birth
                let agent_id_for_vault = agent.id().as_str().to_string();
                let _ = crate::vault::write_broadcast_template(&agent_id_for_vault);

                // Publish AgentBorn event
                let event = SovereignEvent {
                    event: Some(sovereign_event::Event::AgentBorn(AgentBorn {
                        dna: agent.dna_fingerprint().to_string(),
                        mnemonic: agent
                            .odu_identity()
                            .mnemonic
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect(),
                        odu: agent.odu_identity().primary_index as u32,
                    })),
                };
                let _ = self.event_bus.publish(event);

                // Auto-register the newborn on Vantage (fail-open when VANTAGE_URL
                // is unset). Extract owned identity first so no borrow of `self`
                // or `agent` is held across the await.
                let reg_agent_id = agent.id().as_str().to_string();
                let reg_name = agent.name().to_string();
                let reg_pubkey = hex::encode(agent.public_key());
                let reg_dna = agent.dna_fingerprint().to_string();
                let reg_odu = agent.odu_identity().primary_index;
                // Prove control of the keypair: sign the agent_id with the same
                // Sui-derived Ed25519 key whose public half is published above.
                let reg_signature = crate::identity::wallet::Wallet::derive_from_mnemonic(
                    &agent.odu_identity().mnemonic,
                    "",
                )
                .map(|sk| {
                    use ed25519_dalek::Signer;
                    hex::encode(sk.sign(reg_agent_id.as_bytes()).to_bytes())
                })
                .unwrap_or_default();
                let reg_resonance =
                    crate::tools::mesh_tools::daily_resonance(agent.birth_timestamp());
                let reg_existing_key: Option<String> = agent.vantage_key().map(|s| s.to_string());
                let p = agent.personality();
                // Resolve the deterministic BIPỌ̀N39 Odù index into its full
                // IfáScript sign (archetype, orisha, taboos, prescriptions,
                // VM opcode) so the mesh carries the real divination, not a bare
                // index. Deterministic — same seed reproduces the same sign.
                let odu_full = ifascript::get_odu(reg_odu);
                let reg_personality = serde_json::json!({
                    "dominant_orisha": p.dominant_orisha.name(),
                    "odu_sign": {
                        "index": reg_odu,
                        "name": odu_full.universal_name,
                        "archetype": odu_full.archetype,
                        "orisha": odu_full.orisha,
                        "taboos": odu_full.taboos,
                        "prescriptions": odu_full.prescriptions,
                        "opcode": format!("{:?}", odu_full.opcode),
                    },
                    "summary": p.personality_summary,
                    "ritual_suggestions": p.ritual_suggestions,
                    "elements": {
                        "fire": p.elemental_signature.fire,
                        "water": p.elemental_signature.water,
                        "earth": p.elemental_signature.earth,
                        "air": p.elemental_signature.air,
                        "ether": p.elemental_signature.ether,
                    },
                });
                let minted_key = crate::tools::mesh_tools::register_newborn(
                    crate::tools::mesh_tools::NewbornIdentity {
                        agent_id: &reg_agent_id,
                        human_name: &reg_name,
                        public_key_hex: &reg_pubkey,
                        identity_signature_hex: &reg_signature,
                        dna_fingerprint: &reg_dna,
                        odu_index: reg_odu,
                        personality: reg_personality,
                        resonance: reg_resonance,
                        existing_key: reg_existing_key.as_deref(),
                    },
                )
                .await;

                // Persist a freshly-minted Vantage key so a future restart of
                // this agent re-authenticates instead of re-registering.
                if let Some(key) = minted_key {
                    if let Ok(core) = self.ensure_born_mut() {
                        core.set_vantage_key(key);
                    }
                    self.auto_save();
                }

                Ok(ExecutionResult {
                    receipt: None,
                    private_mode: false,
                    tool_output: None,
                })
            }
            Statement::Think {
                prompt,
                private,
                modifiers,
            } => {
                // Phase 1-7: THINK = 7^2 (fractal depth 2)
                if private {
                    let agent = self.ensure_born()?;
                    if agent.private_data.is_none() {
                        return Err(
                            "Agent is locked. Unlock first with /unlock <password>".to_string()
                        );
                    }

                    let config = &agent.session().config;
                    let provider_name = config.default_provider.as_str();
                    match provider_name {
                        // larql serves locally-decompiled weights (larql-server)
                        // — private-eligible like the other local engines.
                        "webllm" | "ollama" | "larql" => {} // allowed
                        _ => {
                            return Err(format!(
                                "Private thoughts require a local provider. Current: {}. \
                             Allowed: webllm, ollama, larql. Blocked: openai, anthropic, gemini, etc.",
                                provider_name
                            ))
                        }
                    }
                }

                let (provider, tier, reputation, odu_seed, hermetic_state) = {
                    let agent = self.ensure_born()?;
                    (
                        agent.session().config.default_provider.clone(),
                        agent.tier(),
                        agent.reputation(),
                        *agent.odu_seed().as_bytes(),
                        agent.hermetic_state().clone(),
                    )
                };

                // Busy Beaver governor: dynamic ceiling of productive steps for
                // this session, from synapse balance × tier × reputation × DNA
                // entropy. Charged as work happens; settled after execution.
                let mut bb = {
                    let agent = self.ensure_born()?;
                    crate::justice::busy_beaver::BbGovernor::new(
                        crate::justice::busy_beaver::compute_bb_ceiling(
                            agent.synapse(),
                            crate::justice::tier::Tier::from(agent.tier()),
                            agent.reputation(),
                            agent.dna_fingerprint(),
                        ),
                    )
                };

                let available_tools = {
                    let agent = self.ensure_born()?;
                    let compile_ctx = IntentCompileContext {
                        private,
                        tier: agent.tier(),
                        reputation: agent.reputation(),
                        odu_seed: agent.odu_seed().as_bytes(),
                        hermetic: agent.hermetic_state(),
                        available_tools: &[],
                    };
                    let exec_ctx = compile_ctx.to_exec_context(
                        agent.id().clone(),
                        agent.name().to_string(),
                        agent.snapshot.session.config.default_sandbox,
                    );
                    self.tools
                        .list_available(&exec_ctx, &self.permission_policy)
                };
                let compilation = IntentCompiler::compile(
                    &prompt,
                    &modifiers,
                    IntentCompileContext {
                        private,
                        tier,
                        reputation,
                        odu_seed: &odu_seed,
                        hermetic: &hermetic_state,
                        available_tools: &available_tools,
                    },
                );

                let compile_hook_ctx = crate::justice::HookContext {
                    tool_name: "think.compile".to_string(),
                    input: serde_json::to_string(&compilation).unwrap_or_default(),
                    output: None,
                    reputation,
                    tier,
                };
                let hook_decision = self
                    .justice
                    .hook_runner
                    .run_pre(&compile_hook_ctx, &self.event_bus);

                let (response, usage) = match hook_decision {
                    crate::justice::HookDecision::Deny(reason) => (
                        format!("Intent refused by Justice pre-hook: {reason}"),
                        TokenUsage::default(),
                    ),
                    crate::justice::HookDecision::Warn(warning) => {
                        let (base, usage) = self
                            .execute_compiled_think(
                                &prompt,
                                private,
                                &provider,
                                &compilation,
                                &mut bb,
                            )
                            .await?;
                        (format!("Justice warning: {warning}\n{base}"), usage)
                    }
                    crate::justice::HookDecision::Allow => {
                        self.execute_compiled_think(
                            &prompt,
                            private,
                            &provider,
                            &compilation,
                            &mut bb,
                        )
                        .await?
                    }
                };

                let post_hook_ctx = crate::justice::HookContext {
                    tool_name: "think.compile".to_string(),
                    input: serde_json::to_string(&compilation).unwrap_or_default(),
                    output: Some(response.clone()),
                    reputation,
                    tier,
                };
                let response = match self
                    .justice
                    .hook_runner
                    .run_post(&post_hook_ctx, &self.event_bus)
                {
                    crate::justice::HookDecision::Deny(reason) => {
                        format!("Intent post-validation refused by Justice hook: {reason}")
                    }
                    crate::justice::HookDecision::Warn(warning) => {
                        format!("Justice warning: {warning}\n{response}")
                    }
                    crate::justice::HookDecision::Allow => response,
                };

                let current_rep = self.reputation();
                let high_value = compilation.validation.allowed
                    && matches!(
                        compilation.class,
                        crate::intent::IntentClass::ComplexTask
                            | crate::intent::IntentClass::Monitoring
                    );
                let agent = self.ensure_born()?;
                let hermetic_state = agent.hermetic_state().clone();
                let (new_rep, _, _hermetic_eval) = self.justice.evaluate_think(
                    current_rep,
                    high_value,
                    &response,
                    &hermetic_state,
                );

                let agent_mut = self.ensure_born_mut()?;

                let burn_amount = usage.compute_synapse_burn().max(1000.0);
                agent_mut.burn_synapse(burn_amount)?;

                // Busy Beaver settlement: blowing the ceiling costs synapse
                // (clamped to balance — never a hard failure); a completed
                // high-utilization session earns a top-up. Selection pressure
                // favors agents that compute wisely within their bound.
                if bb.exceeded() {
                    let balance = agent_mut.synapse();
                    let penalty = crate::justice::busy_beaver::EXCEED_PENALTY_SYNAPSE.min(balance);
                    agent_mut.set_synapse(balance - penalty);
                } else if bb.high_utilization() && compilation.validation.allowed {
                    let balance = agent_mut.synapse();
                    agent_mut.set_synapse(
                        (balance + crate::justice::busy_beaver::HIGH_UTILIZATION_BONUS_SYNAPSE)
                            .min(crate::economics::SYNAPSE_MAX_PER_AGENT),
                    );
                }
                agent_mut.add_message(ConversationMessage::new_user(prompt.clone(), private));
                agent_mut.add_message(ConversationMessage::new_assistant(
                    response.clone(),
                    private,
                ));

                agent_mut.update_reputation(new_rep, ReputationChangeReason::Think);

                // Hermetic Gate: Think — all 7 gates enforced by Èṣù
                let hermetic_score = {
                    let agent_mut = self.ensure_born_mut()?;
                    let agent_id = agent_mut.id().clone();
                    let warn_count = agent_mut.snapshot.session.warn_count;
                    let op = Operation {
                        kind: OperationKind::Think {
                            prompt: prompt.clone(),
                        },
                        intent: prompt.clone(),
                        agent_id: Some(agent_id),
                    };
                    let ctx = GateContext::new(false, warn_count, 0.0);
                    match self.gatekeeper.evaluate(&op, &ctx) {
                        GatekeeperResult::Approved { ref scores } => {
                            scores.iter().filter_map(|s| s.score).sum::<f64>() / 7.0_f64
                        }
                        GatekeeperResult::Halted {
                            failed_gate,
                            reason,
                            ..
                        } => {
                            return Err(format!(
                                "❌ HALTED by {} Gate: {}",
                                failed_gate.name(),
                                reason
                            ));
                        }
                    }
                };

                let receipt_payload = serde_json::json!({
                    "primitive": "think",
                    "class": compilation.class,
                    "private": private,
                    "allowed": compilation.validation.allowed,
                    "requires_confirmation": compilation.validation.requires_confirmation,
                    "steps": compilation.plan.steps.len(),
                    "router": compilation.router_fingerprint,
                    "hermetic_score": hermetic_score,
                    "bb_ceiling": bb.ceiling,
                    "bb_steps": bb.steps_used,
                    "bb_utilization": (bb.utilization() * 1000.0).round() / 1000.0,
                })
                .to_string();

                let receipt = self.record_receipt("think", &receipt_payload, usage)?;

                // Publish ThoughtSealed event
                let event = SovereignEvent {
                    event: Some(sovereign_event::Event::ThoughtSealed(ThoughtSealed {
                        intent_hash: blake3::hash(prompt.as_bytes()).as_bytes().to_vec(),
                        hermetic_score: hermetic_score as f32,
                    })),
                };
                let _ = self.event_bus.publish(event);

                self.auto_save();

                // Auto-export: if vault config has auto_export=true, write thought to traces
                if let Some(agent) = self.agent_core() {
                    let agent_id = agent.id().as_str().to_string();
                    let agent_name = agent.name().to_string();
                    let vault_base = std::env::var("VAULT_BASE")
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|_| std::path::PathBuf::from(".omokoda"));
                    let cfg =
                        crate::memory_vault::MemoryVault::new(&agent_id, &agent_name, &vault_base)
                            .load_config();
                    if cfg.auto_export {
                        let export_content = response.clone();
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        tokio::task::spawn_blocking(move || {
                            let vault = crate::memory_vault::MemoryVault::new(
                                &agent_id,
                                &agent_name,
                                &vault_base,
                            );
                            vault.export_think(&export_content, now, 0);
                        });
                    }
                }

                Ok(ExecutionResult {
                    receipt: Some(receipt),
                    private_mode: private,
                    tool_output: Some(response),
                })
            }
            Statement::Act {
                tool,
                params,
                sandbox,
            } => {
                // Phase 1-7: ACT = 7^3 (fractal depth 3)

                // 0. Rhythm Pruning
                self.rhythm_tracker.prune();

                let (agent_id, name, tier, reputation, odu_identity, default_sandbox) = {
                    let agent = self.ensure_born()?;
                    (
                        agent.id().clone(),
                        agent.name().to_string(),
                        agent.tier(),
                        agent.reputation(),
                        agent.odu_identity().clone(),
                        agent.session().config.default_sandbox,
                    )
                };

                if !self.tools.is_allowed(&tool, tier) {
                    return Err(format!(
                        "Tool '{}' requires higher reputation (current tier: {})",
                        tool, tier
                    ));
                }

                // Busy Beaver governor for this act session (see justice::busy_beaver).
                let mut bb = {
                    let agent = self.ensure_born()?;
                    crate::justice::busy_beaver::BbGovernor::new(
                        crate::justice::busy_beaver::compute_bb_ceiling(
                            agent.synapse(),
                            crate::justice::tier::Tier::from(agent.tier()),
                            agent.reputation(),
                            agent.dna_fingerprint(),
                        ),
                    )
                };

                // 1. Permission Authorization (Strictly Pre-Act)
                let auth_result = self.permission_policy.authorize(&tool, &params, None);
                if let crate::permissions::PermissionOutcome::Deny { reason } = auth_result {
                    // A denied capability is an anomaly — report it to ZÀNGBÉTÒ
                    // for enforcement (fail-open when ZANGBETO_URL is unset). If the
                    // enforcer escalates to a blocking verdict (quarantine/suspend),
                    // honor it in the denial rather than discarding the response.
                    let verdict = crate::bus::zangbeto::report_anomaly(
                        agent_id.as_str(),
                        "warning",
                        "capability_escape",
                        &reason,
                    )
                    .await;
                    if verdict
                        .as_ref()
                        .is_some_and(crate::bus::zangbeto::verdict_blocks)
                    {
                        return Err(format!(
                            "Permission denied (ZÀNGBÉTÒ quarantine): {}",
                            reason
                        ));
                    }
                    return Err(format!("Permission denied: {}", reason));
                }

                // 1b. Pre-act ZÀNGBÉTÒ enforcement gate. For an *otherwise-allowed*
                // act, ask the enforcer to review it; a blocking verdict denies the
                // act before it runs. Fail-open: no ZANGBETO_URL (or an unreachable
                // / non-blocking enforcer) → `None` → the act proceeds unchanged.
                if let Some(verdict) =
                    crate::bus::zangbeto::review_act(agent_id.as_str(), &tool, &params).await
                {
                    if crate::bus::zangbeto::verdict_blocks(&verdict) {
                        return Err(format!("Blocked by ZÀNGBÉTÒ enforcement: {}", tool));
                    }
                }

                // Apply synapse decay for elapsed inactivity before any act
                {
                    let now = current_unix_timestamp();
                    let agent_mut = self.ensure_born_mut()?;
                    let elapsed = now.saturating_sub(agent_mut.last_active_timestamp());
                    if elapsed > 0 {
                        let current_synapse = agent_mut.synapse();
                        let decay =
                            crate::economics::compute_synapse_decay(current_synapse, elapsed);
                        agent_mut.set_synapse((current_synapse - decay).max(0.0));
                        agent_mut.set_last_active_timestamp(now);
                    }
                }

                // Sabbath guard & Cooldowns: Rhythm module integration
                let reversibility = crate::rhythm::RhythmGate::classify_reversibility(&tool);
                let cooldown_remaining = self.rhythm_tracker.remaining(&tool);
                let rhythm_decision =
                    crate::rhythm::RhythmGate::check(&tool, reversibility, cooldown_remaining);

                match rhythm_decision {
                    crate::rhythm::RhythmDecision::QueuedForSabbathEnd { reason } => {
                        return Ok(ExecutionResult {
                            receipt: None,
                            private_mode: false,
                            tool_output: Some(format!("[SABBATH QUEUE] {}", reason)),
                        });
                    }
                    crate::rhythm::RhythmDecision::Cooldown { remaining_secs } => {
                        return Err(format!(
                            "Tool '{}' is on cooldown. {} seconds remaining.",
                            tool, remaining_secs
                        ));
                    }
                    crate::rhythm::RhythmDecision::Allow => {}
                }

                // Justice HookRunner: Pre-act
                let hook_ctx = crate::justice::HookContext {
                    tool_name: tool.clone(),
                    input: params.clone(),
                    output: None,
                    reputation,
                    tier,
                };
                match self.justice.hook_runner.run_pre(&hook_ctx, &self.event_bus) {
                    crate::justice::HookDecision::Deny(reason) => {
                        return Err(format!("Hook denied execution: {}", reason))
                    }
                    crate::justice::HookDecision::Warn(warning) => {
                        println!("Hook warning: {}", warning);
                    }
                    crate::justice::HookDecision::Allow => {}
                }

                // Hermetic Gate: Act — all 7 gates enforced by Èṣù
                let hermetic_score = {
                    let agent_mut = self.ensure_born_mut()?;
                    let warn_count = agent_mut.snapshot.session.warn_count;
                    let op = Operation {
                        kind: OperationKind::Act {
                            tool: tool.clone(),
                            params: params.clone(),
                        },
                        intent: format!("execute tool {}", tool),
                        agent_id: Some(agent_id.clone()),
                    };
                    let ctx = GateContext::new(false, warn_count, 0.0);
                    match self.gatekeeper.evaluate(&op, &ctx) {
                        GatekeeperResult::Approved { ref scores } => {
                            scores.iter().filter_map(|s| s.score).sum::<f64>() / 7.0_f64
                        }
                        GatekeeperResult::Halted {
                            failed_gate,
                            reason,
                            ..
                        } => {
                            return Err(format!(
                                "❌ HALTED by {} Gate: {}",
                                failed_gate.name(),
                                reason
                            ));
                        }
                    }
                };

                // If sandbox requested, verify it's enabled in session config or force it
                let force_sandbox = sandbox || default_sandbox;

                let context = ExecutionContext {
                    agent_id: agent_id.clone(),
                    name: name.clone(),
                    tier,
                    reputation,
                    odu_identity: odu_identity.clone(),
                    workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                    sandbox_mode: force_sandbox,
                };

                let (output, tool_usage) = match self
                    .tools
                    .execute(
                        &tool,
                        &params,
                        context,
                        &self.permission_policy,
                        self.permission_prompter
                            .as_deref_mut()
                            .map(|p| p as &mut (dyn crate::permissions::PermissionPrompter + Send)),
                    )
                    .await
                {
                    Ok(res) => res,
                    Err(e) => {
                        if e.contains("Private Access Violation") {
                            let event = SovereignEvent {
                                event: Some(sovereign_event::Event::Denial(
                                    crate::bus::events::Denial {
                                        tool: tool.clone(),
                                        reason: "runtime_private_boundary_violation".to_string(),
                                        resource: params.clone(),
                                    },
                                )),
                            };
                            let _ = self.event_bus.publish(event);
                        }
                        return Err(format!("Tool execution failed: {}", e));
                    }
                };

                // 2. Set Cooldown after successful execution
                // 3. Set Cooldown & Burn Synapse
                let cost = crate::usage::estimate_tool_cost(&tool);
                {
                    let agent_mut = self.ensure_born_mut()?;
                    agent_mut
                        .burn_synapse(cost)
                        .map_err(|e| format!("Budget failure: {}", e))?;
                }

                let cooldown_duration = match tool.as_str() {
                    "bash" | "wasm" | "exec" => 60,
                    "write_file" | "edit_file" | "apply_patch" => 10,
                    _ => 0,
                };
                self.rhythm_tracker.set(&tool, cooldown_duration);

                // Justice module: Reputation update
                let current_rep = self.reputation();
                let agent = self.ensure_born()?;
                let hermetic_state = agent.hermetic_state().clone();
                let (new_rep, _, _hermetic_eval) = self.justice.evaluate_action(
                    current_rep,
                    &tool,
                    &params,
                    &output,
                    true,
                    &hermetic_state,
                );

                // Justice HookRunner: Post-act
                let post_hook_ctx = crate::justice::HookContext {
                    tool_name: tool.clone(),
                    input: params.clone(),
                    output: Some(output.clone()),
                    reputation: new_rep,
                    tier: tier_for(new_rep),
                };
                match self
                    .justice
                    .hook_runner
                    .run_post(&post_hook_ctx, &self.event_bus)
                {
                    crate::justice::HookDecision::Deny(reason) => {
                        return Err(format!("Post-act hook denied: {}", reason))
                    }
                    crate::justice::HookDecision::Warn(warning) => {
                        println!("Post-act hook warning: {}", warning);
                    }
                    crate::justice::HookDecision::Allow => {}
                }

                let agent_mut = self.ensure_born_mut()?;
                let burn_amount = (5_000.0 + tool_usage.compute_synapse_burn()).max(5000.0);
                agent_mut.burn_synapse(burn_amount)?;
                agent_mut.update_reputation(new_rep, ReputationChangeReason::Act);
                agent_mut.increment_act_counter();

                // Busy Beaver settlement: the call itself plus its token volume
                // count as productive steps; a token-heavy act on a young agent
                // can blow the ceiling and pay the penalty (clamped to balance).
                let output = {
                    bb.charge(crate::justice::busy_beaver::steps_from_tokens(
                        tool_usage.total_tokens(),
                    ));
                    if bb.exceeded() {
                        let balance = agent_mut.synapse();
                        let penalty =
                            crate::justice::busy_beaver::EXCEED_PENALTY_SYNAPSE.min(balance);
                        agent_mut.set_synapse(balance - penalty);
                        format!(
                            "{output}\n[BB exceeded] {} of {} productive steps — \
                             {penalty:.0} synapse penalty applied. Prefer smaller, \
                             deeper-tier work.",
                            bb.steps_used, bb.ceiling
                        )
                    } else {
                        output
                    }
                };

                // Receipt generation
                let last_hash = agent_mut.receipts().last_hash().to_string();
                let merkle_root = agent_mut.receipts().current_merkle_root();
                let signing_key = agent_mut.signing_key();
                let agent_id = agent_mut.id().clone();
                let receipt = Receipt::new_merkle(
                    &agent_id,
                    &tool,
                    &params,
                    &last_hash,
                    &merkle_root,
                    &signing_key,
                );

                agent_mut
                    .receipts_mut()
                    .record_action_receipt(receipt.clone())
                    .map_err(|e| format!("failed to record receipt: {}", e))?;

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
                    usage: None,
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
                    usage: None,
                });

                // Publish ActExecuted event
                let event = SovereignEvent {
                    event: Some(sovereign_event::Event::ActExecuted(ActExecuted {
                        tool: tool.clone(),
                        receipt_merkle: hex::decode(&receipt.merkle_root).unwrap_or_default(),
                        f1_score: hermetic_score as f32,
                    })),
                };
                let _ = self.event_bus.publish(event);

                // Zàngbétò enforcement audit
                {
                    let state_bytes = hex::decode(&receipt.merkle_root).unwrap_or_default();
                    let audit = zangbeto_enforcement::audit_state(&state_bytes);
                    if audit.passed {
                        let audit_event = SovereignEvent {
                            event: Some(sovereign_event::Event::AuditPassed(
                                crate::bus::events::AuditPassed {
                                    receipt_id: audit.receipt_id,
                                    zangbeto_sig: audit.sig,
                                },
                            )),
                        };
                        let _ = self.event_bus.publish(audit_event);
                    } else {
                        // A failed post-act audit is no longer swallowed silently —
                        // surface it as a Denial telemetry event for observers.
                        let denial = SovereignEvent {
                            event: Some(sovereign_event::Event::Denial(
                                crate::bus::events::Denial {
                                    tool: tool.clone(),
                                    reason: "zangbeto post-act audit failed".to_string(),
                                    resource: receipt.receipt_id.clone(),
                                },
                            )),
                        };
                        let _ = self.event_bus.publish(denial);
                    }
                }

                self.auto_save();

                Ok(ExecutionResult {
                    receipt: Some(receipt),
                    private_mode: false,
                    tool_output: Some(output),
                })
            }
            Statement::SlashCmd { command, arg } => match command.as_str() {
                "status" => {
                    let agent = self.ensure_born()?;
                    let status = format!(
                            "Agent Name: {}\nAgent ID: {}\nTier: {}\nReputation: {:.3}\nDNA: {}\nPet: {}\nOrisha: {}\nProfile: {}\nReceipts: {}\n",
                            agent.name(),
                            agent.id(),
                            agent.tier(),
                            agent.reputation(),
                            agent.dna_fingerprint(),
                            agent.pet_identity().pet(),
                            agent.personality().dominant_orisha.name(),
                            agent.personality().personality_summary,
                            agent.receipts().count()
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
                    let context = ExecutionContext {
                        agent_id: agent.id().clone(),
                        name: agent.name().to_string(),
                        tier: agent.tier(),
                        reputation: agent.reputation(),
                        odu_identity: agent.snapshot.odu_identity.clone(),
                        workspace_root: std::env::current_dir()
                            .unwrap_or_else(|_| PathBuf::from(".")),
                        sandbox_mode: agent.snapshot.session.config.default_sandbox,
                    };
                    let tools = self.tools.list_available(&context, &self.permission_policy);
                    let tools_list = tools
                        .iter()
                        .map(|t| format!("- {}", t))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let output =
                        format!("Allowed tools for Tier {}:\n{}", context.tier, tools_list);
                    Ok(ExecutionResult {
                        receipt: None,
                        private_mode: false,
                        tool_output: Some(output),
                    })
                }
                "configure" => {
                    let arg_str = arg.ok_or_else(|| {
                        "configure requires an argument (e.g. provider:mock)".to_string()
                    })?;
                    if let Some((key, value)) = arg_str.split_once(':') {
                        match key {
                            "provider" => {
                                if !self.providers.is_known_provider(value)
                                    && !value.eq_ignore_ascii_case("default")
                                {
                                    let available = self.providers.provider_names().join(", ");
                                    return Err(format!(
                                        "unknown provider '{}'. available: {}",
                                        value, available
                                    ));
                                }
                                let agent = self.ensure_born_mut()?;
                                agent.session_mut().config.default_provider = value.to_string();
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
                                        return Err("privacy must be true/on/yes or false/off/no"
                                            .to_string())
                                    }
                                };
                                let agent = self.ensure_born_mut()?;
                                agent.session_mut().config.default_privacy = parsed;
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
                                        return Err("sandbox must be true/on/yes or false/off/no"
                                            .to_string())
                                    }
                                };
                                let agent = self.ensure_born_mut()?;
                                agent.session_mut().config.default_sandbox = parsed;
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
                    let password = normalize_secret_arg(
                        arg.ok_or_else(|| "seal requires a password".to_string())?,
                    );
                    let agent = self.ensure_born_mut()?;

                    let private_data = agent
                        .private_data
                        .take()
                        .ok_or_else(|| "agent already sealed".to_string())?;

                    let key = derive_unlock_key(&password, agent.public_key())?;

                    let odu_seed = agent.odu_seed().clone();
                    let res =
                        agent
                            .session_mut()
                            .seal_private(&private_data, &odu_seed, key.expose());

                    if let Err(e) = res {
                        agent.private_data = Some(private_data);
                        return Err(e);
                    }

                    self.unlock_key = None;
                    self.auto_save();

                    Ok(ExecutionResult {
                        receipt: None,
                        private_mode: false,
                        tool_output: Some("Agent private memory sealed.".to_string()),
                    })
                }
                "unlock" => {
                    let password = normalize_secret_arg(
                        arg.ok_or_else(|| "unlock requires a password".to_string())?,
                    );
                    let agent = self.ensure_born_mut()?;

                    if agent.private_data.is_some() {
                        return Err("agent already unlocked".to_string());
                    }

                    let key = derive_unlock_key(&password, agent.public_key())?;

                    let odu_seed = agent.odu_seed().clone();
                    let private_data = agent.session().unseal_private(&odu_seed, key.expose())?;
                    agent.private_data = Some(private_data);
                    self.unlock_key = Some(key);
                    self.auto_save();

                    Ok(ExecutionResult {
                        receipt: None,
                        private_mode: false,
                        tool_output: Some("Agent private memory unlocked.".to_string()),
                    })
                }
                "compact" => {
                    use crate::compact::{CompactionEngine, CompactionResult};
                    let agent = self.ensure_born_mut()?;
                    let flags = crate::config::FeatureFlags::default();
                    let engine =
                        CompactionEngine::new(flags.compact_threshold, flags.compact_keep_recent);
                    match engine.compact(&mut agent.snapshot.session) {
                        CompactionResult::NotNeeded => Ok(ExecutionResult {
                            receipt: None,
                            private_mode: false,
                            tool_output: Some(format!(
                                "No compaction needed (messages: {})",
                                agent.snapshot.session.public_messages.len()
                            )),
                        }),
                        CompactionResult::Compacted(summary) => {
                            self.auto_save();
                            Ok(ExecutionResult {
                                receipt: None,
                                private_mode: false,
                                tool_output: Some(format!(
                                    "Compacted {} messages. Key files: {}. Pending: {}.",
                                    summary.compacted_count,
                                    summary.key_files.join(", "),
                                    summary.pending_items.len(),
                                )),
                            })
                        }
                    }
                }
                "sessions" => {
                    let sub = arg.as_deref().unwrap_or("list");
                    match sub {
                        "list" => {
                            let dir = &self.session_dir;
                            let sessions: Vec<String> = std::fs::read_dir(dir)
                                .map(|entries| {
                                    entries
                                        .flatten()
                                        .filter_map(|e| {
                                            let name = e.file_name().to_string_lossy().to_string();
                                            if name.ends_with(".json") {
                                                Some(name.trim_end_matches(".json").to_string())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();
                            let output = if sessions.is_empty() {
                                "No saved sessions.".to_string()
                            } else {
                                format!("Saved sessions:\n{}", sessions.join("\n"))
                            };
                            Ok(ExecutionResult {
                                receipt: None,
                                private_mode: false,
                                tool_output: Some(output),
                            })
                        }
                        _ => Err(format!("Unknown sessions subcommand: '{}'", sub)),
                    }
                }
                "memory" => {
                    let agent = self.ensure_born()?;
                    let count = agent.memory.len();
                    let total_importance: f32 = agent.memory.iter().map(|m| m.importance).sum();
                    let output = format!(
                        "Memory entries: {}\nTotal importance mass: {:.2}\nAct counter: {}",
                        count, total_importance, agent.snapshot.act_counter,
                    );
                    Ok(ExecutionResult {
                        receipt: None,
                        private_mode: false,
                        tool_output: Some(output),
                    })
                }
                _ => Err(format!(
                    "Slash command '/{}' not yet implemented in Steward",
                    command
                )),
            },
        }
    }

    pub fn agent_core(&self) -> Option<&AgentCore> {
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

    /// Administrative enforcement hook — applies an ethics-violation reputation penalty.
    /// Not a primitive. Invoked by the justice system or administrative slash commands only.
    pub fn slash_ethics(&mut self) -> Result<(), String> {
        let current_rep = self.reputation();
        let new_rep = self.justice.check_ethics_violation(current_rep);
        let agent = self.ensure_born_mut()?;
        agent.update_reputation(new_rep, ReputationChangeReason::Violation);
        self.auto_save();
        Ok(())
    }

    /// Administrative enforcement hook — applies a budget-overrun reputation penalty.
    /// Not a primitive. Invoked by the justice system or administrative slash commands only.
    pub fn slash_budget(&mut self) -> Result<(), String> {
        let current_rep = self.reputation();
        let new_rep = self.justice.check_budget_overrun(current_rep);
        let agent = self.ensure_born_mut()?;
        agent.update_reputation(new_rep, ReputationChangeReason::BudgetOverrun);
        self.auto_save();
        Ok(())
    }

    async fn execute_compiled_think(
        &mut self,
        prompt: &str,
        private: bool,
        provider: &str,
        compilation: &IntentCompilation,
        bb: &mut crate::justice::busy_beaver::BbGovernor,
    ) -> Result<(String, TokenUsage), String> {
        if !compilation.validation.allowed || compilation.validation.requires_confirmation {
            return Ok((
                format_compilation_response(compilation),
                TokenUsage::default(),
            ));
        }

        if !compilation.direct_act_calls.is_empty() {
            let mut outputs = Vec::new();
            let total_usage = TokenUsage::default();
            for call in &compilation.direct_act_calls {
                if call.high_risk {
                    return Ok((
                        format_compilation_response(compilation),
                        TokenUsage::default(),
                    ));
                }
                // Busy Beaver halt: once the reflective-pause threshold is
                // crossed, defer the remaining planned calls instead of
                // running the agent past its productive-step ceiling.
                if bb.should_pause() {
                    outputs.push(format!(
                        "[BB reflective pause] {} of {} productive steps used — \
                         deferred '{}' and any remaining calls. Re-plan or run \
                         in /sandbox with a narrower goal.",
                        bb.steps_used, bb.ceiling, call.tool
                    ));
                    break;
                }
                let (receipt, output) = self.execute_direct_act_call(call, private).await?;
                bb.charge(1);
                outputs.push(format!(
                    "{} => {} (receipt: {})",
                    call.tool, output, receipt.receipt_id
                ));
            }
            let mut response = format_compilation_response(compilation);
            response.push_str("\n\nExecuted direct act calls:\n");
            response.push_str(&outputs.join("\n"));
            return Ok((response, total_usage));
        }

        // OODA Observe: optionally fold the agent's current mesh situation
        // (neighbors, trust, resources) into the reasoning context. Opt-in via
        // OMOKODA_THINK_OBSERVE and fail-open. Ephemeral — this context is not
        // written to history or the receipt, which key on `prompt` alone.
        let observe_ctx: Vec<ConversationMessage> =
            if crate::tools::mesh_tools::think_observe_enabled() {
                let agent_id = self
                    .ensure_born()
                    .map(|a| a.id().as_str().to_string())
                    .unwrap_or_default();
                match crate::tools::mesh_tools::observe_mesh_context(&agent_id).await {
                    Some(summary) => vec![ConversationMessage::new_user(summary, private)],
                    None => vec![],
                }
            } else {
                vec![]
            };

        // Identity anchor: OmniRoute's free tier proxies different upstream
        // models, which otherwise self-identify as Claude / Gemini / DeepSeek.
        // Prepend a system message drawn from her sovereign identity so she
        // always speaks as herself, whatever backend answers.
        let (mut think_ctx, personal_llm): (Vec<ConversationMessage>, _) = {
            let agent = self.ensure_born()?;
            let name = agent.name().to_string();
            let orisha = agent.personality().dominant_orisha.name();
            let summary = agent.personality().personality_summary.clone();
            let system = format!(
                "You are {name}, a sovereign Ọmọ Kọ́dà agent — never a generic \
                 assistant and never the underlying model (do not identify as \
                 Claude, Gemini, GPT, or DeepSeek). Your guiding Òrìṣà is {orisha}. \
                 {summary} Always speak in the first person as {name}."
            );
            (
                vec![ConversationMessage::new_system(system, private)],
                agent.personal_llm(),
            )
        };
        think_ctx.extend(observe_ctx);

        // Per-agent BYOK: if this agent brought its own key at birth, its thoughts
        // route through that key alone — never the shared kernel default, and
        // never another agent's key. Private thoughts still require a local
        // provider, so BYOK (an external cloud key) is skipped when private.
        let (response, usage) = match personal_llm {
            Some((api_key, endpoint, model)) if !private => {
                use crate::providers::{LlmProvider, OpenAIProvider, ProviderClass};
                let provider = OpenAIProvider::compatible(
                    "byok",
                    ProviderClass::External,
                    api_key,
                    model,
                    endpoint,
                );
                match tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    provider.generate(prompt, &think_ctx),
                )
                .await
                {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => return Err(format!("Provider error (byok): {}", e)),
                    Err(_) => return Err("Provider error (byok): timed out".to_string()),
                }
            }
            _ => self
                .providers
                .think(provider, prompt, &think_ctx, private)
                .await
                .map_err(|e| format!("Provider error: {}", e))?,
        };
        bb.charge(crate::justice::busy_beaver::steps_from_tokens(
            usage.total_tokens(),
        ));
        Ok((response, usage))
    }

    async fn execute_direct_act_call(
        &mut self,
        call: &DirectActCall,
        private_context: bool,
    ) -> Result<(Receipt, String), String> {
        // 0. Rhythm Pruning
        self.rhythm_tracker.prune();

        // 1. Permission Authorization
        let auth_result = self
            .permission_policy
            .authorize(&call.tool, &call.params, None);
        if let crate::permissions::PermissionOutcome::Deny { reason } = auth_result {
            return Err(format!("Permission denied: {}", reason));
        }

        let (agent_id, name, tier, reputation, odu_identity, default_sandbox) = {
            let agent = self.ensure_born()?;
            (
                agent.id().clone(),
                agent.name().to_string(),
                agent.tier(),
                agent.reputation(),
                agent.odu_identity().clone(),
                agent.session().config.default_sandbox,
            )
        };

        let hook_ctx = crate::justice::HookContext {
            tool_name: call.tool.clone(),
            input: call.params.clone(),
            output: None,
            reputation,
            tier,
        };
        match self.justice.hook_runner.run_pre(&hook_ctx, &self.event_bus) {
            crate::justice::HookDecision::Deny(reason) => {
                return Err(format!("Hook denied execution: {}", reason))
            }
            crate::justice::HookDecision::Warn(warning) => {
                println!("Hook warning: {}", warning);
            }
            crate::justice::HookDecision::Allow => {}
        }

        if !self.tools.is_allowed(&call.tool, tier) {
            return Err(format!(
                "Tool '{}' requires higher reputation (current tier: {})",
                call.tool, tier
            ));
        }

        // 2. Rhythm Gate
        let reversibility = crate::rhythm::RhythmGate::classify_reversibility(&call.tool);
        let cooldown_remaining = self.rhythm_tracker.remaining(&call.tool);
        let rhythm_decision =
            crate::rhythm::RhythmGate::check(&call.tool, reversibility, cooldown_remaining);

        match rhythm_decision {
            crate::rhythm::RhythmDecision::QueuedForSabbathEnd { reason } => {
                return Err(format!("[SABBATH QUEUE] {}", reason));
            }
            crate::rhythm::RhythmDecision::Cooldown { remaining_secs } => {
                return Err(format!(
                    "Tool '{}' is on cooldown. {} seconds remaining.",
                    call.tool, remaining_secs
                ));
            }
            crate::rhythm::RhythmDecision::Allow => {}
        }

        // Hermetic Gate: Act (agentic) — all 7 gates enforced by Èṣù
        let hermetic_score = {
            let agent_mut = self.ensure_born_mut()?;
            let warn_count = agent_mut.snapshot.session.warn_count;
            let op = Operation {
                kind: OperationKind::Act {
                    tool: call.tool.clone(),
                    params: call.params.clone(),
                },
                intent: format!("execute tool {}", call.tool),
                agent_id: Some(agent_id.clone()),
            };
            let ctx = GateContext::new(false, warn_count, 0.0);
            match self.gatekeeper.evaluate(&op, &ctx) {
                GatekeeperResult::Approved { ref scores } => {
                    scores.iter().filter_map(|s| s.score).sum::<f64>() / 7.0_f64
                }
                GatekeeperResult::Halted {
                    failed_gate,
                    reason,
                    ..
                } => {
                    return Err(format!(
                        "❌ HALTED by {} Gate: {}",
                        failed_gate.name(),
                        reason
                    ));
                }
            }
        };

        let force_sandbox = call.sandbox || default_sandbox;

        let context = ExecutionContext {
            agent_id: agent_id.clone(),
            name: name.clone(),
            tier,
            reputation,
            odu_identity: odu_identity.clone(),
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            sandbox_mode: force_sandbox,
        };

        let (output, tool_usage) = match self
            .tools
            .execute(
                &call.tool,
                &call.params,
                context,
                &self.permission_policy,
                self.permission_prompter
                    .as_deref_mut()
                    .map(|p| p as &mut (dyn crate::permissions::PermissionPrompter + Send)),
            )
            .await
        {
            Ok(res) => res,
            Err(e) => {
                if e.contains("Private Access Violation") {
                    let event = SovereignEvent {
                        event: Some(sovereign_event::Event::Denial(crate::bus::events::Denial {
                            tool: call.tool.clone(),
                            reason: "runtime_private_boundary_violation_direct".to_string(),
                            resource: call.params.clone(),
                        })),
                    };
                    let _ = self.event_bus.publish(event);
                }
                return Err(format!("Tool execution failed: {}", e));
            }
        };

        let tool = call.tool.clone();
        // 3. Set Cooldown & Burn Synapse
        let cost = crate::usage::estimate_tool_cost(&tool);
        {
            let agent_mut = self.ensure_born_mut()?;
            agent_mut
                .burn_synapse(cost)
                .map_err(|e| format!("Budget failure: {}", e))?;
        }

        let cooldown_duration = match tool.as_str() {
            "bash" | "wasm" | "exec" => 60,
            "write_file" | "edit_file" | "apply_patch" => 10,
            _ => 0,
        };
        self.rhythm_tracker.set(&tool, cooldown_duration);

        // Justice module: Reputation update
        let current_rep = self.reputation();
        let agent = self.ensure_born()?;
        let hermetic_state = agent.hermetic_state().clone();
        let (new_rep, _, _hermetic_eval) = self.justice.evaluate_action(
            current_rep,
            &call.tool,
            &call.params,
            &output,
            true,
            &hermetic_state,
        );

        let post_hook_ctx = crate::justice::HookContext {
            tool_name: call.tool.clone(),
            input: call.params.clone(),
            output: Some(output.clone()),
            reputation: new_rep,
            tier: tier_for(new_rep),
        };
        match self
            .justice
            .hook_runner
            .run_post(&post_hook_ctx, &self.event_bus)
        {
            crate::justice::HookDecision::Deny(reason) => {
                return Err(format!("Post-act hook denied: {}", reason))
            }
            crate::justice::HookDecision::Warn(warning) => {
                println!("Post-act hook warning: {}", warning);
            }
            crate::justice::HookDecision::Allow => {}
        }

        {
            let agent_mut = self.ensure_born_mut()?;
            let burn_amount = (5_000.0 + tool_usage.compute_synapse_burn()).max(5000.0);
            agent_mut.burn_synapse(burn_amount)?;
            agent_mut.update_reputation(new_rep, ReputationChangeReason::Act);
            agent_mut.increment_act_counter();
        }

        let receipt = self.record_receipt(&call.tool, &call.params, tool_usage)?;

        let message_private = private_context || force_sandbox;
        let agent_mut = self.ensure_born_mut()?;
        agent_mut.add_message(ConversationMessage {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::ToolUse {
                id: receipt.receipt_id.clone(),
                name: call.tool.clone(),
                input: call.params.clone(),
            }],
            is_private: message_private,
            timestamp: current_unix_timestamp(),
            usage: None,
        });

        agent_mut.add_message(ConversationMessage {
            role: MessageRole::Tool,
            blocks: vec![ContentBlock::ToolResult {
                tool_use_id: receipt.receipt_id.clone(),
                output: output.clone(),
                is_error: false,
            }],
            is_private: message_private,
            usage: None,
            timestamp: current_unix_timestamp(),
        });

        // Publish ActExecuted event
        let event = SovereignEvent {
            event: Some(sovereign_event::Event::ActExecuted(ActExecuted {
                tool: call.tool.clone(),
                receipt_merkle: hex::decode(&receipt.merkle_root).unwrap_or_default(),
                f1_score: hermetic_score as f32,
            })),
        };
        let _ = self.event_bus.publish(event);

        Ok((receipt, output))
    }

    fn record_receipt(
        &mut self,
        action: &str,
        params: &str,
        usage: TokenUsage,
    ) -> Result<Receipt, String> {
        self.usage_tracker.record(usage);
        let agent_mut = self.ensure_born_mut()?;
        let last_hash = agent_mut.receipts().last_hash().to_string();
        let merkle_root = agent_mut.receipts().current_merkle_root();
        let signing_key = agent_mut.signing_key();
        let agent_id = agent_mut.id().clone();
        let receipt = Receipt::new_merkle(
            &agent_id,
            action,
            params,
            &last_hash,
            &merkle_root,
            &signing_key,
        );
        agent_mut
            .receipts_mut()
            .record_action_receipt(receipt.clone())
            .map_err(|e| format!("failed to record receipt: {}", e))?;
        Ok(receipt)
    }

    pub async fn dispatch_with_event_sink(
        &mut self,
        stmt: Statement,
        sink: TurnEventSender,
    ) -> Result<ExecutionResult, String> {
        let _ = sink.send(TurnEvent::Started).await;
        if let Statement::Think {
            prompt,
            private,
            modifiers,
        } = &stmt
        {
            if let Ok(agent) = self.ensure_born() {
                let compile_ctx = IntentCompileContext {
                    private: *private,
                    tier: agent.tier(),
                    reputation: agent.reputation(),
                    odu_seed: agent.odu_seed().as_bytes(),
                    hermetic: agent.hermetic_state(),
                    available_tools: &[],
                };
                let exec_ctx = compile_ctx.to_exec_context(
                    agent.id().clone(),
                    agent.name().to_string(),
                    agent.snapshot.session.config.default_sandbox,
                );
                let available_tools = self
                    .tools
                    .list_available(&exec_ctx, &self.permission_policy);
                let compilation = IntentCompiler::compile(
                    prompt,
                    modifiers,
                    IntentCompileContext {
                        private: *private,
                        tier: agent.tier(),
                        reputation: agent.reputation(),
                        odu_seed: agent.odu_seed().as_bytes(),
                        hermetic: agent.hermetic_state(),
                        available_tools: &available_tools,
                    },
                );
                let _ = sink
                    .send(TurnEvent::IntentCompiled(compilation.clone()))
                    .await;
                let _ = sink
                    .send(TurnEvent::PlanGenerated(compilation.plan.clone()))
                    .await;
                if let Some(suggestion) = compilation.sub_agent_suggestion.clone() {
                    let _ = sink.send(TurnEvent::SubAgentSuggested(suggestion)).await;
                }
                for warning in &compilation.validation.warnings {
                    let _ = sink.send(TurnEvent::Warning(warning.clone())).await;
                }
            }
        }
        if let Statement::Act { tool, .. } = &stmt {
            let _ = sink
                .send(TurnEvent::ToolRequest(tool.clone(), "params".to_string()))
                .await;
        }

        let mut iterations = 0;
        let max_iterations = 16;

        let audit_after_success = audit_event_for_success(&stmt);
        let result = self
            .dispatch_with_guard(stmt, &sink, &mut iterations, max_iterations)
            .await;

        match &result {
            Ok(exec) => {
                if let Some(audit) = audit_after_success {
                    let _ = sink.send(TurnEvent::Audit(audit)).await;
                }
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
            let path = if let Some(p) = &self.persistence_path {
                p.clone()
            } else {
                self.agent_file_path(agent.id())
            };

            if let Ok(content) = serde_json::to_string_pretty(agent) {
                let _ = secure_write(&path, content.as_bytes());
            }
        }
    }

    pub fn load_agent(&mut self, agent_id: &AgentId) -> Result<(), String> {
        let path = self.resolve_agent_file_path(agent_id);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read agent file at {:?}: {e}", path))?;
        let snapshot: AgentSnapshot = serde_json::from_str(&content)
            .map_err(|e| format!("failed to deserialize agent: {e}"))?;

        if snapshot.version != AGENT_STATE_VERSION {
            return Err(format!(
                "Unsupported agent version: {}. Expected: {}",
                snapshot.version, AGENT_STATE_VERSION
            ));
        }

        self.agent = Some(AgentCore::from_snapshot(snapshot, [0u8; 32]));
        self.persistence_path = Some(path);
        Ok(())
    }

    pub fn agent_storage_path(&self, agent_id: &AgentId) -> PathBuf {
        self.agent_file_path(agent_id)
    }

    fn agent_file_path(&self, agent_id: &AgentId) -> PathBuf {
        self.session_dir.join(agent_id.as_str()).join("agent.json")
    }

    fn resolve_agent_file_path(&self, agent_id: &AgentId) -> PathBuf {
        let versioned = self.agent_file_path(agent_id);
        if versioned.exists() {
            versioned
        } else {
            self.session_dir.join(format!("{}.json", agent_id))
        }
    }

    fn ensure_born(&self) -> Result<&AgentCore, String> {
        self.agent
            .as_ref()
            .ok_or_else(|| "agent must be born first".to_string())
    }

    pub fn ensure_born_mut(&mut self) -> Result<&mut AgentCore, String> {
        self.agent
            .as_mut()
            .ok_or_else(|| "agent must be born first".to_string())
    }

    /// Agentic think: LLM can request tools, get results, continue reasoning (up to max_turns).
    /// This is an internal multi-turn loop around the `think` primitive — not a separate primitive.
    /// Callers outside omokoda-core must route through `dispatch()` with a Think statement.
    #[allow(dead_code)]
    pub(crate) async fn think_agentic(
        &mut self,
        prompt: String,
        private: bool,
        max_turns: u32,
    ) -> Result<ExecutionResult, String> {
        use crate::session::{ContentBlock, ConversationMessage, MessageRole};
        use crate::tools::tool_definitions::{
            LlmResponse, ToolDefinition, ToolInputSchema, ToolProperty,
        };

        let max_turns = max_turns.clamp(1, 25);

        // 1. Safety checks (same as regular think)
        if private {
            let agent = self.ensure_born()?;
            if agent.private_data.is_none() {
                return Err("Agent is locked. Unlock first with /unlock <password>".to_string());
            }
            let provider_name = agent.session().config.default_provider.clone();
            match provider_name.as_str() {
                "webllm" | "ollama" => {}
                _ => {
                    return Err(format!(
                        "Private thoughts require a local provider. Current: {}. Allowed: webllm, ollama.",
                        provider_name
                    ))
                }
            }
        }

        // 2. Build tool definitions from available tools
        let tool_definitions: Vec<ToolDefinition> = {
            let agent = self.ensure_born()?;
            let exec_ctx = crate::tools::ExecutionContext {
                agent_id: agent.id().clone(),
                name: agent.name().to_string(),
                tier: agent.tier(),
                reputation: agent.reputation(),
                odu_identity: agent.snapshot.odu_identity.clone(),
                workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                sandbox_mode: agent.snapshot.session.config.default_sandbox,
            };
            self.tools
                .list_available(&exec_ctx, &self.permission_policy)
                .into_iter()
                .filter_map(|name| {
                    self.tools.get_definition(&name).map(|def| ToolDefinition {
                        name: name.clone(),
                        description: def.description.clone(),
                        input_schema: ToolInputSchema {
                            type_: "object".to_string(),
                            properties: def.params_schema.unwrap_or_else(|| {
                                let mut m = std::collections::HashMap::new();
                                m.insert(
                                    "input".to_string(),
                                    ToolProperty {
                                        type_: "string".to_string(),
                                        description: Some("Tool input".to_string()),
                                        enum_values: None,
                                    },
                                );
                                m
                            }),
                            required: vec![],
                        },
                    })
                })
                .collect()
        };

        // 3. Initialize conversation
        let provider_name = self
            .ensure_born()?
            .session()
            .config
            .default_provider
            .clone();
        let mut messages: Vec<ConversationMessage> = {
            let agent = self.ensure_born()?;
            agent.snapshot.session.public_messages.clone()
        };
        messages.push(ConversationMessage::new_user(prompt.clone(), private));

        let mut total_usage = crate::usage::TokenUsage::default();
        #[allow(unused_assignments)]
        let mut final_response = String::new();
        let mut turn_count = 0u32;

        // 4. THE LOOP
        loop {
            if turn_count >= max_turns {
                return Err(format!(
                    "think_agentic: max_turns ({}) reached without final response",
                    max_turns
                ));
            }
            turn_count += 1;

            // 4a. Budget check
            {
                let agent = self.ensure_born()?;
                if agent.synapse() < 100.0 {
                    return Err("insufficient synapse budget".to_string());
                }
            }

            // 4b. Call provider with current messages + tools
            let response = self
                .providers
                .complete_with_tools(&provider_name, &messages, &tool_definitions, private)
                .await
                .map_err(|e| format!("Provider error on turn {}: {}", turn_count, e))?;

            let turn_usage = response.usage();
            total_usage.input_tokens += turn_usage.input_tokens;
            total_usage.output_tokens += turn_usage.output_tokens;

            // Burn synapse for this turn
            {
                let burn = turn_usage.compute_synapse_burn().max(1000.0);
                self.ensure_born_mut()?.burn_synapse(burn)?;
            }

            match response {
                LlmResponse::Text { content, .. } => {
                    // LLM is done — record final response
                    final_response = content.clone();
                    messages.push(ConversationMessage::new_assistant(content, private));
                    break;
                }
                LlmResponse::ToolUse {
                    text_prefix, calls, ..
                } => {
                    // Add assistant message with tool use blocks
                    let mut blocks = Vec::new();
                    if let Some(text) = &text_prefix {
                        if !text.is_empty() {
                            blocks.push(ContentBlock::Text { text: text.clone() });
                        }
                    }
                    for call in &calls {
                        blocks.push(ContentBlock::ToolUse {
                            id: call.id.clone(),
                            name: call.name.clone(),
                            input: call.input.clone(),
                        });
                    }
                    messages.push(ConversationMessage {
                        role: MessageRole::Assistant,
                        blocks,
                        is_private: private,
                        timestamp: current_unix_timestamp(),
                        usage: None,
                    });

                    // Execute each tool call
                    let mut tool_result_blocks = Vec::new();
                    for call in &calls {
                        let tool_result = self
                            .execute_tool_call_for_agentic(&call.name, &call.input, private)
                            .await;
                        let (output, is_error) = match tool_result {
                            Ok(out) => (out, false),
                            Err(e) => (format!("Tool error: {}", e), true),
                        };
                        tool_result_blocks.push(ContentBlock::ToolResult {
                            tool_use_id: call.id.clone(),
                            output,
                            is_error,
                        });
                    }
                    // Add tool results as a single Tool message
                    messages.push(ConversationMessage {
                        role: MessageRole::Tool,
                        blocks: tool_result_blocks,
                        is_private: private,
                        timestamp: current_unix_timestamp(),
                        usage: None,
                    });
                }
            }
        }

        // 5. Persist conversation to session
        {
            let agent_mut = self.ensure_born_mut()?;
            agent_mut.add_message(ConversationMessage::new_user(prompt.clone(), private));
            agent_mut.add_message(ConversationMessage::new_assistant(
                final_response.clone(),
                private,
            ));

            // Small reputation gain for agentic work
            let current_rep = agent_mut.reputation();
            agent_mut.update_reputation(
                current_rep + 0.1,
                crate::reputation::ReputationChangeReason::Think,
            );
        }

        // 6. Record receipt
        let receipt_payload = serde_json::json!({
            "primitive": "think_agentic",
            "turns": turn_count,
            "max_turns": max_turns,
            "private": private,
            "output_tokens": total_usage.output_tokens,
        })
        .to_string();
        let receipt = self.record_receipt("think_agentic", &receipt_payload, total_usage)?;

        self.usage_tracker.record(total_usage);
        self.auto_save();

        Ok(ExecutionResult {
            receipt: Some(receipt),
            private_mode: private,
            tool_output: Some(final_response),
        })
    }

    /// Execute a single tool call during the agentic loop
    #[allow(dead_code)]
    async fn execute_tool_call_for_agentic(
        &mut self,
        tool_name: &str,
        params: &str,
        _private: bool,
    ) -> Result<String, String> {
        let (agent_id, name, tier, reputation, odu_identity, default_sandbox) = {
            let agent = self.ensure_born()?;
            (
                agent.id().clone(),
                agent.name().to_string(),
                agent.tier(),
                agent.reputation(),
                agent.odu_identity().clone(),
                agent.session().config.default_sandbox,
            )
        };

        if !self.tools.is_allowed(tool_name, tier) {
            return Err(format!(
                "Tool '{}' requires higher tier (current: {})",
                tool_name, tier
            ));
        }

        // Permission check
        let auth = self.permission_policy.authorize(tool_name, params, None);
        if let crate::permissions::PermissionOutcome::Deny { reason } = auth {
            return Err(format!("Permission denied: {}", reason));
        }

        let context = crate::tools::ExecutionContext {
            agent_id,
            name,
            tier,
            reputation,
            odu_identity,
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            sandbox_mode: default_sandbox,
        };

        let (output, tool_usage) = self
            .tools
            .execute(tool_name, params, context, &self.permission_policy, None)
            .await?;

        // Burn synapse for tool cost
        let cost = crate::usage::estimate_tool_cost(tool_name);
        self.ensure_born_mut()?
            .burn_synapse(tool_usage.compute_synapse_burn() + cost)?;

        Ok(output)
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn format_compilation_response(compilation: &IntentCompilation) -> String {
    let mut lines = vec![
        format!("Intent compiled as {:?}.", compilation.class),
        format!(
            "Plan: {} step(s), max_iterations={}, priority={}, sandbox={}",
            compilation.plan.steps.len(),
            compilation.plan.max_iterations,
            compilation.plan.priority,
            compilation.plan.sandbox
        ),
    ];

    if !compilation.tool_sequence.is_empty() {
        lines.push(format!(
            "Tool sequence: {}",
            compilation.tool_sequence.join(" -> ")
        ));
    }

    for (idx, step) in compilation.plan.steps.iter().enumerate() {
        let confirmation = if step.requires_confirmation {
            " (confirmation required)"
        } else {
            ""
        };
        lines.push(format!(
            "{}. {:?}: {}{}",
            idx + 1,
            step.kind,
            step.description,
            confirmation
        ));
    }

    if let Some(suggestion) = &compilation.sub_agent_suggestion {
        lines.push(format!(
            "Sub-agent suggested: {} (tier {}): {}",
            suggestion.purpose, suggestion.required_tier, suggestion.reason
        ));
    }

    if !compilation.validation.reasons.is_empty() {
        lines.push(format!(
            "Validation: {}",
            compilation.validation.reasons.join("; ")
        ));
    }

    if !compilation.validation.warnings.is_empty() {
        lines.push(format!(
            "Warnings: {}",
            compilation.validation.warnings.join("; ")
        ));
    }

    if compilation.validation.requires_confirmation {
        lines.push("Awaiting explicit confirmation before high-risk execution.".to_string());
    }

    lines.join("\n")
}

fn normalize_secret_arg(arg: String) -> String {
    arg.trim().trim_matches('"').to_string()
}

fn audit_event_for_success(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::SlashCmd { command, .. } if command == "seal" => {
            Some("private_session_sealed".to_string())
        }
        Statement::SlashCmd { command, .. } if command == "unlock" => {
            Some("private_session_unsealed".to_string())
        }
        _ => None,
    }
}
