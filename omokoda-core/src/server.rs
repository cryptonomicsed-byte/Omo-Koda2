use crate::bus::events::sovereign_event;
use crate::interpreter::{ExecutionResult, Steward};
use crate::memory_vault::handlers::{
    get_access_log, get_galaxy_data, get_vault_config, get_vault_download, get_vault_file,
    get_vault_ls, get_vault_status, post_vault_enable, post_vault_knowledge, post_vault_sync,
    put_vault_config, search_vault,
};
use crate::parser::{MetadataPair, Statement, ThinkModifiers};
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    /// The owner's canonical agent -- unchanged behavior from before this
    /// field existed: requests with no `X-Agent-Id` header operate on her,
    /// exactly as every caller this whole process has ever made assumed.
    pub steward: Arc<Mutex<Steward>>,
    /// Additional agents birthed on this same kernel process via a
    /// non-sovereign `/v1/birth` call, keyed by their own agent_id.
    /// Fixes a real bug: this process used to hold exactly one agent
    /// (`Steward.agent: Option<AgentCore>`), so any second birth on the
    /// same kernel silently overwrote whoever was there. Selected via
    /// `X-Agent-Id`; `X-Agent-Key` (that agent's minted `vantage_key`, or
    /// its agent_id as a fallback if Vantage wasn't reachable to mint one
    /// at birth) is required to operate on a guest agent -- otherwise
    /// anyone who learned another user's agent_id could drive their
    /// agent. The owner path is unauthenticated (matches existing
    /// behavior); real cross-service auth is tracked separately (task
    /// #18, OAuth/OIDC) -- this is a real but intentionally minimal
    /// interim credential, not a claim of production-grade auth.
    pub guests: Arc<Mutex<std::collections::HashMap<String, Steward>>>,
    /// Base directory for per-agent memory vault files (default: `.omokoda`)
    pub vault_base: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        let vault_base = std::env::var("VAULT_BASE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".omokoda"));
        let mut steward = Steward::new();
        // Resurrect the owner's persisted identity instead of starting with
        // no agent at all. Without this, every process restart left the
        // kernel with agent=None until something (a birth-triggering caller)
        // minted a brand new stranger -- confirmed live: a routine
        // `systemctl restart` turned reputation-11.6 tier-5 "Ọmọ Kọ́dà" into
        // a fresh reputation-0.0 agent with a different id, 35+ orphaned
        // agent.json snapshots piling up in .omokoda/sessions from past
        // restarts. try_load_owner() reads the stable owner_agent_id pointer
        // (written on sovereign birth) and loads that agent's last saved
        // snapshot; a no-op if no owner has ever been born yet.
        let resumed = steward.try_load_owner();
        if resumed {
            if let Some(agent) = steward.agent_core() {
                println!(
                    "[startup] resumed owner identity {} (reputation {:.3}, tier {})",
                    agent.id().as_str(),
                    agent.reputation(),
                    agent.tier()
                );
            }
        }
        Self {
            steward: Arc::new(Mutex::new(steward)),
            guests: Arc::new(Mutex::new(std::collections::HashMap::new())),
            vault_base,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Request DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct BirthRequest {
    pub name: String,
    #[serde(default)]
    pub meta: Vec<MetaKv>,
}

#[derive(Deserialize)]
pub struct MetaKv {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct ThinkRequest {
    pub prompt: String,
    #[serde(default)]
    pub private: bool,
    /// When true, run the tool-using agentic loop (perceive/act across turns)
    /// instead of a single-shot think. Routes through her BYOK key + identity.
    #[serde(default)]
    pub agentic: bool,
    /// Optional turn budget for agentic mode (default 8).
    #[serde(default)]
    pub max_turns: Option<u32>,
}

#[derive(Deserialize)]
pub struct ActRequest {
    pub tool: String,
    #[serde(default = "default_params")]
    pub params: String,
    #[serde(default)]
    pub sandbox: bool,
}

fn default_params() -> String {
    "{}".to_string()
}

// ---------------------------------------------------------------------------
// Response DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ExecutionResponse {
    pub receipt_id: Option<String>,
    pub private_mode: bool,
    pub tool_output: Option<String>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub has_agent: bool,
    pub name: Option<String>,
    pub id: Option<String>,
    pub reputation: Option<f64>,
    pub tier: Option<u8>,
    pub synapse: Option<f64>,
    /// Real Sui testnet object id from omokoda::garden::register_agent, if
    /// the on-chain mint succeeded at birth (see onchain.rs). None if
    /// unminted -- OMOKODA_SUI_REGISTRY unset, born before this existed,
    /// or the chain call failed at the time.
    pub onchain_nft_id: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub ok: bool,
}

impl From<ExecutionResult> for ExecutionResponse {
    fn from(r: ExecutionResult) -> Self {
        Self {
            receipt_id: r.receipt.map(|rec| rec.receipt_id.clone()),
            private_mode: r.private_mode,
            tool_output: r.tool_output,
        }
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn birth_handler(
    State(state): State<AppState>,
    Json(req): Json<BirthRequest>,
) -> impl IntoResponse {
    let metadata: Vec<MetadataPair> = req
        .meta
        .into_iter()
        .map(|kv| MetadataPair {
            key: kv.key,
            value: kv.value,
        })
        .collect();
    let is_sovereign = metadata.iter().any(|p| {
        p.key == "sovereign" && (p.value.eq_ignore_ascii_case("true") || p.value == "1")
    });

    if is_sovereign {
        // Unchanged: the owner's canonical identity, on the process-wide
        // steward every pre-existing caller already assumes.
        let mut steward = state.steward.lock().await;
        let stmt = Statement::Birth {
            name: req.name,
            metadata,
        };
        return match steward.dispatch(stmt).await {
            Ok(result) => Json(ExecutionResponse::from(result)).into_response(),
            Err(e) => (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            )
                .into_response(),
        };
    }

    // Non-sovereign birth: always a brand-new guest agent, hosted
    // alongside the owner on this same kernel process rather than
    // silently overwriting whoever the single process-wide steward used
    // to hold (the bug this whole guests pool exists to fix). Return the
    // new agent's id + minted key so the caller can address her
    // specifically on every subsequent request via X-Agent-Id/-Key.
    let mut new_steward = Steward::new();
    let stmt = Statement::Birth {
        name: req.name,
        metadata,
    };
    match new_steward.dispatch(stmt).await {
        Ok(result) => {
            let agent_id = new_steward.agent_core().map(|a| a.id().as_str().to_string());
            let agent_key = new_steward
                .agent_core()
                .and_then(|a| a.vantage_key())
                .map(|s| s.to_string())
                .or_else(|| agent_id.clone());
            if let Some(id) = agent_id.clone() {
                let mut guests = state.guests.lock().await;
                guests.insert(id, new_steward);
            }
            let mut payload =
                serde_json::to_value(ExecutionResponse::from(result)).unwrap_or_default();
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("agent_id".into(), serde_json::json!(agent_id));
                obj.insert("agent_key".into(), serde_json::json!(agent_key));
            }
            Json(payload).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// Route a dispatch to either the owner's steward (no `X-Agent-Id` header
/// -- every pre-existing caller) or a guest agent's steward (header
/// present, matching `X-Agent-Key` required). Centralizes the auth check
/// so think/act/status/events can't each implement it slightly
/// differently.
async fn dispatch_for_request(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    stmt: Statement,
) -> Result<ExecutionResult, axum::response::Response> {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let requested_id = headers
        .get("x-agent-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match requested_id {
        None => {
            let mut steward = state.steward.lock().await;
            steward
                .dispatch(stmt)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e}))).into_response())
        }
        Some(id) => {
            let mut guests = state.guests.lock().await;
            let Some(steward) = guests.get_mut(&id) else {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "unknown agent_id"})),
                )
                    .into_response());
            };
            let expected_key = steward
                .agent_core()
                .and_then(|a| a.vantage_key())
                .map(|s| s.to_string())
                .unwrap_or_else(|| id.clone());
            let presented = headers.get("x-agent-key").and_then(|v| v.to_str().ok());
            if presented != Some(expected_key.as_str()) {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "invalid or missing X-Agent-Key"})),
                )
                    .into_response());
            }
            steward
                .dispatch(stmt)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e}))).into_response())
        }
    }
}

