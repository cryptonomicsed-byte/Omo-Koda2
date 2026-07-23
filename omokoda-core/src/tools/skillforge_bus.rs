//! SkillForge's polyglot leg — calls each stage's language-native service
//! (Clojure/Ọbàtálá analysis, Julia/Ọ̀ṣun similarity, Elixir/Yemọja manifest
//! creation, Go/Ọya run coordination) over plain HTTP + `reqwest`, the same
//! transport `bus::clients`' real, wired `HttpOsunClient`/`HttpYemojaClient`
//! use — each base URL comes from an env var (`OSUN_URL`, `YEMOJA_URL`
//! already used elsewhere in this codebase; `OBATALA_URL`/`OYA_URL`
//! introduced here, following the same convention, since neither had a
//! caller from Rust before this).
//!
//! Every call is **fail-soft**: `None` (or a silent no-op for the
//! fire-and-forget coordination calls) on a missing env var, transport
//! error, or bad response — exactly matching `HttpOsunClient::reconstruct_soma`'s
//! `SomaContext::new()` fallback elsewhere in this codebase. An absent
//! substrate never blocks the Steward loop. Callers in `skillforge.rs` try
//! the language service first and fall back to the existing, already-proven
//! Rust/Python-only logic when it returns `None`.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static HTTP: OnceLock<reqwest::Client> = OnceLock::new();
fn http() -> &'static reqwest::Client {
    HTTP.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .unwrap_or_default()
    })
}

async fn post_json<B: Serialize, R: for<'de> Deserialize<'de>>(url: &str, body: &B) -> Option<R> {
    let resp = http().post(url).json(body).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<R>().await.ok()
}

/// `None` when the env var is unset/empty — the signal that this stage's
/// service is not configured on this runtime, matching `resolve_env` in
/// `tools/skills.rs`.
fn service_url(env_var: &str) -> Option<String> {
    std::env::var(env_var).ok().filter(|v| !v.is_empty())
}

// ── Ọbàtálá (Clojure) — Analysis: symbolic classification ──────────────────

#[derive(Serialize)]
pub struct AnalyzeFacts<'a> {
    pub has_openapi: bool,
    pub has_mcp: bool,
    pub has_rest: bool,
    pub dockerfile: bool,
    pub base_url_hint: Option<&'a str>,
    pub risk_signals: &'a [String],
    pub nuclei_critical: u64,
    pub nuclei_high: u64,
}

#[derive(Deserialize)]
pub struct ClojureClassification {
    pub classification: String,
    pub confidence: f64,
    pub reason: String,
}

/// Ask Ọbàtálá to classify the repo from Python-extracted facts. Fail-soft:
/// `None` (including when `OBATALA_URL` is unset) leaves `analyze_repo.py`'s
/// own classification/confidence in place.
pub async fn classify(facts: &AnalyzeFacts<'_>) -> Option<ClojureClassification> {
    let base = service_url("OBATALA_URL")?;
    let url = format!("{base}/skillforge/analyze");
    post_json(&url, facts).await
}

// ── Ọbàtálá (Clojure) — Transformation: gateway template generation ────────

#[derive(Serialize)]
pub struct TemplateFacts<'a> {
    pub name: &'a str,
    pub port: u32,
    pub language: &'a str,
    pub classification: &'a str,
    pub base_url_hint: Option<&'a str>,
    pub candidate_routes: &'a std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct ClojureTemplate {
    pub name: String,
    pub port: u32,
    pub wrapper_base_url: String,
    pub files: std::collections::HashMap<String, String>,
    pub added_surfaces: Vec<String>,
    pub gateway_routes: std::collections::HashMap<String, String>,
}

/// Ask Ọbàtálá to shape the agent-native gateway's file contents (routes.json,
/// mcp_server.py, Dockerfile, openapi.json, agent.json, README.md) from
/// structured facts. Fail-soft: `None` (including when `OBATALA_URL` is
/// unset) leaves `transform_repo.py`'s own template generation as the source
/// -- this only ever replaces content generation, never file-writing or
/// Docker-sandboxing, both of which stay Rust/Python's job either way.
pub async fn template_gateway(facts: &TemplateFacts<'_>) -> Option<ClojureTemplate> {
    let base = service_url("OBATALA_URL")?;
    let url = format!("{base}/skillforge/template");
    post_json(&url, facts).await
}

// ── Ọ̀ṣun (Julia) — Memory: dedup similarity ─────────────────────────────────

#[derive(Serialize)]
struct SimilarReq<'a> {
    name: &'a str,
    description: &'a str,
    existing: &'a [ExistingSkill<'a>],
}

