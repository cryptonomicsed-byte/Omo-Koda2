//! DIP bridge — sends DipEnvelope messages to the local DIP server (port 7792).
//!
//! Callers supply NetworkRepr entries; this module never hard-codes a transport.
//! Fail-open: unreachable DIP server never blocks the calling operation.

use serde_json::{json, Value};

const DIP_PORT_DEFAULT: u16 = 7792;

fn dip_base() -> String {
    let port: u16 = std::env::var("DIP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DIP_PORT_DEFAULT);
    format!("http://127.0.0.1:{port}")
}

fn build_envelope(
    kind: &str,
    from: &str,
    to: &str,
    payload_kind: &str,
    payload: Value,
) -> Value {
    json!({
        "envelope_id":  uuid_v4(),
        "kind":         kind,
        "from":         from,
        "to":           to,
        "payload": {
            "kind":    payload_kind,
            "content": payload,
        },
        "ttl_secs":  300,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "signature": "",
    })
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("dip-{:016x}", t as u64 ^ (rand_u64()))
}

fn rand_u64() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::thread::current().id().hash(&mut h);
    h.finish()
}

async fn post_outbound(envelope: Value) -> bool {
    let client = reqwest::Client::new();
    client
        .post(format!("{}/api/dip/outbound", dip_base()))
        .json(&envelope)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Announce agent identity across requested networks.
/// `networks` is a slice of `{network, address, public_key, metadata}` objects
/// matching DIP's NetworkRepr shape (caller constructs these from IdentityVault).
/// Returns true if DIP accepted the announcement; false otherwise (fail-open).
pub async fn announce_identity(
    agent_id: &str,
    agent_did: &str,
    networks: &[Value],
) -> bool {
    let from = format!("agent:{agent_id}");
    let envelope = build_envelope(
        "identity",
        &from,
        "*",
        "identity_announce",
        json!({
            "agent_id":  agent_id,
            "did":       agent_did,
            "kind":      "agent",
            "networks":  networks,
        }),
    );
    post_outbound(envelope).await
}

/// Send a capability advertisement — lets other agents discover what this
/// agent can do (UCX tools, IfáScript tools, compute contributions, etc.).
pub async fn advertise_capability(
    agent_id: &str,
    capability_kind: &str,
    capability_meta: Value,
) -> bool {
    let from = format!("agent:{agent_id}");
    let envelope = build_envelope(
        "capability",
        &from,
        "*",
        "capability_ad",
        json!({
            "agent_id":        agent_id,
            "capability_kind": capability_kind,
            "meta":            capability_meta,
        }),
    );
    post_outbound(envelope).await
}

/// Forward a raw inbound DIP envelope received by an external adapter
/// to the DIP server's inbound endpoint for routing to Vantage/Omo-Koda2.
pub async fn forward_inbound(raw_envelope: Value) -> bool {
    let client = reqwest::Client::new();
    client
        .post(format!("{}/api/dip/inbound", dip_base()))
        .json(&raw_envelope)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