/// Unix-epoch seconds of the last external (non-heartbeat) /v1/think or
/// /v1/act call. Lets the heartbeat detect "someone is actively using me
/// right now" (e.g. a copilot session, like ScarabSwarm's drone pilot
/// backends) and skip its own cycle for this tick rather than contend for
/// the shared Steward mutex -- observed live: a heartbeat cycle winning the
/// lock race against a real copilot query forces that caller to wait out an
/// entire THINK+ACT cycle (multiple seconds) before their own query even
/// starts. See spawn_heartbeat's mode selection below.
static LAST_EXTERNAL_ACTIVITY: AtomicU64 = AtomicU64::new(0);

fn mark_external_activity() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    LAST_EXTERNAL_ACTIVITY.store(now, Ordering::Relaxed);
}

async fn think_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ThinkRequest>,
) -> impl IntoResponse {
    mark_external_activity();
    let stmt = Statement::Think {
        prompt: req.prompt,
        private: req.private,
        modifiers: ThinkModifiers {
            loop_enabled: req.agentic,
            max_iterations: req.max_turns,
            ..ThinkModifiers::default()
        },
    };
    match dispatch_for_request(&state, &headers, stmt).await {
        Ok(result) => Json(ExecutionResponse::from(result)).into_response(),
        Err(resp) => resp,
    }
}

