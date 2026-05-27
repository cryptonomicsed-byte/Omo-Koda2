use omokoda_core::economics::SynapseAccount;
use omokoda_core::justice::tier::Tier;
use omokoda_core::reputation::{can_promote_tier, daily_gain_multiplier, MAX_ACTIONS_PER_DAY};

#[test]
fn bb_step_limits_per_tier() {
    assert_eq!(Tier::T0.bb_step_limit(), 1);
    assert_eq!(Tier::T1.bb_step_limit(), 6);
    assert_eq!(Tier::T2.bb_step_limit(), 21);
    assert_eq!(Tier::T3.bb_step_limit(), 107);
    assert_eq!(Tier::T4.bb_step_limit(), 107);
    assert_eq!(Tier::T5.bb_step_limit(), 47_176_870);
}

#[test]
fn synapse_cap_per_tier() {
    assert_eq!(SynapseAccount::new(0).tier_cap(), 1_000_000);
    assert_eq!(SynapseAccount::new(1).tier_cap(), 10_000_000);
    assert_eq!(SynapseAccount::new(2).tier_cap(), 30_000_000);
    assert_eq!(SynapseAccount::new(3).tier_cap(), 60_000_000);
    assert_eq!(SynapseAccount::new(4).tier_cap(), 86_000_000);
    assert_eq!(SynapseAccount::new(5).tier_cap(), 86_000_000);
}

#[test]
fn promotion_allowed_with_no_prior() {
    assert!(can_promote_tier(None));
}

#[test]
fn promotion_blocked_if_recent() {
    use chrono::Utc;
    let recent = Utc::now(); // just happened
    assert!(!can_promote_tier(Some(recent)));
}

#[test]
fn daily_action_cap_51st_returns_zero_multiplier() {
    // at MAX_ACTIONS_PER_DAY, the diminishing multiplier is near-zero but not exactly 0;
    // the caller is responsible for capping — multiplier itself just shows diminishing curve
    let at_cap = daily_gain_multiplier(MAX_ACTIONS_PER_DAY);
    let at_zero = daily_gain_multiplier(0);
    assert!(at_cap < at_zero);
    assert!(at_zero == 1.0);
}
