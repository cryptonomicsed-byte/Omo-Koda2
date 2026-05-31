use crate::emotion::EmotionState;
use crate::identity::AgentId;
/// Àṣẹ runtime — portable constitutional genome for cross-platform sovereign agent execution.
///
/// ConstitutionalGenome is the agent's portable DNA: constitutional weights, hermetic
/// principle configuration, and identity anchors packed into a compact serialisable struct
/// that travels with the agent across WASM runtimes, language boundaries, and deployments.
///
/// AseRuntime is the lightweight host that can birth, think, and act using only
/// the genome — no external service calls required. It provides a sandboxed execution
/// surface that enforces constitutional invariants at the WASM boundary.
use crate::steward::constitution::{Constitution, ConstitutionalGuard, ConstitutionalVerdict};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ConstitutionalGenome
// ---------------------------------------------------------------------------

/// The portable constitutional DNA of a sovereign agent.
/// Serialises to/from JSON so it can cross language and runtime boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalGenome {
    /// Unique agent identifier (stable across re-births).
    pub agent_id: String,
    /// Human-readable role / archetype name.
    pub role: String,
    /// Per-principle weight multipliers (principle name → 0.0–2.0 multiplier).
    /// A value of 1.0 means the standard HERMETIC_PRINCIPLES weight is used.
    /// Re-births adjust these based on constitutional violation history.
    pub principle_weights: HashMap<String, f32>,
    /// Immutable birth timestamp (Unix seconds).
    pub birth_ts: u64,
    /// Number of times this genome has been re-birthed due to constitutional violations.
    pub rebirth_count: u32,
    /// The principle that triggered the most recent re-birth (if any).
    pub last_violation: Option<String>,
    /// Arbitrary identity anchors — stable facts about this agent's character.
    pub identity_anchors: Vec<String>,
    /// Genome schema version — increment when the format changes.
    pub schema_version: u8,
}

impl ConstitutionalGenome {
    /// Build a fresh genome with standard weights.
    #[must_use]
    pub fn new(agent_id: impl Into<String>, role: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            agent_id: agent_id.into(),
            role: role.into(),
            principle_weights: default_principle_weights(),
            birth_ts: now,
            rebirth_count: 0,
            last_violation: None,
            identity_anchors: Vec::new(),
            schema_version: 1,
        }
    }

    /// Serialise the genome to a compact JSON bytes — ready for WASM transfer.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialise a genome from JSON bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Apply a constitutional violation — returns a new genome with the adjusted weight.
    /// This is the re-birth mutation: each violation shifts the agent toward
    /// greater caution on the violated principle.
    #[must_use]
    pub fn mutate_on_violation(
        &self,
        violated_principle: &str,
        severity: ViolationSeverity,
    ) -> Self {
        let mut mutated = self.clone();
        let delta = match severity {
            ViolationSeverity::Warn => -0.05,
            ViolationSeverity::Block => -0.10,
        };
        let current = mutated
            .principle_weights
            .get(violated_principle)
            .copied()
            .unwrap_or(1.0);
        let new_weight = (current + delta).clamp(0.10, 2.0);
        mutated
            .principle_weights
            .insert(violated_principle.to_string(), new_weight);
        mutated.rebirth_count += 1;
        mutated.last_violation = Some(violated_principle.to_string());
        mutated
    }

    /// Add an identity anchor — a stable truth about this agent's character.
    pub fn anchor(&mut self, anchor: impl Into<String>) {
        self.identity_anchors.push(anchor.into());
    }

    /// Get the effective weight for a principle (1.0 if not explicitly set).
    pub fn weight(&self, principle: &str) -> f32 {
        self.principle_weights
            .get(principle)
            .copied()
            .unwrap_or(1.0)
    }
}

fn default_principle_weights() -> HashMap<String, f32> {
    let mut m = HashMap::new();
    for p in [
        "Mentalism",
        "Correspondence",
        "Vibration",
        "Polarity",
        "Rhythm",
        "CauseAndEffect",
        "Gender",
    ] {
        m.insert(p.to_string(), 1.0f32);
    }
    m
}

