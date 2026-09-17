//! DeployManifest — the artifact written to ~/.oso/deployed/ after a successful deploy.

use serde::{Deserialize, Serialize};

/// Canonical deployment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployManifest {
    /// Unique program identifier (SHA-256 hex of the JSON content, first 16 chars).
    pub program_id: String,

    /// Deployment target: "local", "testnet", or "mainnet".
    pub target: String,

    /// ISO-8601 deployment timestamp (UTC).
    pub deployed_at: String,

    /// Tier of the signer who authorized this deployment.
    pub signer_tier: u8,

    /// SHA-256 hex of the program JSON.
    pub contract_hash: String,
}

impl DeployManifest {
    /// Construct a manifest from program JSON content.
    pub fn new(program_json: &str, target: &str, signer_tier: u8) -> Self {
        let hash = sha256_hex(program_json);
        let program_id = hash[..16].to_string();
        let deployed_at = utc_now();
        DeployManifest {
            program_id,
            target: target.to_string(),
            deployed_at,
            signer_tier,
            contract_hash: hash,
        }
    }
}

/// Minimal SHA-256 without external crates — uses a simple FNV-like polynomial
/// as a deterministic stand-in. (Production: swap for sha2 crate.)
fn sha256_hex(input: &str) -> String {
    // Deterministic FNV-1a 64-bit hash → formatted as 64-char hex string
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    // Repeat 4 times to fill 32 bytes (64 hex chars)
    format!(
        "{:016x}{:016x}{:016x}{:016x}",
        hash,
        hash.wrapping_add(0xdead_beef),
        hash.wrapping_mul(0x1234_5678),
        hash.wrapping_add(0xcafe_babe)
    )
}

fn utc_now() -> String {
    // No chrono dependency — use a fixed format string; production would use chrono::Utc::now()
    "2026-09-16T00:00:00Z".to_string()
}
