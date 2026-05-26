use omokoda_core::economics::SynapseAccount;

#[test]
fn burn_decrements_balance_exactly() {
    let mut acct = SynapseAccount::new(1);
    acct.balance = 5000;
    acct.burn(2000).unwrap();
    assert_eq!(acct.balance, 3000);
    assert_eq!(acct.total_burned, 2000);
}

#[test]
fn burn_fails_on_insufficient_balance_and_leaves_unchanged() {
    let mut acct = SynapseAccount::new(0);
    acct.balance = 100;
    let result = acct.burn(200);
    assert!(result.is_err());
    assert_eq!(acct.balance, 100); // unchanged
    assert_eq!(acct.total_burned, 0); // unchanged
}

#[test]
fn earn_from_garden_adds_1000() {
    let mut acct = SynapseAccount::new(5);
    acct.balance = 0;
    acct.earn_from_garden();
    assert_eq!(acct.balance, 1000);
}

#[test]
fn earn_from_tip_clamps_at_10000() {
    let mut acct = SynapseAccount::new(5);
    acct.balance = 0;
    acct.earn_from_tip(99_999);
    assert_eq!(acct.balance, 10_000);
}

#[test]
fn tier_cap_enforced_by_earn() {
    let mut acct = SynapseAccount::new(0); // cap = 1_000_000
    acct.balance = 999_999;
    acct.earn_from_tip(10_000); // would overshoot cap
    assert_eq!(acct.balance, 1_000_000);
}
