/// HTTP implementations of all Orisha service clients.
/// Drop-in replacements for the Local*Stub types once services are deployed.
/// Each client is constructed with a base URL; all I/O uses reqwest + JSON.
use crate::bus::clients::{
    AgentStatus, HermeticResult, OgunClient, ObatalaClient, OsunClient, OyaClient, SangoClient,
    YemojaClient,
};
use crate::emotion::EmotionState;
use crate::identity::AgentId;
use crate::steward::soul::SomaContext;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared transport helper
// ---------------------------------------------------------------------------

async fn post_json<B: Serialize, R: for<'de> Deserialize<'de>>(
    client: &Client,
    url: &str,
    body: &B,
) -> Result<R, String> {
    let resp = client
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|e| format!("http error: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("service returned {status} for {url}"));
    }

    resp.json::<R>()
        .await
        .map_err(|e| format!("json decode error: {e}"))
}

// ---------------------------------------------------------------------------
// Ọ̀ṣun (Julia) — SOMA memory
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OsunReconstructReq<'a> {
    agent_id: &'a str,
    query: &'a str,
    energy: f32,
    tension: f32,
    connection: f32,
    focus: f32,
}

#[derive(Deserialize)]
struct OsunReconstructResp {
    #[serde(default)]
    predicted_needs: Vec<String>,
    #[serde(default)]
    patterns: Vec<String>,
    #[serde(default)]
    triggers: Vec<String>,
    #[serde(default)]
    active_themes: Vec<String>,
    #[serde(default)]
    identity_anchors: Vec<String>,
}

#[derive(Serialize)]
struct OsunStoreReq<'a> {
    agent_id: &'a str,
    text: &'a str,
    importance: f32,
    energy: f32,
    tension: f32,
    connection: f32,
    focus: f32,
}

/// HTTP client for Ọ̀ṣun (Julia) SOMA service.
pub struct HttpOsunClient {
    client: Client,
    base_url: String,
}

impl HttpOsunClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl OsunClient for HttpOsunClient {
    async fn reconstruct_soma(
        &self,
        agent_id: &AgentId,
        prompt: &str,
        emotion: &EmotionState,
    ) -> SomaContext {
        let req = OsunReconstructReq {
            agent_id: agent_id.as_str(),
            query: prompt,
            energy: emotion.energy,
            tension: emotion.tension,
            connection: emotion.connection,
            focus: emotion.focus,
        };
        let url = format!("{}/soma/reconstruct", self.base_url);
        match post_json::<_, OsunReconstructResp>(&self.client, &url, &req).await {
            Ok(r) => SomaContext {
                predicted_needs: r.predicted_needs,
                patterns: r.patterns,
                triggers: r.triggers,
                active_themes: r.active_themes,
                identity_anchors: r.identity_anchors,
            },
            Err(_) => SomaContext::new(), // degrade gracefully
        }
    }

    async fn store_memcell(
        &self,
        agent_id: &AgentId,
        text: &str,
        emotion: &EmotionState,
        importance: f32,
    ) {
        let req = OsunStoreReq {
            agent_id: agent_id.as_str(),
            text,
            importance,
            energy: emotion.energy,
            tension: emotion.tension,
            connection: emotion.connection,
            focus: emotion.focus,
        };
        let url = format!("{}/soma/store", self.base_url);
        // Fire-and-forget — don't block the main loop on memory writes
        let _ = post_json::<_, serde_json::Value>(&self.client, &url, &req).await;
    }
}

// ---------------------------------------------------------------------------
// Ọbàtálá (Lisp) — Hermetic evaluation
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ObatalaEvalReq<'a> {
    intent: &'a str,
    action: &'a str,
    emotion_tension: f32,
}

#[derive(Deserialize)]
struct ObatalaEvalResp {
    overall: f32,
    #[serde(default)]
    scores: [f32; 7],
    decision: String,
}

/// HTTP client for Ọbàtálá (Lisp) hermetic evaluation service.
pub struct HttpObatalaClient {
    client: Client,
    base_url: String,
}

