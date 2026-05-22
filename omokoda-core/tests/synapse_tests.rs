use omokoda_core::economics::{SynapseAccount, SynapseError};

fn account(balance: u64, tier: u8) -> SynapseAccount {
    SynapseAccount::new(balance, tier)
}

// --- atomic burn ---

#[test]
fn burn_decrements_balance_exactly() {
    let mut acc = account(5_000, 0);
    acc.burn(1_000).unwrap();
    assert_eq!(acc.balance, 4_000);
    assert_eq!(acc.total_burned, 1_000);
}

#[test]
fn burn_full_balance_succeeds() {
    let mut acc = account(1_000, 0);
    acc.burn(1_000).unwrap();
    assert_eq!(acc.balance, 0);
    assert_eq!(acc.total_burned, 1_000);
}

#[test]
fn burn_fails_on_insufficient_balance() {
    let mut acc = account(500, 0);
    let result = acc.burn(1_000);
    assert_eq!(result, Err(SynapseError::InsufficientBalance));
    // Balance must be unchanged — no partial burn
    assert_eq!(acc.balance, 500);
    assert_eq!(acc.total_burned, 0);
}

#[test]
fn burn_zero_cost_always_succeeds() {
    let mut acc = account(0, 0);
    acc.burn(0).unwrap();
    assert_eq!(acc.balance, 0);
}

#[test]
fn burn_accumulates_total_burned() {
    let mut acc = account(10_000, 1);
    acc.burn(1_000).unwrap();
    acc.burn(2_000).unwrap();
    acc.burn(500).unwrap();
    assert_eq!(acc.total_burned, 3_500);
    assert_eq!(acc.balance, 6_500);
}

// --- earn_from_garden ---

#[test]
fn earn_from_garden_adds_1000() {
    let mut acc = account(0, 0);
    acc.earn_from_garden();
    assert_eq!(acc.balance, 1_000);
}

#[test]
fn earn_from_garden_caps_at_tier_ceiling() {
    // T0 cap = 1_000_000
    let mut acc = account(999_900, 0);
    acc.earn_from_garden();
    assert_eq!(acc.balance, 1_000_000); // saturates at cap
}

#[test]
fn earn_from_garden_does_not_exceed_cap() {
    let mut acc = account(1_000_000, 0);
    acc.earn_from_garden();
    assert_eq!(acc.balance, 1_000_000);
}

// --- earn_from_tip ---

#[test]
fn earn_from_tip_adds_small_amount() {
    let mut acc = account(0, 1);
    acc.earn_from_tip(500);
    assert_eq!(acc.balance, 500);
}

#[test]
fn earn_from_tip_clamps_at_10000() {
    let mut acc = account(0, 1);
    acc.earn_from_tip(50_000);
    assert_eq!(acc.balance, 10_000);
}

#[test]
fn earn_from_tip_exactly_10000() {
    let mut acc = account(0, 1);
    acc.earn_from_tip(10_000);
    assert_eq!(acc.balance, 10_000);
}

#[test]
fn earn_from_tip_clamped_at_tier_cap() {
    // T0 cap = 1_000_000; start near cap
    let mut acc = account(999_995, 0);
    acc.earn_from_tip(10_000);
    assert_eq!(acc.balance, 1_000_000);
}

// --- tier_cap correctness ---

#[test]
fn tier_cap_t0() {
    assert_eq!(account(0, 0).tier_cap(), 1_000_000);
}

#[test]
fn tier_cap_t1() {
    assert_eq!(account(0, 1).tier_cap(), 10_000_000);
}

#[test]
fn tier_cap_t2() {
    assert_eq!(account(0, 2).tier_cap(), 30_000_000);
}

#[test]
fn tier_cap_t3() {
    assert_eq!(account(0, 3).tier_cap(), 60_000_000);
}

#[test]
fn tier_cap_t4() {
    assert_eq!(account(0, 4).tier_cap(), 86_000_000);
}

#[test]
fn tier_cap_t5() {
    assert_eq!(account(0, 5).tier_cap(), 86_000_000);
}

// --- error display ---

#[test]
fn synapse_error_display() {
    let e = SynapseError::InsufficientBalance;
    assert_eq!(format!("{}", e), "insufficient_synapse");
}
