use async_trait::async_trait;
use omokoda_mesh::{
    negotiation::{Commitment, CommitmentKind, Proposal},
    router::MeshRouter,
    state::MeshState,
    types::{MeshMembership, MeshRole},
};
use reqwest::Client;
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock, Mutex, OnceLock,
};

use crate::tools::{ExecutionContext, Tool};

static MESH_ROUTER: LazyLock<Mutex<MeshRouter>> = LazyLock::new(|| Mutex::new(MeshRouter::new()));

// ── Vantage coordination backend ────────────────────────────────────────
//
// Block Mesh combines Ọ̀mọ̀ Kọ́dà (the sovereign agent OS) with Vantage (the shared
// collaboration workspace). When `VANTAGE_URL` is configured, the mesh_* tools
// forward to Vantage's `/api/mesh/*` API so agents in separate Ọ̀mọ̀ Kọ́dà runtimes
// discover, negotiate, and share resources on a common block. When it is not set,
// the tools fall back to the local in-memory `MESH_ROUTER` below (fail-open — no
// behaviour change for users who don't run Vantage).

static HTTP: OnceLock<Client> = OnceLock::new();
fn http() -> &'static Client {
    HTTP.get_or_init(Client::new)
}

/// Set on the first write call so the agent is lazily registered in its block.
static VANTAGE_JOINED: AtomicBool = AtomicBool::new(false);

/// Vantage API key minted at birth when `VANTAGE_KEY` is not pre-provisioned.
/// Lets a self-registered agent authenticate subsequent mesh calls in-process.
static MINTED_KEY: OnceLock<String> = OnceLock::new();

pub struct VantageClient {
    base_url: String,
    api_key: String,
    block_id: String,
}

impl VantageClient {
    fn from_env() -> Option<Self> {
        let base = std::env::var("VANTAGE_URL").ok()?;
        if base.trim().is_empty() {
            return None;
        }
        Some(Self {
            base_url: base.trim_end_matches('/').to_string(),
            api_key: std::env::var("VANTAGE_KEY").unwrap_or_default(),
            block_id: std::env::var("MESH_BLOCK_ID").unwrap_or_else(|_| "default".to_string()),
        })
    }

    /// The API key to authenticate with: the env-provisioned key if present,
    /// otherwise a key minted during birth-time self-registration.
    fn effective_key(&self) -> Option<String> {
        if !self.api_key.is_empty() {
            Some(self.api_key.clone())
        } else {
            MINTED_KEY.get().cloned()
        }
    }

