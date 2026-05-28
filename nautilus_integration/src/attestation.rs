use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttestationError {
    #[error("code measurement mismatch: expected {expected}, got {actual}")]
    MeasurementMismatch { expected: String, actual: String },
    #[error("invalid TEE quote")]
    InvalidQuote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeQuote {
    pub enclave_id: [u8; 32],
    pub code_measurement: [u8; 32],
    pub nonce: [u8; 16],
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AttestationResult {
    pub enclave_id: [u8; 32],
    pub seal_key: [u8; 32],
}

/// Verify a TEE quote and derive a seal key.
/// In production: verify the SGX/TDX quote signature and check against
/// a pinned code measurement from config. Here we use a deterministic stub.
pub fn verify_quote(
    quote: &TeeQuote,
    expected_measurement: &[u8; 32],
) -> Result<AttestationResult, AttestationError> {
    if quote.code_measurement != *expected_measurement {
        return Err(AttestationError::MeasurementMismatch {
            expected: hex::encode(expected_measurement),
            actual: hex::encode(quote.code_measurement),
        });
    }

    // Derive seal key via HKDF(enclave_id || code_measurement)
    let mut hasher = Sha256::new();
    hasher.update(b"omokoda:nautilus:seal_key_v1");
    hasher.update(quote.enclave_id);
    hasher.update(quote.code_measurement);
    let seal_key: [u8; 32] = hasher.finalize().into();

    Ok(AttestationResult {
        enclave_id: quote.enclave_id,
        seal_key,
    })
}
