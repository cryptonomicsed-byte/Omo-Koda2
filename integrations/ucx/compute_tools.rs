//! Omo-Koda2 / UCX compute tools.
//!
//! Exposes two agent tools:
//!   - `ucx_request_compute` — submit a compute job on behalf of the agent
//!   - `ucx_offer_compute`   — register the local machine as a UCX provider
//!
//! These are thin HTTP wrappers around the UCX broker API.
//! The agent's identity (Omo-Koda2 agent_id) becomes the submitter_id in UCX.
//!
//! Dependency direction: Omo-Koda2 calls UCX over HTTP; it does not link
//! ucx-protocol crate directly.  That keeps the agent layer decoupled.

use serde_json::{json, Value};

/// Submit a compute job through the UCX broker.
///
/// `spec` is a JSON object matching the UCX Job wire format.
/// Returns the Allocation JSON or an error string.
pub async fn ucx_request_compute(
    ucx_base_url: &str,
    agent_id: &str,
    agent_key: &str,
    spec: Value,
) -> Result<Value, String> {
    let url = format!("{ucx_base_url}/api/jobs");
    let body = json!({
        "submitter_id": agent_id,
        "workload":     spec.get("workload").cloned().unwrap_or(json!("Generic")),
        "requirements": spec.get("requirements").cloned().unwrap_or(json!({})),
        "constraints":  spec.get("constraints").cloned().unwrap_or(json!({})),
        "runtime_spec": spec.get("runtime_spec").cloned().unwrap_or(json!({})),
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("X-Agent-Key", agent_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let val: Value = resp.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        return Err(format!("ucx_request_compute {status}: {val}"));
    }
    Ok(val)
}

/// Register this agent's machine as a UCX provider via Vantage rendezvous.
///
/// `capability` is a JSON object matching ProviderCapability wire format.
pub async fn ucx_offer_compute(
    vantage_base_url: &str,
    agent_key: &str,
    capability: Value,
) -> Result<Value, String> {
    let url = format!("{vantage_base_url}/api/ucx/providers/register");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("X-Agent-Key", agent_key)
        .json(&capability)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let val: Value = resp.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        return Err(format!("ucx_offer_compute {status}: {val}"));
    }
    Ok(val)
}
