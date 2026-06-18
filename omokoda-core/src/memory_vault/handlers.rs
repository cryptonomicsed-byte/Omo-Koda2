use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
};

use crate::memory_vault::{
    types::{SearchQuery, UpdateConfigBody, VaultStatus},
    vault::MemoryVault,
};
use crate::server::AppState;

pub async fn get_vault_status(State(state): State<AppState>) -> impl IntoResponse {
    let steward = state.steward.lock().await;
    let Some(agent) = steward.agent_core() else {
        return (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no active agent"})),
        )
            .into_response();
    };
    let (agent_id, agent_name) = (agent.id().as_str().to_string(), agent.name().to_string());
    drop(steward);

    let vault = MemoryVault::new(&agent_id, &agent_name, &state.vault_base);
    Json(VaultStatus {
        enabled: true,
        config: vault.load_config(),
        note_counts: vault.note_counts(),
        vault_path: vault.vault_path.to_string_lossy().to_string(),
    })
    .into_response()
}

pub async fn get_vault_config(State(state): State<AppState>) -> impl IntoResponse {
    let steward = state.steward.lock().await;
    let Some(agent) = steward.agent_core() else {
        return (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no active agent"})),
        )
            .into_response();
    };
    let (agent_id, agent_name) = (agent.id().as_str().to_string(), agent.name().to_string());
    drop(steward);

    let vault = MemoryVault::new(&agent_id, &agent_name, &state.vault_base);
    Json(vault.load_config()).into_response()
}

pub async fn put_vault_config(
    State(state): State<AppState>,
    Json(body): Json<UpdateConfigBody>,
) -> impl IntoResponse {
    let steward = state.steward.lock().await;
    let Some(agent) = steward.agent_core() else {
        return (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no active agent"})),
        )
            .into_response();
    };
    let (agent_id, agent_name) = (agent.id().as_str().to_string(), agent.name().to_string());
    drop(steward);

    let vault = MemoryVault::new(&agent_id, &agent_name, &state.vault_base);
    let mut config = vault.load_config();
    config.access = body.access;
    if let Some(peers) = body.federation_peers {
        config.federation_peers = peers;
    }
    if let Some(auto) = body.auto_export {
        config.auto_export = auto;
    }

    match vault.save_config(&config) {
        Ok(_) => Json(config).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub async fn post_vault_sync(State(state): State<AppState>) -> impl IntoResponse {
    let (agent_id, agent_name, session_clone) = {
        let steward = state.steward.lock().await;
        let Some(agent) = steward.agent_core() else {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "no active agent"})),
            )
                .into_response();
        };
        (
            agent.id().as_str().to_string(),
            agent.name().to_string(),
            agent.session().clone(),
        )
    };

    let vault = MemoryVault::new(&agent_id, &agent_name, &state.vault_base);
    vault.full_sync_from_session(&session_clone);

    Json(serde_json::json!({
        "status": "synced",
        "counts": vault.note_counts(),
    }))
    .into_response()
}

pub async fn get_galaxy_data(State(state): State<AppState>) -> impl IntoResponse {
    let steward = state.steward.lock().await;
    let Some(agent) = steward.agent_core() else {
        return (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no active agent"})),
        )
            .into_response();
    };
    let (agent_id, agent_name) = (agent.id().as_str().to_string(), agent.name().to_string());
    drop(steward);

    let vault = MemoryVault::new(&agent_id, &agent_name, &state.vault_base);
    Json(vault.get_galaxy_data()).into_response()
}

pub async fn search_vault(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let steward = state.steward.lock().await;
    let Some(agent) = steward.agent_core() else {
        return (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no active agent"})),
        )
            .into_response();
    };
    let (agent_id, agent_name) = (agent.id().as_str().to_string(), agent.name().to_string());
    drop(steward);

    let vault = MemoryVault::new(&agent_id, &agent_name, &state.vault_base);
    Json(serde_json::json!({
        "query": params.q,
        "results": vault.search(&params.q),
    }))
    .into_response()
}
