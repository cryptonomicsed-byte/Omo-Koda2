//! Ṣàngó (Move / Sui) receipt-recording client.
//!
//! When `SANGO_URL` is set, every completed `act` is reported to the Ṣàngó
//! relay — the bridge between the sovereign kernel and on-chain settlement
//! (OSOVM/elegbara_router on Sui). This is how an agent's action trail becomes
//! a durable, eventually on-chain-anchored receipt instead of living only in
//! local session state.
//!
//! Fail-open: a no-op when `SANGO_URL` is unset, so runtimes without a relay
//! configured are unaffected. Fire-and-forget — receipts are best-effort and
//! must never block or fail an otherwise-successful act.

use std::sync::OnceLock;

use reqwest::Client;
use serde_json::json;

static HTTP: OnceLock<Client> = OnceLock::new();
fn http() -> &'static Client {
    HTTP.get_or_init(Client::new)
}

fn normalize_base(raw: &str) -> Option<String> {
    let url = raw.trim().trim_end_matches('/');
    if url.is_empty() {
        None
    } else {
        Some(url.to_string())
    }
}

fn base_url() -> Option<String> {
    normalize_base(&std::env::var("SANGO_URL").ok()?)
}

/// Report a completed act's receipt to the Ṣàngó relay. Best-effort — the
/// send is fire-and-forget; transport errors and non-2xx responses are
/// swallowed rather than surfaced, since a receipt-recording failure must
/// never fail the act it's describing.
pub async fn write_receipt(agent_id: &str, action_tool: &str, overall_score: f32, decision: &str) {
    let Some(base) = base_url() else { return };
    let _ = http()
        .post(format!("{base}/receipt/record"))
        .json(&json!({
            "agent_id": agent_id,
            "action_tool": action_tool,
            "overall_score": overall_score,
            "decision": decision,
        }))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await;
}
