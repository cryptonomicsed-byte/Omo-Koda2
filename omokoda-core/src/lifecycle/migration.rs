/// AgentCapsule migration core — Phase 14.1
///
/// Provides the data structures and sealing logic for cross-node agent
/// transfer.  The encryption itself (ChaCha20-Poly1305 with destination-node
/// pubkey) is stubbed with a placeholder for Phase 14.2 once the node-to-node
/// key-agreement protocol is finalised.  All structural fields, state machine,
/// and hash/signature helpers are production-ready.
use serde::{Deserialize, Serialize};

/// Encrypted migration capsule for cross-node agent transfer.
///
/// # Security model
/// 1. `capsule_hash`      — BLAKE3 of all plaintext fields (agent_id, source,
///    dest, ts, vault ciphertext).  Commit-then-encrypt.
/// 2. `source_node_sig`   — Ed25519 signature of `capsule_hash` by the source
///    node's long-term identity key.  Destination verifies before decrypting.
/// 3. `encrypted_vault`   — vault bytes encrypted with the destination node's
///    public key (key-agreement cipher TBD in Phase 14.2; currently a
///    placeholder zero-encryption for structural completeness).
/// 4. `vault_nonce`       — 12-byte ChaCha20-Poly1305 nonce.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentCapsule {
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub source_node_pubkey: String,
    #[serde(default)]
    pub destination_node_pubkey: String,
    #[serde(default)]
    pub migration_timestamp: u64,
    /// Nostr event id for the kind 31022 migration-intent event.
    #[serde(default)]
    pub migration_intent_nostr_event_id: Option<String>,
    /// Vault ciphertext encrypted with destination node's pubkey.
    #[serde(default)]
    pub encrypted_vault: Vec<u8>,
    /// ChaCha20-Poly1305 nonce (12 bytes).
    #[serde(default)]
    pub vault_nonce: [u8; 12],
    /// BLAKE3 hash of all plaintext fields committed before encryption.
    #[serde(default)]
    pub capsule_hash: [u8; 32],
    /// Hex-encoded Ed25519 signature of `capsule_hash` by source node.
    #[serde(default)]
    pub source_node_sig: String,
}

/// State machine for a live migration workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationState {
    /// Migration intent published as Nostr kind 31022.
    IntentPublished { nostr_event_id: String },
    /// Capsule sealed; vault ciphertext ready for transfer.
    CapsuleSealed,
    /// Transfer initiated toward destination node.
    TransferInitiated,
    /// Destination node confirmed receipt.
    ReceivedByDestination,
    /// Agent is running on destination; tombstone pending on source.
    LandingComplete,
    /// Source node archived (read-only tombstone retained for audit trail).
    SourceArchived,
}

impl AgentCapsule {
    /// Seal a migration capsule from raw vault bytes.
    ///
    /// `vault_data`             — serialised `IdentityVaultData` plaintext
    ///                            (caller's responsibility to encrypt before
    ///                            passing; Phase 14.2 will move encryption here).
    /// `agent_id`               — agent identifier.
    /// `source_node_pubkey`     — hex Ed25519 pubkey of the sending node.
    /// `destination_node_pubkey`— hex Ed25519 pubkey of the receiving node.
    ///
    /// # Phase 14.2 note
    /// The `encrypted_vault` field is currently set to `vault_data` unchanged
    /// (identity transform) and `vault_nonce` is zeroed.  Phase 14.2 will
    /// replace this with proper ChaCha20-Poly1305 under a node-to-node ECDH
    /// shared secret.  The `source_node_sig` is empty until the node's signing
    /// key is wired in (also Phase 14.2).
    pub fn seal(
        vault_data: &[u8],
        agent_id: &str,
        source_node_pubkey: &str,
        destination_node_pubkey: &str,
    ) -> Result<Self, String> {
        let migration_timestamp = current_unix_ts();
        // Phase 14.2: replace with ChaCha20-Poly1305 under ECDH shared secret.
        let encrypted_vault = vault_data.to_vec();
        let vault_nonce = [0u8; 12];

        let capsule_hash = Self::capsule_content_hash(
            agent_id,
            source_node_pubkey,
            destination_node_pubkey,
            migration_timestamp,
            &encrypted_vault,
        );

        // Phase 14.2: replace with Ed25519 sig from source node identity key.
        let source_node_sig = String::new();

        Ok(AgentCapsule {
            agent_id: agent_id.to_string(),
            source_node_pubkey: source_node_pubkey.to_string(),
            destination_node_pubkey: destination_node_pubkey.to_string(),
            migration_timestamp,
            migration_intent_nostr_event_id: None,
            encrypted_vault,
            vault_nonce,
            capsule_hash,
            source_node_sig,
        })
    }

