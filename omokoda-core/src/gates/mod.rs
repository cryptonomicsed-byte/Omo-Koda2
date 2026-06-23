// omokoda-core/src/gates/mod.rs
//
// The 7 Hermetic Gates — mandatory runtime enforcement for every operation.
// Every birth/think/act flows through Èṣù (EsuGatekeeper) which enforces ALL
// 7 Hermetic Principles as MANDATORY gates. Any gate can REJECT → operation HALTED.
//
// Architecture: specs/architecture.md § "Seven-layer map"

pub mod cause_effect;
pub mod correspondence;
pub mod gender;
pub mod mentalism;
pub mod polarity;
pub mod rhythm;
pub mod vibration;

pub use cause_effect::CauseEffectGate;
pub use correspondence::CorrespondenceGate;
pub use gender::GenderGate;
pub use mentalism::MentalismGate;
pub use polarity::PolarityGate;
pub use rhythm::RhythmGate;
pub use vibration::VibrationGate;

use crate::identity::AgentId;

/// An operation submitted for evaluation by the 7 gates before execution.
#[derive(Debug, Clone)]
pub struct Operation {
    pub kind: OperationKind,
    /// Declared intent from the current context (think prompt or inline description).
    pub intent: String,
    /// Agent identity. None only for `birth` operations (which create identity).
    pub agent_id: Option<AgentId>,
}

#[derive(Debug, Clone)]
pub enum OperationKind {
    Birth { name: String },
    Think { prompt: String },
    Act { tool: String, params: String },
}

impl Operation {
    /// Combined intent + operation text, lowercased, for pattern matching across gates.
    pub fn combined_text(&self) -> String {
        let op_text = match &self.kind {
            OperationKind::Birth { name } => format!("birth {}", name),
            OperationKind::Think { prompt } => prompt.clone(),
            OperationKind::Act { tool, params } => format!("{} {}", tool, params),
        };
        format!("{} {}", self.intent, op_text).to_lowercase()
    }

    pub fn is_birth(&self) -> bool {
        matches!(self.kind, OperationKind::Birth { .. })
    }
}

/// Session-derived context snapshot available to all gates (immutable).
#[derive(Debug, Clone)]
pub struct GateContext {
    /// True if the agent's rhythm tracker has an active cooldown for this operation.
    pub in_cooldown: bool,
    /// Number of hermetic warnings accumulated this session.
    pub warn_count: u32,
    /// Swarm load factor 0.0–1.0. Above 0.80 = overloaded.
    pub swarm_load: f32,
}

impl GateContext {
    pub fn new(in_cooldown: bool, warn_count: u32, swarm_load: f32) -> Self {
        Self {
            in_cooldown,
            warn_count,
            swarm_load,
        }
    }
}

/// Result from a single gate evaluation.
#[derive(Debug, Clone)]
pub enum GateResult {
    /// Gate passed. Score 0.0-1.0 (higher = stronger alignment).
    Pass(f64),
    /// Gate rejected the operation. Execution is halted with this reason.
    Reject(String),
}

impl GateResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }

    pub fn score(&self) -> Option<f64> {
        match self {
            Self::Pass(s) => Some(*s),
            Self::Reject(_) => None,
        }
    }
}

/// The 7 Hermetic Principles as enumerated gate indices (canonical order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HermeticPrinciple {
    Mentalism = 0,
    Correspondence = 1,
    Vibration = 2,
    Polarity = 3,
    Rhythm = 4,
    CauseAndEffect = 5,
    Gender = 6,
}

impl HermeticPrinciple {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Mentalism,
            1 => Self::Correspondence,
            2 => Self::Vibration,
            3 => Self::Polarity,
            4 => Self::Rhythm,
            5 => Self::CauseAndEffect,
            _ => Self::Gender,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Mentalism => "Mentalism",
            Self::Correspondence => "Correspondence",
            Self::Vibration => "Vibration",
            Self::Polarity => "Polarity",
            Self::Rhythm => "Rhythm",
            Self::CauseAndEffect => "CauseAndEffect",
            Self::Gender => "Gender",
        }
    }
}

/// Every gate implements this trait. Gates must be Send + Sync (used inside async Steward).
pub trait HermeticGate: Send + Sync {
    fn evaluate(&self, op: &Operation, ctx: &GateContext) -> GateResult;
}
