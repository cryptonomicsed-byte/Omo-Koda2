//! UCX ComputeReceipt storage and Zàngbétò anchoring for Omo-Koda2 agents.
//!
//! When an agent submits a compute job and it completes, this module:
//!   1. Fetches the ComputeReceipt from UCX broker
//!   2. Stores it in the agent's local receipt store
//!   3. Links zangbeto_anchor if present (settlement proof)
//!   4. Emits an ARP ActionReceipt wrapping the ComputeReceipt

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Local record of a completed compute job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJobRecord {
    pub job_id:          String,
    pub provider_id:     String,
    pub completed_at:    String,
    pub gpu_seconds:     f64,
    pub cpu_seconds:     f64,
    pub amount_cents:    u64,
    pub currency:        String,
    pub artifact_hash:   Option<String>,
    pub execution_hash:  Option<String>,
    pub zangbeto_anchor: Option<String>,
    pub receipt_hash:    Option<String>,
    /// ARP ActionReceipt id emitted for this compute job.
    pub arp_receipt_id:  Option<String>,
}

/// Fetch a ComputeReceipt from the UCX broker and store it locally.
///
/// Returns the stored `ComputeJobRecord` or an error.
pub async fn fetch_and_store(
    ucx_base: &str,
    job_id: &str,
    provider_id: &str,
) -> Result<ComputeJobRecord, String> {
    let url = format!("{ucx_base}/api/jobs/{job_id}/{provider_id}/receipt");
    let resp = reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("fetch_and_store HTTP error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("fetch_and_store {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let val: Value = resp.json().await
        .map_err(|e| format!("fetch_and_store parse error: {e}"))?;

    let record = ComputeJobRecord {
        job_id:          val["job_id"].as_str().unwrap_or(job_id).to_string(),
        provider_id:     val["provider_id"].as_str().unwrap_or(provider_id).to_string(),
        completed_at:    val["completed_at"].as_str().unwrap_or("").to_string(),
        gpu_seconds:     val["resources"]["gpu_seconds"].as_f64().unwrap_or(0.0),
        cpu_seconds:     val["resources"]["cpu_seconds"].as_f64().unwrap_or(0.0),
        amount_cents:    val["billing"]["amount_cents"].as_u64().unwrap_or(0),
        currency:        val["billing"]["currency"].as_str().unwrap_or("USD").to_string(),
        artifact_hash:   val["verification"]["artifact_hash"].as_str().map(str::to_string),
        execution_hash:  val["verification"]["execution_hash"].as_str().map(str::to_string),
        zangbeto_anchor: val["zangbeto_anchor"].as_str().map(str::to_string),
        receipt_hash:    val["receipt_hash"].as_str().map(str::to_string),
        arp_receipt_id:  None,
    };

    Ok(record)
}

/// Emit an ARP ActionReceipt for a completed compute job.
/// Links the ComputeJobRecord's execution_hash as evidence.
pub async fn emit_arp_receipt(
    vantage_base: &str,
    agent_id: &str,
    agent_key: &str,
    record: &ComputeJobRecord,
) -> Option<String> {
    let evidence = record.execution_hash.as_deref()
        .or(record.artifact_hash.as_deref())
        .unwrap_or("no-hash");

    let client = reqwest::Client::new();
    let url = format!("{vantage_base}/api/arp/receipts");
    let body = json!({
        "agent_id":    agent_id,
        "receipt_kind": "Compute",
        "action_id":   record.job_id,
        "summary":     format!(
            "compute: provider={} gpu_seconds={:.1} cost={} cents",
            record.provider_id, record.gpu_seconds, record.amount_cents
        ),
        "evidence_hash": evidence,
        "zangbeto_anchor": record.zangbeto_anchor,
        "metadata": {
            "provider_id":  record.provider_id,
            "gpu_seconds":  record.gpu_seconds,
            "amount_cents": record.amount_cents,
        },
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {agent_key}"))
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;

    if resp.status().is_success() {
        let val: Value = resp.json().await.ok()?;
        val["receipt_id"].as_str().map(str::to_string)
    } else {
        None
    }
}

/// Notify OSOVM of a completed GPU job (gap #22: UCX → OSOVM settlement).
///
/// POSTs a GPU_CONTRIBUTION opcode (0x3f) to the OSOVM run endpoint so the
/// VM can authorize Dopamine allocation.  Fail-open: returns None on error.
pub async fn notify_osovm(
    osovm_base: &str,
    agent_id:   &str,
    record:     &ComputeJobRecord,
) -> Option<Value> {
    let receipt_hash = record.receipt_hash.as_deref()
        .or(record.execution_hash.as_deref())
        .unwrap_or("unknown");

    let body = json!({
        "opcode": "GPU_CONTRIBUTION",
        "args": {
            "agent_id":     agent_id,
            "job_id":       record.job_id,
            "provider_id":  record.provider_id,
            "gpu_seconds":  record.gpu_seconds,
            "receipt_hash": receipt_hash,
            "f1_score":     null,
        }
    });

    let resp = reqwest::Client::new()
        .post(&format!("{osovm_base}/run"))
        .json(&body)
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
        .ok()?;

    if resp.status().is_success() {
        resp.json::<Value>().await.ok()
    } else {
        None
    }
}

/// Full pipeline: fetch receipt → emit ARP → notify OSOVM → return enriched record.
/// Fail-open on ARP and OSOVM: returns record even if either call fails.
pub async fn fetch_and_emit(
    ucx_base:    &str,
    vantage_base: &str,
    agent_id:    &str,
    agent_key:   &str,
    job_id:      &str,
    provider_id: &str,
) -> Result<ComputeJobRecord, String> {
    let mut record = fetch_and_store(ucx_base, job_id, provider_id).await?;

    let arp_id = emit_arp_receipt(vantage_base, agent_id, agent_key, &record).await;
    record.arp_receipt_id = arp_id;

    // Gap #22: UCX → OSOVM settlement — GPU_CONTRIBUTION opcode authorizes Dopamine
    if let Ok(osovm_url) = std::env::var("OSOVM_URL") {
        let _ = notify_osovm(&osovm_url, agent_id, &record).await;
    }

    Ok(record)
}
