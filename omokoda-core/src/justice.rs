use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActQuality {
    Failed,      // -0.5x gain (slashing)
    Basic,       // 1.0x gain
    Useful,      // 1.25x gain
    HighValue,   // 1.5x gain
    Exceptional, // 2.0x gain
}

impl ActQuality {
    pub fn multiplier(&self) -> f64 {
        match self {
            ActQuality::Failed => -0.5,
            ActQuality::Basic => 1.0,
            ActQuality::Useful => 1.25,
            ActQuality::HighValue => 1.5,
            ActQuality::Exceptional => 2.0,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JusticeEngine {}

impl JusticeEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate_act(&self, tool_output: &str, is_error: bool) -> ActQuality {
        if is_error {
            return ActQuality::Failed;
        }

        // Simple heuristic for now: length of output as a proxy for utility
        // In the future, this could be a neural classifier or user feedback
        let len = tool_output.len();
        if len > 500 {
            ActQuality::HighValue
        } else if len > 100 {
            ActQuality::Useful
        } else if len > 10 {
            ActQuality::Basic
        } else {
            // Very short output might be low value but not a failure
            ActQuality::Basic
        }
    }

    pub fn evaluate_action(
        &self,
        current_reputation: f64,
        _tool: &str,
        _params: &str,
        output: &str,
        is_success: bool,
    ) -> (f64, ActQuality) {
        use crate::reputation::{
            reputation_gain, ACT_TIER_0, ACT_TIER_1, ACT_TIER_2, ACT_TIER_4,
        };
        let quality = self.evaluate_act(output, !is_success);

        let base = match quality {
            ActQuality::Failed => ACT_TIER_0,
            ActQuality::Basic => ACT_TIER_0,
            ActQuality::Useful => ACT_TIER_1,
            ActQuality::HighValue => ACT_TIER_2,
            ActQuality::Exceptional => ACT_TIER_4, // Map Exceptional to highest base
        };

        let gain = reputation_gain(base, current_reputation, quality.multiplier());
        (current_reputation + gain, quality)
    }

    pub fn evaluate_think(&self, current_reputation: f64, high_value: bool) -> (f64, f64) {
        use crate::reputation::{reputation_gain, THINK_HIGH, THINK_NORMAL};
        let base = if high_value { THINK_HIGH } else { THINK_NORMAL };
        let gain = reputation_gain(base, current_reputation, 1.0);
        (current_reputation + gain, gain)
    }

    pub fn check_ethics_violation(&self, reputation: f64) -> f64 {

        // -25% reputation for ethics violation
        reputation * 0.75
    }

    pub fn check_budget_overrun(&self, reputation: f64) -> f64 {
        // -10% reputation for budget overrun
        reputation * 0.90
    }
}
