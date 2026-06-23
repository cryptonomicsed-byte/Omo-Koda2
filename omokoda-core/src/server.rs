use crate::bus::events::sovereign_event;
use crate::interpreter::{ExecutionResult, Steward};
use crate::memory_vault::handlers::{
    get_access_log, get_galaxy_data, get_vault_config, get_vault_download, get_vault_file,
    get_vault_status, post_vault_enable, post_vault_knowledge, post_vault_sync, put_vault_config,
    search_vault,
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
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub steward: Arc<Mutex<Steward>>,
    /// Base directory for per-agent memory vault files (default: `.omokoda`)
    pub vault_base: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        let vault_base = std::env::var("VAULT_BASE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".omokoda"));
        Self {
            steward: Arc::new(Mutex::new(Steward::new())),
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
    let mut steward = state.steward.lock().await;
    let stmt = Statement::Birth {
        name: req.name,
        metadata: req
            .meta
            .into_iter()
            .map(|kv| MetadataPair {
                key: kv.key,
                value: kv.value,
            })
            .collect(),
    };
    match steward.dispatch(stmt).await {
        Ok(result) => Json(ExecutionResponse::from(result)).into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

async fn think_handler(
    State(state): State<AppState>,
    Json(req): Json<ThinkRequest>,
) -> impl IntoResponse {
    let mut steward = state.steward.lock().await;
    let stmt = Statement::Think {
        prompt: req.prompt,
        private: req.private,
        modifiers: ThinkModifiers::default(),
    };
    match steward.dispatch(stmt).await {
        Ok(result) => Json(ExecutionResponse::from(result)).into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

async fn act_handler(
    State(state): State<AppState>,
    Json(req): Json<ActRequest>,
) -> impl IntoResponse {
    let mut steward = state.steward.lock().await;
    let stmt = Statement::Act {
        tool: req.tool,
        params: req.params,
        sandbox: req.sandbox,
    };
    match steward.dispatch(stmt).await {
        Ok(result) => Json(ExecutionResponse::from(result)).into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    let steward = state.steward.lock().await;
    let agent = steward.agent_core();
    Json(StatusResponse {
        has_agent: agent.is_some(),
        name: agent.map(|a| a.name().to_string()),
        id: agent.map(|a| a.id().as_str().to_string()),
        reputation: agent.map(|a| a.reputation()),
        tier: agent.map(|a| a.tier()),
        synapse: agent.map(|a| a.synapse()),
    })
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
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
        // Memory vault endpoints
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
        .route("/v1/vault/file/*path", get(get_vault_file))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn start_server(port: u16) -> Result<(), std::io::Error> {
    let state = AppState::new();
    let router = create_router(state);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Ọmọ Kọ́dà HTTP server listening on {addr}");
    axum::serve(listener, router).await
}
