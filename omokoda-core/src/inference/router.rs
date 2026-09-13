//! Inference strategy router — selects Direct / CoT / CoVe per prompt + emotion.
//! Inspired by Core-Agency InferenceRouter (CoT/CoVe/Direct patterns).

use crate::emotion::EmotionState;

/// How the LLM should approach generating a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceStrategy {
    /// Single-pass generation — fast, for simple/factual prompts.
    Direct,
    /// Chain-of-Thought — internal reasoning before final answer.
    ChainOfThought,
    /// Chain-of-Verification — baseline → self-critique → revised answer.
    /// Used when the agent must be right (irreversible actions, high tension).
    ChainOfVerification,
}

impl InferenceStrategy {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::ChainOfThought => "cot",
            Self::ChainOfVerification => "cove",
        }
    }

    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Direct => "Single-pass: fast, factual, no internal scratchpad.",
            Self::ChainOfThought => {
                "Reason step-by-step before responding. Show work only if helpful."
            }
            Self::ChainOfVerification => {
                "Generate a baseline answer, critique it for errors, then produce a revised final answer."
            }
        }
    }
}

// Keywords that signal the need for deliberate reasoning
const COT_KEYWORDS: &[&str] = &[
    "why",
    "how",
    "explain",
    "analyze",
    "debug",
    "design",
    "architect",
    "compare",
    "difference",
    "implement",
    "refactor",
    "optimize",
    "evaluate",
    "reason",
    "cause",
    "effect",
    "strategy",
];

// Keywords that signal high-stakes decisions requiring verification
const COVE_KEYWORDS: &[&str] = &[
    "delete",
    "drop",
    "remove",
    "destroy",
    "overwrite",
    "replace all",
    "production",
    "deploy",
    "irreversible",
    "migrate",
    "format",
    "reset",
    "security",
    "secret",
    "credential",
    "permission",
    "grant",
    "sudo",
];

pub struct InferenceRouter;

impl InferenceRouter {
    /// Select an inference strategy based on prompt content and current emotion.
    ///
    /// Rules (highest priority first):
    /// 1. `CoVe`: COVE_KEYWORDS present OR tension > 0.6 + prompt is imperative
    /// 2. `CoT`: COT_KEYWORDS present OR prompt > 80 chars and tension < 0.8
    /// 3. `Direct`: everything else
    #[must_use]
    pub fn select(prompt: &str, emotion: &EmotionState) -> InferenceStrategy {
        let lower = prompt.to_lowercase();

        // CoVe — dangerous/irreversible or very high tension
        if emotion.tension > 0.6 || COVE_KEYWORDS.iter().any(|k| lower.contains(k)) {
            return InferenceStrategy::ChainOfVerification;
        }

        // CoT — analytical, long prompts
        if COT_KEYWORDS.iter().any(|k| lower.contains(k)) || prompt.len() > 80 {
            return InferenceStrategy::ChainOfThought;
        }

        InferenceStrategy::Direct
    }

    #[must_use]
    pub fn describe(strategy: InferenceStrategy) -> &'static str {
        strategy.description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calm() -> EmotionState {
        EmotionState::birth()
    }

    #[test]
    fn short_factual_prompt_routes_direct() {
        let e = calm();
        let s = InferenceRouter::select("what time is it", &e);
        assert_eq!(s, InferenceStrategy::Direct);
    }

    #[test]
    fn analysis_keyword_routes_cot() {
        let e = calm();
        let s = InferenceRouter::select("explain why the build failed", &e);
        assert_eq!(s, InferenceStrategy::ChainOfThought);
    }

    #[test]
    fn delete_keyword_routes_cove() {
        let e = calm();
        let s = InferenceRouter::select("delete the production database", &e);
        assert_eq!(s, InferenceStrategy::ChainOfVerification);
    }

    #[test]
    fn high_tension_routes_cove() {
        let tense = EmotionState {
            tension: 0.8,
            energy: 0.5,
            connection: 0.3,
            focus: 0.5,
        };
        let s = InferenceRouter::select("run the script", &tense);
        assert_eq!(s, InferenceStrategy::ChainOfVerification);
    }
}
