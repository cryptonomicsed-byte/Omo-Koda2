use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;

pub struct OduKeys;

impl OduKeys {
    /// Derives K_0 from K_root and agent context.
    pub fn derive_k0(
        k_root: &[u8; 32],
        agent_id: &str,
        birth_timestamp: u64,
        chain_id: &str,
    ) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(agent_id.as_bytes());
        hasher.update(&birth_timestamp.to_le_bytes());
        hasher.update(chain_id.as_bytes());
        let salt = hasher.finalize();

        let hk = Hkdf::<Sha256>::new(Some(salt.as_bytes()), k_root);
        let mut k0 = [0u8; 32];
        hk.expand(b"omokoda:initial_key", &mut k0)
            .expect("HKDF expansion failed for K0");
        k0
    }

    /// Derives K_n+1 from K_n using key rotation logic.
    pub fn rotate_key(
        kn: &[u8; 32],
        hermetic_seed: &[u8; 32],
        act_counter: u64,
        epoch_nonce: &[u8; 32],
    ) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(hermetic_seed);
        hasher.update(&act_counter.to_le_bytes());
        hasher.update(epoch_nonce);
        let odu_vector = hasher.finalize();

        let nonce = Nonce::from_slice(&odu_vector.as_bytes()[..12]);
        let cipher = ChaCha20Poly1305::new(kn.into());

        let plaintext = [0u8; 32];
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("ChaCha20Poly1305 encryption failed for key rotation");

        let mut kn_plus_1 = [0u8; 32];
        kn_plus_1.copy_from_slice(&ciphertext[..32]);
        kn_plus_1
    }

    /// Derives current_key for private memory encryption.
    pub fn derive_current_key(kn: &[u8; 32], act_timestamp: u64) -> [u8; 32] {
        let hk = Hkdf::<Sha256>::new(Some(&act_timestamp.to_le_bytes()), kn);
        let mut current_key = [0u8; 32];
        hk.expand(b"private_memory", &mut current_key)
            .expect("HKDF expansion failed for current_key");
        current_key
    }
}
