use serde::{Deserialize, Serialize};

/// Tension level — escalates to human review at Critical.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TensionLevel {
    Calm,     // 0–25
    Rising,   // 26–50
    High,     // 51–75
    Critical, // 76–100 — triggers human review
}

impl TensionLevel {
    pub fn from_score(score: f64) -> Self {
        if score <= 25.0 {
            Self::Calm
        } else if score <= 50.0 {
            Self::Rising
        } else if score <= 75.0 {
            Self::High
        } else {
            Self::Critical
        }
    }
}

/// Tension tracker — monitors narrative/state tension across acts.
/// From eternal-orisa-loom: escalates to human review when tension > 85.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensionTracker {
    pub score: f64,
    pub peak: f64,
    pub review_threshold: f64,
}

impl Default for TensionTracker {
    fn default() -> Self {
        Self {
            score: 0.0,
            peak: 0.0,
            review_threshold: 85.0,
        }
    }
}

impl TensionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_delta(&mut self, delta: f64) {
        self.score = (self.score + delta).clamp(0.0, 100.0);
        if self.score > self.peak {
            self.peak = self.score;
        }
    }

    pub fn level(&self) -> TensionLevel {
        TensionLevel::from_score(self.score)
    }

    pub fn requires_human_review(&self) -> bool {
        self.score >= self.review_threshold
    }

    pub fn decay(&mut self, rate: f64) {
        self.score = (self.score - rate).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tension_starts_calm() {
        let t = TensionTracker::new();
        assert_eq!(t.level(), TensionLevel::Calm);
        assert!(!t.requires_human_review());
    }

    #[test]
    fn tension_escalates_to_critical() {
        let mut t = TensionTracker::new();
        t.apply_delta(90.0);
        assert_eq!(t.level(), TensionLevel::Critical);
        assert!(t.requires_human_review());
    }

    #[test]
    fn tension_clamped_at_100() {
        let mut t = TensionTracker::new();
        t.apply_delta(200.0);
        assert_eq!(t.score, 100.0);
    }

    #[test]
    fn tension_decays() {
        let mut t = TensionTracker::new();
        t.apply_delta(50.0);
        t.decay(20.0);
        assert!((t.score - 30.0).abs() < 1e-9);
    }
}