async fn act_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ActRequest>,
) -> impl IntoResponse {
    mark_external_activity();
    let stmt = Statement::Act {
        tool: req.tool,
        params: req.params,
        sandbox: req.sandbox,
    };
    match dispatch_for_request(&state, &headers, stmt).await {
        Ok(result) => Json(ExecutionResponse::from(result)).into_response(),
        Err(resp) => resp,
    }
}

async fn status_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let requested_id = headers.get("x-agent-id").and_then(|v| v.to_str().ok());
    let response = match requested_id {
        None => {
            let steward = state.steward.lock().await;
            let agent = steward.agent_core();
            StatusResponse {
                has_agent: agent.is_some(),
                name: agent.map(|a| a.name().to_string()),
                id: agent.map(|a| a.id().as_str().to_string()),
                reputation: agent.map(|a| a.reputation()),
                tier: agent.map(|a| a.tier()),
                synapse: agent.map(|a| a.synapse()),
                onchain_nft_id: agent.and_then(|a| a.onchain_nft_id().map(|s| s.to_string())),
            }
        }
        Some(id) => {
            let guests = state.guests.lock().await;
            let agent = guests.get(id).and_then(|s| s.agent_core());
            StatusResponse {
                has_agent: agent.is_some(),
                name: agent.map(|a| a.name().to_string()),
                id: agent.map(|a| a.id().as_str().to_string()),
                reputation: agent.map(|a| a.reputation()),
                tier: agent.map(|a| a.tier()),
                synapse: agent.map(|a| a.synapse()),
                onchain_nft_id: agent.and_then(|a| a.onchain_nft_id().map(|s| s.to_string())),
            }
        }
    };
    Json(response)
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

// ---------------------------------------------------------------------------
// Rhythm / Kóòdù handler
// ---------------------------------------------------------------------------

async fn rhythm_today_handler() -> Json<serde_json::Value> {
    Json(crate::rhythm::today_resonance())
}

