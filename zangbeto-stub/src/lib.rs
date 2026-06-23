//! Zàngbétò CI stub.
//!
//! Mirrors the public API surface of the real `zangbeto-enforcement` crate so that
//! omokoda-core compiles and tests pass in CI without needing the real repo.

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Result of a Zàngbétò state audit.
pub struct AuditResult {
    /// Unique receipt identifier for this audit event.
    pub receipt_id: String,
    /// 32-byte deterministic signature (blake3-style hash of state in stub).
    pub sig: Vec<u8>,
    /// Whether the audit passed (stub always passes).
    pub passed: bool,
}

/// Audit a state snapshot represented as raw bytes.
///
/// In the real crate this runs the Zàngbétò enforcement VM; the stub
/// returns a deterministic SHA-256 digest of the input as the "signature".
pub fn audit_state(state_bytes: &[u8]) -> AuditResult {
    let mut hasher = Sha256::new();
    hasher.update(state_bytes);
    let sig: Vec<u8> = hasher.finalize().to_vec();
    AuditResult {
        receipt_id: Uuid::new_v4().to_string(),
        sig,
        passed: true,
    }
}