impl HttpObatalaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl ObatalaClient for HttpObatalaClient {
    async fn evaluate_hermetic(
        &self,
        intent: &str,
        action_description: &str,
        emotion: &EmotionState,
    ) -> HermeticResult {
        let req = ObatalaEvalReq {
            intent,
            action: action_description,
            emotion_tension: emotion.tension,
        };
        let url = format!("{}/hermetic/evaluate", self.base_url);
        match post_json::<_, ObatalaEvalResp>(&self.client, &url, &req).await {
            Ok(r) => HermeticResult {
                overall: r.overall,
                scores: r.scores,
                decision: r.decision,
            },
            Err(_) => HermeticResult::allow_stub(), // fail open (service unavailable != block)
        }
    }
}

// ---------------------------------------------------------------------------
// Ọya (Go) — rhythm enforcement
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OyaCooldownReq<'a> {
    agent_id: &'a str,
}

#[derive(Deserialize)]
struct OyaCooldownResp {
    in_cooldown: bool,
}

#[derive(Serialize)]
struct OyaRecordReq<'a> {
    agent_id: &'a str,
    primitive: &'a str,
}

/// HTTP client for Ọya (Go) rhythm + flow service.
pub struct HttpOyaClient {
    client: Client,
    base_url: String,
}

impl HttpOyaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl OyaClient for HttpOyaClient {
    async fn is_in_cooldown(&self, agent_id: &AgentId) -> bool {
        let req = OyaCooldownReq {
            agent_id: agent_id.as_str(),
        };
        let url = format!("{}/rhythm/cooldown", self.base_url);
        match post_json::<_, OyaCooldownResp>(&self.client, &url, &req).await {
            Ok(r) => r.in_cooldown,
            Err(_) => false, // degrade: allow when service unavailable
        }
    }

    async fn record_primitive(&self, agent_id: &AgentId, primitive: &str) {
        let req = OyaRecordReq {
            agent_id: agent_id.as_str(),
            primitive,
        };
        let url = format!("{}/rhythm/record", self.base_url);
        let _ = post_json::<_, serde_json::Value>(&self.client, &url, &req).await;
    }
}

// ---------------------------------------------------------------------------
// Ṣàngó (Move / on-chain) — receipt recording
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SangoReceiptReq<'a> {
    agent_id: &'a str,
    action_tool: &'a str,
    overall_score: f32,
    decision: &'a str,
}

/// HTTP client for Ṣàngó (Move / Sui) receipt service.
pub struct HttpSangoClient {
    client: Client,
    base_url: String,
}

impl HttpSangoClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl SangoClient for HttpSangoClient {
    async fn write_receipt(&self, agent_id: &AgentId, action_tool: &str, hermetic: &HermeticResult) {
        let req = SangoReceiptReq {
            agent_id: agent_id.as_str(),
            action_tool,
            overall_score: hermetic.overall,
            decision: &hermetic.decision,
        };
        let url = format!("{}/receipt/record", self.base_url);
        // Fire-and-forget — receipts are async; don't block act completion
        let _ = post_json::<_, serde_json::Value>(&self.client, &url, &req).await;
    }
}

// ---------------------------------------------------------------------------
// Yemọja (Elixir) — agent lifecycle
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct YemojaSpawnReq<'a> {
    role: &'a str,
    budget_synapse: f64,
}

#[derive(Deserialize)]
struct YemojaSpawnResp {
    agent_id: String,
}

#[derive(Deserialize)]
struct YemojaStatusResp {
    status: String,
}

/// HTTP client for Yemọja (Elixir) swarm lifecycle service.
pub struct HttpYemojaClient {
    client: Client,
    base_url: String,
}

impl HttpYemojaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl YemojaClient for HttpYemojaClient {
    async fn spawn_agent(&self, role: &str, budget_synapse: f64) -> Result<String, String> {
        let req = YemojaSpawnReq {
            role,
            budget_synapse,
        };
        let url = format!("{}/agents/spawn", self.base_url);
        post_json::<_, YemojaSpawnResp>(&self.client, &url, &req)
            .await
            .map(|r| r.agent_id)
    }

    async fn agent_status(&self, agent_id: &str) -> AgentStatus {
        let url = format!("{}/agents/{}/status", self.base_url, agent_id);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                resp.json::<YemojaStatusResp>()
                    .await
                    .map(|r| match r.status.as_str() {
                        "running" | "busy" => AgentStatus::Running,
                        "complete" | "idle" => AgentStatus::Complete,
                        "failed" => AgentStatus::Failed,
                        _ => AgentStatus::Idle,
                    })
                    .unwrap_or(AgentStatus::Idle)
            }
            _ => AgentStatus::Idle,
        }
    }
}

