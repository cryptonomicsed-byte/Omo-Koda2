// User identity resolution — all user ID routing enters here before any memory or hive access.
// Primary identifier: Sui wallet address (or zkLogin).
// Fallback: cryptographic seed derived from interaction metadata for non-wallet users.
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use thiserror::Error;

/// Controls what data a user session produces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PrivacyMode {
    /// Full hive participation — public profile shared, memories consented to aggregation.
    #[default]
    Public,
    /// Private session — local inference only, minimal public footprint, no hive contribution.
    Private,
    /// No persistent storage of any kind. Agent respects `/private` or incognito flag.
    Incognito,
}

impl PrivacyMode {
    pub fn from_flag(private: bool, incognito: bool) -> Self {
        if incognito {
            Self::Incognito
        } else if private {
            Self::Private
        } else {
            Self::Public
        }
    }
}

/// A resolved user identity — created by Rust gatekeeper on first contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    /// Hex-encoded Sui wallet address, or SHA3-256 of interaction metadata for non-wallet users.
    pub id: String,
    pub privacy: PrivacyMode,
    /// true = verified Sui wallet or zkLogin; false = generated seed (anonymous user).
    pub is_wallet: bool,
}

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("invalid wallet address: {0}")]
    InvalidWallet(String),
    #[error("seed generation failed: {0}")]
    SeedFailure(String),
}

impl UserIdentity {
    /// Resolve from a Sui wallet address (0x-prefixed hex).
    pub fn from_wallet(address: &str) -> Result<Self, IdentityError> {
        let addr = address.trim();
        if !addr.starts_with("0x") || addr.len() < 10 {
            return Err(IdentityError::InvalidWallet(addr.to_string()));
        }
        // Validate hex content after 0x prefix.
        if hex::decode(&addr[2..]).is_err() {
            return Err(IdentityError::InvalidWallet(addr.to_string()));
        }
        Ok(Self {
            id: addr.to_lowercase(),
            privacy: PrivacyMode::Public,
            is_wallet: true,
        })
    }

    /// Derive a deterministic pseudonymous ID from interaction metadata (timestamp, IP hash, etc.)
    /// for users without a Sui wallet. Uses SHA3-256 so the ID is non-reversible.
    pub fn from_seed(interaction_metadata: &str) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(b"omokoda-user-seed-v1:");
        hasher.update(interaction_metadata.as_bytes());
        let hash = hasher.finalize();
        Self {
            id: format!("seed:{}", hex::encode(hash)),
            privacy: PrivacyMode::Private, // seed users default to Private until they opt in
            is_wallet: false,
        }
    }

    /// Attempt wallet resolution; fall back to seed derivation.
    /// This is the main entry point called by the Rust gatekeeper on first contact.
    pub fn resolve(wallet: Option<&str>, metadata: &str) -> Self {
        match wallet {
            Some(addr) if !addr.is_empty() => {
                Self::from_wallet(addr).unwrap_or_else(|_| Self::from_seed(metadata))
            }
            _ => Self::from_seed(metadata),
        }
    }

    /// Apply a privacy override from a `/private` or `incognito` command.
    pub fn apply_privacy(&mut self, mode: PrivacyMode) {
        self.privacy = mode;
    }

    /// No data persisted whatsoever.
    pub fn is_incognito(&self) -> bool {
        self.privacy == PrivacyMode::Incognito
    }

    /// Memory cells and public profile may be stored (not incognito).
    pub fn allows_storage(&self) -> bool {
        self.privacy != PrivacyMode::Incognito
    }

    /// User has consented to their data contributing to the shared public hive.
    pub fn allows_hive_contribution(&self) -> bool {
        self.privacy == PrivacyMode::Public
    }

    /// Private or incognito — local-only inference required.
    pub fn requires_local_provider(&self) -> bool {
        self.privacy != PrivacyMode::Public
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_resolution_validates_hex_prefix() {
        assert!(UserIdentity::from_wallet("0xdeadbeef00112233445566778899aabb").is_ok());
        assert!(UserIdentity::from_wallet("notawalletaddress").is_err());
        assert!(UserIdentity::from_wallet("0x").is_err());
    }

    #[test]
    fn seed_is_deterministic() {
        let id1 = UserIdentity::from_seed("ts=1700000000,region=us-west");
        let id2 = UserIdentity::from_seed("ts=1700000000,region=us-west");
        assert_eq!(id1.id, id2.id);
        assert!(!id1.is_wallet);
        assert!(id1.id.starts_with("seed:"));
    }

    #[test]
    fn seed_differs_per_metadata() {
        let a = UserIdentity::from_seed("session-A");
        let b = UserIdentity::from_seed("session-B");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn resolve_prefers_wallet() {
        let id = UserIdentity::resolve(Some("0xdeadbeef00112233445566778899aabb"), "fallback");
        assert!(id.is_wallet);
    }

    #[test]
    fn resolve_falls_back_to_seed() {
        let id = UserIdentity::resolve(None, "session-meta");
        assert!(!id.is_wallet);
        assert!(id.id.starts_with("seed:"));
    }

    #[test]
    fn resolve_invalid_wallet_falls_back() {
        let id = UserIdentity::resolve(Some("bad-wallet"), "session-meta");
        assert!(!id.is_wallet);
    }

    #[test]
    fn privacy_modes_control_capabilities() {
        let mut id = UserIdentity::from_seed("test");
        id.apply_privacy(PrivacyMode::Incognito);
        assert!(id.is_incognito());
        assert!(!id.allows_storage());
        assert!(!id.allows_hive_contribution());
        assert!(id.requires_local_provider());

        id.apply_privacy(PrivacyMode::Private);
        assert!(!id.is_incognito());
        assert!(id.allows_storage());
        assert!(!id.allows_hive_contribution());
        assert!(id.requires_local_provider());

        id.apply_privacy(PrivacyMode::Public);
        assert!(id.allows_hive_contribution());
        assert!(!id.requires_local_provider());
    }
}