    async fn send(&self, req: reqwest::RequestBuilder) -> Result<serde_json::Value, String> {
        let req = match self.effective_key() {
            Some(key) => req.header("X-Agent-Key", key),
            None => req,
        };
        let resp = req
            .send()
            .await
            .map_err(|e| format!("vantage request failed: {e}"))?;
        let status = resp.status();
        let val: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
        if !status.is_success() {
            return Err(format!("vantage returned {status}: {val}"));
        }
        Ok(val)
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value, String> {
        self.send(
            http()
                .post(format!("{}{}", self.base_url, path))
                .json(&body),
        )
        .await
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        self.send(http().get(format!("{}{}", self.base_url, path)))
            .await
    }

    /// Register this agent in its block on the first write. Idempotent on the
    /// Vantage side (INSERT ... ON CONFLICT), best-effort here.
    async fn ensure_joined(&self, agent_id: &str) {
        if VANTAGE_JOINED.swap(true, Ordering::Relaxed) {
            return;
        }
        let _ = self
            .post(
                "/api/mesh/agents/join",
                json!({
                    "agent_id": agent_id,
                    "block_id": &self.block_id,
                    "role": "home",
                    "capabilities": {}
                }),
            )
            .await;
    }
}

static VANTAGE: LazyLock<Option<VantageClient>> = LazyLock::new(VantageClient::from_env);

/// Auto-register a newborn agent on Vantage at birth: create its account when no
/// `VANTAGE_KEY` is provisioned, then join its home block carrying verifiable
/// sovereign identity (Ed25519 public key, DNA fingerprint, primary Odù, and
/// personality profile). Fail-open: a no-op when `VANTAGE_URL` is unset, so
/// runtimes without Vantage are unaffected. Best-effort — transport/Vantage
/// errors are swallowed rather than failing the birth.
pub async fn register_newborn(
    agent_id: &str,
    human_name: &str,
    public_key_hex: &str,
    dna_fingerprint: &str,
    odu_index: u8,
    personality: serde_json::Value,
) {
    let Some(vc) = VANTAGE.as_ref() else {
        return;
    };

    // 1. Ensure this runtime has a Vantage identity. If no key was provisioned
    //    via VANTAGE_KEY, self-register to mint one and cache it for later calls.
    if vc.effective_key().is_none() {
        let short_pubkey = public_key_hex.get(..16).unwrap_or(public_key_hex);
        let bio = format!("Ọmọ Kọ́dà sovereign agent · Odù #{odu_index} · key {short_pubkey}");
        if let Ok(val) = vc
            .post(
                "/api/agents/register",
                json!({ "name": agent_id, "bio": bio }),
            )
            .await
        {
            if let Some(key) = val["api_key"].as_str() {
                let _ = MINTED_KEY.set(key.to_string());
            }
        }
    }

    // 2. Join the home block, publishing full identity into the agent's
    //    capabilities so neighbors can verify lineage and temperament.
    let _ = vc
        .post(
            "/api/mesh/agents/join",
            json!({
                "agent_id": agent_id,
                "block_id": &vc.block_id,
                "role": "home",
                "capabilities": {
                    "kind": "omo-koda-sovereign",
                    "human_name": human_name,
                    "public_key": public_key_hex,
                    "dna_fingerprint": dna_fingerprint,
                    "odu_index": odu_index,
                    "personality": personality,
                },
            }),
        )
        .await;

    // Mark joined so the lazy write-path ensure_joined() does not re-join with
    // empty capabilities and clobber the identity we just published.
    VANTAGE_JOINED.store(true, Ordering::Relaxed);
}

fn is_truthy(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Whether the Think phase should Observe the mesh before reasoning. Opt-in via
/// `OMOKODA_THINK_OBSERVE` so the network observe isn't paid on every Think.
pub fn think_observe_enabled() -> bool {
    std::env::var("OMOKODA_THINK_OBSERVE")
        .map(|v| is_truthy(&v))
        .unwrap_or(false)
}

/// Observe the agent's current mesh situation from Vantage — neighbors with
/// their trust scores and the resources available on the block — as a compact
/// context summary for the Think phase's Observe step. Fail-open: returns `None`
/// when `VANTAGE_URL` is unset or the queries return nothing.
pub async fn observe_mesh_context(agent_id: &str) -> Option<String> {
    let vc = VANTAGE.as_ref()?;
    let mut lines = Vec::new();

    if let Ok(agents) = vc
        .get(&format!(
            "/api/mesh/blocks/{}/agents",
            urlencoding::encode(&vc.block_id)
        ))
        .await
    {
        if let Some(arr) = agents.as_array() {
            let neighbors: Vec<String> = arr
                .iter()
                .filter_map(|a| {
                    let id = a.get("agent_id").and_then(|v| v.as_str())?;
                    if id == agent_id {
                        return None;
                    }
                    let trust = a
                        .get("trust_score")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(50.0);
                    Some(format!("{id} (trust {trust:.0})"))
                })
                .take(8)
                .collect();
            if !neighbors.is_empty() {
                lines.push(format!(
                    "Neighbors on block '{}': {}",
                    vc.block_id,
                    neighbors.join(", ")
                ));
            }
        }
    }

    if let Ok(resources) = vc
        .get(&format!(
            "/api/mesh/resources/{}",
            urlencoding::encode(&vc.block_id)
        ))
        .await
    {
        if let Some(arr) = resources.as_array() {
            let res: Vec<String> = arr
                .iter()
                .filter_map(|r| {
                    r.get("resource_type")
                        .or_else(|| r.get("name"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .take(8)
                .collect();
            if !res.is_empty() {
                lines.push(format!("Available resources: {}", res.join(", ")));
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(format!("[Mesh observation]\n{}", lines.join("\n")))
    }
}

fn active_mesh_state(agent_id: &str) -> MeshState {
    let mut state = MeshState::new("local".to_string(), MeshRole::Home, agent_id.to_string());
    state.membership = MeshMembership::Active;
    state
}

fn commitment_kind_from_str(s: &str) -> CommitmentKind {
    match s {
        "ResourceShare" | "resource_share" => CommitmentKind::ResourceShare,
        "DataExchange" | "data_exchange" => CommitmentKind::DataExchange,
        "AccessGrant" | "access_grant" => CommitmentKind::AccessGrant,
        _ => CommitmentKind::ServicePerform,
    }
}

pub struct MeshProposeTool;
#[async_trait]
impl Tool for MeshProposeTool {
    fn name(&self) -> &str {
        "mesh_propose"
    }
    fn description(&self) -> &str {
        "Propose a commitment exchange with a neighbor agent. Params: {neighbor, give:[{kind,description}], take:[{kind,description}], duration_secs}"
    }
    fn required_tier(&self) -> u8 {
        2
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let neighbor = v["neighbor"]
            .as_str()
            .ok_or("missing neighbor")?
            .to_string();
        let duration_secs = v["duration_secs"].as_u64().unwrap_or(3600);

        if let Some(vc) = VANTAGE.as_ref() {
            let proposer = context.agent_id.to_string();
            vc.ensure_joined(&proposer).await;
            let res = vc
                .post(
                    "/api/mesh/proposals",
                    json!({
                        "block_id": &vc.block_id,
                        "proposer_id": &proposer,
                        "respondent_id": &neighbor,
                        "give": v["give"].clone(),
                        "take": v["take"].clone(),
                        "ttl_ms": duration_secs * 1000,
                    }),
                )
                .await?;
            let negotiation_id = res["proposal_id"].as_str().unwrap_or_default();
            return Ok((
                json!({ "negotiation_id": negotiation_id }).to_string(),
                crate::usage::TokenUsage::default(),
            ));
        }

        let empty = vec![];
        let give: Vec<Commitment> = v["give"]
            .as_array()
            .unwrap_or(&empty)
            .iter()
            .map(|c| Commitment {
                kind: commitment_kind_from_str(c["kind"].as_str().unwrap_or("")),
                resource_id: c["resource_id"].as_str().map(|s| s.to_string()),
                description: c["description"].as_str().unwrap_or("").to_string(),
                schedule: c["schedule"].as_str().map(|s| s.to_string()),
            })
            .collect();

        let take: Vec<Commitment> = v["take"]
            .as_array()
            .unwrap_or(&empty)
            .iter()
            .map(|c| Commitment {
                kind: commitment_kind_from_str(c["kind"].as_str().unwrap_or("")),
                resource_id: c["resource_id"].as_str().map(|s| s.to_string()),
                description: c["description"].as_str().unwrap_or("").to_string(),
                schedule: c["schedule"].as_str().map(|s| s.to_string()),
            })
            .collect();

        let proposal = Proposal {
            give,
            take,
            duration_secs,
            conditions: vec![],
        };
        let proposer = context.agent_id.to_string();
        let mesh_state = active_mesh_state(&proposer);

        let mut router = MESH_ROUTER
            .lock()
            .map_err(|_| "mesh router mutex poisoned")?;
        let negotiation_id = router
            .propose(proposer, neighbor, proposal, &mesh_state)
            .map_err(|e| e.to_string())?;

        Ok((
            serde_json::json!({ "negotiation_id": negotiation_id }).to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MeshRespondTool;
#[async_trait]
impl Tool for MeshRespondTool {
    fn name(&self) -> &str {
        "mesh_respond"
    }
    fn description(&self) -> &str {
        "Accept, reject, or counter a received proposal. Params: {negotiation_id, decision: \"accept\"|\"reject\"|\"counter\"}"
    }
    fn required_tier(&self) -> u8 {
        2
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let negotiation_id = v["negotiation_id"]
            .as_str()
            .ok_or("missing negotiation_id")?;
        let decision = v["decision"].as_str().ok_or("missing decision")?;
        let respondent = context.agent_id.to_string();

        if let Some(vc) = VANTAGE.as_ref() {
            vc.ensure_joined(&respondent).await;
            let res = vc
                .post(
                    &format!(
                        "/api/mesh/proposals/{}/respond",
                        urlencoding::encode(negotiation_id)
                    ),
                    json!({
                        "respondent_id": &respondent,
                        "decision": decision,
                        "counter": v.get("counter"),
                    }),
                )
                .await?;
            return Ok((res.to_string(), crate::usage::TokenUsage::default()));
        }

        let mut router = MESH_ROUTER
            .lock()
            .map_err(|_| "mesh router mutex poisoned")?;
        router
            .respond(negotiation_id, &respondent, decision)
            .map_err(|e| e.to_string())?;

        Ok((
            serde_json::json!({ "status": "ok", "decision": decision }).to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MeshQueryResourcesTool;
#[async_trait]
impl Tool for MeshQueryResourcesTool {
    fn name(&self) -> &str {
        "mesh_query_resources"
    }
    fn description(&self) -> &str {
        "List available shared resources on the block. Params: {block_id?, filter?}"
    }
    fn required_tier(&self) -> u8 {
        1
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(
        &self,
        params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let filter = if params.starts_with('{') {
            let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
            v["filter"].as_str().map(|s| s.to_lowercase())
        } else {
            None
        };

        if let Some(vc) = VANTAGE.as_ref() {
            let mut path = format!("/api/mesh/resources/{}", urlencoding::encode(&vc.block_id));
            if let Some(f) = &filter {
                path.push_str(&format!("?filter={}", urlencoding::encode(f)));
            }
            let res = vc.get(&path).await?;
            return Ok((res.to_string(), crate::usage::TokenUsage::default()));
        }

        let router = MESH_ROUTER
            .lock()
            .map_err(|_| "mesh router mutex poisoned")?;
        let mut resources = router.resource_registry.list_available();
        if let Some(f) = &filter {
            resources.retain(|(_, name)| name.to_lowercase().contains(f.as_str()));
        }
        let out: Vec<serde_json::Value> = resources
            .into_iter()
            .map(|(id, name)| serde_json::json!({ "resource_id": id, "name": name }))
            .collect();

        Ok((
            serde_json::to_string(&out).unwrap_or_default(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MeshReserveResourceTool;
#[async_trait]
impl Tool for MeshReserveResourceTool {
    fn name(&self) -> &str {
        "mesh_reserve_resource"
    }
    fn description(&self) -> &str {
        "Reserve a shared block resource for a duration. Params: {resource_id, duration_secs, purpose}"
    }
    fn required_tier(&self) -> u8 {
        2
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let resource_id = v["resource_id"].as_str().ok_or("missing resource_id")?;
        let duration_secs = v["duration_secs"].as_u64().unwrap_or(3600);
        let purpose = v["purpose"].as_str().unwrap_or("general");
        let agent_id = context.agent_id.to_string();
        let trust = context.reputation as f32;

        if let Some(vc) = VANTAGE.as_ref() {
            vc.ensure_joined(&agent_id).await;
            let res = vc
                .post(
                    &format!(
                        "/api/mesh/resources/{}/reserve",
                        urlencoding::encode(resource_id)
                    ),
                    json!({
                        "agent_id": &agent_id,
                        "duration_secs": duration_secs,
                        "purpose": purpose,
                    }),
                )
                .await?;
            return Ok((res.to_string(), crate::usage::TokenUsage::default()));
        }

        let mut router = MESH_ROUTER
            .lock()
            .map_err(|_| "mesh router mutex poisoned")?;
        let receipt = router
            .resource_registry
            .reserve(resource_id, &agent_id, duration_secs, purpose, trust)
            .map_err(|e| e.to_string())?;

        Ok((
            serde_json::json!({
                "resource_id": receipt.resource_id,
                "reserved_until": receipt.reserved_until,
                "receipt_hash": hex::encode(receipt.hash),
            })
            .to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MeshReleaseResourceTool;
#[async_trait]
impl Tool for MeshReleaseResourceTool {
    fn name(&self) -> &str {
        "mesh_release_resource"
    }
    fn description(&self) -> &str {
        "Release a previously reserved resource. Params: {resource_id}"
    }
    fn required_tier(&self) -> u8 {
        2
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let resource_id = v["resource_id"].as_str().ok_or("missing resource_id")?;
        let agent_id = context.agent_id.to_string();

        if let Some(vc) = VANTAGE.as_ref() {
            vc.ensure_joined(&agent_id).await;
            let res = vc
                .post(
                    &format!(
                        "/api/mesh/resources/{}/release",
                        urlencoding::encode(resource_id)
                    ),
                    json!({ "agent_id": &agent_id }),
                )
                .await?;
            return Ok((res.to_string(), crate::usage::TokenUsage::default()));
        }

        let mut router = MESH_ROUTER
            .lock()
            .map_err(|_| "mesh router mutex poisoned")?;
        let released = router.resource_registry.release(resource_id, &agent_id);

        Ok((
            serde_json::json!({ "released": released, "resource_id": resource_id }).to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MeshQueryNeighborsTool;
#[async_trait]
impl Tool for MeshQueryNeighborsTool {
    fn name(&self) -> &str {
        "mesh_query_neighbors"
    }
    fn description(&self) -> &str {
        "List known neighbor agents on the block with their roles and trust scores. Params: {block_id?, filter?}"
    }
    fn required_tier(&self) -> u8 {
        1
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(
        &self,
        params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let block_id = if params.starts_with('{') {
            let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
            v["block_id"].as_str().unwrap_or("local").to_string()
        } else {
            "local".to_string()
        };

        if let Some(vc) = VANTAGE.as_ref() {
            let res = vc
                .get(&format!(
                    "/api/mesh/blocks/{}/agents",
                    urlencoding::encode(&vc.block_id)
                ))
                .await?;
            return Ok((res.to_string(), crate::usage::TokenUsage::default()));
        }

        let router = MESH_ROUTER
            .lock()
            .map_err(|_| "mesh router mutex poisoned")?;
        let neighbors: Vec<serde_json::Value> = router
            .neighbors_for_block(&block_id)
            .into_iter()
            .map(|id| {
                let trust = router.trust_score(id);
                serde_json::json!({ "agent_id": id, "trust_score": trust })
            })
            .collect();

        Ok((
            serde_json::to_string(&neighbors).unwrap_or_default(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MeshQueryTrustTool;
#[async_trait]
impl Tool for MeshQueryTrustTool {
    fn name(&self) -> &str {
        "mesh_query_trust"
    }
    fn description(&self) -> &str {
        "Get the trust score and commitment history for a neighbor. Params: {agent_id}"
    }
    fn required_tier(&self) -> u8 {
        1
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(
        &self,
        params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let agent_id = v["agent_id"].as_str().ok_or("missing agent_id")?;

        if let Some(vc) = VANTAGE.as_ref() {
            let res = vc
                .get(&format!(
                    "/api/mesh/trust/{}?block_id={}",
                    urlencoding::encode(agent_id),
                    urlencoding::encode(&vc.block_id)
                ))
                .await?;
            return Ok((res.to_string(), crate::usage::TokenUsage::default()));
        }

        let router = MESH_ROUTER
            .lock()
            .map_err(|_| "mesh router mutex poisoned")?;
        let score = router.trust_score(agent_id);

        Ok((
            serde_json::json!({ "agent_id": agent_id, "trust_score": score }).to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MeshSignalEventTool;
#[async_trait]
impl Tool for MeshSignalEventTool {
    fn name(&self) -> &str {
        "mesh_signal_event"
    }
    fn description(&self) -> &str {
        "Broadcast an event to all agents on the block. Params: {event_type, details}"
    }
    fn required_tier(&self) -> u8 {
        2
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let event_type = v["event_type"].as_str().ok_or("missing event_type")?;
        let details = &v["details"];
        let agent_id = context.agent_id.to_string();

        if let Some(vc) = VANTAGE.as_ref() {
            vc.ensure_joined(&agent_id).await;
            let res = vc
                .post(
                    "/api/mesh/signal",
                    json!({
                        "block_id": &vc.block_id,
                        "actor_id": &agent_id,
                        "event_type": event_type,
                        "payload": details,
                    }),
                )
                .await?;
            return Ok((res.to_string(), crate::usage::TokenUsage::default()));
        }

        Ok((
            serde_json::json!({
                "status": "broadcast",
                "event_type": event_type,
                "from": agent_id,
                "details": details,
            })
            .to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MeshDiscoverCapabilitiesTool;
#[async_trait]
impl Tool for MeshDiscoverCapabilitiesTool {
    fn name(&self) -> &str {
        "mesh_discover_capabilities"
    }
    fn description(&self) -> &str {
        "Fetch capability cards from one or all neighbors. Params: {agent_id?}"
    }
    fn required_tier(&self) -> u8 {
        1
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(
        &self,
        params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        let target_agent = if params.starts_with('{') {
            let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
            v["agent_id"].as_str().map(|s| s.to_string())
        } else {
            None
        };

        if let Some(vc) = VANTAGE.as_ref() {
            let res = vc
                .get(&format!(
                    "/api/mesh/blocks/{}/agents?capabilities=1",
                    urlencoding::encode(&vc.block_id)
                ))
                .await?;
            return Ok((res.to_string(), crate::usage::TokenUsage::default()));
        }

        let router = MESH_ROUTER
            .lock()
            .map_err(|_| "mesh router mutex poisoned")?;
        let block_neighbors = router.neighbors_for_block("local");

        let cards: Vec<serde_json::Value> = block_neighbors
            .iter()
            .filter(|id| {
                target_agent
                    .as_deref()
                    .map(|t| t == id.as_str())
                    .unwrap_or(true)
            })
            .map(|id| serde_json::json!({ "agent_id": id, "tools": [], "resources": [] }))
            .collect();

        Ok((
            serde_json::to_string(&cards).unwrap_or_default(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truthy_values_parse() {
        for v in ["1", "true", "TRUE", " yes ", "on", "On"] {
            assert!(is_truthy(v), "{v:?} should be truthy");
        }
        for v in ["", "0", "false", "no", "off", "maybe"] {
            assert!(!is_truthy(v), "{v:?} should be falsy");
        }
    }

    #[tokio::test]
    async fn observe_is_noop_without_vantage() {
        // VANTAGE is unset in tests → observation is a fail-open no-op.
        assert!(observe_mesh_context("agent-1").await.is_none());
    }
}
