//! TEE sealing for Tier-1 private memory — the Nautilus/Seal envelope.
//!
//! Private memory entries are always encrypted in software (ChaCha20-Poly1305
//! under the Living Odu key, see `AgentCore::encrypt_memory_entry`). When a
//! Nautilus enclave is available, this module adds a **second envelope**:
//! the software ciphertext is sealed with the enclave's AES-256-GCM key,
//! bound to the agent id so one agent's seal can never open another's
//! (`nautilus_integration::sealed_memory`).
//!
//!   plaintext ──ChaCha20(odu key)──▶ ciphertext ──TEE seal──▶ sealed blob
//!
//! Configuration (fail-open; unsealed = software-only, never a crash):
//!   OMOKODA_TEE_SEAL=1        enable the envelope
//!   NAUTILUS_SEAL_KEY=<hex>   64 hex chars — the attested enclave seal key.
//!                             In production this comes from
//!                             `attestation::verify_quote` after a Nautilus
//!                             handshake; the env var is the injection point
//!                             for the key the enclave released.

use nautilus_integration::sealed_memory::{seal, unseal, SealedMemory};
use serde::{Deserialize, Serialize};

/// A TEE-sealed envelope around software ciphertext. Serialized into the
/// memory entry's ciphertext slot, prefixed so readers can distinguish it
/// from a bare software ciphertext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeEnvelope {
    pub version: u8,
    pub sealed: SealedMemory,
}

/// Marker prefix for serialized envelopes. A bare ChaCha ciphertext is
/// binary and cannot collide with this ASCII prefix at offset 0.
pub const TEE_ENVELOPE_PREFIX: &[u8] = b"OMOKODA-TEE1:";

pub struct TeeSealer {
    seal_key: [u8; 32],
}

impl TeeSealer {
    /// Build from the environment. `None` = TEE sealing disabled or key
    /// missing/malformed — the caller stays on software-only encryption.
    pub fn from_env() -> Option<Self> {
        if std::env::var("OMOKODA_TEE_SEAL").ok().as_deref() != Some("1") {
            return None;
        }
        let hex_key = std::env::var("NAUTILUS_SEAL_KEY").ok()?;
        let bytes = hex::decode(hex_key.trim()).ok()?;
        let seal_key: [u8; 32] = bytes.try_into().ok()?;
        Some(Self { seal_key })
    }

    /// For tests and for wiring the key straight from a completed Nautilus
    /// attestation (`AttestationResult::seal_key`).
    pub fn with_key(seal_key: [u8; 32]) -> Self {
        Self { seal_key }
    }

    /// Seal software ciphertext into a TEE envelope, bound to `agent_id`.
    pub fn seal_bytes(&self, ciphertext: &[u8], agent_id: &str) -> Result<Vec<u8>, String> {
        let sealed = seal(ciphertext, &self.seal_key, agent_id)
            .map_err(|e| format!("TEE seal failed: {e}"))?;
        let envelope = TeeEnvelope { version: 1, sealed };
        let json =
            serde_json::to_vec(&envelope).map_err(|e| format!("TEE envelope encode: {e}"))?;
        let mut out = Vec::with_capacity(TEE_ENVELOPE_PREFIX.len() + json.len());
        out.extend_from_slice(TEE_ENVELOPE_PREFIX);
        out.extend_from_slice(&json);
        Ok(out)
    }

    /// Unseal an envelope back to the software ciphertext. Fails on agent
    /// mismatch or tampering (AES-GCM authentication).
    pub fn unseal_bytes(&self, enveloped: &[u8], agent_id: &str) -> Result<Vec<u8>, String> {
        let json = enveloped
            .strip_prefix(TEE_ENVELOPE_PREFIX)
            .ok_or("not a TEE envelope")?;
        let envelope: TeeEnvelope =
            serde_json::from_slice(json).map_err(|e| format!("TEE envelope decode: {e}"))?;
        unseal(&envelope.sealed, &self.seal_key, agent_id)
            .map_err(|e| format!("TEE unseal failed: {e}"))
    }
}

/// True if `bytes` carry the TEE envelope prefix.
pub fn is_tee_enveloped(bytes: &[u8]) -> bool {
    bytes.starts_with(TEE_ENVELOPE_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> [u8; 32] {
        [7u8; 32]
    }

    #[test]
    fn seal_unseal_round_trips() {
        let sealer = TeeSealer::with_key(key());
        let ciphertext = b"software-encrypted bytes";
        let enveloped = sealer.seal_bytes(ciphertext, "agent-luna").unwrap();
        assert!(is_tee_enveloped(&enveloped));
        let back = sealer.unseal_bytes(&enveloped, "agent-luna").unwrap();
        assert_eq!(back, ciphertext);
    }

    #[test]
    fn unseal_rejects_wrong_agent() {
        // The seal key is bound to the agent id — cross-agent unsealing fails.
        let sealer = TeeSealer::with_key(key());
        let enveloped = sealer.seal_bytes(b"secret", "agent-luna").unwrap();
        assert!(sealer.unseal_bytes(&enveloped, "agent-mallory").is_err());
    }

    #[test]
    fn unseal_rejects_tampering() {
        let sealer = TeeSealer::with_key(key());
        let mut enveloped = sealer.seal_bytes(b"secret", "agent-luna").unwrap();
        let last = enveloped.len() - 1;
        enveloped[last] ^= 0xFF;
        assert!(sealer.unseal_bytes(&enveloped, "agent-luna").is_err());
    }

    #[test]
    fn bare_ciphertext_is_not_an_envelope() {
        assert!(!is_tee_enveloped(b"\x8a\x01binary chacha bytes"));
        let sealer = TeeSealer::with_key(key());
        assert!(sealer.unseal_bytes(b"random", "agent-luna").is_err());
    }

    #[test]
    fn from_env_disabled_without_flag() {
        std::env::remove_var("OMOKODA_TEE_SEAL");
        assert!(TeeSealer::from_env().is_none());
    }
}
