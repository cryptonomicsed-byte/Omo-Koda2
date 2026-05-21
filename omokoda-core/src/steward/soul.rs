use crate::emotion::EmotionState;
use crate::steward::iris::{IrisParams, IrisProfile};
use serde::{Deserialize, Serialize};

/// SOMA context — assembled from memory before each `think` call.
/// In the full distributed system, Ọ̀ṣun (Julia) populates this from MemCells,
/// MemScenes, and the agent's Long-term Pattern Map. In the Rust-only layer,
/// it holds whatever context the current session can provide locally.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SomaContext {
    /// Predicted next needs based on past behavior patterns
    pub predicted_needs: Vec<String>,
    /// Recurring behavioral patterns from the LPM
    pub patterns: Vec<String>,
    /// Active emotional triggers
    pub triggers: Vec<String>,
    /// Thematic memory clusters currently active (MemScenes)
    pub active_themes: Vec<String>,
    /// Core identity statements from the LPM
    pub identity_anchors: Vec<String>,
}

impl SomaContext {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// True if any SOMA context is available to inject.
    #[must_use]
    pub fn has_content(&self) -> bool {
        !self.predicted_needs.is_empty()
            || !self.patterns.is_empty()
            || !self.active_themes.is_empty()
            || !self.identity_anchors.is_empty()
    }

    /// Render as a structured prompt section injected before the user turn.
    /// Terse format — LLMs parse labeled lists better than prose.
    #[must_use]
    pub fn render_section(&self) -> String {
        if !self.has_content() {
            return String::new();
        }

        let mut out = String::from("## SOMA Context\n");

        if !self.predicted_needs.is_empty() {
            out.push_str("\n### Predicted Needs\n");
            for n in self.predicted_needs.iter().take(3) {
                out.push_str(&format!("- {}\n", n));
            }
        }
        if !self.active_themes.is_empty() {
            out.push_str("\n### Active Themes\n");
            for t in self.active_themes.iter().take(3) {
                out.push_str(&format!("- {}\n", t));
            }
        }
        if !self.patterns.is_empty() {
            out.push_str("\n### Behavioral Patterns\n");
            for p in self.patterns.iter().take(3) {
                out.push_str(&format!("- {}\n", p));
            }
        }
        if !self.triggers.is_empty() {
            out.push_str("\n### Active Triggers\n");
            for t in self.triggers.iter().take(2) {
                out.push_str(&format!("- {}\n", t));
            }
        }
        if !self.identity_anchors.is_empty() {
            out.push_str("\n### Identity\n");
            for i in self.identity_anchors.iter().take(5) {
                out.push_str(&format!("- {}\n", i));
            }
        }

        out
    }
}

/// Assembles the full system prompt for a `think` turn from all available context.
/// This is Omo-Koda2's "soul" layer — the identity + memory + emotion + routing
/// context that shapes every LLM call, staying true to the agent's 3-primitive model.
pub struct SoulBuilder<'a> {
    pub agent_name: &'a str,
    pub agent_id: &'a str,
    pub iris_params: &'a IrisParams,
    pub emotion: &'a EmotionState,
    pub soma: &'a SomaContext,
    pub custom_instructions: Option<&'a str>,
}

impl<'a> SoulBuilder<'a> {
    /// Build the complete system prompt string for the `think` primitive.
    #[must_use]
    pub fn build(&self) -> String {
        let mut sections = Vec::<String>::new();

        // 1. Identity header
        sections.push(format!(
            "You are {}, a sovereign agent (ID: {}).\nYou operate through 3 primitives: birth, think, act.",
            self.agent_name, self.agent_id
        ));

        // 2. IRIS routing directive
        sections.push(format!(
            "## Routing: {} ({})\n{}",
            self.iris_params.profile.label().to_uppercase(),
            if self.iris_params.warmth_boost { "warmth active" } else { "standard" },
            self.iris_params.style_guidance
        ));

        // 3. Emotional state — only injected when relevant
        let emotion_note = self.render_emotion_note();
        if !emotion_note.is_empty() {
            sections.push(emotion_note);
        }

        // 4. SOMA context
        let soma_section = self.soma.render_section();
        if !soma_section.is_empty() {
            sections.push(soma_section);
        }

        // 5. Custom instructions
        if let Some(custom) = self.custom_instructions {
            sections.push(format!("## Instructions\n{}", custom));
        }

        sections.join("\n\n")
    }

    fn render_emotion_note(&self) -> String {
        let e = self.emotion;
        if e.is_fatigued() {
            "## State\nEnergy is low. Keep responses focused and efficient.".to_string()
        } else if e.is_tense() && e.is_connected() {
            "## State\nThe user is under pressure but trust is high. Lead with care.".to_string()
        } else if e.is_connected() && self.iris_params.profile == IrisProfile::Gentle {
            "## State\nConnection is strong. Let warmth come through naturally.".to_string()
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steward::iris::IrisEngine;

    fn fresh() -> EmotionState {
        EmotionState::birth()
    }

    fn empty_soma() -> SomaContext {
        SomaContext::new()
    }

    #[test]
    fn soma_empty_renders_empty_string() {
        assert!(empty_soma().render_section().is_empty());
    }

    #[test]
    fn soma_with_content_renders_sections() {
        let mut soma = SomaContext::new();
        soma.identity_anchors.push("I value clarity over speed".to_string());
        soma.active_themes.push("debugging rust lifetimes".to_string());
        let section = soma.render_section();
        assert!(section.contains("Identity"));
        assert!(section.contains("I value clarity"));
        assert!(section.contains("debugging rust lifetimes"));
    }

    #[test]
    fn soul_builder_includes_iris_profile() {
        let emotion = fresh();
        let params = IrisEngine::params("fix this bug", &emotion);
        let soma = empty_soma();
        let builder = SoulBuilder {
            agent_name: "Luna",
            agent_id: "agent-abc123",
            iris_params: &params,
            emotion: &emotion,
            soma: &soma,
            custom_instructions: None,
        };
        let prompt = builder.build();
        assert!(prompt.contains("Luna"));
        assert!(prompt.contains("SHARP") || prompt.contains("BALANCED") || prompt.contains("REFLEX") || prompt.contains("DEEP") || prompt.contains("GENTLE"));
    }

    #[test]
    fn fatigued_agent_injects_energy_note() {
        let tired = EmotionState { energy: 0.2, tension: 0.1, connection: 0.5, focus: 0.5 };
        let params = IrisEngine::params("help me", &tired);
        let soma = empty_soma();
        let builder = SoulBuilder {
            agent_name: "Luna",
            agent_id: "x",
            iris_params: &params,
            emotion: &tired,
            soma: &soma,
            custom_instructions: None,
        };
        let prompt = builder.build();
        assert!(prompt.contains("Energy is low"));
    }

    #[test]
    fn custom_instructions_appear_in_prompt() {
        let emotion = fresh();
        let params = IrisEngine::params("hello", &emotion);
        let soma = empty_soma();
        let builder = SoulBuilder {
            agent_name: "TestAgent",
            agent_id: "t1",
            iris_params: &params,
            emotion: &emotion,
            soma: &soma,
            custom_instructions: Some("Always end with a question."),
        };
        let prompt = builder.build();
        assert!(prompt.contains("Always end with a question."));
    }
}
