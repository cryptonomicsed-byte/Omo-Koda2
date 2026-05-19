use crate::identity::odu::{OduIdentity, OduSeed};
use crate::identity::AgentId;
use argon2::{Algorithm, Argon2, Params, Version};
use blake3;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

pub const SESSION_VERSION: u32 = 1;
pub const ENCRYPTED_SESSION_VERSION: u32 = 1;
pub const ARGON2_MEMORY_KB: u32 = 65536;
pub const ARGON2_ITERATIONS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 1;
pub const ARGON2_OUTPUT_LEN: u32 = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub version: u32,
    pub agent_id: AgentId,
    pub name: String,
    pub birth_timestamp: u64,
    pub reputation: f64,
    pub config: SessionConfig,
    pub public_messages: Vec<ConversationMessage>,
    pub encrypted_private: Option<EncryptedSession>,
    pub warn_count: u32,
    pub cooldown_active: bool,
    pub think_history: Vec<String>,
    pub swarm_agents: Vec<AgentId>,
}

pub type SessionState = Session;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionConfig {
    pub default_provider: String,
    pub default_privacy: bool,
    pub default_sandbox: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_provider: "ollama".to_string(),
            default_privacy: true,
            default_sandbox: true,
        }
    }
}

/// Versioned encrypted private session envelope.
///
/// Security invariants:
/// - `private_ciphertext` is the only persisted representation of private messages and Odu private data.
/// - `nonce` is generated randomly for every seal/rotation.
/// - `salt` is public KDF salt; it is not a secret.
/// - Argon2id parameters are persisted so future migrations can reject or upgrade old envelopes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedSession {
    pub version: u32,
    pub private_ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub salt: [u8; 16],
    pub key_version: u32,
    pub kdf: KdfParams,
}

pub type EncryptedData = EncryptedSession;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KdfParams {
    pub algorithm: String,
    pub memory_kb: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub output_len: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            algorithm: "argon2id-v0x13".to_string(),
            memory_kb: ARGON2_MEMORY_KB,
            iterations: ARGON2_ITERATIONS,
            parallelism: ARGON2_PARALLELISM,
            output_len: ARGON2_OUTPUT_LEN,
        }
    }
}

/// Zeroizing wrapper for passphrase-derived unlock keys held by the Steward.
#[derive(Clone, PartialEq, Eq)]
pub struct SensitiveKey([u8; 32]);

impl SensitiveKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn expose(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn zeroize_now(&mut self) {
        self.0.zeroize();
    }
}

impl std::fmt::Debug for SensitiveKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SensitiveKey([redacted])")
    }
}

