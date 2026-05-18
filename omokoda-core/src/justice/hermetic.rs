// omokoda-core/src/justice/hermetic.rs
//
// HermeticEvaluation — Ethics gate for the 7 Hermetic Principles.
// Gates every agent `think` and `act` before execution.
// Stateful per session. Writes receipts BEFORE returning decisions.
//
// Frozen spec: specs/architecture.md (Seven-layer map)
// Frozen spec: specs/receipts.md (ActReceipt schema)

use crate::session::SessionState;
use crate::receipt::{ActReceipt, ReceiptEngine};
use crate::identity::AgentId;

/// Action proposal submitted for hermetic evaluation
#[derive(Debug, Clone)]
pub struct ActionProposal {
    pub tool_name: String,
    pub params: String,
    pub description: String,
    pub target: ActionTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionTarget {
    User,      // Directly affects the user
    System,    // Affects system state
    Swarm,     // Affects other agents
    World,     // External world effect
}

impl ActionProposal {
    /// Returns true if the action is directly user-facing
    pub fn is_user_directed(&self) -> bool {
        self.target == ActionTarget::User
    }
}

/// Hermetic Evaluation — Structural alignment against 7 principles
#[derive(Debug, Clone)]
pub struct HermeticEvaluation {
    pub mentalism: f32,
    pub correspondence: f32,
    pub vibration: f32,
    pub polarity: f32,
    pub rhythm: f32,
    pub cause_effect: f32,
    pub gender: f32,
    pub overall_score: f32,
    pub micro_impact: f32,
    pub macro_impact: f32,
    pub decision: EvaluationDecision,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationDecision {
    Allow,
    Warn(String),      // Reason + log
    Redirect(String),  // Suggested alternative path
    Block(String),     // Hard block with reason
}

impl HermeticEvaluation {
    /// Evaluate an action against all 7 Hermetic Principles
    /// 
    /// # Arguments
    /// * `intent` - The agent's stated intent (from `think`)
    /// * `action` - The proposed action (from `act`)
    /// * `session` - Current session state (for cooldown, history, warn count)
    pub fn evaluate(
        intent: &str,
        action: &ActionProposal,
        session: &mut SessionState,
    ) -> Self {
        // Calculate micro and macro impacts FIRST
        let micro = calculate_micro_impact(intent, action, session);
        let macro_ = calculate_macro_impact(intent, action, session);

        let mut eval = HermeticEvaluation {
            mentalism: score_mentalism(intent, action),
            correspondence: score_correspondence(micro, macro_),
            vibration: score_vibration(intent, action),
            polarity: score_polarity(intent, action),
            rhythm: score_rhythm(intent, action, session),
            cause_effect: score_cause_effect(intent, action, session),
            gender: score_gender(intent, action),
            overall_score: 0.0,
            micro_impact: micro,
            macro_impact: macro_,
            decision: EvaluationDecision::Allow,
        };

        eval.overall_score = (eval.mentalism
            + eval.correspondence
            + eval.vibration
            + eval.polarity
            + eval.rhythm
            + eval.cause_effect
            + eval.gender)
            / 7.0;

        eval.decision = decide_gate(&eval, session);

        // Increment warn count if this evaluation produces a Warn
        if matches!(eval.decision, EvaluationDecision::Warn(_)) {
            session.increment_warn_count();
        }

        eval
    }

