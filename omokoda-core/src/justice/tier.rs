use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    T0 = 0,
    T1 = 1,
    T2 = 2,
    T3 = 3,
    T4 = 4,
    T5 = 5,
}

impl Tier {
    pub fn from_reputation(rep: f64) -> Self {
        if rep >= 100.0 {
            Self::T5
        } else if rep > 80.0 {
            Self::T4
        } else if rep > 60.0 {
            Self::T3
        } else if rep > 40.0 {
            Self::T2
        } else if rep > 20.0 {
            Self::T1
        } else {
            Self::T0
        }
    }

    pub fn bb_step_limit(self) -> u64 {
        match self {
            Self::T0 => 1,
            Self::T1 => 6,
            Self::T2 => 21,
            Self::T3 | Self::T4 => 107,
            Self::T5 => 47_176_870,
        }
    }

    pub fn synapse_cap(self) -> u64 {
        match self {
            Self::T0 => 1_000_000,
            Self::T1 => 10_000_000,
            Self::T2 => 30_000_000,
            Self::T3 => 60_000_000,
            Self::T4 | Self::T5 => 86_000_000,
        }
    }

    pub fn synapse_efficiency(self) -> f64 {
        match self {
            Self::T0 => 1.00,
            Self::T1 => 0.95,
            Self::T2 => 0.90,
            Self::T3 => 0.85,
            Self::T4 => 0.80,
            Self::T5 => 0.75,
        }
    }

    pub fn decay_rate_percent(self) -> f64 {
        match self {
            Self::T0 => 12.0,
            Self::T1 => 10.0,
            Self::T2 => 8.0,
            Self::T3 => 7.0,
            Self::T4 => 5.0,
            Self::T5 => 4.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t5_bb_step_limit() {
        assert_eq!(Tier::T5.bb_step_limit(), 47_176_870);
    }

    #[test]
    fn t0_bb_step_limit() {
        assert_eq!(Tier::T0.bb_step_limit(), 1);
    }

    #[test]
    fn synapse_cap_t4_t5_equal() {
        assert_eq!(Tier::T4.synapse_cap(), Tier::T5.synapse_cap());
        assert_eq!(Tier::T5.synapse_cap(), 86_000_000);
    }
}