impl Drop for SensitiveKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub blocks: Vec<ContentBlock>,
    pub is_private: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: String,
    },
    ToolResult {
        tool_use_id: String,
        output: String,
        is_error: bool,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivateSessionData {
    pub odu_seed: OduSeed,
    pub odu_identity: OduIdentity,
    pub private_messages: Vec<ConversationMessage>,
}

impl Session {
    pub fn new(agent_id: AgentId, name: String, birth_timestamp: u64) -> Self {
        Self {
            version: SESSION_VERSION,
            agent_id,
            name,
            birth_timestamp,
            reputation: 0.0,
            config: SessionConfig::default(),
            public_messages: Vec::new(),
            encrypted_private: None,
            warn_count: 0,
            cooldown_active: false,
            think_history: Vec::new(),
            swarm_agents: Vec::new(),
        }
    }

    pub fn apply_metadata(&mut self, key: &str, value: &str) {
        match key {
            "provider" => self.config.default_provider = value.to_string(),
            "privacy" => self.config.default_privacy = value == "true",
            "sandbox" => self.config.default_sandbox = value == "true",
            _ => {}
        }
    }

    pub fn warn_count_this_session(&self) -> u32 {
        self.warn_count
    }

    pub fn increment_warn_count(&mut self) {
        self.warn_count += 1;
    }

    pub fn is_in_cooldown(&self) -> bool {
        self.cooldown_active
    }

    pub fn set_cooldown(&mut self, active: bool) {
        self.cooldown_active = active;
    }

    pub fn recent_thinks(&self) -> &[String] {
        &self.think_history
    }

    pub fn swarm_size(&self) -> usize {
        self.swarm_agents.len()
    }

    pub fn add_message(&mut self, message: ConversationMessage, reputation: f64) {
        if !message.is_private {
            self.push_public(message, reputation);
        }
    }

    pub fn push_public(&mut self, message: ConversationMessage, reputation: f64) {
        self.public_messages.push(message);
        let engine = crate::memory::MemoryEngine::new();
        engine.compress(&mut self.public_messages, reputation);
    }

    pub fn seal_private(
        &mut self,
        private_data: &PrivateSessionData,
        odu_seed: &OduSeed,
        password_key: &[u8; 32],
    ) -> Result<(), String> {
        self.seal_private_with_version(private_data, odu_seed, password_key, 1)
    }

    fn seal_private_with_version(
        &mut self,
        private_data: &PrivateSessionData,
        odu_seed: &OduSeed,
        password_key: &[u8; 32],
        key_version: u32,
    ) -> Result<(), String> {
        let salt = generate_salt(&self.agent_id, self.birth_timestamp);
        let mut key = derive_session_key(odu_seed, &salt, password_key, key_version);

        let mut json = serde_json::to_string(private_data)
            .map_err(|e| format!("failed to serialize private data: {e}"))?;

        let cipher = ChaCha20Poly1305::new(&key.into());
        key.zeroize();
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|e| format!("encryption failed: {e}"))?;
        json.zeroize();

        self.encrypted_private = Some(EncryptedSession {
            version: ENCRYPTED_SESSION_VERSION,
            private_ciphertext: ciphertext,
            nonce: nonce_bytes,
            salt,
            key_version,
            kdf: KdfParams::default(),
        });

        Ok(())
    }

    pub fn unseal_private(
        &self,
        odu_seed: &OduSeed,
        password_key: &[u8; 32],
    ) -> Result<PrivateSessionData, String> {
        let data = self
            .encrypted_private
            .as_ref()
            .ok_or_else(|| "no encrypted private data found".to_string())?;

        let mut key = derive_session_key(odu_seed, &data.salt, password_key, data.key_version);
        let cipher = ChaCha20Poly1305::new(&key.into());
        key.zeroize();
        let nonce = Nonce::from_slice(&data.nonce);

        let mut plaintext = cipher
            .decrypt(nonce, data.private_ciphertext.as_slice())
            .map_err(|e| format!("decryption failed: {e}"))?;

        let private_data: PrivateSessionData = serde_json::from_slice(&plaintext)
            .map_err(|e| format!("failed to deserialize private data: {e}"))?;
        plaintext.zeroize();

        Ok(private_data)
    }

    pub fn rotate_key(
        &mut self,
        odu_seed: &OduSeed,
        old_password_key: &[u8; 32],
        new_password_key: &[u8; 32],
    ) -> Result<(), String> {
        let private_data = self.unseal_private(odu_seed, old_password_key)?;
        let new_version = self
            .encrypted_private
            .as_ref()
            .map(|d| d.key_version)
            .unwrap_or(0)
            + 1;
        self.seal_private_with_version(&private_data, odu_seed, new_password_key, new_version)?;
        Ok(())
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize session: {e}"))?;
        secure_write(path, json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let session: Self = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        session.migrate()
    }

    pub fn migrate(self) -> Result<Self, String> {
        match self.version {
            SESSION_VERSION => Ok(self),
            other => Err(format!(
                "unsupported session version {other}; expected {SESSION_VERSION}"
            )),
        }
    }

    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("failed to export session: {e}"))
    }
}

impl PrivateSessionData {
    pub fn push_private(&mut self, message: ConversationMessage, reputation: f64) {
        self.private_messages.push(message);
        let engine = crate::memory::MemoryEngine::new();
        engine.compress(&mut self.private_messages, reputation);
    }
}

impl Drop for PrivateSessionData {
    fn drop(&mut self) {
        self.odu_seed.0.zeroize();
        self.odu_identity.mnemonic.zeroize();
        for message in &mut self.private_messages {
            message.zeroize_contents();
        }
        self.private_messages.clear();
    }
}

