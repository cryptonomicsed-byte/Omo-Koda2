use crate::identity::odu::{OduIdentity, OduSeed};
use crate::interpreter::AgentId;
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
            default_provider: "ollama".to_string(),
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
pub struct PrivateSessionData {
    pub odu_seed: OduSeed,
    pub odu_identity: OduIdentity,
    pub private_messages: Vec<ConversationMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub blocks: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
        tool_name: String,
        output: String,
        is_error: bool,
    },
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
            "privacy" => self.config.default_privacy = value == "private" || value == "true",
            "sandbox" => self.config.default_sandbox = value == "sandbox" || value == "true",
            _ => {}
        }
    }

    pub fn push_public(&mut self, message: ConversationMessage) {
        self.public_messages.push(message);
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), SessionError> {
        let encoded = serde_json::to_string_pretty(self).map_err(SessionError::Encode)?;
        fs::write(path, encoded).map_err(SessionError::Io)
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, SessionError> {
        let encoded = fs::read_to_string(path).map_err(SessionError::Io)?;
        let session: Session = serde_json::from_str(&encoded).map_err(SessionError::Decode)?;
        if session.version != SESSION_VERSION {
            return Err(SessionError::UnsupportedVersion(session.version));
        }
        Ok(session)
    }

    pub fn encrypt_private(
        &mut self,
        private_data: &PrivateSessionData,
        passphrase: &str,
    ) -> Result<(), SessionError> {
        let salt = generate_salt(&self.agent_id, self.birth_timestamp);
        let key = derive_key(passphrase, &salt)?;
        
        let plaintext = serde_json::to_vec(private_data).map_err(SessionError::Encode)?;
        
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|_| SessionError::Crypto)?;
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_slice())
            .map_err(|_| SessionError::Crypto)?;
            
        self.encrypted_private = Some(EncryptedData {
            ciphertext,
            nonce: nonce_bytes,
            salt,
        });
        
        Ok(())
    }

    pub fn decrypt_private(&self, passphrase: &str) -> Result<PrivateSessionData, SessionError> {
        let encrypted = self
            .encrypted_private
            .as_ref()
            .ok_or(SessionError::NoPrivateData)?;
            
        let key = derive_key(passphrase, &encrypted.salt)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|_| SessionError::Crypto)?;
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_slice())
            .map_err(|_| SessionError::Crypto)?;
            
        let private_data: PrivateSessionData =
            serde_json::from_slice(&plaintext).map_err(SessionError::Decode)?;
            
        Ok(private_data)
    }
}

fn generate_salt(agent_id: &AgentId, birth_timestamp: u64) -> [u8; 16] {
    let chain_id = "omokoda-v1";
    let mut hasher = blake3::Hasher::new();
    hasher.update(agent_id.as_str().as_bytes());
    hasher.update(&birth_timestamp.to_le_bytes());
    hasher.update(chain_id.as_bytes());
    let hash = hasher.finalize();
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&hash.as_bytes()[..16]);
    salt
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], SessionError> {
    use argon2::{Algorithm, Argon2, Params, Version};
    
    let params = Params::new(65536, 3, 1, Some(32)).map_err(|_| SessionError::Crypto)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| SessionError::Crypto)?;
        
    Ok(key)
}

impl ConversationMessage {
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            blocks: vec![ContentBlock::Text { text: text.into() }],
        }
    }
}

#[derive(Debug)]
pub enum SessionError {
    Io(std::io::Error),
    Encode(serde_json::Error),
    Decode(serde_json::Error),
    UnsupportedVersion(u32),
    Crypto,
    NoPrivateData,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::Io(error) => write!(f, "session I/O error: {error}"),
            SessionError::Encode(error) => write!(f, "session encode error: {error}"),
            SessionError::Decode(error) => write!(f, "session decode error: {error}"),
            SessionError::UnsupportedVersion(version) => {
                write!(f, "unsupported session version: {version}")
            }
            SessionError::Crypto => write!(f, "cryptographic error"),
            SessionError::NoPrivateData => write!(f, "no private data found in session"),
        }
    }
}

impl std::error::Error for SessionError {}
