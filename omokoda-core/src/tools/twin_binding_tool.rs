//! `twin_binding` tool — lets an agent publish a real Twin Binding (kind
//! 1903, cryptonomicsed-byte/ip-layer's schemas/twin_binding.md) once it
//! actually has a live VeilSim/OSOVM twin, wiring `ip_layer::
//! publish_twin_binding` into the `act` surface the same way
//! onchain_tools.rs wires `onchain::settle_transaction_tax`.
//!
//! Deliberately NOT wired into the birth flow (unlike the IP Root, which
//! ip_layer.rs publishes automatically at birth): there is no sim_id to
//! bind until a real VeilSim twin exists, so this is an on-demand,
//! agent-initiated call, not a birth-time side effect.

use async_trait::async_trait;
use serde_json::json;

use crate::ip_layer::publish_twin_binding;
use crate::tools::{ExecutionContext, Tool};

pub struct TwinBindingTool;

#[async_trait]
impl Tool for TwinBindingTool {
    fn name(&self) -> &str {
        "twin_binding"
    }

    fn description(&self) -> &str {
        "Publish a real Twin Binding (kind 1903) claiming a VeilSim sim_id IS this agent's \
         digital twin, signed with this agent's own NIP-06 identity and published to the same \
         relay its IP Root used at birth. Requires a real sim_id from an actual VeilSim/OSOVM \
         run — does not fabricate one. Fail-open: returns an event id on success, an explicit \
         error if nothing accepted the event. Params: {sim_id, twin_kind, fidelity?, \
         osovm_veil_receipt_id?}"
    }

    fn required_tier(&self) -> u8 {
        // Same class as nostr_identity_tool (publishing your own signed
        // claim about yourself, no funds/on-chain settlement involved) but
        // it IS a write (a real, hard-to-retract public claim on a relay),
        // so tier 1 rather than tier 0's read-only bar.
        1
    }

    fn is_write_operation(&self) -> bool {
        true
    }

    fn params_schema(&self) -> Option<serde_json::Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "sim_id": { "type": "string", "minLength": 1 },
                "twin_kind": { "type": "string", "minLength": 1 },
                "fidelity": { "type": "string" },
                "osovm_veil_receipt_id": { "type": "string" }
            },
            "required": ["sim_id", "twin_kind"],
            "additionalProperties": false
        }))
    }

    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let sim_id = v["sim_id"].as_str().ok_or("missing sim_id")?;
        let twin_kind = v["twin_kind"].as_str().ok_or("missing twin_kind")?;
        let fidelity = v["fidelity"].as_str();
        let osovm_veil_receipt_id = v["osovm_veil_receipt_id"].as_str();

        match publish_twin_binding(
            &context.odu_identity.mnemonic,
            sim_id,
            twin_kind,
            fidelity,
            osovm_veil_receipt_id,
        )
        .await
        {
            Some(event_id) => Ok((
                json!({ "event_id": event_id, "kind": 1903 }).to_string(),
                crate::usage::TokenUsage::default(),
            )),
            None => Err(
                "twin_binding: no relay accepted the event. This means IP_LAYER_RELAY_URL / \
                 BUZZ_RELAY_URL is unset or unreachable, or sim_id/twin_kind was blank. Check \
                 kernel stderr and retry once a relay is reachable."
                    .to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::AgentId;
    use crate::tools::ToolRegistry;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn ctx() -> ExecutionContext {
        ExecutionContext {
            agent_id: AgentId::from_str("agent-twin-binding-test"),
            name: "test-agent".to_string(),
            tier: 1,
            reputation: 100.0,
            odu_identity: crate::identity::odu::OduIdentity {
                primary_index: 0,
                mnemonic: bipon39::entropy_to_mnemonic(&[7u8; 32])
                    .expect("test entropy should produce a valid mnemonic")
                    .join(" "),
            },
            workspace_root: std::env::current_dir().unwrap(),
            sandbox_mode: false,
        }
    }

    #[test]
    fn is_registered_and_discoverable() {
        let registry = ToolRegistry::new();
        assert!(registry.exists("twin_binding"));
    }

    #[tokio::test]
    async fn missing_sim_id_is_a_clean_param_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tool = TwinBindingTool;
        let result = tool
            .execute(r#"{"twin_kind":"veilsim-1to1"}"#, &ctx())
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sim_id"));
    }

    #[tokio::test]
    async fn unreachable_relay_is_a_clean_execution_error_not_a_panic() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("IP_LAYER_RELAY_URL", "ws://127.0.0.1:19999");
        }
        let tool = TwinBindingTool;
        let result = tool
            .execute(r#"{"sim_id":"sim-1","twin_kind":"veilsim-1to1"}"#, &ctx())
            .await;
        unsafe {
            std::env::remove_var("IP_LAYER_RELAY_URL");
        }
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no relay accepted"));
    }

    #[test]
    fn twin_binding_tool_is_tier_1_write() {
        let tool = TwinBindingTool;
        assert_eq!(tool.required_tier(), 1);
        assert!(tool.is_write_operation());
    }
}