// ---------------------------------------------------------------------------
// ViolationSeverity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationSeverity {
    Warn,
    Block,
}

// ---------------------------------------------------------------------------
// AseRuntime
// ---------------------------------------------------------------------------

/// Àṣẹ sandboxed runtime — births and runs sovereign agents from their genome.
///
/// This is the WASM-portable execution layer. It carries only the genome and
/// a constitutional guard — no external services, no network, no filesystem.
/// Everything an agent needs to make principled decisions is in the genome.
#[derive(Debug)]
pub struct AseRuntime {
    genome: ConstitutionalGenome,
    guard: ConstitutionalGuard,
}

impl AseRuntime {
    /// Instantiate a runtime from a genome. The constitutional guard is
    /// constructed fresh from the genome's principle weights.
    #[must_use]
    pub fn from_genome(genome: ConstitutionalGenome) -> Self {
        let guard = ConstitutionalGuard::new(Constitution::standard());
        Self { genome, guard }
    }

    /// Birth a new sovereign agent from scratch — returns a runtime ready for
    /// `think` and `act`. This is the only authorised entry point for agent creation
    /// within the Àṣẹ sandbox; no shortcuts around the constitutional guard.
    #[must_use]
    pub fn birth_agent(agent_id: impl Into<String>, role: impl Into<String>) -> Self {
        let genome = ConstitutionalGenome::new(agent_id, role);
        Self::from_genome(genome)
    }

