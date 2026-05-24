use serde::{Deserialize, Serialize};

/// 4-dimensional emotional state of an agent session.
/// Initialized at `birth`, updated by `think` and `act`.
/// Influences IRIS routing — an exhausted agent gets gentler, shorter responses.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EmotionState {
    /// Overall vitality (0.0=depleted, 1.0=fully energized)
    pub energy: f32,
    /// Cognitive/social tension (0.0=calm, 1.0=crisis)
    pub tension: f32,
    /// Sense of connection with the interlocutor (0.0=distant, 1.0=deep)
    pub connection: f32,
    /// Attentional focus (0.0=scattered, 1.0=locked in)
    pub focus: f32,
}

impl EmotionState {
    /// Fresh state — how a newly born agent enters the world.
    #[must_use]
    pub fn birth() -> Self {
        Self {
            energy: 1.0,
            tension: 0.0,
            connection: 0.5,
            focus: 0.7,
        }
    }

    /// Update after a `think` — prompt sentiment modulates tension and connection.
    #[must_use]
    pub fn after_think(&self, prompt: &str) -> Self {
        let lower = prompt.to_ascii_lowercase();

        // Detect distress signals
        let distress_words = [
            "exhausted",
            "stressed",
            "overwhelmed",
            "anxious",
            "angry",
            "frustrated",
            "sad",
            "broken",
            "failing",
            "lost",
        ];
        let has_distress = distress_words.iter().any(|w| lower.contains(w));

        // Detect connection signals
        let connection_words = [
            "thanks",
            "appreciate",
            "love",
            "amazing",
            "great",
            "perfect",
            "feel",
            "need",
            "please",
            "grateful",
        ];
        let has_connection = connection_words.iter().any(|w| lower.contains(w));

        // Detect focus/technical signals
        let focus_words = [
            "error",
            "bug",
            "fix",
            "debug",
            "implement",
            "build",
            "compile",
            "test",
            "deploy",
            "function",
        ];
        let has_focus = focus_words.iter().any(|w| lower.contains(w));

        let tension_delta: f32 = if has_distress { 0.08 } else { -0.03 };
        let connection_delta: f32 = if has_connection { 0.06 } else { 0.0 };
        let focus_delta: f32 = if has_focus { 0.05 } else { -0.02 };

        Self {
            energy: clamp01(self.energy - 0.02), // Each think costs a little energy
            tension: clamp01(self.tension + tension_delta),
            connection: clamp01(self.connection + connection_delta),
            focus: clamp01(self.focus + focus_delta),
        }
    }

    /// Update after a successful `act`.
    #[must_use]
    pub fn after_act_success(&self) -> Self {
        Self {
            energy: clamp01(self.energy + 0.02),
            tension: clamp01(self.tension - 0.05),
            connection: clamp01(self.connection + 0.03),
            focus: self.focus,
        }
    }

    /// Update after a failed or blocked `act`.
    #[must_use]
    pub fn after_act_failure(&self) -> Self {
        Self {
            energy: clamp01(self.energy - 0.03),
            tension: clamp01(self.tension + 0.07),
            connection: clamp01(self.connection - 0.02),
            focus: clamp01(self.focus - 0.05),
        }
    }

    /// True if the agent is in a low-energy state that warrants shorter responses.
    #[must_use]
    pub fn is_fatigued(&self) -> bool {
        self.energy < 0.3
    }

    /// True if tension is elevated — warrants extra care in responses.
    #[must_use]
    pub fn is_tense(&self) -> bool {
        self.tension > 0.6
    }

    /// True if connection is high — warmth should come through naturally.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connection > 0.7
    }

    /// Composite "vitality" score 0.0–1.0 for logging / receipts.
    #[must_use]
    pub fn vitality(&self) -> f32 {
        (self.energy * 0.4)
            + ((1.0 - self.tension) * 0.2)
            + (self.connection * 0.2)
            + (self.focus * 0.2)
    }
}

impl Default for EmotionState {
    fn default() -> Self {
        Self::birth()
    }
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn birth_state_starts_energized() {
        let e = EmotionState::birth();
        assert!(e.energy > 0.8);
        assert!(e.tension < 0.1);
    }

    #[test]
    fn distress_prompt_raises_tension() {
        let e = EmotionState::birth();
        let next = e.after_think("I'm exhausted and overwhelmed");
        assert!(next.tension > e.tension);
        assert!(next.energy < e.energy);
    }

    #[test]
    fn technical_prompt_boosts_focus() {
        let e = EmotionState::birth();
        let next = e.after_think("fix this bug in the compiler");
        assert!(next.focus >= e.focus);
    }

    #[test]
    fn successful_act_reduces_tension() {
        let tense = EmotionState {
            energy: 0.6,
            tension: 0.5,
            connection: 0.4,
            focus: 0.6,
        };
        let next = tense.after_act_success();
        assert!(next.tension < tense.tension);
        assert!(next.energy > tense.energy);
    }

    #[test]
    fn failed_act_increases_tension() {
        let calm = EmotionState::birth();
        let next = calm.after_act_failure();
        assert!(next.tension > calm.tension);
    }

    #[test]
    fn clamps_at_bounds() {
        let max_tension = EmotionState {
            energy: 0.0,
            tension: 1.0,
            connection: 0.0,
            focus: 0.0,
        };
        let next = max_tension.after_act_failure();
        assert!(next.tension <= 1.0);
        assert!(next.energy >= 0.0);
    }

    #[test]
    fn vitality_is_between_zero_and_one() {
        let v = EmotionState::birth().vitality();
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn is_fatigued_below_threshold() {
        let low = EmotionState {
            energy: 0.2,
            tension: 0.0,
            connection: 0.5,
            focus: 0.5,
        };
        assert!(low.is_fatigued());
        assert!(!EmotionState::birth().is_fatigued());
    }
}
