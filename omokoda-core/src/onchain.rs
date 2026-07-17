//! Real, optional on-chain birth minting via a deployed Sui Move contract
//! (`omokoda::garden::register_agent`, package
//! `0x380e0599702b7ebd9005b02f36dd611cff209c94ca678f051233346cf7dbf22e`,
//! testnet). Shells out to the `sui` CLI -- the same "no SDK, real
//! binary, subprocess call" pattern already used for Hermes, since a full
//! Sui Rust SDK is a much heavier dependency than this kernel needs for
//! one entry-function call.
//!
//! Fail-open by design, matching Vantage registration's own pattern: if
//! `sui` isn't installed, gas is exhausted, or the network call fails,
//! birth proceeds without an on-chain record rather than blocking --
//! being born is never allowed to depend on blockchain availability.
//!
//! Honest scope: `garden::register_agent` mints a real object
//! (`AgentInfo { name, owner, reputation, tier }`) but the deployed
//! contract has no update entry function yet -- this is a real mint, not
//! yet a dynamic NFT whose on-chain fields evolve with the agent. That's
//! separate, real follow-up work (a package upgrade), not done here.
//!
//! Configured via env vars, nothing hardcoded beyond the constants above
//! which name the actual deployed package this kernel talks to:
//!   OMOKODA_SUI_REGISTRY    - shared AgentRegistry object id (required;
//!                             unset = minting silently skipped)
//!   OMOKODA_SUI_GAS_BUDGET  - optional, default 20_000_000 MIST (~0.02 SUI)

const GARDEN_PACKAGE: &str = "0x380e0599702b7ebd9005b02f36dd611cff209c94ca678f051233346cf7dbf22e";
const DEFAULT_GAS_BUDGET: &str = "20000000";

/// Mint a real on-chain `AgentInfo` object for a newborn agent. Returns
/// the minted object's id on success, `None` on any failure (missing
/// config, missing `sui` binary, insufficient gas, network error) --
/// callers must treat `None` as "no on-chain record yet", never as a
/// reason to fail the birth itself.
pub async fn mint_onchain_agent(name: &str) -> Option<String> {
    let registry = std::env::var("OMOKODA_SUI_REGISTRY").ok()?;
    let gas_budget =
        std::env::var("OMOKODA_SUI_GAS_BUDGET").unwrap_or_else(|_| DEFAULT_GAS_BUDGET.to_string());

    // Move `vector<u8>` argument as a JSON array of byte values --
    // `sui client call`'s CLI arg-parsing accepts this literally.
    let name_arg = format!(
        "[{}]",
        name.bytes().map(|b| b.to_string()).collect::<Vec<_>>().join(",")
    );

    let output = tokio::process::Command::new("sui")
        .args([
            "client",
            "call",
            "--package",
            GARDEN_PACKAGE,
            "--module",
            "garden",
            "--function",
            "register_agent",
            "--args",
            &registry,
            &name_arg,
            "--gas-budget",
            &gas_budget,
            "--json",
        ])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        eprintln!(
            "[onchain] register_agent failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    extract_minted_agent_info_id(&json)
}

fn extract_minted_agent_info_id(json: &serde_json::Value) -> Option<String> {
    let changes = json.get("objectChanges")?.as_array()?;
    changes.iter().find_map(|o| {
        let obj_type = o.get("objectType")?.as_str()?;
        let change_type = o.get("type")?.as_str()?;
        if change_type == "created" && obj_type.ends_with("::garden::AgentInfo") {
            o.get("objectId")?.as_str().map(|s| s.to_string())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_the_agent_info_object_from_a_real_response_shape() {
        // Shape verified live against a real testnet transaction
        // (objectId 0xcb792f0c...), trimmed to the fields this parser
        // actually reads.
        let json = serde_json::json!({
            "objectChanges": [
                {
                    "type": "mutated",
                    "objectType": "0x2::coin::Coin<0x2::sui::SUI>",
                    "objectId": "0xsomecoin"
                },
                {
                    "type": "created",
                    "objectType": "0x380e0599702b7ebd9005b02f36dd611cff209c94ca678f051233346cf7dbf22e::garden::AgentInfo",
                    "objectId": "0xcb792f0c6c4b8cb55e7fd0fafb7896ba0b98f6d6f33bd010d8692cff1935c034"
                }
            ]
        });
        assert_eq!(
            extract_minted_agent_info_id(&json),
            Some("0xcb792f0c6c4b8cb55e7fd0fafb7896ba0b98f6d6f33bd010d8692cff1935c034".to_string())
        );
    }

    #[test]
    fn no_agent_info_object_returns_none() {
        let json = serde_json::json!({
            "objectChanges": [
                {"type": "mutated", "objectType": "0x2::coin::Coin<0x2::sui::SUI>", "objectId": "0xsomecoin"}
            ]
        });
        assert_eq!(extract_minted_agent_info_id(&json), None);
    }

    #[test]
    fn malformed_response_returns_none_not_a_panic() {
        let json = serde_json::json!({"unexpected": "shape"});
        assert_eq!(extract_minted_agent_info_id(&json), None);
    }
}
