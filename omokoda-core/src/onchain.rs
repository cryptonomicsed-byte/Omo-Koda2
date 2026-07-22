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
//! being born is never allowed to depend on blockchain availability. Same
//! fail-open discipline applies to the update calls below: a failed
//! on-chain update never blocks or fails a real think/act.
//!
//! `register_agent` mints the real `AgentInfo` object at birth.
//! `update_agent_stats` and `update_glyph_signal` (package v2, upgraded
//! live -- see GARDEN_PACKAGE_V2) make it genuinely dynamic: reputation/
//! tier are mutated in place on the existing object, and the glyph-index
//! divination signal (see divination.rs) is attached via Sui dynamic
//! fields -- traits that did not exist at mint time and evolve as the
//! agent actually thinks. Both are owner-gated in Move (only the minting
//! wallet's address may call them on a given AgentInfo).
//!
//! Configured via env vars, nothing hardcoded beyond the constants above
//! which name the actual deployed package this kernel talks to:
//!   OMOKODA_SUI_REGISTRY    - shared AgentRegistry object id (required;
//!                             unset = minting silently skipped)
//!   OMOKODA_SUI_GAS_BUDGET  - optional, default 20_000_000 MIST (~0.02 SUI)

const GARDEN_PACKAGE: &str = "0x380e0599702b7ebd9005b02f36dd611cff209c94ca678f051233346cf7dbf22e";
/// `update_agent_stats` and `update_glyph_signal` only exist from package
/// version 2 onward (a Sui upgrade doesn't retrofit new functions onto the
/// original package id -- the bytecode at GARDEN_PACKAGE is immutable).
/// register_agent works at either address; the update functions require
/// this one.
const GARDEN_PACKAGE_V2: &str =
    "0xb2108b39f975bf9e20972a8752df0c7b0f014d9f91031696affecd760632b630";
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
        name.bytes()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(",")
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

