//! Walrus blob anchoring — Tier 3 of the fractal memory (`specs/memory-fractal.md`).
//!
//! Dense sub-graphs (REM fold archives, media) do not belong on-chain; they
//! belong in Walrus, with only the blob id + BLAKE3 hash anchored in the
//! receipt chain / on Sui. Anchor the hash, not the data.
//!
//! Configuration (fail-open — unset means "not configured", never a crash):
//!   WALRUS_PUBLISHER_URL   e.g. https://publisher.walrus-testnet.walrus.space
//!   WALRUS_AGGREGATOR_URL  e.g. https://aggregator.walrus-testnet.walrus.space
//!   WALRUS_EPOCHS          storage duration in epochs (default 5)
//!
//! Wire protocol (Walrus HTTP API):
//!   store  PUT {publisher}/v1/blobs?epochs=N   body = raw bytes
//!   read   GET {aggregator}/v1/blobs/{blob_id}
//!
//! The store response reports either `newlyCreated.blobObject.blobId` or
//! `alreadyCertified.blobId` (idempotent re-upload) — both are success.

use serde::{Deserialize, Serialize};

use crate::memory::memdir::OduDirectory;

pub const DEFAULT_EPOCHS: u32 = 5;

#[derive(Debug, Clone)]
pub struct WalrusConfig {
    pub publisher_url: String,
    pub aggregator_url: String,
    pub epochs: u32,
}

impl WalrusConfig {
    /// Read config from the environment. `None` = Walrus not configured on
    /// this runtime (the caller should skip anchoring, not fail).
    pub fn from_env() -> Option<Self> {
        let publisher_url = std::env::var("WALRUS_PUBLISHER_URL").ok()?;
        let aggregator_url = std::env::var("WALRUS_AGGREGATOR_URL").ok()?;
        if publisher_url.is_empty() || aggregator_url.is_empty() {
            return None;
        }
        let epochs = std::env::var("WALRUS_EPOCHS")
            .ok()
            .and_then(|e| e.parse().ok())
            .unwrap_or(DEFAULT_EPOCHS);
        Some(Self {
            publisher_url,
            aggregator_url,
            epochs,
        })
    }
}

/// What gets anchored in the receipt chain after a successful store: enough
/// to locate the blob (id) and to prove what it contained (hash), never the
/// data itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalrusAnchor {
    pub blob_id: String,
    /// BLAKE3 of the stored bytes — verifiable against a later `read_blob`.
    pub blake3_hex: String,
    pub byte_len: usize,
}

impl WalrusAnchor {
    /// Receipt payload for `record_receipt("walrus_anchor", …)` — the JSON
    /// that gets hash-committed into the agent's receipt chain.
    pub fn receipt_payload(&self, label: &str) -> String {
        serde_json::json!({
            "kind": "walrus_anchor",
            "label": label,
            "blob_id": self.blob_id,
            "blake3": self.blake3_hex,
            "byte_len": self.byte_len,
        })
        .to_string()
    }
}

