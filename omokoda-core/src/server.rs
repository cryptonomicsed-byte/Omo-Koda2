use crate::bus::events::sovereign_event;
use crate::interpreter::{ExecutionResult, Steward};
use crate::parser::{MetadataPair, Statement, ThinkModifiers};
use crate::vault::{
    galaxy_data, insert_knowledge, list_files, load_vault_config, read_file, save_vault_config,
    KnowledgeTriple, VaultConfig,
};
use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub steward: Arc<Mutex<Steward>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            steward: Arc::new(Mutex::new(Steward::new())),
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
    Json(StatusResponse {
        has_agent: steward.agent_core().is_some(),
        name: steward.agent_core().map(|a| a.name().to_string()),
        id: steward.agent_core().map(|a| a.id().as_str().to_string()),
        reputation: steward.agent_core().map(|a| a.reputation()),
        tier: steward.agent_core().map(|a| a.tier()),
        synapse: steward.agent_core().map(|a| a.synapse()),
    })
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

// ---------------------------------------------------------------------------
// Vault handlers
// ---------------------------------------------------------------------------

async fn vault_files_handler(State(state): State<AppState>) -> impl IntoResponse {
    let agent_id = {
        let s = state.steward.lock().await;
        s.agent_core().map(|a| a.id().as_str().to_string())
    };
    match agent_id {
        None => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no agent born"})),
        )
            .into_response(),
        Some(id) => Json(list_files(&id)).into_response(),
    }
}

async fn vault_read_file_handler(
    State(state): State<AppState>,
    Path(rel): Path<String>,
) -> impl IntoResponse {
    let agent_id = {
        let s = state.steward.lock().await;
        s.agent_core().map(|a| a.id().as_str().to_string())
    };
    match agent_id {
        None => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no agent born"})),
        )
            .into_response(),
        Some(id) => match read_file(&id, &rel) {
            Ok(content) => Json(serde_json::json!({"path": rel, "content": content})).into_response(),
            Err(e) => (
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": e})),
            )
                .into_response(),
        },
    }
}

#[derive(Deserialize)]
pub struct KnowledgeRequest {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    #[serde(default)]
    pub confidence: Option<f64>,
}

async fn vault_knowledge_handler(
    State(state): State<AppState>,
    Json(req): Json<KnowledgeRequest>,
) -> impl IntoResponse {
    let agent_id = {
        let s = state.steward.lock().await;
        s.agent_core().map(|a| a.id().as_str().to_string())
    };
    match agent_id {
        None => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no agent born"})),
        )
            .into_response(),
        Some(id) => {
            let triple = KnowledgeTriple {
                subject: req.subject,
                predicate: req.predicate,
                object: req.object,
                confidence: req.confidence.unwrap_or(1.0),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            match insert_knowledge(&id, triple) {
                Ok(()) => Json(serde_json::json!({"ok": true})).into_response(),
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
    }
}

async fn vault_galaxy_handler(State(state): State<AppState>) -> impl IntoResponse {
    let agent_id = {
        let s = state.steward.lock().await;
        s.agent_core().map(|a| a.id().as_str().to_string())
    };
    match agent_id {
        None => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no agent born"})),
        )
            .into_response(),
        Some(id) => Json(galaxy_data(&id)).into_response(),
    }
}

#[derive(Deserialize)]
pub struct SyncRequest {
    #[serde(default)]
    pub content: String,
}

async fn vault_sync_handler(
    State(state): State<AppState>,
    Json(req): Json<SyncRequest>,
) -> impl IntoResponse {
    let agent_id = {
        let s = state.steward.lock().await;
        s.agent_core().map(|a| a.id().as_str().to_string())
    };
    match agent_id {
        None => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no agent born"})),
        )
            .into_response(),
        Some(id) => match crate::vault::vault_sync(&id, &req.content) {
            Ok(()) => Json(serde_json::json!({"ok": true})).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response(),
        },
    }
}

async fn vault_config_get_handler(State(state): State<AppState>) -> impl IntoResponse {
    let agent_id = {
        let s = state.steward.lock().await;
        s.agent_core().map(|a| a.id().as_str().to_string())
    };
    match agent_id {
        None => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no agent born"})),
        )
            .into_response(),
        Some(id) => Json(load_vault_config(&id)).into_response(),
    }
}

async fn vault_config_put_handler(
    State(state): State<AppState>,
    Json(cfg): Json<VaultConfig>,
) -> impl IntoResponse {
    let agent_id = {
        let s = state.steward.lock().await;
        s.agent_core().map(|a| a.id().as_str().to_string())
    };
    match agent_id {
        None => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no agent born"})),
        )
            .into_response(),
        Some(id) => match save_vault_config(&id, &cfg) {
            Ok(()) => Json(serde_json::json!({"ok": true})).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response(),
        },
    }
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
            "resource": e.resource,
        }),
        Some(sovereign_event::Event::Audit(e)) => json!({
            "type": "audit",
            "event_type": e.event_type,
            "details": e.details,
            "timestamp": e.timestamp,
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
        // Vault routes
        .route("/v1/vault/files", get(vault_files_handler))
        .route("/v1/vault/file/*rel", get(vault_read_file_handler))
        .route("/v1/vault/knowledge", post(vault_knowledge_handler))
        .route("/v1/vault/galaxy", get(vault_galaxy_handler))
        .route("/v1/vault/sync", post(vault_sync_handler))
        .route("/v1/vault/config", get(vault_config_get_handler))
        .route("/v1/vault/config", put(vault_config_put_handler))
        // Rhythm route
        .route("/v1/rhythm/today", get(rhythm_today_handler))
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
