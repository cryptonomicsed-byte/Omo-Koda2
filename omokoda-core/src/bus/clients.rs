use crate::emotion::EmotionState;
use crate::identity::AgentId;
use crate::steward::iris::IrisParams;
use crate::steward::soul::SomaContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Result of a Hermetic principle evaluation (from Ọbàtálá / Lisp service).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermeticResult {
    /// Overall alignment score 0.0–1.0
    pub overall: f32,
    /// Per-principle scores: [mentalism, correspondence, vibration, polarity, rhythm, cause_effect, gender]
    pub scores: [f32; 7],
    /// "Allow", "Warn: ...", or "Block: ..."
    pub decision: String,
}

impl HermeticResult {
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        self.decision.starts_with("Allow")
    }

    #[must_use]
    pub fn is_blocked(&self) -> bool {
        self.decision.starts_with("Block")
    }

    /// Always-allow stub — used when no Hermetic service is configured.
    #[must_use]
    pub fn allow_stub() -> Self {
        Self {
            overall: 0.85,
            scores: [0.85; 7],
            decision: "Allow".to_string(),
        }
    }
}

/// Ọ̀ṣun (Julia) client — SOMA reconstruction and memory operations.
/// Called during `think` to retrieve contextual memory before the LLM call.
#[async_trait]
pub trait OsunClient: Send + Sync {
    /// Retrieve SOMA context for the current prompt + emotion state.
    async fn reconstruct_soma(
        &self,
        agent_id: &AgentId,
        prompt: &str,
        emotion: &EmotionState,
    ) -> SomaContext;

    /// Store a MemCell after a completed `think` turn.
    async fn store_memcell(
        &self,
        agent_id: &AgentId,
        text: &str,
        emotion: &EmotionState,
        importance: f32,
    );
}

/// Ọbàtálá (Lisp) client — Hermetic principle evaluation.
/// Called during `think` to gate the LLM call, and during `act` to gate tool execution.
#[async_trait]
pub trait ObatalaClient: Send + Sync {
    /// Evaluate `intent` + `action_description` against the 7 Hermetic principles.
    async fn evaluate_hermetic(
        &self,
        intent: &str,
        action_description: &str,
        emotion: &EmotionState,
    ) -> HermeticResult;
}

/// Ọya (Go) client — rhythm enforcement and inter-service transport.
#[async_trait]
pub trait OyaClient: Send + Sync {
    /// Check if the agent is in a rhythm cooldown period.
    async fn is_in_cooldown(&self, agent_id: &AgentId) -> bool;

    /// Record a completed primitive for rhythm tracking.
    async fn record_primitive(&self, agent_id: &AgentId, primitive: &str);
}

/// Ṣàngó (Move) client — on-chain receipt and reputation.
/// Called after `act` to write an immutable receipt.
#[async_trait]
pub trait SangoClient: Send + Sync {
    async fn write_receipt(
        &self,
        agent_id: &AgentId,
        action_tool: &str,
        hermetic: &HermeticResult,
    );
}

/// Yemọja (Elixir) client — agent lifecycle and swarm coordination.
/// Routes to omokoda-swarm supervision trees for sub-agent spawning and delegation.
#[async_trait]
pub trait YemojaClient: Send + Sync {
    /// Spawn a sub-agent with a given role and Synapse budget.
    async fn spawn_agent(&self, role: &str, budget_synapse: f64) -> Result<String, String>;
    /// Query the live status of a spawned sub-agent by ID.
    async fn agent_status(&self, agent_id: &str) -> AgentStatus;
}

/// Ògún (Python) client — tool execution and external integrations.
/// Routes to the Python execution layer for data processing and rapid prototyping.
#[async_trait]
pub trait OgunClient: Send + Sync {
    /// Execute a named Python tool with JSON input; returns JSON output.
    async fn execute_tool(&self, tool_name: &str, input_json: &str) -> Result<String, String>;
}

/// Status of a spawned sub-agent in the Yemọja supervision tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Running,
    Complete,
    Failed,
}

// ─── Stub implementations (no-ops until real services are deployed) ──────────

/// Local stub — SOMA reconstruction returns empty context.
/// Replace with a real `OsunClient` implementation once the Julia service is deployed.
pub struct LocalOsunStub;

