use crate::identity::odu::{OduIdentity, OduSeed};
use crate::identity::AgentId;
use argon2::{Argon2, Algorithm, Params, Version};
use blake3;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const SESSION_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub version: u32,
    pub agent_id: AgentId,
    pub name: String,
    pub birth_timestamp: u64,
    pub reputation: f64,
    pub config: SessionConfig,
    pub public_messages: Vec<ConversationMessage>,
    pub encrypted_private: Option<EncryptedData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionConfig {
    pub default_provider: String,
    pub default_privacy: bool,
    pub default_sandbox: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_provider: "default".to_string(),
            default_privacy: true,
            default_sandbox: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub salt: [u8; 16],
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

    pub fn add_message(&mut self, message: ConversationMessage) {
        if !message.is_private {
            self.push_public(message);
        }
    }

    pub fn push_public(&mut self, message: ConversationMessage) {
        self.public_messages.push(message);
        if self.public_messages.len() > 100 {
            // RACK eviction: remove first 20 messages
            self.public_messages.drain(0..20);
        }
    }

    pub fn seal_private(
        &mut self,
        private_data: &PrivateSessionData,
        odu_seed: &OduSeed,
        password_key: &[u8; 32],
    ) -> Result<(), String> {
        let salt = generate_salt(&self.agent_id, self.birth_timestamp);
        let key = derive_session_key(odu_seed, &salt, password_key);

        let json = serde_json::to_string(private_data)
            .map_err(|e| format!("failed to serialize private data: {e}"))?;

        let cipher = ChaCha20Poly1305::new(&key.into());
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|e| format!("encryption failed: {e}"))?;

        self.encrypted_private = Some(EncryptedData {
            ciphertext,
            nonce: nonce_bytes,
            salt,
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

        let key = derive_session_key(odu_seed, &data.salt, password_key);
        let cipher = ChaCha20Poly1305::new(&key.into());
        let nonce = Nonce::from_slice(&data.nonce);

        let plaintext = cipher
            .decrypt(nonce, data.ciphertext.as_slice())
            .map_err(|e| format!("decryption failed: {e}"))?;

        let private_data: PrivateSessionData = serde_json::from_slice(&plaintext)
            .map_err(|e| format!("failed to deserialize private data: {e}"))?;

        Ok(private_data)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize session: {e}"))?;
        fs::write(path, json).map_err(|e| format!("failed to write session file: {e}"))?;
        Ok(())
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }
}

impl PrivateSessionData {
    pub fn push_private(&mut self, message: ConversationMessage) {
        self.private_messages.push(message);
        if self.private_messages.len() > 1000 {
            // RACK eviction: remove first 200 messages and add summary
            self.private_messages.drain(0..200);
            let summary = ConversationMessage {
                role: MessageRole::System,
                blocks: vec![ContentBlock::Text {
                    text: "[SYSTEM SUMMARY OF EVICTED MESSAGES]".to_string(),
                }],
                is_private: true,
                timestamp: current_unix_timestamp(),
            };
            self.private_messages.insert(0, summary);
        }
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

const ARGON2_MEMORY_KB: u32 = 65536;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 1;
const ARGON2_OUTPUT_LEN: u32 = 32;
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

fn derive_session_key(
    odu_seed: &OduSeed,
    salt: &[u8; 16],
    password_key: &[u8; 32],
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

    let mut combined_salt = Vec::with_capacity(48);
    combined_salt.extend_from_slice(salt);
    combined_salt.extend_from_slice(odu_seed.as_bytes());

    argon2
        .hash_password_into(password_key, &combined_salt, &mut okm)
        .expect("Argon2 key derivation failed");
    okm
}

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