impl ConversationMessage {
    pub fn zeroize_contents(&mut self) {
        for block in &mut self.blocks {
            match block {
                ContentBlock::Text { text } => text.zeroize(),
                ContentBlock::ToolUse { id, name, input } => {
                    id.zeroize();
                    name.zeroize();
                    input.zeroize();
                }
                ContentBlock::ToolResult {
                    tool_use_id,
                    output,
                    is_error: _,
                } => {
                    tool_use_id.zeroize();
                    output.zeroize();
                }
            }
        }
        self.blocks.clear();
    }
}

impl ConversationMessage {
    pub fn new_user(content: String, is_private: bool) -> Self {
        Self {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text { text: content }],
            is_private,
            timestamp: current_unix_timestamp(),
        }
    }

    pub fn new_assistant(content: String, is_private: bool) -> Self {
        Self {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::Text { text: content }],
            is_private,
            timestamp: current_unix_timestamp(),
        }
    }

    pub fn user_text(text: &str) -> Self {
        Self::new_user(text.to_string(), false)
    }

    pub fn assistant_text(text: &str) -> Self {
        Self::new_assistant(text.to_string(), false)
    }
}

const SESSION_CHAIN_ID: &str = "omokoda-main";

pub fn generate_salt(agent_id: &AgentId, birth_timestamp: u64) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(agent_id.as_str().as_bytes());
    hasher.update(&birth_timestamp.to_be_bytes());
    hasher.update(SESSION_CHAIN_ID.as_bytes());
    let result = hasher.finalize();
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&result.as_bytes()[..16]);
    salt
}

/// Derive the AEAD key with Argon2id using frozen session parameters.
///
/// The Odu seed participates in the KDF salt so ciphertext is bound to the born agent. The
/// passphrase-derived `password_key` remains the ownership secret and must be zeroized by callers.
fn derive_session_key(
    odu_seed: &OduSeed,
    salt: &[u8; 16],
    password_key: &[u8; 32],
    key_version: u32,
) -> [u8; 32] {
    let params = Params::new(
        ARGON2_MEMORY_KB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(ARGON2_OUTPUT_LEN as usize),
    )
    .expect("invalid Argon2 parameters");

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut okm = [0u8; 32];

    let mut combined_salt = Vec::with_capacity(52);
    combined_salt.extend_from_slice(salt);
    combined_salt.extend_from_slice(odu_seed.as_bytes());
    combined_salt.extend_from_slice(&key_version.to_be_bytes());

    argon2
        .hash_password_into(password_key, &combined_salt, &mut okm)
        .expect("Argon2 key derivation failed");
    okm
}

pub fn derive_unlock_key(
    password: &str,
    agent_public_key: &[u8; 32],
) -> Result<SensitiveKey, String> {
    let params = Params::new(
        ARGON2_MEMORY_KB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(ARGON2_OUTPUT_LEN as usize),
    )
    .map_err(|e| format!("invalid Argon2 parameters: {e}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut salt = [0u8; 32];
    let mut hasher = blake3::Hasher::new();
    hasher.update(agent_public_key);
    hasher.update(b"omokoda-unlock-key-v1");
    salt.copy_from_slice(hasher.finalize().as_bytes());

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| format!("unlock key derivation failed: {e}"))?;
    salt.zeroize();
    Ok(SensitiveKey::new(key))
}

pub fn secure_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create session dir: {e}"))?;
            set_strict_dir_permissions(parent)?;
        }
    }
    std::fs::write(path, bytes).map_err(|e| format!("failed to write session file: {e}"))?;
    std::fs::File::open(path)
        .map_err(|e| format!("failed to open file for sync: {e}"))?
        .sync_all()
        .map_err(|e| format!("failed to sync file: {e}"))?;
    set_strict_file_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
pub fn set_strict_dir_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("failed to set session dir permissions: {e}"))
}

#[cfg(not(unix))]
pub fn set_strict_dir_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
pub fn set_strict_file_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("failed to set session file permissions: {e}"))
}

#[cfg(not(unix))]
pub fn set_strict_file_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
