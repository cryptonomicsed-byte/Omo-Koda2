use serde::{Deserialize, Serialize};

/// The 11 Òrìṣà-archetype reasoning lobes of the Wisdom module.
/// Each lobe has a distinct voice and veto condition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WisdomLobe {
    Esu,      // Gateway — checks routing and permission first
    Obatala,  // Clarity — rejects ambiguous or impure reasoning
    Osun,     // Depth — amplifies emotional/relational weight
    Yemoja,   // Creation — evaluates generative potential
    Ogun,     // Execution — pragmatic feasibility check
    Sango,    // Justice — checks for fairness and consequences
    Oya,      // Flow — temporal rhythm and change detection
    Orunmila, // Wisdom — highest-order synthesis
    Eshu,     // Trickster — stress-tests conclusions
    Shango,   // Power — authority and capability check
    Oshun,    // Harmony — beauty, balance, resolution check
}

impl WisdomLobe {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Esu => "esu",
            Self::Obatala => "obatala",
            Self::Osun => "osun",
            Self::Yemoja => "yemoja",
            Self::Ogun => "ogun",
            Self::Sango => "sango",
            Self::Oya => "oya",
            Self::Orunmila => "orunmila",
            Self::Eshu => "eshu",
            Self::Shango => "shango",
            Self::Oshun => "oshun",
        }
    }

    pub fn all() -> [WisdomLobe; 11] {
        [
            Self::Esu,
            Self::Obatala,
            Self::Osun,
            Self::Yemoja,
            Self::Ogun,
            Self::Sango,
            Self::Oya,
            Self::Orunmila,
            Self::Eshu,
            Self::Shango,
            Self::Oshun,
        ]
    }
}

/// Result from ensemble deliberation.
#[derive(Debug, Clone)]
pub struct EnsembleResult {
    pub consensus_score: f64, // 0.0 = complete disagreement, 1.0 = unanimous
    pub vetoed_by: Vec<WisdomLobe>,
    pub synthesis: String,
}

impl EnsembleResult {
    pub fn is_vetoed(&self) -> bool {
        !self.vetoed_by.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lobes_returns_eleven() {
        assert_eq!(WisdomLobe::all().len(), 11);
    }

    #[test]
    fn all_lobes_have_string_repr() {
        for lobe in WisdomLobe::all() {
            assert!(!lobe.as_str().is_empty());
        }
    }

    #[test]
    fn ensemble_result_vetoed_check() {
        let r = EnsembleResult {
            consensus_score: 0.5,
            vetoed_by: vec![WisdomLobe::Obatala],
            synthesis: "blocked".to_string(),
        };
        assert!(r.is_vetoed());
    }
}
