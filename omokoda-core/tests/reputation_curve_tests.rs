use omokoda_core::reputation::{
    difficulty, daily_gain_multiplier, reputation_gain, tier_for,
    DIMINISHING_RETURNS_BASE, MAX_ACTIONS_PER_DAY,
};

// --- difficulty formula: 1.0 / (1.0 + rep/25.0) ---

#[test]
fn difficulty_at_zero_is_1() {
    let d = difficulty(0.0);
    assert!((d - 1.0).abs() < 1e-10, "difficulty(0) should be 1.0, got {}", d);
}

#[test]
fn difficulty_at_25_is_0_5() {
    let d = difficulty(25.0);
    assert!((d - 0.5).abs() < 1e-10, "difficulty(25) should be 0.5, got {}", d);
}

#[test]
fn difficulty_at_50_is_approx_0_333() {
    let d = difficulty(50.0);
    let expected = 1.0 / 3.0;
    assert!((d - expected).abs() < 1e-6, "difficulty(50) should be ~0.333, got {}", d);
}

#[test]
fn difficulty_at_75_is_0_25() {
    let d = difficulty(75.0);
    let expected = 1.0 / 4.0;
    assert!((d - expected).abs() < 1e-10, "difficulty(75) should be 0.25, got {}", d);
}

#[test]
fn difficulty_is_strictly_decreasing() {
    let d0 = difficulty(0.0);
    let d25 = difficulty(25.0);
    let d50 = difficulty(50.0);
    let d100 = difficulty(100.0);
    assert!(d0 > d25 && d25 > d50 && d50 > d100);
}

#[test]
fn difficulty_never_negative() {
    for rep in [0.0, 1.0, 25.0, 50.0, 100.0, 1000.0] {
        assert!(difficulty(rep) > 0.0);
    }
}

// --- reputation_gain formula ---

#[test]
fn reputation_gain_zero_multiplier_is_zero() {
    let gain = reputation_gain(0.1, 50.0, 0.0);
    assert_eq!(gain, 0.0);
}

#[test]
fn reputation_gain_scales_with_difficulty() {
    let low_rep = reputation_gain(0.1, 0.0, 1.0);
    let high_rep = reputation_gain(0.1, 100.0, 1.0);
    assert!(low_rep > high_rep, "gain should be higher at low reputation");
}

// --- tier_for: must match EXACT existing thresholds (these are tested by interpreter_tests) ---

#[test]
fn tier_for_20_is_t0() {
    // boundary: tier_for uses > (exclusive), so 20.0 is NOT > 20.0 → T0
    assert_eq!(tier_for(20.0), 0);
}

#[test]
fn tier_for_20_001_is_t1() {
    assert_eq!(tier_for(20.001), 1);
}

#[test]
fn tier_for_0_is_t0() {
    assert_eq!(tier_for(0.0), 0);
}

#[test]
fn tier_for_100_is_t5() {
    assert_eq!(tier_for(100.0), 5);
}

#[test]
fn tier_for_99_999_is_t4() {
    assert_eq!(tier_for(99.999), 4);
}

#[test]
fn tier_for_80_001_is_t4() {
    assert_eq!(tier_for(80.001), 4);
}

#[test]
fn tier_for_80_exactly_is_t3() {
    assert_eq!(tier_for(80.0), 3);
}

// --- diminishing returns formula ---

#[test]
fn diminishing_returns_base_constant() {
    assert!((DIMINISHING_RETURNS_BASE - 0.995).abs() < 1e-10);
}

#[test]
fn daily_multiplier_0_is_1() {
    assert!((daily_gain_multiplier(0) - 1.0).abs() < 1e-10);
}

#[test]
fn daily_multiplier_50_is_zero() {
    // 50 == MAX_ACTIONS_PER_DAY
    assert_eq!(daily_gain_multiplier(MAX_ACTIONS_PER_DAY), 0.0);
}

#[test]
fn daily_multiplier_49_is_above_zero() {
    let m = daily_gain_multiplier(49);
    assert!(m > 0.0 && m < 1.0, "multiplier at 49 should be (0,1), got {}", m);
}

#[test]
fn daily_multiplier_is_monotonically_decreasing() {
    let mut prev = daily_gain_multiplier(0);
    for n in 1..MAX_ACTIONS_PER_DAY {
        let curr = daily_gain_multiplier(n);
        assert!(curr < prev, "multiplier should decrease at action {}", n);
        prev = curr;
    }
}