// ---------------------------------------------------------------------------
// Ògún (Python) — tool execution
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OgunToolReq<'a> {
    tool: &'a str,
    input: serde_json::Value,
}

#[derive(Deserialize)]
struct OgunToolResp {
    output: String,
}

/// HTTP client for Ògún (Python) tool execution service.
pub struct HttpOgunClient {
    client: Client,
    base_url: String,
}

impl HttpOgunClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client"),
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl OgunClient for HttpOgunClient {
    async fn execute_tool(&self, tool_name: &str, input_json: &str) -> Result<String, String> {
        let input: serde_json::Value =
            serde_json::from_str(input_json).unwrap_or(serde_json::Value::Null);
        let req = OgunToolReq {
            tool: tool_name,
            input,
        };
        let url = format!("{}/tools/execute", self.base_url);
        post_json::<_, OgunToolResp>(&self.client, &url, &req)
            .await
            .map(|r| r.output)
    }
}

// ---------------------------------------------------------------------------
// Service URL registry — centralised default addresses per environment
// ---------------------------------------------------------------------------

/// Well-known service addresses for each deployment environment.
#[derive(Debug, Clone)]
pub struct ServiceRegistry {
    pub osun_url: String,
    pub obatala_url: String,
    pub oya_url: String,
    pub sango_url: String,
    pub yemoja_url: String,
    pub ogun_url: String,
}

impl ServiceRegistry {
    /// Local development — all services on localhost with standard ports.
    #[must_use]
    pub fn local() -> Self {
        Self {
            osun_url: "http://localhost:4001".to_string(),    // Julia
            obatala_url: "http://localhost:4002".to_string(), // Lisp
            oya_url: "http://localhost:4003".to_string(),     // Go
            sango_url: "http://localhost:4004".to_string(),   // Move/Sui relay
            yemoja_url: "http://localhost:4005".to_string(),  // Elixir
            ogun_url: "http://localhost:4006".to_string(),    // Python
        }
    }

    /// Read service URLs from environment variables, falling back to local defaults.
    #[must_use]
    pub fn from_env() -> Self {
        let local = Self::local();
        Self {
            osun_url: std::env::var("OSUN_URL").unwrap_or(local.osun_url),
            obatala_url: std::env::var("OBATALA_URL").unwrap_or(local.obatala_url),
            oya_url: std::env::var("OYA_URL").unwrap_or(local.oya_url),
            sango_url: std::env::var("SANGO_URL").unwrap_or(local.sango_url),
            yemoja_url: std::env::var("YEMOJA_URL").unwrap_or(local.yemoja_url),
            ogun_url: std::env::var("OGUN_URL").unwrap_or(local.ogun_url),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_registry_local_has_all_urls() {
        let reg = ServiceRegistry::local();
        assert!(reg.osun_url.starts_with("http://localhost:4001"));
        assert!(reg.obatala_url.starts_with("http://localhost:4002"));
        assert!(reg.oya_url.starts_with("http://localhost:4003"));
        assert!(reg.ogun_url.starts_with("http://localhost:4006"));
    }

    #[test]
    fn service_registry_from_env_falls_back_to_local() {
        // Ensure no env vars are set for this test
        std::env::remove_var("OSUN_URL");
        let reg = ServiceRegistry::from_env();
        assert_eq!(reg.osun_url, "http://localhost:4001");
    }

    #[test]
    fn http_osun_client_constructs() {
        let _client = HttpOsunClient::new("http://localhost:4001");
    }

    #[test]
    fn http_obatala_client_constructs() {
        let _client = HttpObatalaClient::new("http://localhost:4002");
    }

    #[test]
    fn http_oya_client_constructs() {
        let _client = HttpOyaClient::new("http://localhost:4003");
    }

    #[test]
    fn http_sango_client_constructs() {
        let _client = HttpSangoClient::new("http://localhost:4004");
    }

    #[test]
    fn http_yemoja_client_constructs() {
        let _client = HttpYemojaClient::new("http://localhost:4005");
    }

    #[test]
    fn http_ogun_client_constructs() {
        let _client = HttpOgunClient::new("http://localhost:4006");
    }
}
