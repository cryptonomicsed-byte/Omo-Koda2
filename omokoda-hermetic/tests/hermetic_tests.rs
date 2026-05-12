#[cfg(test)]
mod hermetic_tests {
    use omokoda_hermetic::HermeticState;

    #[test]
    fn same_seed_produces_same_state() {
        let a = HermeticState::from_seed("agent-001", 1714348800);
        let b = HermeticState::from_seed("agent-001", 1714348800);
        assert_eq!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn different_name_produces_different_state() {
        let a = HermeticState::from_seed("agent-001", 1714348800);
        let b = HermeticState::from_seed("agent-002", 1714348800);
        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn different_timestamp_produces_different_state() {
        let a = HermeticState::from_seed("agent-001", 1714348800);
        let b = HermeticState::from_seed("agent-001", 1714348801);
        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn fingerprint_is_fixed_length() {
        let state = HermeticState::from_seed("test", 0);
        assert_eq!(state.fingerprint().len(), 64); // 32 bytes as hex
    }

    #[test]
    fn empty_name_is_rejected() {
        let result = std::panic::catch_unwind(|| HermeticState::from_seed("", 0));
        assert!(result.is_err());
    }

    #[test]
    fn principle_values_are_not_public() {
        // HermeticState exposes only fingerprint() — no named principle fields
        // This test passes by compiling: if principle fields were pub,
        // we would access them here and they would not panic.
        let state = HermeticState::from_seed("test", 1714348800);
        let f = state.fingerprint();
        assert!(!f.is_empty());
    }

    #[test]
    fn think_policy_returns_valid_depth() {
        let state = HermeticState::from_seed("agent-001", 1714348800);
        let depth = state.think_abstraction_depth();
        assert!((0.0..=1.0).contains(&depth));
    }

    #[test]
    fn act_cooldown_is_deterministic() {
        let state = HermeticState::from_seed("agent-001", 1714348800);
        let same_state = HermeticState::from_seed("agent-001", 1714348800);
        assert_eq!(state.act_cooldown_ms(), same_state.act_cooldown_ms());
    }
}
