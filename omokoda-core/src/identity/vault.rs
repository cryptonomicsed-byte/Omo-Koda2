use rand::Rng;
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub agent_id: String,
    pub tool: String,
    pub expiry: u64,
    pub signature: Vec<u8>,
}

impl CapabilityToken {
    pub fn sign(agent_id: &str, tool: &str, k_root: &[u8; 32]) -> Self {
        let expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600; // 1 hour
            
        let message = format!("{}:{}:{}", agent_id, tool, expiry);
        let mut hmac = Hmac::<Sha256>::new_from_slice(k_root).expect("HMAC can take key of any size");
        hmac.update(message.as_bytes());
        let signature = hmac.finalize().into_bytes().to_vec();
        
        Self {
            agent_id: agent_id.to_string(),
            tool: tool.to_string(),
            expiry,
            signature,
        }
    }

    pub fn verify(&self, k_root: &[u8; 32]) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if self.expiry < now { return false; }
        
        let message = format!("{}:{}:{}", self.agent_id, self.tool, self.expiry);
        let mut hmac = Hmac::<Sha256>::new_from_slice(k_root).expect("HMAC can take key of any size");
        hmac.update(message.as_bytes());
        
        hmac.verify_slice(&self.signature).is_ok()
    }
}

pub struct SealVault;

impl SealVault {
    /// Generates an internal secret (K_root) inside the vault.
    pub fn generate_internal_secret() -> [u8; 32] {
        let mut k_root = [0u8; 32];
        rand::thread_rng().fill(&mut k_root);
        k_root
    }

    /// Generates a deterministic internal secret (K_root) for a given name and seed.
    pub fn generate_deterministic_secret(name: &str, master_seed: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(master_seed);
        hasher.finalize().into()
    }

    /// Reconstructs K_root from threshold shares.
    pub fn reconstruct_secret(_shares: Vec<Vec<u8>>) -> Result<[u8; 32], String> {
        Err("Threshold reconstruction not implemented in stub".to_string())
    }
}
