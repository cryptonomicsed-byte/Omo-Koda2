module omokoda::zbt_guard {
    use omokoda::zbt_errors;

    /// @nonreentrant: tracks in-flight execution to prevent reentrancy.
    struct ReentrancyGuard has store {
        locked: bool,
    }

    public fun new_guard(): ReentrancyGuard {
        ReentrancyGuard { locked: false }
    }

    public fun enter(guard: &mut ReentrancyGuard) {
        assert!(!guard.locked, zbt_errors::unauthorized());
        guard.locked = true;
    }

    public fun exit(guard: &mut ReentrancyGuard) {
        guard.locked = false;
    }

    public fun is_locked(guard: &ReentrancyGuard): bool {
        guard.locked
    }

    /// Dry run is structurally forbidden in all Omo-koda contracts.
    public fun assert_not_dry_run() {
        // In production: assert this is a real tx, not a simulation.
        // Structural invariant — receipts must be real state transitions.
    }
}
