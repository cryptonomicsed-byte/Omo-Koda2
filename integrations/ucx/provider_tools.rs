//! UCX provider registration tools for Omo-Koda2 agents.
//!
//! Allows an agent to register its local machine (or a remote GPU node) as a
//! UCX compute provider via Vantage /api/ucx/providers/register, so other
//! agents can discover and submit jobs to it.

use serde_json::{json, Value};

/// Register this agent's machine as a UCX native provider.
///
/// Publishes the provider capability to Vantage so it appears in VantageDiscovery
/// queries. The UCX broker on this machine must also be running and reachable.
pub async fn register_provider(
    vantage_base: &str,
    agent_id: &str,
    agent_key: &str,
    ucx_broker_url: &str,
    capability: Value,
) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let url = format!("{vantage_base}/api/ucx/providers/register");

    let body = json!({
        "agent_id":       agent_id,
        "broker_url":     ucx_broker_url,
        "capability":     capability,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {agent_key}"))
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("register_provider HTTP error: {e}"))?;

    if resp.status().is_success() {
        resp.json::<Value>().await
            .map_err(|e| format!("register_provider parse error: {e}"))
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(format!("register_provider {status}: {body}"))
    }
}

/// Deregister this agent's UCX provider from Vantage.
pub async fn deregister_provider(
    vantage_base: &str,
    agent_id: &str,
    agent_key: &str,
    provider_id: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{vantage_base}/api/ucx/providers/{provider_id}");

    let resp = client
        .delete(&url)
        .header("Authorization", format!("Bearer {agent_key}"))
        .query(&[("agent_id", agent_id)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("deregister_provider HTTP error: {e}"))?;

    if resp.status().is_success() || resp.status().as_u16() == 404 {
        Ok(())
    } else {
        Err(format!("deregister_provider {}: {}", resp.status(), resp.text().await.unwrap_or_default()))
    }
}

/// List all UCX providers visible to this agent via Vantage.
pub async fn list_providers(
    vantage_base: &str,
    agent_id: &str,
    agent_key: &str,
) -> Result<Vec<Value>, String> {
    let client = reqwest::Client::new();
    let url = format!("{vantage_base}/api/ucx/providers");

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {agent_key}"))
        .query(&[("agent_id", agent_id)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("list_providers HTTP error: {e}"))?;

    if resp.status().is_success() {
        let val: Value = resp.json().await
            .map_err(|e| format!("list_providers parse error: {e}"))?;
        Ok(val.as_array().cloned().unwrap_or_default())
    } else {
        Err(format!("list_providers {}", resp.status()))
    }
}

/// Build a ProviderCapability JSON from local machine specs.
/// Uses env vars or sensible defaults for GPU/CPU discovery.
pub fn build_local_capability(
    agent_id: &str,
    provider_id: &str,
    broker_url: &str,
) -> Value {
    let vram_gb = std::env::var("UCX_VRAM_GB")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let ram_gb = std::env::var("UCX_RAM_GB")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(8.0);
    let cpu_cores = std::env::var("UCX_CPU_CORES")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(4);

    json!({
        "provider_id":     provider_id,
        "owner_agent_id":  agent_id,
        "tier":            "Personal",
        "trust":           "Standard",
        "gpu": if vram_gb > 0.0 { json!({
            "vendor":   std::env::var("UCX_GPU_VENDOR").unwrap_or_else(|_| "Nvidia".into()),
            "model":    std::env::var("UCX_GPU_MODEL").unwrap_or_else(|_| "local".into()),
            "vram_gb":  vram_gb,
            "fp16":     false,
            "bf16":     false,
            "cuda":     false,
            "rocm":     false,
            "count":    1,
        }) } else { Value::Null },
        "cpu": {
            "cores": cpu_cores,
            "arch":  "Aarch64",
        },
        "ram_gb":                ram_gb,
        "disk_gb":               0.0,
        "runtimes":              [],
        "price_gpu_hour_cents":  null,
        "price_cpu_hour_cents":  0,
        "policy_deny":           [],
        "regions":               ["local"],
        "broker_url":            broker_url,
    })
}