#[async_trait]
impl OsunClient for LocalOsunStub {
    async fn reconstruct_soma(
        &self,
        _agent_id: &AgentId,
        _prompt: &str,
        _emotion: &EmotionState,
    ) -> SomaContext {
        SomaContext::new()
    }

    async fn store_memcell(
        &self,
        _agent_id: &AgentId,
        _text: &str,
        _emotion: &EmotionState,
        _importance: f32,
    ) {
    }
}

/// Local stub — always returns Allow with neutral scores.
pub struct LocalObatalaStub;

#[async_trait]
impl ObatalaClient for LocalObatalaStub {
    async fn evaluate_hermetic(
        &self,
        _intent: &str,
        _action_description: &str,
        _emotion: &EmotionState,
    ) -> HermeticResult {
        HermeticResult::allow_stub()
    }
}

/// Local stub — never in cooldown, recording is a no-op.
pub struct LocalOyaStub;

#[async_trait]
impl OyaClient for LocalOyaStub {
    async fn is_in_cooldown(&self, _agent_id: &AgentId) -> bool {
        false
    }

    async fn record_primitive(&self, _agent_id: &AgentId, _primitive: &str) {}
}

/// Local stub — receipt writing is a no-op until Sui integration is ready.
pub struct LocalSangoStub;

/// Local stub — sub-agent spawning returns a placeholder ID.
/// Replace with a real `YemojaClient` once omokoda-swarm HTTP API is deployed.
pub struct LocalYemojaStub;

/// Local stub — Python tool execution returns a stub JSON payload.
/// Replace with a real `OgunClient` once the Python execution service is deployed.
pub struct LocalOgunStub;

#[async_trait]
impl SangoClient for LocalSangoStub {
    async fn write_receipt(
        &self,
        _agent_id: &AgentId,
        _action_tool: &str,
        _hermetic: &HermeticResult,
    ) {
    }
}

#[async_trait]
impl YemojaClient for LocalYemojaStub {
    async fn spawn_agent(&self, _role: &str, _budget_synapse: f64) -> Result<String, String> {
        Ok("stub-agent-id".to_string())
    }

    async fn agent_status(&self, _agent_id: &str) -> AgentStatus {
        AgentStatus::Idle
    }
}

#[async_trait]
impl OgunClient for LocalOgunStub {
    async fn execute_tool(&self, tool_name: &str, _input_json: &str) -> Result<String, String> {
        Ok(format!(r#"{{"stub": true, "tool": "{}"}}"#, tool_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::AgentId;

    fn agent() -> AgentId {
        AgentId::new("test-fingerprint-bus-clients")
    }

    #[tokio::test]
    async fn osun_stub_returns_empty_soma() {
        let stub = LocalOsunStub;
        let soma = stub
            .reconstruct_soma(&agent(), "hello", &EmotionState::birth())
            .await;
        assert!(!soma.has_content());
    }

    #[tokio::test]
    async fn obatala_stub_allows() {
        let stub = LocalObatalaStub;
        let result = stub
            .evaluate_hermetic("help user debug", "read file", &EmotionState::birth())
            .await;
        assert!(result.is_allowed());
        assert!(!result.is_blocked());
        assert_eq!(result.scores.len(), 7);
    }

    #[tokio::test]
    async fn oya_stub_never_in_cooldown() {
        let stub = LocalOyaStub;
        assert!(!stub.is_in_cooldown(&agent()).await);
    }

    #[test]
    fn hermetic_result_block_detection() {
        let mut r = HermeticResult::allow_stub();
        r.decision = "Block: deception detected".to_string();
        assert!(r.is_blocked());
        assert!(!r.is_allowed());
    }

    #[tokio::test]
    async fn yemoja_stub_spawns_stub_agent() {
        let stub = LocalYemojaStub;
        let result = stub.spawn_agent("analyst", 1000.0).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "stub-agent-id");
    }

    #[tokio::test]
    async fn yemoja_stub_status_is_idle() {
        let stub = LocalYemojaStub;
        assert_eq!(stub.agent_status("any-id").await, AgentStatus::Idle);
    }

    #[tokio::test]
    async fn ogun_stub_returns_stub_json() {
        let stub = LocalOgunStub;
        let result = stub.execute_tool("data_transform", r#"{"input": "x"}"#).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("stub"));
        assert!(json.contains("data_transform"));
    }
}
