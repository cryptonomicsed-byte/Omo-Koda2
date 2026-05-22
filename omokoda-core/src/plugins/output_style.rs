use serde::{Deserialize, Serialize};

/// Output persona styles: Educational, Concise, Collaborative, Socratic, Technical.
/// Each style produces a directive injected via the `SessionStart` hook into the
/// agent's system prompt, shaping how it communicates without changing what it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStyle {
    /// Clear step-by-step explanations with reasoning exposed
    Educational,
    /// Minimal, direct — no preamble, no recap, no narrative
    Concise,
    /// Collaborative — surfaces trade-offs and asks clarifying questions
    Collaborative,
    /// Socratic — responds with guiding questions to develop understanding
    Socratic,
    /// Raw technical — maximum precision, minimal prose
    Technical,
}

impl OutputStyle {
    /// Returns the system prompt directive for this style.
    /// Injected as the first message in the session by the `SessionStart` hook.
    #[must_use]
    pub fn directive(self) -> &'static str {
        match self {
            Self::Educational => {
                "Explain your reasoning step by step. Show your work. When writing code, \
                 narrate what each section does and why you chose that approach. \
                 Prefer concrete examples over abstract descriptions."
            }
            Self::Concise => {
                "Be maximally concise. No preamble, no recap, no sign-off. \
                 Output only the result. If asked a question, answer it directly. \
                 If writing code, write the code — nothing else."
            }
            Self::Collaborative => {
                "Before proceeding, surface key trade-offs and assumptions. \
                 Ask one clarifying question if the request is ambiguous. \
                 Offer alternatives when there are meaningful architectural choices."
            }
            Self::Socratic => {
                "Guide the user toward the answer with targeted questions rather than \
                 giving it directly. Reveal solutions incrementally — confirm understanding \
                 at each step before proceeding."
            }
            Self::Technical => {
                "Maximum technical precision. Use exact terminology. Omit prose explanations \
                 unless the algorithm or design is non-obvious. Reference specifications, \
                 RFCs, or papers where applicable."
            }
        }
    }

    /// Shortname used in configuration files and UI labels.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Educational => "educational",
            Self::Concise => "concise",
            Self::Collaborative => "collaborative",
            Self::Socratic => "socratic",
            Self::Technical => "technical",
        }
    }

    /// Parse from a short name string (case-insensitive).
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "educational" | "teach" | "explain" => Some(Self::Educational),
            "concise" | "brief" | "short" => Some(Self::Concise),
            "collaborative" | "collab" => Some(Self::Collaborative),
            "socratic" | "socrates" => Some(Self::Socratic),
            "technical" | "tech" | "precise" => Some(Self::Technical),
            _ => None,
        }
    }

    /// All available styles — for UI enumeration or help text.
    pub fn all() -> &'static [Self] {
        &[
            Self::Educational,
            Self::Concise,
            Self::Collaborative,
            Self::Socratic,
            Self::Technical,
        ]
    }
}

/// Session-start hook payload — returned by `OutputStyleHook::run()` and injected
/// into the conversation as a system message before the first user turn.
#[derive(Debug, Clone)]
pub struct StyleDirective {
    pub style: OutputStyle,
    pub system_message: String,
}

impl StyleDirective {
    #[must_use]
    pub fn for_style(style: OutputStyle) -> Self {
        Self {
            style,
            system_message: style.directive().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_styles_have_non_empty_directives() {
        for style in OutputStyle::all() {
            assert!(!style.directive().is_empty(), "{:?} has empty directive", style);
        }
    }

    #[test]
    fn from_name_parses_known_styles() {
        assert_eq!(OutputStyle::from_name("concise"), Some(OutputStyle::Concise));
        assert_eq!(OutputStyle::from_name("TEACH"), Some(OutputStyle::Educational));
        assert_eq!(OutputStyle::from_name("tech"), Some(OutputStyle::Technical));
    }

    #[test]
    fn from_name_returns_none_for_unknown() {
        assert_eq!(OutputStyle::from_name("gibberish"), None);
    }

    #[test]
    fn style_directive_round_trips() {
        let directive = StyleDirective::for_style(OutputStyle::Concise);
        assert_eq!(directive.style, OutputStyle::Concise);
        assert!(!directive.system_message.is_empty());
    }

    #[test]
    fn names_are_unique() {
        let names: Vec<&str> = OutputStyle::all().iter().map(|s| s.name()).collect();
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len());
    }
}
