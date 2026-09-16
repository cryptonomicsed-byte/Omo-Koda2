//! If-Script causal gate — bridges live agent state into the hermetic gate.
//!
//! Positioned between the permission-policy check and tool execution in
//! `execute_tool_call_for_agentic`.  The hermetic gate is a structural
//! constraint (tier×Odù×access×memory coherence); it is NOT a policy gate
//! (that's `PermissionPolicy`) and NOT a rhythm gate (that's Ọya).  Each
//! gate owns one concern.

use ifascript::{
    cosmogram::AccessClass,
    hermetic::{default_gate, GateContext},
    soul::MemoryTier,
};

pub struct CausalGateInput<'a> {
    pub tier: u8,
    pub odu_id: u8,
    pub tool_name: &'a str,
}

pub struct CausalDecision {
    pub allowed: bool,
    pub warnings: usize,
    pub denial_reason: Option<String>,
}

/// Evaluate whether the agent's (tier, Odù, tool) triple satisfies the
/// structural hermetic constraints.  Never panics — always returns a decision.
pub fn evaluate_causal_gate(input: &CausalGateInput<'_>) -> CausalDecision {
    let gate = default_gate();
    let access = access_class_for_tier(input.tier);
    let memory = memory_tier_for_tier(input.tier);
    let ctx = GateContext {
        tier: input.tier,
        odu_id: input.odu_id as u16,
        access_class: &access,
        memory_tier: &memory,
    };
    let result = gate.validate_all(&ctx);
    let denial_reason = if !result.allowed {
        Some(
            result
                .violations
                .iter()
                .filter(|v| v.should_block())
                .map(|v| v.message.clone())
                .collect::<Vec<_>>()
                .join("; "),
        )
    } else {
        None
    };
    CausalDecision {
        allowed: result.allowed,
        warnings: result.warnings,
        denial_reason,
    }
}

pub fn access_class_for_tier(tier: u8) -> AccessClass {
    match tier {
        0..=2 => AccessClass::Public,
        3..=5 => AccessClass::Sealed,
        _ => AccessClass::Council,
    }
}

pub fn memory_tier_for_tier(tier: u8) -> MemoryTier {
    match tier {
        0 => MemoryTier::Tier0Existential,
        1 => MemoryTier::Tier1Deep,
        2..=3 => MemoryTier::Tier2Operational,
        _ => MemoryTier::Tier3Contributable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier1_base_odu_allowed() {
        let decision = evaluate_causal_gate(&CausalGateInput {
            tier: 1,
            odu_id: 0,
            tool_name: "read_file",
        });
        assert!(decision.allowed, "tier-1 base Odù should pass");
    }

    #[test]
    fn tier1_odu_in_range_allowed() {
        let decision = evaluate_causal_gate(&CausalGateInput {
            tier: 1,
            odu_id: 200,
            tool_name: "think",
        });
        assert!(decision.allowed);
    }
}
