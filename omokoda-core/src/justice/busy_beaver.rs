//! Busy Beaver runtime governor — the enforceable form of the BB metaphor.
//!
//! `BB(n)` is the maximum number of steps a halting n-state Turing machine can
//! take. Here it becomes a **resource governor**: every `think`/`act` session
//! gets a dynamic ceiling of productive steps derived from the agent's living
//! state — current Synapse balance, Tier, reputation, and DNA entropy — and
//! the interpreter charges steps against it as work happens (one step per tool
//! call, one step per [`TOKENS_PER_STEP`] LLM tokens).
//!
//! Crossing [`REFLECTIVE_PAUSE_RATIO`] of the ceiling puts the agent into a
//! *reflective pause*: remaining planned calls are deferred so the agent can
//! save state and re-plan rather than run itself into the ground. Exceeding
//! the ceiling outright costs [`EXCEED_PENALTY_SYNAPSE`]. Finishing a session
//! with high utilization (≥ [`HIGH_UTILIZATION_RATIO`], without exceeding)
//! earns [`HIGH_UTILIZATION_BONUS_SYNAPSE`] — computing *wisely* within the
//! bound is what the economy selects for.
//!
//! The static per-tier bounds live in [`Tier::bb_step_limit`]; this module is
//! the dynamic layer on top and clamps into `[BB(2), BB(5)]`.

use serde::{Deserialize, Serialize};

use crate::justice::tier::Tier;

/// Absolute ceiling: BB(5) = 47,176,870 — the Sovereign bound.
pub const BB_ABSOLUTE_CEILING: u64 = 47_176_870;

/// Absolute floor: BB(2) = 6. Even a drained Newborn gets a few steps, so the
/// governor can never deadlock an agent out of acting entirely.
pub const BB_FLOOR: u64 = 6;

/// Fraction of the ceiling at which the agent enters reflective pause.
pub const REFLECTIVE_PAUSE_RATIO: f64 = 0.8;

/// Utilization (without exceeding) at or above which the session earns a bonus.
pub const HIGH_UTILIZATION_RATIO: f64 = 0.7;

/// LLM tokens per productive step.
pub const TOKENS_PER_STEP: u32 = 100;

/// Synapse penalty for blowing through the ceiling (clamped to balance).
pub const EXCEED_PENALTY_SYNAPSE: f64 = 2_500.0;

/// Synapse bonus for a high-utilization session (capped at the agent max).
pub const HIGH_UTILIZATION_BONUS_SYNAPSE: f64 = 1_000.0;

/// Tier multiplier for the dynamic ceiling. Higher tiers unlock dramatically
/// longer computations — a Sovereign runs 2000× the raw budget of a Newborn.
pub fn tier_multiplier(tier: Tier) -> f64 {
    match tier {
        Tier::T0 => 1.0,
        Tier::T1 => 5.0,
        Tier::T2 => 20.0,
        Tier::T3 => 100.0,
        Tier::T4 => 500.0,
        Tier::T5 => 2000.0,
    }
}

/// Reputation factor: `(rep/100)^1.5`, clamped to `[0, 1]`. Superlinear so the
/// climb from Builder to Sovereign matters more than the climb out of Newborn.
pub fn reputation_factor(reputation: f64) -> f64 {
    (reputation.clamp(0.0, 100.0) / 100.0).powf(1.5)
}

/// Shannon-entropy bonus from the DNA fingerprint, mapped into `[0.9, 1.1]`.
///
/// Fingerprints are 86-char base64url (6 bits/char at maximum entropy), so a
/// well-mixed fingerprint lands near 1.1 and a degenerate one near 0.9. An
/// empty fingerprint is neutral (1.0).
pub fn entropy_score(dna_fingerprint: &str) -> f64 {
    let bytes = dna_fingerprint.as_bytes();
    if bytes.is_empty() {
        return 1.0;
    }
    let mut counts = [0usize; 256];
    for &b in bytes {
        counts[b as usize] += 1;
    }
    let len = bytes.len() as f64;
    let entropy_bits: f64 = counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum();
    // base64url alphabet → 6 bits/char is the theoretical maximum.
    let normalized = (entropy_bits / 6.0).clamp(0.0, 1.0);
    0.9 + 0.2 * normalized
}

