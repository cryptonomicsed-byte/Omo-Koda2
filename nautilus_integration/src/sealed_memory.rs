use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Error)]
pub enum SealError {
    #[error("encryption failed")]
    EncryptionFailed,
    #[error("decryption failed — data tampered or wrong key")]
    DecryptionFailed,
    #[error("key derivation failed")]
    KeyDerivation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedMemory {
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub agent_id_hash: [u8; 32],
}

#[derive(Zeroize, ZeroizeOnDrop)]
struct SealKey([u8; 32]);

/// Seal plaintext memory under a per-TEE key.
/// The agent_id is bound into the key to prevent cross-agent unsealing.
pub fn seal(plaintext: &[u8], key: &[u8; 32], agent_id: &str) -> Result<SealedMemory, SealError> {
    let bound_key = derive_agent_key(key, agent_id);
    let cipher = Aes256Gcm::new_from_slice(&bound_key.0).map_err(|_| SealError::KeyDerivation)?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| SealError::EncryptionFailed)?;

    let agent_id_hash = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(agent_id.as_bytes());
        h.finalize().into()
    };

    Ok(SealedMemory {
        nonce: nonce_bytes,
        ciphertext,
        agent_id_hash,
    })
}

/// Unseal memory. Fails if the agent_id doesn't match or data was tampered.
pub fn unseal(sealed: &SealedMemory, key: &[u8; 32], agent_id: &str) -> Result<Vec<u8>, SealError> {
    let bound_key = derive_agent_key(key, agent_id);
    let cipher = Aes256Gcm::new_from_slice(&bound_key.0).map_err(|_| SealError::KeyDerivation)?;

    let nonce = Nonce::from_slice(&sealed.nonce);
    cipher
        .decrypt(nonce, sealed.ciphertext.as_ref())
        .map_err(|_| SealError::DecryptionFailed)
}

fn derive_agent_key(base_key: &[u8; 32], agent_id: &str) -> SealKey {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"omokoda:seal_key_binding_v1");
    h.update(base_key);
    h.update(agent_id.as_bytes());
    SealKey(h.finalize().into())
}
