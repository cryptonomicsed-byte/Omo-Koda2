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

    /// Build from a verified Nautilus attestation quote. This is the real
    /// connection the module doc comment above has described since before
    /// this function existed -- `verify_quote` was fully implemented but
    /// never actually called from anywhere in this crate; this closes
    /// that gap the same way `dream.rs`/`walrus.rs` were wired to their
    /// callers earlier in this memory-system build.
    ///
    /// Still honest about what it verifies: `nautilus_integration::
    /// attestation::verify_quote` checks `quote.code_measurement` against
    /// an expected value and derives a key from the quote's own fields --
    /// it does not verify a real hardware attestation signature (SGX/TDX)
    /// yet, since no real enclave is deployed. Once one is, `TeeQuote`'s
    /// fields come from that hardware and this call site does not need to
    /// change at all -- the seam is already correct, only the quote's
    /// provenance upgrades.
    pub fn from_attestation(
        quote: &nautilus_integration::attestation::TeeQuote,
        expected_measurement: &[u8; 32],
    ) -> Result<Self, String> {
        let result = nautilus_integration::attestation::verify_quote(quote, expected_measurement)
            .map_err(|e| format!("Nautilus attestation failed: {e}"))?;
        Ok(Self {
            seal_key: result.seal_key,
        })
    }

    /// Build from a real Sui Seal fetch (see `memory::seal_bridge`) --
    /// the DEK Seal's key servers released after the on-chain
    /// `seal_approve_agent_memory` policy passed for this specific agent.
    pub fn from_seal_dek(dek: [u8; 32]) -> Self {
        Self { seal_key: dek }
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

    #[test]
    fn from_attestation_seals_and_unseals_when_measurement_matches() {
        use nautilus_integration::attestation::TeeQuote;

        let measurement = [9u8; 32];
        let quote = TeeQuote {
            enclave_id: [1u8; 32],
            code_measurement: measurement,
            nonce: [2u8; 16],
            signature: vec![3u8; 8],
        };
        let sealer = TeeSealer::from_attestation(&quote, &measurement)
            .expect("matching measurement must succeed");
        let enveloped = sealer.seal_bytes(b"attested secret", "agent-luna").unwrap();
        assert_eq!(
            sealer.unseal_bytes(&enveloped, "agent-luna").unwrap(),
            b"attested secret"
        );
    }

    #[test]
    fn from_attestation_rejects_measurement_mismatch() {
        use nautilus_integration::attestation::TeeQuote;

        let quote = TeeQuote {
            enclave_id: [1u8; 32],
            code_measurement: [9u8; 32],
            nonce: [2u8; 16],
            signature: vec![3u8; 8],
        };
        let wrong_expected = [8u8; 32];
        assert!(TeeSealer::from_attestation(&quote, &wrong_expected).is_err());
    }

    #[test]
    fn from_seal_dek_round_trips() {
        let dek = [5u8; 32];
        let sealer = TeeSealer::from_seal_dek(dek);
        let enveloped = sealer.seal_bytes(b"seal-sourced secret", "agent-luna").unwrap();
        assert_eq!(
            sealer.unseal_bytes(&enveloped, "agent-luna").unwrap(),
            b"seal-sourced secret"
        );
    }
}
