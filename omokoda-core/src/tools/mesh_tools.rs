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

    async fn send(&self, req: reqwest::RequestBuilder) -> Result<serde_json::Value, String> {
        let req = if self.api_key.is_empty() {
            req
        } else {
            req.header("X-Agent-Key", &self.api_key)
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