    /// Compute the BLAKE3 commitment hash for capsule content fields.
    ///
    /// Hash input: agent_id || "|" || source || "|" || dest || "|"
    ///             || ts_be_bytes || "|" || vault_bytes
    ///
    /// This is a deterministic, collision-resistant fingerprint of all fields
    /// that must be signed and verified before decryption.
    pub fn capsule_content_hash(
        agent_id: &str,
        source: &str,
        dest: &str,
        ts: u64,
        vault: &[u8],
    ) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(agent_id.as_bytes());
        hasher.update(b"|");
        hasher.update(source.as_bytes());
        hasher.update(b"|");
        hasher.update(dest.as_bytes());
        hasher.update(b"|");
        hasher.update(&ts.to_be_bytes());
        hasher.update(b"|");
        hasher.update(vault);
        *hasher.finalize().as_bytes()
    }
}

fn current_unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_hash_is_deterministic() {
        let h1 = AgentCapsule::capsule_content_hash(
            "agent-abc123",
            "src_pubkey",
            "dst_pubkey",
            1_700_000_000,
            b"vault_bytes",
        );
        let h2 = AgentCapsule::capsule_content_hash(
            "agent-abc123",
            "src_pubkey",
            "dst_pubkey",
            1_700_000_000,
            b"vault_bytes",
        );
        assert_eq!(h1, h2);
    }

    #[test]
    fn capsule_hash_changes_with_any_field() {
        let base = AgentCapsule::capsule_content_hash(
            "agent-abc",
            "src",
            "dst",
            1_000,
            b"vault",
        );
        let diff_agent = AgentCapsule::capsule_content_hash(
            "agent-xyz",
            "src",
            "dst",
            1_000,
            b"vault",
        );
        let diff_vault = AgentCapsule::capsule_content_hash(
            "agent-abc",
            "src",
            "dst",
            1_000,
            b"different_vault",
        );
        assert_ne!(base, diff_agent);
        assert_ne!(base, diff_vault);
    }

    #[test]
    fn seal_produces_valid_capsule() {
        let capsule = AgentCapsule::seal(
            b"fake_vault_data",
            "agent-test",
            "src_pubkey_hex",
            "dst_pubkey_hex",
        )
        .expect("seal must succeed");

        assert_eq!(capsule.agent_id, "agent-test");
        assert_eq!(capsule.source_node_pubkey, "src_pubkey_hex");
        assert_eq!(capsule.destination_node_pubkey, "dst_pubkey_hex");
        assert_eq!(capsule.encrypted_vault, b"fake_vault_data");
        assert_eq!(capsule.capsule_hash.len(), 32);

        // Hash must match independent computation
        let expected_hash = AgentCapsule::capsule_content_hash(
            "agent-test",
            "src_pubkey_hex",
            "dst_pubkey_hex",
            capsule.migration_timestamp,
            b"fake_vault_data",
        );
        assert_eq!(capsule.capsule_hash, expected_hash);
    }

    #[test]
    fn migration_state_serialises() {
        let state = MigrationState::IntentPublished {
            nostr_event_id: "event123".to_string(),
        };
        let json = serde_json::to_string(&state).expect("must serialise");
        assert!(json.contains("IntentPublished"));
    }
}
