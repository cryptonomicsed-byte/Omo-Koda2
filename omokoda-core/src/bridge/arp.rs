//! ARP bridge — builds and submits AgentLifecycle ActionReceipts for the
//! three canonical lifecycle events: birth, think, and act.
//!
//! Uses the Vantage /api/arp/receipts endpoint. Fail-open: Vantage being
//! unreachable never blocks birth/think/act.
//!
//! Every receipt now carries a `Gix1` wire envelope (Phase 1 Step 7 of the
//! GIX spec). The `gix1_for_receipt` hand-roll has been replaced with
//! canonical `gix_types::Gix1::new`.

use gix_types::{Gix1, GixKind, GixNamespace, RoutingHints};
use serde_json::{json, Value};

fn vantage_base() -> String {
    std::env::var("VANTAGE_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn api_key() -> String {
    std::env::var("VANTAGE_API_KEY").unwrap_or_default()
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn gix1_for_receipt(receipt_id: &str) -> Value {
    let created_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let env = Gix1::new(
        GixKind::Receipt,
        GixNamespace::ArpReceipt,
        receipt_id.as_bytes(),
        None,
        created_at_ms,
        RoutingHints::default(),
    );

    json!({
        "canonical_id": hex::encode(env.canonical_id),
        "glyph":        env.glyph.to_string(),
        "kind":         "receipt",
        "odu_base":     env.odu_base,
        "odu_composed": env.odu_composed,
        "namespace":    "arp_receipt",
        "version":      env.version,
        "envelope_hash": hex::encode(env.integrity.envelope_hash),
    })
}

fn make_receipt(
    agent_id: &str,
    action_kind: &str,
    target: &str,
    outcome: &str,
    payload: Value,
    previous_hash: Option<&str>,
) -> Value {
    let receipt_id = uuid_v4();
    let gix1 = gix1_for_receipt(&receipt_id);
    json!({
        "receipt_id":  receipt_id,
        "gix1":        gix1,
        "kind":        "AgentLifecycle",
        "kind_ext":    action_kind,
        "principal": {
            "principal_id": agent_id,
            "kind": "Agent",
            "agent_id": agent_id,
            "session_id": null,
            "agent_tier": null,
            "capabilities": [],
        },
        "action": {
            "kind":    action_kind,
            "target":  target,
            "outcome": outcome,
            "params":  payload,
        },
        "evidence_ids": [],
        "witness_attestations": [],
        "throne_evaluations": [],
        "consensus_receipt": null,
        "physical_attestation": null,
        "zangbeto_anchor": null,
        "nostr_event_id": null,
        "timestamp": now_secs(),
        "execution_id": null,
        "previous_hash": previous_hash,
        "signature": "",
    })
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let seed = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    let mut h = seed;
    n.hash(&mut h);
    std::thread::current().id().hash(&mut h);
    let v = h.finish();
    format!("{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (v >> 32) as u32,
        (v >> 16) as u16,
        (v >> 4) as u16 & 0x0fff,
        ((v & 0x3fff) | 0x8000) as u16,
        v & 0xffffffffffff_u64,
    )
}

async fn post_receipt(receipt: Value) -> bool {
    let client = reqwest::Client::new();
    let key = api_key();
    let mut req = client
        .post(format!("{}/api/arp/receipts", vantage_base()))
        .json(&receipt)
        .timeout(std::time::Duration::from_secs(4));
    if !key.is_empty() {
        req = req.header("X-Agent-Key", &key);
    }
    req.send().await.map(|r| r.status().is_success()).unwrap_or(false)
}

/// Birth receipt — emitted once at agent creation.
pub async fn receipt_birth(
    agent_id: &str,
    genesis_receipt_id: &str,
    agent_name: &str,
) -> bool {
    let receipt = make_receipt(
        agent_id,
        "birth",
        genesis_receipt_id,
        "success",
        json!({ "agent_name": agent_name, "genesis_receipt_id": genesis_receipt_id }),
        None,
    );
    post_receipt(receipt).await
}

/// Think receipt — emitted after each THINK execution.
pub async fn receipt_think(
    agent_id: &str,
    think_id: &str,
    prompt_summary: &str,
    previous_hash: Option<&str>,
) -> bool {
    let receipt = make_receipt(
        agent_id,
        "think",
        think_id,
        "success",
        json!({ "prompt_summary": prompt_summary }),
        previous_hash,
    );
    post_receipt(receipt).await
}

/// Act receipt — emitted after each ACT/tool execution.
pub async fn receipt_act(
    agent_id: &str,
    act_id: &str,
    tool_name: &str,
    outcome: &str,
    previous_hash: Option<&str>,
) -> bool {
    let receipt = make_receipt(
        agent_id,
        "act",
        act_id,
        outcome,
        json!({ "tool_name": tool_name }),
        previous_hash,
    );
    post_receipt(receipt).await
}