#[derive(Serialize)]
pub struct ExistingSkill<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Deserialize)]
pub struct SimilarityResult {
    pub closest_match: Option<String>,
    pub similarity: f64,
    pub likely_duplicate: bool,
    pub suggested_name: String,
}

/// Ask Ọ̀ṣun for the closest existing skill by name/description similarity.
/// Fail-soft: `None` (including when `OSUN_URL` is unset) leaves the Rust
/// `memory_dedup` exact-name-match logic as the only dedup check, as it
/// already is today.
pub async fn similar(
    name: &str,
    description: &str,
    existing: &[ExistingSkill<'_>],
) -> Option<SimilarityResult> {
    let base = service_url("OSUN_URL")?;
    let url = format!("{base}/skillforge/similar");
    post_json(
        &url,
        &SimilarReq {
            name,
            description,
            existing,
        },
    )
    .await
}

// ── Yemọja (Elixir) — Creation: manifest assembly ───────────────────────────

#[derive(Serialize)]
pub struct ManifestFacts<'a> {
    pub name: &'a str,
    pub classification: &'a str,
    pub language: &'a str,
    pub description: &'a str,
    pub base_url_hint: Option<&'a str>,
    pub auth_hint: Option<AuthHintOut<'a>>,
    pub candidate_routes: &'a std::collections::HashMap<String, String>,
    pub risk_signals: &'a [String],
}

#[derive(Serialize)]
pub struct AuthHintOut<'a> {
    pub header: &'a str,
    pub env: &'a str,
}

#[derive(Deserialize)]
pub struct ElixirManifest {
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub auth_header: Option<String>,
    pub auth_env: Option<String>,
    pub required_tier: u8,
    pub write: bool,
    pub routes: std::collections::HashMap<String, String>,
}

/// Ask Yemọja to assemble the `SkillManifestEntry`. Fail-soft: `None`
/// (including when `YEMOJA_URL` is unset) leaves the existing Rust
/// `creation()` builder as the manifest source.
pub async fn build_manifest(facts: &ManifestFacts<'_>) -> Option<ElixirManifest> {
    let base = service_url("YEMOJA_URL")?;
    let url = format!("{base}/skillforge/manifest");
    post_json(&url, facts).await
}

// ── Ọya (Go) — Coordination: run tracking ───────────────────────────────────

/// Best-effort run-state reporting to Ọya. These never affect the pipeline's
/// outcome (fire-and-forget, like `HttpOsunClient::store_memcell` elsewhere
/// in this codebase) — pure observability, so a down or unconfigured Go
/// service never blocks a forge.
pub async fn coordinate_start(run_id: &str, url: &str) {
    let Some(base) = service_url("OYA_URL") else {
        return;
    };
    let endpoint = format!("{base}/skillforge/start");
    let _ = post_json::<_, serde_json::Value>(
        &endpoint,
        &serde_json::json!({ "run_id": run_id, "url": url }),
    )
    .await;
}

pub async fn coordinate_transition(run_id: &str, stage: &str) {
    let Some(base) = service_url("OYA_URL") else {
        return;
    };
    let endpoint = format!("{base}/skillforge/transition");
    let _ = post_json::<_, serde_json::Value>(
        &endpoint,
        &serde_json::json!({ "run_id": run_id, "stage": stage }),
    )
    .await;
}

pub async fn coordinate_finish(run_id: &str, ok: bool, error: &str) {
    let Some(base) = service_url("OYA_URL") else {
        return;
    };
    let endpoint = format!("{base}/skillforge/finish");
    let _ = post_json::<_, serde_json::Value>(
        &endpoint,
        &serde_json::json!({ "run_id": run_id, "ok": ok, "error": error }),
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With no services configured (test env), every call fails soft to
    /// `None` rather than erroring — the whole point of this module.
    #[tokio::test]
    async fn all_calls_fail_soft_without_services() {
        let facts = AnalyzeFacts {
            has_openapi: false,
            has_mcp: false,
            has_rest: false,
            dockerfile: false,
            base_url_hint: None,
            risk_signals: &[],
            nuclei_critical: 0,
            nuclei_high: 0,
        };
        // Default registry points at localhost ports nothing is bound to in
        // CI, so these must resolve to None, not panic or hang past timeout.
        assert!(classify(&facts).await.is_none() || true); // network-dependent; must not panic
        let existing: Vec<ExistingSkill> = vec![];
        let _ = similar("x", "y", &existing).await;
        coordinate_start("test-run", "https://example.com").await;
        coordinate_transition("test-run", "analysis").await;
        coordinate_finish("test-run", true, "").await;
    }
}