/// Mutate an existing AgentInfo's reputation/tier in place. Returns true
/// on a real, confirmed on-chain success. Fail-open: any failure is
/// logged and returns false, never propagated as an error the caller
/// must handle -- an on-chain stat refresh is a nice-to-have, not a
/// dependency for real/act to function.
pub async fn update_onchain_stats(nft_id: &str, reputation: u64, tier: u8) -> bool {
    let gas_budget =
        std::env::var("OMOKODA_SUI_GAS_BUDGET").unwrap_or_else(|_| DEFAULT_GAS_BUDGET.to_string());
    let reputation_arg = reputation.to_string();
    let tier_arg = tier.to_string();

    let Ok(output) = tokio::process::Command::new("sui")
        .args([
            "client",
            "call",
            "--package",
            GARDEN_PACKAGE_V2,
            "--module",
            "garden",
            "--function",
            "update_agent_stats",
            "--args",
            nft_id,
            &reputation_arg,
            &tier_arg,
            "--gas-budget",
            &gas_budget,
            "--json",
        ])
        .output()
        .await
    else {
        return false;
    };

    if !output.status.success() {
        eprintln!(
            "[onchain] update_agent_stats failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return false;
    }
    true
}

/// Attach/refresh the agent's glyph-index divination signal on-chain
/// (see divination.rs::recurrence_signal) -- the genuinely dynamic part
/// of the NFT: these fields did not exist at mint and evolve as the
/// agent actually thinks. Same fail-open contract as update_onchain_stats.
pub async fn update_onchain_glyph_signal(
    nft_id: &str,
    dominant_glyph: u8,
    recurrence_count: u64,
    timestamp: u64,
) -> bool {
    let gas_budget =
        std::env::var("OMOKODA_SUI_GAS_BUDGET").unwrap_or_else(|_| DEFAULT_GAS_BUDGET.to_string());
    let glyph_arg = dominant_glyph.to_string();
    let recurrence_arg = recurrence_count.to_string();
    let timestamp_arg = timestamp.to_string();

    let Ok(output) = tokio::process::Command::new("sui")
        .args([
            "client",
            "call",
            "--package",
            GARDEN_PACKAGE_V2,
            "--module",
            "garden",
            "--function",
            "update_glyph_signal",
            "--args",
            nft_id,
            &glyph_arg,
            &recurrence_arg,
            &timestamp_arg,
            "--gas-budget",
            &gas_budget,
            "--json",
        ])
        .output()
        .await
    else {
        return false;
    };

    if !output.status.success() {
        eprintln!(
            "[onchain] update_glyph_signal failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return false;
    }
    true
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

// ---------------------------------------------------------------------------
// Ṣàngó — SkillForge Audit stage: on-chain audit receipts
// ---------------------------------------------------------------------------

/// Standalone package, published because the `garden` package's UpgradeCap
/// is owned by an address not present in this deployment's keystore --
/// same "separate named package constant" pattern as `GARDEN_PACKAGE` /
/// `GARDEN_PACKAGE_V2` above. Module `audit`, function `record`.
const SKILLFORGE_AUDIT_PACKAGE: &str =
    "0x8f15cdd07cd9eedd403d461aa6ea4ae6b6a2e0c69ac0c2a3c1ea440475a57425";

/// Anchor one SkillForge audit decision on-chain: a durable, content-
/// addressed (hash-only, never the raw name/URL) proof that this repo was
/// reviewed and what the verdict was. Fail-open, matching every other
/// on-chain call in this module: `None`/`OMOKODA_SUI_REGISTRY` unset never
/// blocks or fails the forge -- the receipt is a nice-to-have audit trail,
/// not a dependency for SkillForge to function. Reuses `OMOKODA_SUI_REGISTRY`
/// only as the "is on-chain configured at all" signal (this call takes no
/// registry object argument), so a runtime with on-chain birth minting
/// enabled gets audit anchoring for free.
pub async fn record_skillforge_audit(
    skill_name: &str,
    source_url: &str,
    risk_score: u32,
    requires_review: bool,
    approved: bool,
) -> Option<String> {
    std::env::var("OMOKODA_SUI_REGISTRY").ok()?;
    let gas_budget =
        std::env::var("OMOKODA_SUI_GAS_BUDGET").unwrap_or_else(|_| DEFAULT_GAS_BUDGET.to_string());

    let name_hash = blake3::hash(skill_name.as_bytes());
    let url_hash = blake3::hash(source_url.as_bytes());
    let to_vec_arg = |h: &blake3::Hash| {
        format!(
            "[{}]",
            h.as_bytes()
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let output = tokio::process::Command::new("sui")
        .args([
            "client",
            "call",
            "--package",
            SKILLFORGE_AUDIT_PACKAGE,
            "--module",
            "audit",
            "--function",
            "record",
            "--args",
            &to_vec_arg(&name_hash),
            &to_vec_arg(&url_hash),
            &risk_score.to_string(),
            &requires_review.to_string(),
            &approved.to_string(),
            &timestamp.to_string(),
            "--gas-budget",
            &gas_budget,
            "--json",
        ])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        eprintln!(
            "[onchain] skillforge_audit record failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let changes = json.get("objectChanges")?.as_array()?;
    changes.iter().find_map(|o| {
        let obj_type = o.get("objectType")?.as_str()?;
        let change_type = o.get("type")?.as_str()?;
        if change_type == "created" && obj_type.ends_with("::audit::AuditReceipt") {
            o.get("objectId")?.as_str().map(|s| s.to_string())
        } else {
            None
        }
    })
}

/// Transfer ownership of an already-minted on-chain object (agent NFT,
/// skill-audit receipt, etc.) to a new Sui address via the generic
/// `sui client transfer` CLI verb -- there is no bespoke Move entry
/// function for this in `garden.move`, only the initial
/// `transfer::public_transfer` at mint time, so the generic CLI transfer
/// is the real mechanism. Returns `true` on a successful on-chain
/// transfer, `false` on any failure (bad address, `sui` missing,
/// insufficient gas, object not owned by this wallet).
pub async fn transfer_object(object_id: &str, to_address: &str) -> bool {
    let gas_budget =
        std::env::var("OMOKODA_SUI_GAS_BUDGET").unwrap_or_else(|_| DEFAULT_GAS_BUDGET.to_string());

    let output = tokio::process::Command::new("sui")
        .args([
            "client",
            "transfer",
            "--object-id",
            object_id,
            "--to",
            to_address,
            "--gas-budget",
            &gas_budget,
            "--json",
        ])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => true,
        Ok(o) => {
            eprintln!(
                "[onchain] transfer_object failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            false
        }
        Err(e) => {
            eprintln!("[onchain] transfer_object: sui binary unavailable: {e}");
            false
        }
    }
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
