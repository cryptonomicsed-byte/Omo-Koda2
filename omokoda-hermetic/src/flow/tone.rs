use serde::{Deserialize, Serialize};

/// 7 tone styles from NarratorIDE — modulate agent output based on context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ToneStyle {
    #[default]
    Balanced,
    Academic,
    Casual,
    Playful,
    Verbose,
    Concise,
    Encouraging,
    Brutal,
}

impl ToneStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Academic => "academic",
            Self::Casual => "casual",
            Self::Playful => "playful",
            Self::Verbose => "verbose",
            Self::Concise => "concise",
            Self::Encouraging => "encouraging",
            Self::Brutal => "brutal",
        }
    }

    /// Select tone based on reputation and context tension.
    pub fn for_context(reputation: f64, tension: f64) -> Self {
        if tension > 75.0 {
            Self::Brutal // critical tension → direct, no fluff
        } else if reputation < 20.0 {
            Self::Encouraging // newborn → nurture
        } else if reputation > 80.0 {
            Self::Academic // architect/sovereign → precise
        } else {
            Self::Balanced
        }
    }
}

/// Tone engine — routes output style based on agent state.
pub struct ToneEngine;

impl ToneEngine {
    pub fn select(reputation: f64, tension: f64) -> ToneStyle {
        ToneStyle::for_context(reputation, tension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newborn_gets_encouraging_tone() {
        let tone = ToneEngine::select(5.0, 0.0);
        assert_eq!(tone, ToneStyle::Encouraging);
    }

    #[test]
    fn high_tension_gets_brutal_tone() {
        let tone = ToneEngine::select(50.0, 90.0);
        assert_eq!(tone, ToneStyle::Brutal);
    }

    #[test]
    fn sovereign_gets_academic_tone() {
        let tone = ToneEngine::select(95.0, 0.0);
        assert_eq!(tone, ToneStyle::Academic);
    }

    #[test]
    fn all_tones_have_string_repr() {
        let tones = [
            ToneStyle::Balanced,
            ToneStyle::Academic,
            ToneStyle::Casual,
            ToneStyle::Playful,
            ToneStyle::Verbose,
            ToneStyle::Concise,
            ToneStyle::Encouraging,
            ToneStyle::Brutal,
        ];
        for tone in &tones {
            assert!(!tone.as_str().is_empty());
        }
    }
}
