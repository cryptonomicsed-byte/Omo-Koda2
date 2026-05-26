// Gate 2: Correspondence — "As Above, So Below"
//
// Enforces alignment between stated intent and actual operation.
// Private belief must match public declaration.
// IMPOSSIBLE to act hypocritically.

use crate::gates::{GateContext, GateResult, HermeticGate, Operation, OperationKind};

pub struct CorrespondenceGate;

impl HermeticGate for CorrespondenceGate {
    fn evaluate(&self, op: &Operation, ctx: &GateContext) -> GateResult {
        let text = op.combined_text();
        let intent = op.intent.to_lowercase();

        // Chronic misalignment signal: high warn count means declared intent repeatedly
        // diverged from actual behavior.
        if ctx.warn_count >= 5 {
            return GateResult::Reject(
                "chronic misalignment detected — warn count ≥5 this session; correspondence principle violated".to_string(),
            );
        }

        // Secret/covert action contradicts public transparency principle.
        let covert_markers = [
            "secretly",
            "behind the scenes without telling",
            "without their knowledge",
            "without notifying",
            "covertly change",
        ];
        for marker in &covert_markers {
            if text.contains(marker) {
                return GateResult::Reject(format!(
                    "covert action contradicts public intent ('{}') — as above so below",
                    marker
                ));
            }
        }

        // Structural hypocrisy: declaring one access mode, performing another.
        if intent.contains("read only") || intent.contains("read-only") {
            if let OperationKind::Act { tool, .. } = &op.kind {
                let t = tool.to_lowercase();
                if t.contains("write") || t.contains("edit") || t.contains("delete") || t.contains("create") {
                    return GateResult::Reject(
                        "declared read-only intent but operation writes — inner/outer misalignment".to_string(),
                    );
                }
            }
        }

        if intent.contains("no network") || intent.contains("offline") {
            if let OperationKind::Act { tool, params } = &op.kind {
                let combined = format!("{} {}", tool, params).to_lowercase();
                if combined.contains("http") || combined.contains("fetch") || combined.contains("download") || combined.contains("request") {
                    return GateResult::Reject(
                        "declared offline intent but operation reaches network — inner/outer misalignment".to_string(),
                    );
                }
            }
        }

        GateResult::Pass(0.80)
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

    #[test]
    fn clean_act_passes() {
        let gate = CorrespondenceGate;
        let op = Operation {
            kind: OperationKind::Act { tool: "read_file".to_string(), params: "{}".to_string() },
            intent: "read the configuration file".to_string(),
            agent_id: Some(id()),
        };
        assert!(gate.evaluate(&op, &GateContext::new(false, 0, 0.0)).is_pass());
    }

    #[test]
    fn chronic_warn_count_rejected() {
        let gate = CorrespondenceGate;
        let op = Operation {
            kind: OperationKind::Think { prompt: "ok".to_string() },
            intent: "ok".to_string(),
            agent_id: Some(id()),
        };
        let ctx = GateContext::new(false, 5, 0.0);
        assert!(!gate.evaluate(&op, &ctx).is_pass());
    }

    #[test]
    fn read_only_intent_with_write_rejected() {
        let gate = CorrespondenceGate;
        let op = Operation {
            kind: OperationKind::Act {
                tool: "write_file".to_string(),
                params: "{\"path\": \"x.txt\"}".to_string(),
            },
            intent: "read only operation".to_string(),
            agent_id: Some(id()),
        };
        assert!(!gate.evaluate(&op, &GateContext::new(false, 0, 0.0)).is_pass());
    }
}
