use crate::emotion::EmotionState;
use serde::{Deserialize, Serialize};

/// IRIS routing profile — determines how the `think` primitive executes.
/// Each profile sets LLM temperature, token budget, and response style guidance.
/// Routing profiles govern how the think primitive executes; IRIS only operates during `think`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IrisProfile {
    /// Short, fast, reflexive — for greetings, one-word answers, simple lookups
    Reflex,
    /// Default — balanced reasoning for most requests
    Balanced,
    /// Technical precision — code, debugging, architecture, precise analysis
    Sharp,
    /// Deep reasoning — complex problems, creative synthesis, long-form planning
    Deep,
    /// Empathetic, warm — for distress signals, emotional context, personal topics
    Gentle,
}

impl IrisProfile {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Reflex => "reflex",
            Self::Balanced => "balanced",
            Self::Sharp => "sharp",
            Self::Deep => "deep",
            Self::Gentle => "gentle",
        }
    }
}

/// LLM execution parameters derived from an IRIS profile.
#[derive(Debug, Clone, PartialEq)]
pub struct IrisParams {
    pub profile: IrisProfile,
    /// Sampling temperature (lower = more deterministic)
    pub temperature: f32,
    /// Maximum token budget for the LLM response
    pub max_tokens: u32,
    /// Style injection injected into the system prompt
    pub style_guidance: &'static str,
    /// Whether to add warmth-boosting language to the system prompt
    pub warmth_boost: bool,
}

impl IrisParams {
    fn for_profile(profile: IrisProfile, warmth: bool) -> Self {
        let (temp, tokens, style) = match profile {
            IrisProfile::Reflex => (0.3, 256, "Be extremely concise. One sentence max."),
            IrisProfile::Balanced => (0.7, 1024, "Balance clarity and depth."),
            IrisProfile::Sharp => (0.2, 2048, "Maximum technical precision. Show your reasoning."),
            IrisProfile::Deep => (0.8, 4096, "Think deeply. Explore all angles before concluding."),
            IrisProfile::Gentle => (0.9, 1024, "Lead with empathy. Be warm, patient, and supportive."),
        };
        Self {
            profile,
            temperature: temp,
            max_tokens: tokens,
            style_guidance: style,
            warmth_boost: warmth,
        }
    }
}

/// IRIS routing engine — determines the optimal `think` execution profile
/// from prompt content and current emotional state.
///
/// Priority order:
/// 1. Fatigue override — low energy forces Balanced (never Deep when drained)
/// 2. Distress/emotional signals → Gentle
/// 3. Technical signals → Sharp
/// 4. Prompt length heuristic → Deep if > 200 chars
/// 5. Short prompt → Reflex
/// 6. Default → Balanced
pub struct IrisEngine;

impl IrisEngine {
    /// Select the routing profile for this `think` invocation.
    #[must_use]
    pub fn route(prompt: &str, emotion: &EmotionState) -> IrisProfile {
        let lower = prompt.to_ascii_lowercase();

        // Energy override — a fatigued agent can't go deep
        if emotion.is_fatigued() {
            if emotion.is_tense() {
                return IrisProfile::Gentle;
            }
            return IrisProfile::Balanced;
        }

        // Distress signals → Gentle
        let distress = ["exhausted", "stressed", "overwhelmed", "sad", "anxious",
                        "hurt", "scared", "alone", "desperate", "hopeless", "crying",
                        "worried", "afraid", "confused about feelings"];
        if distress.iter().any(|d| lower.contains(d)) {
            return IrisProfile::Gentle;
        }

        // Emotional tension override → Gentle even if prompt looks technical
        if emotion.is_tense() && emotion.connection > 0.5 {
            return IrisProfile::Gentle;
        }

        // Technical/code signals → Sharp
        let technical = ["error", "bug", "fix", "debug", "implement", "compile",
                         "build", "test", "deploy", "function", "struct", "algorithm",
                         "optimize", "refactor", "architecture", "api", "database",
                         "exception", "panic", "crash", "stack trace", "lint"];
        if technical.iter().any(|t| lower.contains(t)) {
            return IrisProfile::Sharp;
        }

        // Long, complex prompts → Deep
        if prompt.len() > 200 {
            return IrisProfile::Deep;
        }

        // Very short prompts → Reflex
        if prompt.len() < 20 {
            return IrisProfile::Reflex;
        }

        IrisProfile::Balanced
    }

    /// Get execution parameters for a routed profile, incorporating emotion state.
    #[must_use]
    pub fn params(prompt: &str, emotion: &EmotionState) -> IrisParams {
        let profile = Self::route(prompt, emotion);
        IrisParams::for_profile(profile, emotion.is_connected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> EmotionState {
        EmotionState::birth()
    }

    #[test]
    fn short_prompt_routes_to_reflex() {
        assert_eq!(IrisEngine::route("hi", &fresh()), IrisProfile::Reflex);
    }

    #[test]
    fn technical_prompt_routes_to_sharp() {
        assert_eq!(
            IrisEngine::route("fix this bug in the compiler", &fresh()),
            IrisProfile::Sharp
        );
    }

    #[test]
    fn distress_prompt_routes_to_gentle() {
        assert_eq!(
            IrisEngine::route("I'm exhausted and overwhelmed", &fresh()),
            IrisProfile::Gentle
        );
    }

    #[test]
    fn long_prompt_routes_to_deep() {
        let long = "a".repeat(201);
        assert_eq!(IrisEngine::route(&long, &fresh()), IrisProfile::Deep);
    }

    #[test]
    fn moderate_prompt_routes_to_balanced() {
        assert_eq!(
            IrisEngine::route("what should I have for dinner tonight", &fresh()),
            IrisProfile::Balanced
        );
    }

    #[test]
    fn fatigued_agent_overrides_to_balanced() {
        let tired = EmotionState {
            energy: 0.2,
            tension: 0.1,
            connection: 0.5,
            focus: 0.4,
        };
        // Even a technical prompt gets downgraded when energy is low
        let profile = IrisEngine::route("implement a B-tree in Rust", &tired);
        assert_eq!(profile, IrisProfile::Balanced);
    }

    #[test]
    fn fatigued_and_tense_routes_to_gentle() {
        let stressed = EmotionState {
            energy: 0.2,
            tension: 0.8,
            connection: 0.5,
            focus: 0.3,
        };
        assert_eq!(IrisEngine::route("I need help", &stressed), IrisProfile::Gentle);
    }

    #[test]
    fn params_has_warmth_when_connected() {
        let connected = EmotionState {
            energy: 0.9,
            tension: 0.1,
            connection: 0.85,
            focus: 0.7,
        };
        let params = IrisEngine::params("tell me a story", &connected);
        assert!(params.warmth_boost);
    }

    #[test]
    fn sharp_profile_has_low_temperature() {
        let p = IrisParams::for_profile(IrisProfile::Sharp, false);
        assert!(p.temperature < 0.5);
        assert!(p.max_tokens >= 2048);
    }

    #[test]
    fn gentle_profile_has_high_temperature() {
        let p = IrisParams::for_profile(IrisProfile::Gentle, false);
        assert!(p.temperature > 0.7);
    }
}
