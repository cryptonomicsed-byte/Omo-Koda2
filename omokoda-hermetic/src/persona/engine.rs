use serde::{Deserialize, Serialize};

/// 8 language-specific reasoning personas from NarratorIDE.
/// Maps to the 11 Òrìṣà archetypes in the Wisdom module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LanguagePersona {
    RustEngineer,
    GoPragmatist,
    PythonCreative,
    HaskellLogician,
    JavaArchitect,
    JavaScriptAlchemist,
    ElixirFlowMind,
    MoveGuardian,
}

impl LanguagePersona {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RustEngineer => "rust_engineer",
            Self::GoPragmatist => "go_pragmatist",
            Self::PythonCreative => "python_creative",
            Self::HaskellLogician => "haskell_logician",
            Self::JavaArchitect => "java_architect",
            Self::JavaScriptAlchemist => "js_alchemist",
            Self::ElixirFlowMind => "elixir_flow_mind",
            Self::MoveGuardian => "move_guardian",
        }
    }

    pub fn all() -> [LanguagePersona; 8] {
        [
            Self::RustEngineer,
            Self::GoPragmatist,
            Self::PythonCreative,
            Self::HaskellLogician,
            Self::JavaArchitect,
            Self::JavaScriptAlchemist,
            Self::ElixirFlowMind,
            Self::MoveGuardian,
        ]
    }
}

/// Resolved persona profile with behavioral parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaProfile {
    pub persona: LanguagePersona,
    pub verbosity: f64,
    pub formality: f64,
    pub precision: f64,
}

impl PersonaProfile {
    pub fn for_persona(persona: LanguagePersona) -> Self {
        let (verbosity, formality, precision) = match &persona {
            LanguagePersona::RustEngineer => (0.7, 0.8, 1.0),
            LanguagePersona::GoPragmatist => (0.4, 0.6, 0.8),
            LanguagePersona::PythonCreative => (0.6, 0.3, 0.6),
            LanguagePersona::HaskellLogician => (0.9, 0.9, 1.0),
            LanguagePersona::JavaArchitect => (0.8, 0.9, 0.9),
            LanguagePersona::JavaScriptAlchemist => (0.5, 0.3, 0.5),
            LanguagePersona::ElixirFlowMind => (0.6, 0.6, 0.8),
            LanguagePersona::MoveGuardian => (0.7, 0.8, 1.0),
        };
        Self {
            persona,
            verbosity,
            formality,
            precision,
        }
    }
}

/// Persona engine — selects persona based on agent tier.
pub struct PersonaEngine;

impl PersonaEngine {
    pub fn select_for_tier(tier: u8) -> LanguagePersona {
        match tier {
            0 | 1 => LanguagePersona::PythonCreative,
            2 => LanguagePersona::GoPragmatist,
            3 => LanguagePersona::ElixirFlowMind,
            4 => LanguagePersona::RustEngineer,
            _ => LanguagePersona::MoveGuardian,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_personas_returns_eight() {
        assert_eq!(LanguagePersona::all().len(), 8);
    }

    #[test]
    fn all_personas_have_string_repr() {
        for p in LanguagePersona::all() {
            assert!(!p.as_str().is_empty());
        }
    }

    #[test]
    fn rust_engineer_has_max_precision() {
        let profile = PersonaProfile::for_persona(LanguagePersona::RustEngineer);
        assert!((profile.precision - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sovereign_tier_gets_move_guardian() {
        assert_eq!(
            PersonaEngine::select_for_tier(5),
            LanguagePersona::MoveGuardian
        );
    }
}
