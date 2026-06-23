use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tier {
    T0 = 0,
    T1 = 1,
    T2 = 2,
    T3 = 3,
    T4 = 4,
    T5 = 5,
}

impl Tier {
    /// Convert from reputation score to Tier using closed-lower-bound ranges:
    /// [0, 20) → T0, [20, 40) → T1, [40, 60) → T2, [60, 80) → T3,
    /// [80, 100) → T4, [100, ∞) → T5
    ///
    /// NOTE: This differs intentionally from the existing `tier_for()` in reputation.rs,
    /// which uses exclusive-upper-bound (`>`) thresholds. Both functions coexist.
    /// Use `tier_for()` for backward-compatible tier lookups (tested by interpreter_tests).
    /// Use `Tier::from_reputation()` when you need the typed Tier enum.
    pub fn from_reputation(rep: f64) -> Self {
        if rep < 20.0 {
            Tier::T0
        } else if rep < 40.0 {
            Tier::T1
        } else if rep < 60.0 {
            Tier::T2
        } else if rep < 80.0 {
            Tier::T3
        } else if rep < 100.0 {
            Tier::T4
        } else {
            Tier::T5
        }
    }

    /// Busy Beaver step limit for this tier.
    /// BB(1)=1, BB(2)=6, BB(3)=21, BB(4)=107, BB(5)=47,176,870 (Σ function, known values).
    /// T3 and T4 share BB(4) bound; T5 unlocks BB(5) at Sovereign Tier.
    pub fn bb_step_limit(self) -> u64 {
        match self {
            Tier::T0 => 1,
            Tier::T1 => 6,
            Tier::T2 => 21,
            Tier::T3 | Tier::T4 => 107,
            Tier::T5 => 47_176_870,
        }
    }

    /// Synapse cost efficiency — higher tiers pay proportionally less per unit of work.
    pub fn synapse_efficiency(self) -> f64 {
        match self {
            Tier::T0 => 1.0,
            Tier::T1 => 0.95,
            Tier::T2 => 0.90,
            Tier::T3 => 0.85,
            Tier::T4 => 0.80,
            Tier::T5 => 0.75,
        }
    }

    /// Daily Synapse decay rate as a percentage.
    pub fn decay_rate_percent(self) -> f64 {
        match self {
            Tier::T0 => 12.0,
            Tier::T1 => 10.0,
            Tier::T2 => 9.0,
            Tier::T3 => 8.0,
            Tier::T4 => 6.0,
            Tier::T5 => 4.0,
        }
    }

    /// Maximum Synapse balance this tier can hold.
    pub fn synapse_cap(self) -> u64 {
        match self {
            Tier::T0 => 1_000_000,
            Tier::T1 => 10_000_000,
            Tier::T2 => 30_000_000,
            Tier::T3 => 60_000_000,
            Tier::T4 | Tier::T5 => 86_000_000,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Tier {
    fn from(v: u8) -> Self {
        match v {
            0 => Tier::T0,
            1 => Tier::T1,
            2 => Tier::T2,
            3 => Tier::T3,
            4 => Tier::T4,
            _ => Tier::T5,
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "T{}", self.as_u8())
    }
}