/// Extract the blob id from a Walrus publisher store response.
/// Pure — testable without a network.
pub fn parse_store_response(json: &serde_json::Value) -> Option<String> {
    if let Some(id) = json
        .pointer("/newlyCreated/blobObject/blobId")
        .and_then(|v| v.as_str())
    {
        return Some(id.to_string());
    }
    json.pointer("/alreadyCertified/blobId")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// Serialize one archived REM fold (macro node + its micro entries) into a
/// canonical JSON blob and its BLAKE3 hash. `None` if `macro_id` has no
/// archived fold. Pure — the export does not mutate the directory.
pub fn export_fold_json(dir: &OduDirectory, macro_id: &str) -> Option<(String, String)> {
    let micro = dir.archived_folds.get(macro_id)?;
    let macro_node = dir.entries.get(macro_id);
    let blob = serde_json::json!({
        "kind": "rem_fold_archive",
        "version": 1,
        "macro_id": macro_id,
        "macro_node": macro_node,
        "entries": micro,
    });
    let json = serde_json::to_string(&blob).ok()?;
    let hash = blake3::hash(json.as_bytes()).to_hex().to_string();
    Some((json, hash))
}

/// HTTP client for the Walrus publisher/aggregator pair.
pub struct WalrusClient {
    config: WalrusConfig,
    http: reqwest::Client,
}

impl WalrusClient {
    pub fn new(config: WalrusConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// From env, or `None` when Walrus is not configured.
    pub fn from_env() -> Option<Self> {
        WalrusConfig::from_env().map(Self::new)
    }

    /// Store raw bytes; returns the anchor (blob id + hash + length).
    pub async fn store_blob(&self, bytes: Vec<u8>) -> Result<WalrusAnchor, String> {
        let blake3_hex = blake3::hash(&bytes).to_hex().to_string();
        let byte_len = bytes.len();
        let url = format!(
            "{}/v1/blobs?epochs={}",
            self.config.publisher_url.trim_end_matches('/'),
            self.config.epochs
        );
        let resp = self
            .http
            .put(&url)
            .body(bytes)
            .send()
            .await
            .map_err(|e| format!("walrus store failed: {e}"))?;
        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("walrus store: invalid response: {e}"))?;
        if !status.is_success() {
            return Err(format!("walrus store returned {status}: {body}"));
        }
        let blob_id = parse_store_response(&body)
            .ok_or_else(|| format!("walrus store: no blobId in response: {body}"))?;
        Ok(WalrusAnchor {
            blob_id,
            blake3_hex,
            byte_len,
        })
    }

    /// Read a blob back from the aggregator.
    pub async fn read_blob(&self, blob_id: &str) -> Result<Vec<u8>, String> {
        let url = format!(
            "{}/v1/blobs/{}",
            self.config.aggregator_url.trim_end_matches('/'),
            urlencoding::encode(blob_id)
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("walrus read failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("walrus read returned {}", resp.status()));
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("walrus read: body error: {e}"))
    }

    /// Verify a fetched blob against its anchor.
    pub fn verify(anchor: &WalrusAnchor, bytes: &[u8]) -> bool {
        bytes.len() == anchor.byte_len
            && blake3::hash(bytes).to_hex().to_string() == anchor.blake3_hex
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::memdir::OduEntry;

    #[test]
    fn parse_newly_created_response() {
        let v = serde_json::json!({
            "newlyCreated": {
                "blobObject": {"id": "0xabc", "blobId": "b64blobid", "size": 42}
            }
        });
        assert_eq!(parse_store_response(&v).as_deref(), Some("b64blobid"));
    }

    #[test]
    fn parse_already_certified_response() {
        let v = serde_json::json!({
            "alreadyCertified": {"blobId": "existing-id", "endEpoch": 99}
        });
        assert_eq!(parse_store_response(&v).as_deref(), Some("existing-id"));
    }

    #[test]
    fn parse_rejects_unknown_shape() {
        assert!(parse_store_response(&serde_json::json!({"error": "nope"})).is_none());
    }

    #[test]
    fn export_fold_json_round_trips_and_hashes() {
        let mut dir = OduDirectory::new();
        let mut macro_node = OduEntry::new("rem:topics/x:100", "[REM fold] 2 entries", "topics/x");
        macro_node.tags.push("rem-fold".to_string());
        dir.insert(macro_node);
        dir.archive_fold(
            "rem:topics/x:100",
            vec![
                OduEntry::new("a", "first", "topics/x"),
                OduEntry::new("b", "second", "topics/x"),
            ],
        );

        let (json, hash) = export_fold_json(&dir, "rem:topics/x:100").expect("fold exists");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["kind"], "rem_fold_archive");
        assert_eq!(parsed["entries"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["macro_node"]["id"], "rem:topics/x:100");
        assert_eq!(hash, blake3::hash(json.as_bytes()).to_hex().to_string());

        assert!(export_fold_json(&dir, "rem:missing:0").is_none());
    }

    #[test]
    fn anchor_receipt_payload_carries_hash_not_data() {
        let anchor = WalrusAnchor {
            blob_id: "blob-1".to_string(),
            blake3_hex: "aa".repeat(32),
            byte_len: 1024,
        };
        let payload: serde_json::Value =
            serde_json::from_str(&anchor.receipt_payload("rem:topics/x:100")).unwrap();
        assert_eq!(payload["kind"], "walrus_anchor");
        assert_eq!(payload["blob_id"], "blob-1");
        assert_eq!(payload["byte_len"], 1024);
        assert!(payload.get("entries").is_none(), "never the data itself");
    }

    #[test]
    fn verify_checks_hash_and_length() {
        let bytes = b"hello walrus".to_vec();
        let anchor = WalrusAnchor {
            blob_id: "x".to_string(),
            blake3_hex: blake3::hash(&bytes).to_hex().to_string(),
            byte_len: bytes.len(),
        };
        assert!(WalrusClient::verify(&anchor, &bytes));
        assert!(!WalrusClient::verify(&anchor, b"tampered"));
    }

    #[test]
    fn config_from_env_requires_both_urls() {
        // Not set in the test environment → not configured.
        std::env::remove_var("WALRUS_PUBLISHER_URL");
        std::env::remove_var("WALRUS_AGGREGATOR_URL");
        assert!(WalrusConfig::from_env().is_none());
    }
}
