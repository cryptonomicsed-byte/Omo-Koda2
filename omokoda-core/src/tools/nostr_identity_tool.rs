//! nostr_identity tool — surfaces an agent's derived NIP-06 secp256k1
//! identity (npub only, never the private key) via the same tool-registry
//! path every other kernel capability uses. Follows wallet_tools.rs's
//! precedent: raw key material never crosses the tool-call boundary into
//! agent-visible context, exactly like wallet signing stays server-side.
//! Read-only, tier 0 -- deriving and viewing your own public identity is
//! not a privileged operation.

use async_trait::async_trait;
use serde_json::json;

use crate::identity::nip06::derive_nip06_identity;
use crate::tools::{ExecutionContext, Tool};

pub struct NostrIdentityTool;

#[async_trait]
impl Tool for NostrIdentityTool {
    fn name(&self) -> &str {
        "nostr_identity"
    }
    fn description(&self) -> &str {
        "Get this agent's NIP-06 secp256k1/Nostr public identity (npub), \
         deterministically derived from the same BIPON39 birth seed as \
         its native identity. Returns npub and hex pubkey only -- never \
         the private key. Params: none."
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(
        &self,
        _params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let identity = derive_nip06_identity(&context.odu_identity.mnemonic, 0)?;
        let result = json!({
            "npub": identity.npub,
            "public_key_hex": identity.public_key_hex,
        });
        Ok((result.to_string(), crate::usage::TokenUsage::default()))
    }
}