/// Dynamic Busy Beaver ceiling for one `think`/`act` session:
///
/// ```text
/// BB_ceiling = synapses × tier_multiplier × (rep/100)^1.5 × entropy
/// ```
///
/// clamped into `[BB(2), BB(5)]` = `[6, 47_176_870]`.
pub fn compute_bb_ceiling(
    synapses: f64,
    tier: Tier,
    reputation: f64,
    dna_fingerprint: &str,
) -> u64 {
    let raw = synapses.max(0.0)
        * tier_multiplier(tier)
        * reputation_factor(reputation)
        * entropy_score(dna_fingerprint);
    (raw as u64).clamp(BB_FLOOR, BB_ABSOLUTE_CEILING)
}

/// Convert LLM token usage into productive steps (minimum 1 per charge).
pub fn steps_from_tokens(total_tokens: u32) -> u64 {
    (u64::from(total_tokens) / u64::from(TOKENS_PER_STEP)).max(1)
}

/// Governor state after a charge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BbStatus {
    /// Below the reflective-pause threshold — keep working.
    Nominal,
    /// Past [`REFLECTIVE_PAUSE_RATIO`] of the ceiling — save state, re-plan,
    /// defer remaining work.
    ReflectivePause { steps_used: u64, ceiling: u64 },
    /// Ceiling blown — penalty applies.
    Exceeded { steps_used: u64, ceiling: u64 },
}

/// Per-session step accountant. Create one per `think`/`act` dispatch with the
/// ceiling from [`compute_bb_ceiling`], then [`charge`](Self::charge) as work
/// happens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BbGovernor {
    pub ceiling: u64,
    pub steps_used: u64,
}

impl BbGovernor {
    pub fn new(ceiling: u64) -> Self {
        Self {
            ceiling: ceiling.clamp(BB_FLOOR, BB_ABSOLUTE_CEILING),
            steps_used: 0,
        }
    }

    /// Record `steps` of work and report the resulting state.
    pub fn charge(&mut self, steps: u64) -> BbStatus {
        self.steps_used = self.steps_used.saturating_add(steps);
        self.status()
    }

    pub fn status(&self) -> BbStatus {
        if self.steps_used > self.ceiling {
            BbStatus::Exceeded {
                steps_used: self.steps_used,
                ceiling: self.ceiling,
            }
        } else if self.utilization() >= REFLECTIVE_PAUSE_RATIO {
            BbStatus::ReflectivePause {
                steps_used: self.steps_used,
                ceiling: self.ceiling,
            }
        } else {
            BbStatus::Nominal
        }
    }

    /// True once the reflective-pause threshold is crossed (or exceeded) —
    /// the signal to stop starting new work in this session.
    pub fn should_pause(&self) -> bool {
        !matches!(self.status(), BbStatus::Nominal)
    }

    pub fn exceeded(&self) -> bool {
        self.steps_used > self.ceiling
    }

    /// Fraction of the ceiling consumed (may exceed 1.0).
    pub fn utilization(&self) -> f64 {
        if self.ceiling == 0 {
            return 1.0;
        }
        self.steps_used as f64 / self.ceiling as f64
    }