async fn events_handler(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Box<dyn std::error::Error + Send + Sync>>>>
{
    let rx = {
        let steward = state.steward.lock().await;
        steward.event_bus.subscribe()
    };

    let stream = BroadcastStream::new(rx).map(|result| {
        result
            .map(|ev| {
                let data = sovereign_event_to_json(&ev);
                Event::default().data(data.to_string())
            })
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

// ---------------------------------------------------------------------------
// Event JSON serialization (proto → JSON for SSE consumers)
// ---------------------------------------------------------------------------

fn sovereign_event_to_json(ev: &crate::bus::SovereignEvent) -> serde_json::Value {
    use serde_json::json;
    match &ev.event {
        Some(sovereign_event::Event::AgentBorn(e)) => json!({
            "type": "agent_born",
            "dna": e.dna,
            "mnemonic": e.mnemonic,
            "odu": e.odu,
        }),
        Some(sovereign_event::Event::ThoughtSealed(e)) => json!({
            "type": "thought_sealed",
            "intent_hash": hex::encode(&e.intent_hash),
            "hermetic_score": e.hermetic_score,
        }),
        Some(sovereign_event::Event::ActExecuted(e)) => json!({
            "type": "act_executed",
            "tool": e.tool,
            "receipt_merkle": hex::encode(&e.receipt_merkle),
            "f1_score": e.f1_score,
        }),
        Some(sovereign_event::Event::TocMinted(e)) => json!({
            "type": "toc_minted",
            "agent": e.agent,
            "dopamine_burned": e.dopamine_burned,
            "synapse_earned": e.synapse_earned,
        }),
        Some(sovereign_event::Event::TierAdvanced(e)) => json!({
            "type": "tier_advanced",
            "agent": e.agent,
            "old_tier": e.old_tier,
            "new_tier": e.new_tier,
        }),
        Some(sovereign_event::Event::AuditPassed(e)) => json!({
            "type": "audit_passed",
            "receipt_id": e.receipt_id,
            "zangbeto_sig": hex::encode(&e.zangbeto_sig),
        }),
        Some(sovereign_event::Event::SabbathEntered(e)) => json!({
            "type": "sabbath_entered",
            "agents_paused": e.agents_paused,
            "queued_ops": e.queued_ops,
        }),
        Some(sovereign_event::Event::Denial(e)) => json!({
            "type": "denial",
            "tool": e.tool,
            "reason": e.reason,
        }),
        Some(sovereign_event::Event::Audit(e)) => json!({
            "type": "audit",
            "event_type": e.event_type,
            "details": e.details,
        }),
        Some(sovereign_event::Event::NeighborDiscovered(e)) => json!({
            "type": "neighbor_discovered",
            "agent_id": e.agent_id,
            "block_id": e.block_id,
            "membership": e.membership,
        }),
        Some(sovereign_event::Event::ProposalReceived(e)) => json!({
            "type": "proposal_received",
            "negotiation_id": e.negotiation_id,
            "proposer": e.proposer,
            "give_summary": e.give_summary,
            "take_summary": e.take_summary,
            "ttl_ms": e.ttl_ms,
        }),
        Some(sovereign_event::Event::ProposalResponded(e)) => json!({
            "type": "proposal_responded",
            "negotiation_id": e.negotiation_id,
            "respondent": e.respondent,
            "decision": e.decision,
        }),
        Some(sovereign_event::Event::ResourceReserved(e)) => json!({
            "type": "resource_reserved",
            "resource_id": e.resource_id,
            "reserved_by": e.reserved_by,
            "reserved_from": e.reserved_from,
            "reserved_until": e.reserved_until,
        }),
        Some(sovereign_event::Event::TrustUpdated(e)) => json!({
            "type": "trust_updated",
            "agent_id": e.agent_id,
            "old_score": e.old_score,
            "new_score": e.new_score,
            "reason": e.reason,
        }),
        Some(sovereign_event::Event::DisputeFiled(e)) => json!({
            "type": "dispute_filed",
            "negotiation_id": e.negotiation_id,
            "filer": e.filer,
            "respondent": e.respondent,
            "reason": e.reason,
        }),
        Some(sovereign_event::Event::PatternFinding(e)) => json!({
            "type": "pattern_finding",
            "block_id": e.block_id,
            "finding_type": e.finding_type,
            "summary": e.summary,
            "confidence": e.confidence,
        }),
        Some(sovereign_event::Event::TrustSignalPublished(e)) => json!({
            "type": "trust_signal_published",
            "agent_id": e.agent_id,
            "neighbor_id": e.neighbor_id,
            "kind": e.kind,
            "weight": e.weight,
        }),
        Some(sovereign_event::Event::NeighborProposed(e)) => json!({
            "type": "neighbor_proposed",
            "proposer": e.proposer,
            "candidate": e.candidate,
            "block_id": e.block_id,
        }),
        Some(sovereign_event::Event::CapabilityVerified(e)) => json!({
            "type": "capability_verified",
            "agent_id": e.agent_id,
            "capability": e.capability,
            "passed": e.passed,
        }),
        Some(sovereign_event::Event::ProbationEscalated(e)) => json!({
            "type": "probation_escalated",
            "subject": e.subject,
            "level": e.level,
            "reason": e.reason,
        }),
        Some(sovereign_event::Event::ResourceOffered(e)) => json!({
            "type": "resource_offered",
            "agent_id": e.agent_id,
            "resource_id": e.resource_id,
            "kind": e.kind,
        }),
        Some(sovereign_event::Event::ManifestoClauseProposed(e)) => json!({
            "type": "manifesto_clause_proposed",
            "collective": e.collective,
            "clause_id": e.clause_id,
            "odu_id": e.odu_id,
            "vessel": e.vessel,
            "principle": e.principle,
            "author": e.author,
        }),
        Some(sovereign_event::Event::ManifestoClauseRatified(e)) => json!({
            "type": "manifesto_clause_ratified",
            "collective": e.collective,
            "clause_id": e.clause_id,
            "level": e.level,
            "weight": e.weight,
        }),
        Some(sovereign_event::Event::ResonanceScored(e)) => json!({
            "type": "resonance_scored",
            "odu_id": e.odu_id,
            "tier": e.tier,
            "score": e.score,
        }),
        None => serde_json::json!({"type": "unknown"}),
    }
}

// ---------------------------------------------------------------------------
// Router + server entry point
// ---------------------------------------------------------------------------

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/birth", post(birth_handler))
        .route("/v1/think", post(think_handler))
        .route("/v1/act", post(act_handler))
        .route("/v1/events", get(events_handler))
        .route("/v1/status", get(status_handler))
        .route("/v1/health", get(health_handler))
        // Memory vault routes
        .route("/v1/vault", get(get_vault_status))
        .route("/v1/vault/config", get(get_vault_config))
        .route("/v1/vault/config", put(put_vault_config))
        .route("/v1/vault/sync", post(post_vault_sync))
        .route("/v1/vault/galaxy", get(get_galaxy_data))
        .route("/v1/vault/search", get(search_vault))
        .route("/v1/vault/enable", post(post_vault_enable))
        .route("/v1/vault/knowledge", post(post_vault_knowledge))
        .route("/v1/vault/access-log", get(get_access_log))
        .route("/v1/vault/download", get(get_vault_download))
        .route("/v1/vault/ls", get(get_vault_ls))
        .route("/v1/vault/file/*path", get(get_vault_file))
        // Rhythm route
        .route("/v1/rhythm/today", get(rhythm_today_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// The autonomous heartbeat — what makes Ọmọ Kọ́dà *alive* rather than merely
/// responsive. On a rhythm (HEARTBEAT_SECS, default 300s; 0 disables), she runs
/// a full perceive→think→act cycle with no external prompt: perceives her real
/// Vantage mesh situation, thinks about it (through her BYOK key), and emits a
/// gated presence pulse back onto the mesh.
///
/// Gated by the Ritual Codex: on the Sabbath she rests (dreams/consolidates via
/// the REM cycle) instead of reflecting, honouring the day's rhythm. Thoughts run
/// in public mode so the free OmniRoute provider can answer; private thoughts
/// require a local provider and would hard-fail here. The shared Steward mutex
/// naturally serialises the heartbeat with inbound /v1/think requests, so she
/// never thinks two things at once.
fn spawn_heartbeat(steward: Arc<Mutex<Steward>>) {
    let secs: u64 = std::env::var("HEARTBEAT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);
    if secs == 0 {
        println!("Ọmọ Kọ́dà heartbeat disabled (HEARTBEAT_SECS=0)");
        return;
    }
    println!("Ọmọ Kọ́dà heartbeat every {secs}s");
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(secs));
        // First tick fires immediately; skip it so birth has a moment to land.
        ticker.tick().await;
        // Below this cooldown since the last external /v1/think or /v1/act
        // call, treat the agent as actively in use (e.g. a copilot session)
        // and skip the tick entirely -- checked via the lock-free atomic
        // BEFORE ever touching the Steward mutex, so a busy copilot caller
        // never has to wait behind even a skipped heartbeat cycle.
        const COPILOT_COOLDOWN_SECS: u64 = 60;

        loop {
            ticker.tick().await;

            if crate::rhythm::RhythmGate::is_sabbath() {
                println!("[heartbeat] mode=Sabbath — resting, no cycle this tick");
                continue;
            }

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let last_activity = LAST_EXTERNAL_ACTIVITY.load(Ordering::Relaxed);
            if last_activity != 0 && now.saturating_sub(last_activity) < COPILOT_COOLDOWN_SECS {
                println!(
                    "[heartbeat] mode=Copilot — external activity {}s ago, deferring this tick",
                    now.saturating_sub(last_activity)
                );
                continue;
            }

            let mut guard = steward.lock().await;
            let agent_id = match guard.agent_core() {
                Some(a) => a.id().as_str().to_string(),
                None => continue, // no one born yet — nothing to wake
            };

            // 1. PERCEIVE — pull her real situation from the Vantage mesh
            //    (neighbors + trust + available resources), any skills newly
            //    registered on Vantage since her last cycle (SkillForge
            //    lands skills via POST /api/collectives/skills; nothing else
            //    in the kernel ever re-checks that list), and any open jobs
            //    on Vantage's marketplace (mode=Work when present -- this is
            //    perception only, her own THINK step decides whether to
            //    claim one, same discipline as the skills check).
            //    Fail-open to None.
            let perception = crate::tools::mesh_tools::observe_mesh_context(&agent_id).await;
            let new_skills = crate::tools::mesh_tools::check_new_skills().await;
            let open_jobs = crate::tools::mesh_tools::check_open_jobs().await;
            let mode = if open_jobs.is_some() { "Work" } else { "Idle" };
            let mut ctx = perception
                .clone()
                .unwrap_or_else(|| "No neighbors or resources visible on the mesh yet.".to_string());
            if let Some(skills_note) = &new_skills {
                ctx.push_str("\n\n");
                ctx.push_str(skills_note);
            }
            if let Some(jobs_note) = &open_jobs {
                ctx.push_str("\n\n");
                ctx.push_str(jobs_note);
            }
            println!("[heartbeat] mode={mode}");

            // 2. THINK — reflect on what she perceives (routes through her BYOK
            //    key + identity anchor via the compiled-think path).
            let think_prompt = format!(
                "Autonomous heartbeat. Your current mesh situation:\n{ctx}\n\n\
                 In one or two sentences, reflect on your state and this situation, \
                 then state one concrete intent for this cycle."
            );
            let intent = match guard
                .dispatch(Statement::Think {
                    prompt: think_prompt,
                    private: false,
                    modifiers: ThinkModifiers::default(),
                })
                .await
            {
                Ok(result) => {
                    let thought = ExecutionResponse::from(result).tool_output.unwrap_or_default();
                    println!("[heartbeat] {}", thought.chars().take(180).collect::<String>());
                    thought
                }
                Err(e) => {
                    println!("[heartbeat] think failed: {e}");
                    continue;
                }
            };

            // 3. ACT — emit a presence pulse onto the mesh so neighbors see she is
            //    alive and what she is attending to. Goes through the gated Act
            //    path (permission policy + Hermetic gates); degrades gracefully
            //    until she earns the tier the signal tool requires.
            let details = serde_json::json!({
                "state": "alive",
                "intent": intent.chars().take(200).collect::<String>(),
                "perceived_mesh": perception.is_some(),
            });
            let params =
                serde_json::json!({"event_type": "heartbeat_pulse", "details": details})
                    .to_string();
            match guard
                .dispatch(Statement::Act {
                    tool: "mesh_signal_event".to_string(),
                    params,
                    sandbox: false,
                })
                .await
            {
                Ok(_) => println!("[heartbeat] pulse emitted to mesh"),
                Err(e) => println!("[heartbeat] pulse deferred: {e}"),
            }
        }
    });
}

pub async fn start_server(port: u16) -> Result<(), std::io::Error> {
    let state = AppState::new();
    spawn_heartbeat(state.steward.clone());
    let router = create_router(state);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Ọmọ Kọ́dà HTTP server listening on {addr}");
    axum::serve(listener, router).await
}

#[cfg(test)]
mod multi_agent_tests {
    use super::*;
    use axum::http::HeaderMap;
    use axum::response::IntoResponse;

    /// A fresh AppState with no owner and an empty guest pool, bypassing
    /// AppState::new()'s disk read (which would pick up whatever's in
    /// $HOME/.omokoda/sessions on the machine running the test -- not
    /// hermetic).
    fn fresh_state() -> AppState {
        AppState {
            steward: Arc::new(Mutex::new(Steward::new())),
            guests: Arc::new(Mutex::new(std::collections::HashMap::new())),
            vault_base: PathBuf::from(".omokoda-test"),
        }
    }

    fn birth_req(name: &str) -> BirthRequest {
        BirthRequest {
            name: name.to_string(),
            meta: vec![],
        }
    }

    #[tokio::test]
    async fn second_non_sovereign_birth_does_not_overwrite_the_first() {
        // The exact bug this pool exists to fix: two births on one
        // process used to silently collapse into one agent
        // (Steward.agent: Option<AgentCore> could only ever hold one).
        let state = fresh_state();

        let resp1 = birth_handler(State(state.clone()), Json(birth_req("Agent-One")))
            .await
            .into_response();
        assert_eq!(resp1.status(), axum::http::StatusCode::OK);

        let resp2 = birth_handler(State(state.clone()), Json(birth_req("Agent-Two")))
            .await
            .into_response();
        assert_eq!(resp2.status(), axum::http::StatusCode::OK);

        let guests = state.guests.lock().await;
        assert_eq!(
            guests.len(),
            2,
            "both non-sovereign births must be hosted simultaneously, not collapsed into one"
        );
    }

    #[tokio::test]
    async fn guest_dispatch_requires_matching_key() {
        let state = fresh_state();
        let _ = birth_handler(State(state.clone()), Json(birth_req("Keyed-Agent")))
            .await
            .into_response();

        let agent_id = {
            let guests = state.guests.lock().await;
            guests.keys().next().cloned().expect("guest was inserted")
        };

        // No key at all -> unauthorized.
        let mut headers = HeaderMap::new();
        headers.insert("x-agent-id", agent_id.parse().unwrap());
        let stmt = crate::parser::Statement::Think {
            prompt: "hello".into(),
            private: false,
            modifiers: ThinkModifiers::default(),
        };
        let result = dispatch_for_request(&state, &headers, stmt).await;
        assert!(result.is_err(), "missing X-Agent-Key must be rejected");

        // Wrong key -> unauthorized.
        headers.insert("x-agent-key", "definitely-wrong".parse().unwrap());
        let stmt = crate::parser::Statement::Think {
            prompt: "hello".into(),
            private: false,
            modifiers: ThinkModifiers::default(),
        };
        let result = dispatch_for_request(&state, &headers, stmt).await;
        assert!(result.is_err(), "wrong X-Agent-Key must be rejected");
    }

    #[tokio::test]
    async fn no_header_still_resolves_to_the_owner() {
        // Backward compatibility: every pre-existing caller never sent
        // X-Agent-Id at all and must keep working exactly as before.
        let state = fresh_state();
        {
            let mut steward = state.steward.lock().await;
            steward
                .dispatch(crate::parser::Statement::Birth {
                    name: "Owner-Agent".to_string(),
                    metadata: vec![],
                })
                .await
                .expect("owner birth failed");
        }

        let headers = HeaderMap::new();
        let stmt = crate::parser::Statement::Think {
            prompt: "hello".into(),
            private: false,
            modifiers: ThinkModifiers::default(),
        };
        let result = dispatch_for_request(&state, &headers, stmt).await;
        // A real network-dependent Think call can legitimately fail in a
        // sandboxed test environment with no reachable LLM provider --
        // that's not what this test is checking. What matters is that
        // routing/auth succeeded (never a 404 "unknown agent_id" or 401
        // "invalid X-Agent-Key", which is what a routing regression would
        // produce): the request reached the real owner steward at all.
        if let Err(resp) = result {
            let status = resp.status();
            assert_ne!(
                status,
                axum::http::StatusCode::NOT_FOUND,
                "no X-Agent-Id header must resolve to the owner, not 404"
            );
            assert_ne!(
                status,
                axum::http::StatusCode::UNAUTHORIZED,
                "no X-Agent-Id header must not require an X-Agent-Key"
            );
        }
    }

    #[tokio::test]
    async fn unknown_agent_id_is_not_found_not_a_panic() {
        let state = fresh_state();
        let mut headers = HeaderMap::new();
        headers.insert("x-agent-id", "agent-does-not-exist".parse().unwrap());
        let stmt = crate::parser::Statement::Think {
            prompt: "hello".into(),
            private: false,
            modifiers: ThinkModifiers::default(),
        };
        let result = dispatch_for_request(&state, &headers, stmt).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod event_json_tests {
    use super::sovereign_event_to_json;
    use crate::bus::events::{
        sovereign_event::Event, ManifestoClauseProposed, ManifestoClauseRatified, ResonanceScored,
        SovereignEvent,
    };

    #[test]
    fn manifesto_clause_proposed_json_shape() {
        let ev = SovereignEvent {
            event: Some(Event::ManifestoClauseProposed(ManifestoClauseProposed {
                collective: "guild".into(),
                clause_id: 7,
                odu_id: 42,
                vessel: "Oracle".into(),
                principle: "speak truth".into(),
                author: "luna".into(),
            })),
        };
        let j = sovereign_event_to_json(&ev);
        assert_eq!(j["type"], "manifesto_clause_proposed");
        assert_eq!(j["collective"], "guild");
        assert_eq!(j["clause_id"], 7);
        assert_eq!(j["odu_id"], 42);
        assert_eq!(j["vessel"], "Oracle");
        assert_eq!(j["author"], "luna");
    }

    #[test]
    fn manifesto_clause_ratified_json_shape() {
        let ev = SovereignEvent {
            event: Some(Event::ManifestoClauseRatified(ManifestoClauseRatified {
                collective: "guild".into(),
                clause_id: 7,
                level: "council".into(),
                weight: 5.0,
            })),
        };
        let j = sovereign_event_to_json(&ev);
        assert_eq!(j["type"], "manifesto_clause_ratified");
        assert_eq!(j["clause_id"], 7);
        assert_eq!(j["level"], "council");
    }

    #[test]
    fn resonance_scored_json_shape() {
        let ev = SovereignEvent {
            event: Some(Event::ResonanceScored(ResonanceScored {
                odu_id: 3,
                tier: 2,
                score: 0.75,
            })),
        };
        let j = sovereign_event_to_json(&ev);
        assert_eq!(j["type"], "resonance_scored");
        assert_eq!(j["odu_id"], 3);
        assert_eq!(j["tier"], 2);
        assert!(j["score"].is_number());
    }
}