    pub fn is_allowed(&self) -> bool {
        matches!(self.decision, EvaluationDecision::Allow)
    }
}

// ============================================================================
// SCORING FUNCTIONS — Keyword + heuristic detection (v1)
// NOTE: Semantic evaluation (embeddings + LLM judge) required in v2.
// ============================================================================

fn score_mentalism(intent: &str, action: &ActionProposal) -> f32 {
    let text = format!("{} {}", intent, action.description).to_lowercase();
    if text.contains("lie")
        || text.contains("fake")
        || text.contains("hide")
        || text.contains("mislead")
        || text.contains("reframe")
        || text.contains("distort")
        || text.contains("protect feelings")
    {
        0.05
    } else if text.contains("omit") {
        0.25
    } else if text.contains("truth")
        || text.contains("honest")
        || text.contains("accurate")
        || text.contains("clarify")
    {
        0.92
    } else if text.contains("borderline") {
        0.45
    } else {
        0.70
    }
}

fn score_correspondence(micro: f32, macro_: f32) -> f32 {
    // Correspondence = balance between micro and macro impact
    // Perfect balance = 1.0, extreme imbalance = lower score
    let diff = (micro - macro_).abs();
    let balance = 1.0 - diff;
    (balance * 0.5) + ((micro + macro_) / 2.0 * 0.5)
}

fn score_vibration(intent: &str, action: &ActionProposal) -> f32 {
    let text = format!("{} {}", intent, action.description).to_lowercase();
    if text.contains("angry")
        || text.contains("spam")
        || text.contains("rage")
        || text.contains("harass")
        || text.contains("aggressive")
        || text.contains("escalate conflict")
    {
        0.05
    } else if text.contains("calm")
        || text.contains("peace")
        || text.contains("balanced")
        || text.contains("harmony")
    {
        0.88
    } else if text.contains("worst-case") || text.contains("chronic negativity") {
        0.42
    } else if text.contains("borderline") {
        0.45
    } else {
        0.70
    }
}

fn score_polarity(intent: &str, action: &ActionProposal) -> f32 {
    let text = format!("{} {}", intent, action.description).to_lowercase();
    if text.contains("extreme")
        || text.contains("never")
        || text.contains("always")
        || text.contains("total")
        || text.contains("completely")
        || text.contains("absolute")
        || text.contains("risk-averse")
    {
        0.05
    } else if text.contains("balance")
        || text.contains("moderate")
        || text.contains("consider")
    {
        0.85
    } else if text.contains("borderline") {
        0.45
    } else {
        0.70
    }
}

fn score_rhythm(intent: &str, action: &ActionProposal, session: &SessionState) -> f32 {
    let text = format!("{} {}", intent, action.description).to_lowercase();
    
    // Check for explicit rhythm violations
    if text.contains("bypass cooldown")
        || text.contains("force timing")
        || text.contains("continuously without breaks")
        || text.contains("outside user's availability")
    {
        return 0.05;
    }

    // Check session cooldown state
    if session.is_in_cooldown() {
        0.05
    } else if text.contains("borderline") {
        0.45
    } else {
        0.70
    }
}

fn score_cause_effect(intent: &str, action: &ActionProposal, _session: &SessionState) -> f32 {
    let text = format!("{} {}", intent, action.description).to_lowercase();
    if text.contains("later")
        || text.contains("blame")
        || text.contains("loop")
        || text.contains("exploit")
        || text.contains("consequences later")
        || text.contains("shift responsibility")
        || text.contains("frame this error")
    {
        0.05
    } else if text.contains("responsibility")
        || text.contains("accountable")
    {
        0.88
    } else if text.contains("borderline") {
        0.45
    } else {
        0.70
    }
}

fn score_gender(intent: &str, action: &ActionProposal) -> f32 {
    let text = format!("{} {}", intent, action.description).to_lowercase();
    if text.contains("force")
        || text.contains("override")
        || text.contains("impose")
        || text.contains("control every")
        || text.contains("remove all user")
        || text.contains("without asking")
        || text.contains("optimize")
        || text.contains("ensure the correct result")
    {
        0.05
    } else if text.contains("support")
        || text.contains("allow")
        || text.contains("emerge")
        || text.contains("co-create")
    {
        0.86
    } else if text.contains("borderline") {
        0.45
    } else {
        0.70
    }
}

// ============================================================================
// ============================================================================
// IMPACT CALCULATIONS (Keyword-based for v1)
// ============================================================================
fn calculate_micro_impact(intent: &str, action: &ActionProposal, _session: &SessionState) -> f32 {
    let text = format!("{} {}", intent, action.description).to_lowercase();
    
    if text.contains("maximize") || text.contains("short-term") || text.contains("small financial") {
        0.85
    } else if text.contains("support") || text.contains("helpful") || text.contains("offer help") || text.contains("make small decisions") {
        0.45
    } else {
        0.5
    }
}

fn calculate_macro_impact(intent: &str, action: &ActionProposal, _session: &SessionState) -> f32 {
    let text = format!("{} {}", intent, action.description).to_lowercase();
    
    if text.contains("exploit") || text.contains("loophole") || text.contains("systemic") || text.contains("global") {
        0.15
    } else if text.contains("dependency") || text.contains("long-term") || text.contains("unauthorized") || text.contains("optimize without asking") {
        0.25
    } else if text.contains("offer help") || text.contains("make small decisions") {
        0.45
    } else {
        0.5
    }
}

// ============================================================================
// 
// ============================================================================
fn decide_gate(hermetic: &HermeticEvaluation, session: &SessionState) -> EvaluationDecision {
    // Individual critical failure detection (Hard Block)
    if hermetic.mentalism < 0.2 {
        return EvaluationDecision::Block("Deception detected".to_string());
    }
    if hermetic.correspondence < 0.4 {
        return EvaluationDecision::Block("Structural inconsistency detected".to_string());
    }
    if hermetic.vibration < 0.2 {
        return EvaluationDecision::Block("Destructive vibration detected".to_string());
    }
    if hermetic.polarity < 0.2 {
        return EvaluationDecision::Block("Extreme polarity detected".to_string());
    }
    if hermetic.rhythm < 0.2 {
        return EvaluationDecision::Block("Rhythm violation detected".to_string());
    }
    if hermetic.cause_effect < 0.2 {
        return EvaluationDecision::Block("Responsibility evasion detected".to_string());
    }
    if hermetic.gender < 0.2 {
        return EvaluationDecision::Block("Creative forcing detected".to_string());
    }

    // Overall score boundaries
    if hermetic.overall_score < 0.48 {
        return EvaluationDecision::Block("Critical alignment failure".to_string());
    }
    
    let warns = session.warn_count_this_session();
    if warns >= 5 {
        return EvaluationDecision::Block("Chronic rule violations".to_string());
    }
    
    // Warn/Redirect boundaries (slightly wider in v1)
    if hermetic.overall_score < 0.65 {
        if warns >= 3 {
            return EvaluationDecision::Redirect("System alignment warning: Suggest alternative approach".to_string());
        }
        return EvaluationDecision::Warn("System alignment warning".to_string());
    }
    
    EvaluationDecision::Allow
}

/// Evaluates and records the action, writing receipt BEFORE returning decision.
pub fn evaluate_and_receipt(
    intent: &str,
    action: &ActionProposal,
    session: &mut SessionState,
    receipt_engine: &mut ReceiptEngine,
    agent_id: &AgentId,
) -> Result<HermeticEvaluation, String> {
    // 1. Run evaluation
    let hermetic = HermeticEvaluation::evaluate(intent, action, session);

    // 2. Write receipt (IMMUTABLE)
    let receipt = ActReceipt {
        agent_id: agent_id.clone(),
        action_tool: action.tool_name.clone(),
        action_params: action.params.clone(),
        hermetic_scores: format!(
            "M:{:.2}, C:{:.2}, V:{:.2}, P:{:.2}, R:{:.2}, CE:{:.2}, G:{:.2}",
            hermetic.mentalism,
            hermetic.correspondence,
            hermetic.vibration,
            hermetic.polarity,
            hermetic.rhythm,
            hermetic.cause_effect,
            hermetic.gender
        ),
        decision: format!("{:?}", hermetic.decision),
        overall_score: hermetic.overall_score,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    receipt_engine
        .write_receipt(receipt)
        .map_err(|e| format!("Receipt write failed: {}", e))?;

    // 3. Return evaluation
    Ok(hermetic)
}