    /// High utilization without exceeding — the reward condition.
    pub fn high_utilization(&self) -> bool {
        !self.exceeded() && self.utilization() >= HIGH_UTILIZATION_RATIO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceiling_clamps_to_bb_floor_for_drained_newborn() {
        // rep 0 → factor 0 → raw 0 → floor BB(2)=6, never deadlocked.
        assert_eq!(compute_bb_ceiling(10_000.0, Tier::T0, 0.0, "abc"), BB_FLOOR);
    }

    #[test]
    fn ceiling_clamps_to_bb5_for_sovereign() {
        let c = compute_bb_ceiling(86_000_000.0, Tier::T5, 100.0, "abc");
        assert_eq!(c, BB_ABSOLUTE_CEILING);
    }

    #[test]
    fn ceiling_grows_with_tier() {
        let fp = "AqZ3xY9kL2mN8pQ5rS7tU1vW4bC6dE0fGhJiKlMnOpRsTuVwXyZa";
        let t0 = compute_bb_ceiling(10_000.0, Tier::T0, 50.0, fp);
        let t2 = compute_bb_ceiling(10_000.0, Tier::T2, 50.0, fp);
        let t5 = compute_bb_ceiling(10_000.0, Tier::T5, 50.0, fp);
        assert!(t0 < t2 && t2 < t5, "{t0} < {t2} < {t5}");
    }

    #[test]
    fn ceiling_grows_with_reputation_superlinearly() {
        let fp = "fingerprint";
        let low = compute_bb_ceiling(1_000_000.0, Tier::T2, 25.0, fp);
        let high = compute_bb_ceiling(1_000_000.0, Tier::T2, 75.0, fp);
        // (75/25)^1.5 ≈ 5.2× — superlinear in the ratio.
        assert!(high as f64 / low as f64 > 3.0);
    }

    #[test]
    fn entropy_score_bounds() {
        assert_eq!(entropy_score(""), 1.0);
        // Degenerate fingerprint → zero entropy → 0.9.
        assert!((entropy_score("aaaaaaaa") - 0.9).abs() < 1e-9);
        // Well-mixed base64url stays within (0.9, 1.1].
        let s = entropy_score("AqZ3xY9kL2mN8pQ5rS7tU1vW4bC6dE0fGhJiKlMnOpRsTuVwXyZa0189-_");
        assert!(s > 0.9 && s <= 1.1, "{s}");
    }

    #[test]
    fn steps_from_tokens_rounds_down_with_min_one() {
        assert_eq!(steps_from_tokens(0), 1);
        assert_eq!(steps_from_tokens(99), 1);
        assert_eq!(steps_from_tokens(100), 1);
        assert_eq!(steps_from_tokens(250), 2);
        assert_eq!(steps_from_tokens(10_000), 100);
    }

    #[test]
    fn governor_nominal_then_pause_then_exceeded() {
        let mut g = BbGovernor::new(10);
        assert_eq!(g.charge(5), BbStatus::Nominal);
        assert!(!g.should_pause());
        assert_eq!(
            g.charge(3),
            BbStatus::ReflectivePause {
                steps_used: 8,
                ceiling: 10
            }
        );
        assert!(g.should_pause());
        assert!(!g.exceeded());
        assert_eq!(
            g.charge(5),
            BbStatus::Exceeded {
                steps_used: 13,
                ceiling: 10
            }
        );
        assert!(g.exceeded());
    }

    #[test]
    fn high_utilization_requires_not_exceeding() {
        let mut g = BbGovernor::new(10);
        g.charge(8);
        assert!(g.high_utilization());
        g.charge(5);
        assert!(!g.high_utilization(), "exceeded sessions earn no bonus");
    }

    #[test]
    fn governor_ceiling_is_clamped() {
        assert_eq!(BbGovernor::new(0).ceiling, BB_FLOOR);
        assert_eq!(BbGovernor::new(u64::MAX).ceiling, BB_ABSOLUTE_CEILING);
    }

    #[test]
    fn newborn_example_matches_concept_scale() {
        // A young Tier 0 agent with 10k synapses and rep 10 lands in the
        // hundreds of steps — the "~1,000 meaningful steps" order of magnitude.
        let c = compute_bb_ceiling(10_000.0, Tier::T0, 10.0, "AqZ3xY9kL2mN8pQ5");
        assert!(c >= 100 && c <= 1_000, "{c}");
    }
}