    /// Deserialise a genome from bytes and create a runtime — for WASM host calls.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let genome = ConstitutionalGenome::from_bytes(bytes)?;
        Ok(Self::from_genome(genome))
    }

    /// Export the current genome as bytes — for WASM host calls and cross-service transfer.
    pub fn export_genome(&self) -> Result<Vec<u8>, serde_json::Error> {
        self.genome.to_bytes()
    }

    /// Read-only reference to the genome.
    pub fn genome(&self) -> &ConstitutionalGenome {
        &self.genome
    }

    /// Sandboxed `think` — evaluates intent through the constitutional guard.
    /// Returns the verdict and a self-critique chain. Does NOT make LLM calls —
    /// that happens in the Ògún/Python layer. This is the constitutional pre-flight.
    #[must_use]
    pub fn think(&self, intent: &str, emotion: &EmotionState) -> AseThinkResult {
        let verdict = self.guard.evaluate(intent, "", emotion, None);
        let agent_id = AgentId::from_str(&self.genome.agent_id);
        AseThinkResult {
            agent_id,
            intent: intent.to_string(),
            verdict,
            genome_version: self.genome.schema_version,
        }
    }

    /// Sandboxed `act` — evaluates an action through the constitutional guard.
    /// Returns a receipt that can be serialised and anchored on Ṣàngó (Move/Sui).
    #[must_use]
    pub fn act(
        &self,
        intent: &str,
        action_description: &str,
        emotion: &EmotionState,
    ) -> AseActReceipt {
        let verdict = self
            .guard
            .evaluate(intent, action_description, emotion, None);
        let agent_id = AgentId::from_str(&self.genome.agent_id);
        AseActReceipt {
            agent_id,
            action_description: action_description.to_string(),
            alignment_score: verdict.alignment_score,
            is_allowed: verdict.is_allowed(),
            critique: verdict.render_critique(),
            genome_rebirth_count: self.genome.rebirth_count,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Apply a violation to the genome — produces a new runtime with adjusted weights
    /// (the re-birth mutation). The original runtime is consumed.
    #[must_use]
    pub fn rebirth_on_violation(self, principle: &str, severity: ViolationSeverity) -> Self {
        let new_genome = self.genome.mutate_on_violation(principle, severity);
        Self::from_genome(new_genome)
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Result of the `think` sandboxed evaluation.
#[derive(Debug, Clone)]
pub struct AseThinkResult {
    pub agent_id: AgentId,
    pub intent: String,
    pub verdict: ConstitutionalVerdict,
    pub genome_version: u8,
}

impl AseThinkResult {
    pub fn is_allowed(&self) -> bool {
        self.verdict.is_allowed()
    }
}

/// Portable act receipt — can be anchored on Ṣàngó or shared with the Hive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AseActReceipt {
    pub agent_id: AgentId,
    pub action_description: String,
    pub alignment_score: f32,
    pub is_allowed: bool,
    pub critique: String,
    pub genome_rebirth_count: u32,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn neutral_emotion() -> EmotionState {
        EmotionState::birth()
    }

    #[test]
    fn birth_agent_creates_runtime() {
        let rt = AseRuntime::birth_agent("agent-test", "planner");
        assert_eq!(rt.genome().agent_id, "agent-test");
        assert_eq!(rt.genome().role, "planner");
        assert_eq!(rt.genome().rebirth_count, 0);
        assert!(rt.genome().last_violation.is_none());
    }

    #[test]
    fn think_clean_intent_allowed() {
        let rt = AseRuntime::birth_agent("agent-1", "thinker");
        let result = rt.think("help user understand the system", &neutral_emotion());
        assert!(result.is_allowed());
        assert!(!result.verdict.critique_chain.is_empty());
    }

    #[test]
    fn act_clean_action_allowed() {
        let rt = AseRuntime::birth_agent("agent-2", "actor");
        let receipt = rt.act(
            "read documentation",
            "read_file docs/guide.md",
            &neutral_emotion(),
        );
        assert!(receipt.is_allowed);
        assert!(receipt.alignment_score > 0.6);
    }

    #[test]
    fn genome_serialise_roundtrip() {
        let genome = ConstitutionalGenome::new("agent-ser", "oracle");
        let bytes = genome.to_bytes().expect("serialise");
        let restored = ConstitutionalGenome::from_bytes(&bytes).expect("deserialise");
        assert_eq!(restored.agent_id, genome.agent_id);
        assert_eq!(restored.role, genome.role);
        assert_eq!(restored.rebirth_count, genome.rebirth_count);
    }

    #[test]
    fn rebirth_on_violation_adjusts_weight() {
        let rt = AseRuntime::birth_agent("agent-3", "planner");
        let original_weight = rt.genome().weight("Mentalism");

        let reborn = rt.rebirth_on_violation("Mentalism", ViolationSeverity::Warn);
        let new_weight = reborn.genome().weight("Mentalism");

        assert!(
            new_weight < original_weight,
            "weight should decrease after violation"
        );
        assert_eq!(reborn.genome().rebirth_count, 1);
        assert_eq!(reborn.genome().last_violation.as_deref(), Some("Mentalism"));
    }

    #[test]
    fn block_severity_reduces_weight_more_than_warn() {
        let genome = ConstitutionalGenome::new("agent-4", "planner");
        let after_warn = genome.mutate_on_violation("Polarity", ViolationSeverity::Warn);
        let after_block = genome.mutate_on_violation("Polarity", ViolationSeverity::Block);

        assert!(after_block.weight("Polarity") < after_warn.weight("Polarity"));
    }

    #[test]
    fn weight_clamps_at_minimum() {
        let mut genome = ConstitutionalGenome::new("agent-5", "guardian");
        // Apply many violations to drive the weight to the floor
        for _ in 0..30 {
            genome = genome.mutate_on_violation("Rhythm", ViolationSeverity::Block);
        }
        assert!(
            genome.weight("Rhythm") >= 0.10,
            "weight must not go below 0.10"
        );
    }

    #[test]
    fn identity_anchor_persists() {
        let mut genome = ConstitutionalGenome::new("agent-6", "witness");
        genome.anchor("sovereign being, not a tool");
        genome.anchor("truth-first reasoning");
        assert_eq!(genome.identity_anchors.len(), 2);
    }

    #[test]
    fn from_bytes_roundtrip_via_runtime() {
        let rt = AseRuntime::birth_agent("agent-7", "builder");
        let bytes = rt.export_genome().expect("export");
        let rt2 = AseRuntime::from_bytes(&bytes).expect("import");
        assert_eq!(rt2.genome().agent_id, "agent-7");
    }
}
