//! Walrus tool — store/read blobs on decentralized storage through `act`.
//!
//! `store` returns a [`WalrusAnchor`](crate::memory::walrus::WalrusAnchor)
//! as JSON: blob id + BLAKE3 + length. The act receipt hash-commits the
//! anchor (the params and output pass through the normal receipt path), so
//! the chain holds the proof while Walrus holds the bytes.

use async_trait::async_trait;

use crate::memory::walrus::WalrusClient;
use crate::tools::{ExecutionContext, Tool};
use crate::usage::TokenUsage;

/// Cap read output relayed into the session; larger blobs should be fetched
/// out-of-band with the anchor as proof.
const READ_OUTPUT_CAP: usize = 64 * 1024;

pub struct WalrusTool;

#[async_trait]
impl Tool for WalrusTool {
    fn name(&self) -> &str {
        "walrus"
    }
    fn description(&self) -> &str {
        "Walrus decentralized blob storage — {\"op\":\"store\",\"content\":\"…\"} \
         returns {blob_id, blake3, byte_len}; {\"op\":\"read\",\"blob_id\":\"…\"} \
         fetches a blob. Set WALRUS_PUBLISHER_URL and WALRUS_AGGREGATOR_URL to enable."
    }
    fn required_tier(&self) -> u8 {
        2
    }
    fn is_write_operation(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let v: serde_json::Value =
            serde_json::from_str(params).map_err(|e| format!("invalid params: {e}"))?;
        let op = v
            .get("op")
            .and_then(|o| o.as_str())
            .ok_or("missing 'op' — \"store\" or \"read\"")?;

        let client = WalrusClient::from_env().ok_or(
            "walrus is not configured — set WALRUS_PUBLISHER_URL and WALRUS_AGGREGATOR_URL",
        )?;

        match op {
            "store" => {
                let content = v
                    .get("content")
                    .and_then(|c| c.as_str())
                    .ok_or("store requires 'content'")?;
                let anchor = client.store_blob(content.as_bytes().to_vec()).await?;
                serde_json::to_string(&anchor)
                    .map(|s| (s, TokenUsage::default()))
                    .map_err(|e| format!("anchor serialization failed: {e}"))
            }
            "read" => {
                let blob_id = v
                    .get("blob_id")
                    .and_then(|b| b.as_str())
                    .ok_or("read requires 'blob_id'")?;
                let bytes = client.read_blob(blob_id).await?;
                let mut text = String::from_utf8_lossy(&bytes).into_owned();
                if text.len() > READ_OUTPUT_CAP {
                    text.truncate(READ_OUTPUT_CAP);
                    text.push_str("\n…[truncated]");
                }
                Ok((text, TokenUsage::default()))
            }
            other => Err(format!("unknown op '{other}' — use \"store\" or \"read\"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ExecutionContext {
        ExecutionContext {
            agent_id: crate::identity::AgentId::from_str("agent-test"),
            name: "luna".to_string(),
            tier: 2,
            reputation: 50.0,
            odu_identity: crate::identity::odu::OduIdentity {
                primary_index: 0,
                mnemonic: String::new(),
            },
            workspace_root: std::env::temp_dir(),
            sandbox_mode: false,
        }
    }

    #[test]
    fn walrus_tool_is_creator_tier_write() {
        let t = WalrusTool;
        assert_eq!(t.name(), "walrus");
        assert_eq!(t.required_tier(), 2);
        assert!(t.is_write_operation());
    }

    #[tokio::test]
    async fn rejects_bad_params_before_network() {
        let t = WalrusTool;
        assert!(t.execute("not json", &ctx()).await.is_err());
        assert!(t.execute("{}", &ctx()).await.is_err());
    }

    #[tokio::test]
    async fn unconfigured_env_is_a_clean_error() {
        std::env::remove_var("WALRUS_PUBLISHER_URL");
        std::env::remove_var("WALRUS_AGGREGATOR_URL");
        let t = WalrusTool;
        let err = t
            .execute(r#"{"op":"store","content":"x"}"#, &ctx())
            .await
            .unwrap_err();
        assert!(err.contains("not configured"));
    }
}
