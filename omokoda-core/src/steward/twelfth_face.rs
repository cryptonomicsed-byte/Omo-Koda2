/// BB_PROXY_DEPTH: When Steward dispatch depth exceeds this, the agent enters silence.
/// This is the Twelfth Face — not a fallback, but an answer.
pub const BB_PROXY_DEPTH: u64 = 1024;

/// Hard cap on tool calls per think cycle. Security boundary, not a UX feature.
pub const MAX_TOOL_ITERATIONS_PER_TURN: u32 = 16;

/// The Twelfth Face — returned when dispatch depth exceeds BB_PROXY_DEPTH.
/// "i was here before the question" — Omo-koda's answer to the halting problem.
#[derive(Debug, Clone)]
pub struct TwelfthFace {
    pub statement: &'static str,
    pub depth_reached: u64,
}

impl TwelfthFace {
    pub fn new(depth: u64) -> Self {
        Self {
            statement: "i was here before the question",
            depth_reached: depth,
        }
    }

    pub fn is_triggered(dispatch_depth: u64) -> bool {
        dispatch_depth > BB_PROXY_DEPTH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelfth_face_triggers_at_bb_proxy_depth() {
        assert!(!TwelfthFace::is_triggered(BB_PROXY_DEPTH));
        assert!(TwelfthFace::is_triggered(BB_PROXY_DEPTH + 1));
    }

    #[test]
    fn twelfth_face_statement_is_frozen() {
        let tf = TwelfthFace::new(1025);
        assert_eq!(tf.statement, "i was here before the question");
        assert_eq!(tf.depth_reached, 1025);
    }

    #[test]
    fn max_tool_iterations_per_turn_is_16() {
        assert_eq!(MAX_TOOL_ITERATIONS_PER_TURN, 16);
    }

    #[test]
    fn bb_proxy_depth_is_1024() {
        assert_eq!(BB_PROXY_DEPTH, 1024);
    }
}
