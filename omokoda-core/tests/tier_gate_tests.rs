use omokoda_core::justice::tier::Tier;
use omokoda_core::reputation::{
    can_promote_tier, daily_gain_multiplier, MAX_ACTIONS_PER_DAY, MIN_DAYS_BETWEEN_PROMOTIONS,
};
use chrono::{Duration, Utc};

// --- BB step limits ---

#[test]
fn bb_step_limit_t0() {
    assert_eq!(Tier::T0.bb_step_limit(), 1);
}

#[test]
fn bb_step_limit_t1() {
    assert_eq!(Tier::T1.bb_step_limit(), 6);
}

#[test]
fn bb_step_limit_t2() {
    assert_eq!(Tier::T2.bb_step_limit(), 21);
}

#[test]
fn bb_step_limit_t3() {
    assert_eq!(Tier::T3.bb_step_limit(), 107);
}

#[test]
fn bb_step_limit_t4() {
    assert_eq!(Tier::T4.bb_step_limit(), 107);
}

#[test]
fn bb_step_limit_t5() {
    assert_eq!(Tier::T5.bb_step_limit(), 47_176_870);
}

// --- Synapse caps ---

#[test]
fn synapse_cap_t0() {
    assert_eq!(Tier::T0.synapse_cap(), 1_000_000);
}

#[test]
fn synapse_cap_t1() {
    assert_eq!(Tier::T1.synapse_cap(), 10_000_000);
}

#[test]
fn synapse_cap_t2() {
    assert_eq!(Tier::T2.synapse_cap(), 30_000_000);
}

#[test]
fn synapse_cap_t3() {
    assert_eq!(Tier::T3.synapse_cap(), 60_000_000);
}

#[test]
fn synapse_cap_t4() {
    assert_eq!(Tier::T4.synapse_cap(), 86_000_000);
}

#[test]
fn synapse_cap_t5() {
    assert_eq!(Tier::T5.synapse_cap(), 86_000_000);
}

// --- from_reputation boundaries ---

#[test]
fn tier_from_rep_zero() {
    assert_eq!(Tier::from_reputation(0.0), Tier::T0);
}

#[test]
fn tier_from_rep_19_999() {
    assert_eq!(Tier::from_reputation(19.999), Tier::T0);
}

#[test]
fn tier_from_rep_20_exactly() {
    // Tier::from_reputation uses < thresholds, so 20.0 maps to T1
    assert_eq!(Tier::from_reputation(20.0), Tier::T1);
}

#[test]
fn tier_from_rep_100_exactly() {
    assert_eq!(Tier::from_reputation(100.0), Tier::T5);
}

#[test]
fn tier_from_rep_sovereign() {
    assert_eq!(Tier::from_reputation(999.0), Tier::T5);
}

// --- 7-day promotion gate ---

#[test]
fn promotion_allowed_on_first_time() {
    assert!(can_promote_tier(None));
}

#[test]
fn promotion_blocked_within_7_days() {
    let recent = Utc::now() - Duration::days(3);
    assert!(!can_promote_tier(Some(recent)));
}

#[test]
fn promotion_allowed_after_7_days() {
    let old = Utc::now() - Duration::days(MIN_DAYS_BETWEEN_PROMOTIONS as i64);
    assert!(can_promote_tier(Some(old)));
}

#[test]
fn promotion_blocked_at_6_days() {
    let six_days_ago = Utc::now() - Duration::days(6);
    assert!(!can_promote_tier(Some(six_days_ago)));
}

// --- Daily action cap ---

#[test]
fn daily_multiplier_at_zero_actions() {
    assert!((daily_gain_multiplier(0) - 1.0).abs() < 1e-10);
}

#[test]
fn daily_multiplier_decreases_with_actions() {
    let m0 = daily_gain_multiplier(0);
    let m10 = daily_gain_multiplier(10);
    let m49 = daily_gain_multiplier(49);
    assert!(m0 > m10);
    assert!(m10 > m49);
    assert!(m49 > 0.0);
}

#[test]
fn daily_multiplier_at_cap_is_zero() {
    assert_eq!(daily_gain_multiplier(MAX_ACTIONS_PER_DAY), 0.0);
}

#[test]
fn daily_multiplier_beyond_cap_is_zero() {
    assert_eq!(daily_gain_multiplier(MAX_ACTIONS_PER_DAY + 10), 0.0);
}

#[test]
fn daily_multiplier_at_49_approximately_0_778() {
    let m = daily_gain_multiplier(49);
    // 0.995^49 ≈ 0.7817
    assert!(m > 0.75 && m < 0.85, "multiplier at 49 should be ~0.78, got {}", m);
}

// --- Tier round-trip ---

#[test]
fn tier_from_u8_round_trip() {
    for i in 0u8..=5 {
        let t = Tier::from(i);
        assert_eq!(t.as_u8(), i);
    }
}

#[test]
fn tier_u8_overflow_maps_to_t5() {
    assert_eq!(Tier::from(255u8), Tier::T5);
}
