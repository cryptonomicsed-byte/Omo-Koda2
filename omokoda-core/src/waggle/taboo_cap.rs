//! Èṣù's taboo-capability issuer — the minting side of the substrate's
//! authenticated-taboo gate (Connection Map v2 follow-on #1).
//!
//! The adversarial benchmark proved that field-level anomaly detection cannot
//! catch a lone taboo-griefer hiding in legitimate concurrent load: taboo
//! censors a *path*, so a spammed taboo is a denial-of-service, and rate
//! statistics wash out under load. The fix is authentication, not more
//! detection — only an agent bearing a capability Èṣù minted may leave a taboo,
//! and Èṣù mints one only for a verified Ọbàtálá-lineage identity, a higher bar
//! than general onboarding.
//!
//! Èṣù holds an ed25519 signing key. Waggle core (Go, `crypto/ed25519`) is
//! configured with the matching public key and *verifies* — it never issues.
//! The token is `hex(payload) "." hex(signature)`, the payload the compact JSON
//! below signed as raw bytes. Ed25519 is deterministic (RFC 8032), so the Go
//! verifier and this Rust issuer agree byte-for-byte with no shared runtime; the
//! test below pins the exact same vector the Go test does.

use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use serde::Serialize;

/// The signed grant. Field order and json names match the Go `tabooCapPayload`
/// exactly — the same bytes must hash to the same signature on both sides, and
/// the exact payload bytes travel inside the token, so no canonicalization is
/// needed beyond agreeing on this layout.
#[derive(Serialize)]
struct TabooCapPayload<'a> {
    agent: &'a str,
    scope: &'a str,
    lineage: &'a str,
    iat: i64,
    exp: i64,
}

/// Mints ed25519-signed taboo capabilities. Session-scoped: construct from
/// Èṣù's configured seed, hand the public key to the daemon, issue tokens to
/// lineage-verified agents.
pub struct TabooCapIssuer {
    key: SigningKey,
}

impl TabooCapIssuer {
    /// Build the issuer from a 32-byte seed (Èṣù's configured secret).
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            key: SigningKey::from_bytes(seed),
        }
    }

    /// The hex ed25519 public key to configure `waggled -taboo-auth-key` with.
    pub fn public_key_hex(&self) -> String {
        let vk: VerifyingKey = self.key.verifying_key();
        hex::encode(vk.to_bytes())
    }

    /// Issue a taboo capability for `agent`, valid for `ttl_secs`, **only** if
    /// `lineage` is a verified Ọbàtálá lineage. Returns `None` when the lineage
    /// bar is not met — the crossroads keeper does not mint a censor-token for
    /// an identity that has not proven its house.
    pub fn issue(&self, agent: &str, lineage: &str, ttl_secs: i64) -> Option<String> {
        if !is_obatala_lineage(lineage) {
            return None;
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Some(self.sign_payload(agent, lineage, now, now + ttl_secs))
    }

    /// Deterministic core of `issue`, exposed for the cross-language vector
    /// test (fixed iat/exp instead of the wall clock).
    fn sign_payload(&self, agent: &str, lineage: &str, iat: i64, exp: i64) -> String {
        let payload = TabooCapPayload {
            agent,
            scope: "taboo",
            lineage,
            iat,
            exp,
        };
        let bytes = serde_json::to_vec(&payload).expect("payload serializes");
        let sig = self.key.sign(&bytes);
        format!("{}.{}", hex::encode(&bytes), hex::encode(sig.to_bytes()))
    }
}

/// The Ọbàtálá-lineage bar for minting a taboo capability. Ọbàtálá is the
/// shaper of heads — an identity in his lineage has a verified provenance, not
/// merely a passed onboarding. Encoded here as an `obatala:` provenance tag
/// with a non-empty proof; in a full deployment this consults the identity
/// registry (BIPON39/Vantage) rather than a string shape, but the gate — a bar
/// strictly higher than general capability issuance — is the same.
pub fn is_obatala_lineage(lineage: &str) -> bool {
    lineage
        .strip_prefix("obatala:")
        .is_some_and(|proof| !proof.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The canonical cross-language vector, identical to the Go
    // core/tabooauth_test.go pin. Same seed + same payload ⇒ same public key and
    // same signature, proving the issuer and verifier interoperate.
    const VECTOR_SEED: [u8; 32] = [1u8; 32];
    const VECTOR_PUBKEY: &str =
        "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c";
    const VECTOR_TOKEN: &str = "7b226167656e74223a226f626174616c612d6368696c642d31222c2273636f7065223a227461626f6f222c226c696e65616765223a226f626174616c613a7665726966696564222c22696174223a313030303030303030302c22657870223a323030303030303030307d.85aaac79be8c3f3cc232b10e4ecfa99e68261151faef1fc1fbd2568c15477d904413213d91429d341f2d2ad5667a7f13406504d48f1e5cec2ebe740171c02308";

    #[test]
    fn matches_go_vector() {
        let issuer = TabooCapIssuer::from_seed(&VECTOR_SEED);
        assert_eq!(issuer.public_key_hex(), VECTOR_PUBKEY, "public key drift");
        let token = issuer.sign_payload("obatala-child-1", "obatala:verified", 1_000_000_000, 2_000_000_000);
        assert_eq!(token, VECTOR_TOKEN, "signature/token drift from Go verifier");
    }

    #[test]
    fn lineage_bar() {
        assert!(is_obatala_lineage("obatala:verified"));
        assert!(is_obatala_lineage("obatala:bipon39-attestation-0xabc"));
        assert!(!is_obatala_lineage("obatala:"));
        assert!(!is_obatala_lineage("obatala:   "));
        assert!(!is_obatala_lineage("general-onboarding"));
        assert!(!is_obatala_lineage(""));
    }

    #[test]
    fn issue_gated_on_lineage() {
        let issuer = TabooCapIssuer::from_seed(&VECTOR_SEED);
        assert!(issuer.issue("a", "obatala:verified", 3600).is_some());
        assert!(issuer.issue("a", "just-any-agent", 3600).is_none());
    }
}
