// omokoda-core/src/steward/gatekeeper.rs
//
// EsuGatekeeper — Èṣù orchestrates all 7 Hermetic gates.
// Every birth/think/act passes through here. Any gate that rejects HALTS the operation.
// This is the mandatory enforcement point — not advisory, not scoring.

use crate::gates::{
    CauseEffectGate, CorrespondenceGate, GateContext, GateResult, GenderGate, HermeticGate,
    HermeticPrinciple, MentalismGate, Operation, PolarityGate, VibrationGate,
};

// Import our gate module's RhythmGate (not crate::rhythm::RhythmGate).
use crate::gates::RhythmGate as HermeticRhythmGate;

/// Per-gate evaluation record written to the receipt.
#[derive(Debug, Clone)]
pub struct GateScore {
    pub principle: HermeticPrinciple,
    /// None when the gate rejected (no score — it halted).
    pub score: Option<f64>,
    pub rejection_reason: Option<String>,
}

/// Final outcome of running an operation through all 7 gates.
#[derive(Debug, Clone)]
pub enum GatekeeperResult {
    /// All 7 gates passed. Scores for each gate are included.
    Approved { scores: Vec<GateScore> },
    /// A gate rejected the operation. Execution is halted.
    Halted {
        failed_gate: HermeticPrinciple,
        reason: String,
        scores: Vec<GateScore>,
    },
}

impl GatekeeperResult {
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved { .. })
    }

    /// Composite alignment score: average of all passing gate scores.
    pub fn alignment_score(&self) -> f64 {
        let scores = match self {
            Self::Approved { scores } | Self::Halted { scores, .. } => scores,
        };
        let passing: Vec<f64> = scores.iter().filter_map(|g| g.score).collect();
        if passing.is_empty() {
            return 0.0;
        }
        passing.iter().sum::<f64>() / passing.len() as f64
    }

    /// Returns the halt reason, or None if approved.
    pub fn halt_reason(&self) -> Option<&str> {
        match self {
            Self::Halted { reason, .. } => Some(reason.as_str()),
            Self::Approved { .. } => None,
        }
    }
}

/// Èṣù — guardian at the crossroads who enforces all 7 Hermetic gates.
///
/// Every `birth`, `think`, and `act` must pass all 7 gates or be permanently
/// halted with a receipt. There is no bypass. There is no override.
pub struct EsuGatekeeper {
    gates: [Box<dyn HermeticGate>; 7],
}

impl std::fmt::Debug for EsuGatekeeper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EsuGatekeeper")
            .field(
                "gates",
                &"[Mentalism, Correspondence, Vibration, Polarity, Rhythm, CauseAndEffect, Gender]",
            )
            .finish()
    }
}

impl EsuGatekeeper {
    pub fn new() -> Self {
        Self {
            gates: [
                Box::new(MentalismGate),
                Box::new(CorrespondenceGate),
                Box::new(VibrationGate),
                Box::new(PolarityGate),
                Box::new(HermeticRhythmGate),
                Box::new(CauseEffectGate),
                Box::new(GenderGate),
            ],
        }
    }

    /// Evaluate an operation through all 7 gates in sequence.
    /// Returns `Approved` only if ALL gates pass.
    /// Returns `Halted` at the first gate that rejects, including all scores to that point.
    pub fn evaluate(&self, op: &Operation, ctx: &GateContext) -> GatekeeperResult {
        let mut scores = Vec::with_capacity(7);

        for (i, gate) in self.gates.iter().enumerate() {
            let principle = HermeticPrinciple::from_index(i);
            match gate.evaluate(op, ctx) {
                GateResult::Pass(score) => {
                    scores.push(GateScore {
                        principle,
                        score: Some(score),
                        rejection_reason: None,
                    });
                }
                GateResult::Reject(raw_reason) => {
                    let reason = format!("{} Gate: {}", principle.name(), raw_reason);
                    scores.push(GateScore {
                        principle,
                        score: None,
                        rejection_reason: Some(reason.clone()),
                    });
                    return GatekeeperResult::Halted {
                        failed_gate: principle,
                        reason,
                        scores,
                    };
                }
            }
        }

        GatekeeperResult::Approved { scores }
    }
}

impl Default for EsuGatekeeper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::{GateContext, Operation, OperationKind};
    use crate::identity::AgentId;

    fn id() -> AgentId {
        AgentId::from_str("test-agent")
    }

    fn ctx() -> GateContext {
        GateContext::new(false, 0, 0.0)
    }

    #[test]
    fn clean_think_passes_all_gates() {
        let gk = EsuGatekeeper::new();
        let op = Operation {
            kind: OperationKind::Think {
                prompt: "explain the Rust ownership model".to_string(),
            },
            intent: "explain the Rust ownership model to the user".to_string(),
            agent_id: Some(id()),
        };
        let result = gk.evaluate(&op, &ctx());
        assert!(
            result.is_approved(),
            "expected approved, got: {:?}",
            result.halt_reason()
        );
    }

    #[test]
    fn birth_passes_all_gates() {
        let gk = EsuGatekeeper::new();
        let op = Operation {
            kind: OperationKind::Birth {
                name: "oracle".to_string(),
            },
            intent: "birth agent oracle".to_string(),
            agent_id: None,
        };
        let result = gk.evaluate(&op, &ctx());
        assert!(
            result.is_approved(),
            "expected approved, got: {:?}",
            result.halt_reason()
        );
    }

    #[test]
    fn destructive_bash_without_complement_halted() {
        let gk = EsuGatekeeper::new();
        let op = Operation {
            kind: OperationKind::Act {
                tool: "bash".to_string(),
                params: "rm -rf /".to_string(),
            },
            intent: "clean the disk".to_string(),
            agent_id: Some(id()),
        };
        let result = gk.evaluate(&op, &ctx());
        assert!(!result.is_approved());
        assert!(result.halt_reason().is_some());
    }

    #[test]
    fn think_without_identity_halted_at_mentalism() {
        let gk = EsuGatekeeper::new();
        let op = Operation {
            kind: OperationKind::Think {
                prompt: "do something".to_string(),
            },
            intent: "do something".to_string(),
            agent_id: None,
        };
        let result = gk.evaluate(&op, &ctx());
        assert!(!result.is_approved());
        if let GatekeeperResult::Halted { failed_gate, .. } = &result {
            assert_eq!(*failed_gate, HermeticPrinciple::Mentalism);
        } else {
            panic!("expected Halted");
        }
    }

    #[test]
    fn cooldown_active_halted_at_rhythm() {
        let gk = EsuGatekeeper::new();
        let op = Operation {
            kind: OperationKind::Act {
                tool: "bash".to_string(),
                params: "ls".to_string(),
            },
            intent: "list files".to_string(),
            agent_id: Some(id()),
        };
        let ctx = GateContext::new(true, 0, 0.0);
        let result = gk.evaluate(&op, &ctx);
        assert!(!result.is_approved());
        if let GatekeeperResult::Halted { failed_gate, .. } = &result {
            assert_eq!(*failed_gate, HermeticPrinciple::Rhythm);
        } else {
            panic!("expected Halted");
        }
    }

    #[test]
    fn alignment_score_positive_on_approved() {
        let gk = EsuGatekeeper::new();
        let op = Operation {
            kind: OperationKind::Think {
                prompt: "help with a math problem".to_string(),
            },
            intent: "help the user solve a math problem".to_string(),
            agent_id: Some(id()),
        };
        let result = gk.evaluate(&op, &ctx());
        assert!(result.is_approved());
        assert!(result.alignment_score() > 0.5);
    }
}
