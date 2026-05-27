use omokoda_core::reputation::{daily_gain_multiplier, difficulty, DIMINISHING_RETURNS_BASE};

#[test]
fn difficulty_at_zero_rep_is_one() {
    assert_eq!(difficulty(0.0), 1.0);
}

#[test]
fn difficulty_at_25_rep_is_half() {
    let d = difficulty(25.0);
    assert!((d - 0.5).abs() < 1e-10, "difficulty(25) = {}", d);
}

#[test]
fn difficulty_at_50_rep_is_one_third() {
    let d = difficulty(50.0);
    assert!((d - (1.0 / 3.0)).abs() < 1e-10, "difficulty(50) = {}", d);
}

#[test]
fn diminishing_returns_zero_actions_is_one() {
    assert_eq!(daily_gain_multiplier(0), 1.0);
}

#[test]
fn diminishing_returns_50_actions_is_less_than_one() {
    let m = daily_gain_multiplier(50);
    assert!(m < 1.0 && m > 0.5, "multiplier(50) = {}", m);
    // 0.995^50 ≈ 0.778
    let expected = DIMINISHING_RETURNS_BASE.powi(50);
    assert!((m - expected).abs() < 1e-10);
}

#[test]
fn t5_reputation_threshold_is_100() {
    use omokoda_core::reputation::tier_for;
    assert_eq!(tier_for(100.0), 5);
    assert_eq!(tier_for(99.999), 4);
}
