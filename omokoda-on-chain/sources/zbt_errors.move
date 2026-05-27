module omokoda::zbt_errors {
    const E_UNAUTHORIZED: u64 = 100;
    const E_INVALID_RECEIPT: u64 = 101;
    const E_DRY_RUN_FORBIDDEN: u64 = 102;
    const E_SABBATH_QUEUED: u64 = 103;
    const E_INSUFFICIENT_SYNAPSE: u64 = 104;
    const E_TIER_GATE_FAILED: u64 = 105;
    const E_REPLAY_DETECTED: u64 = 106;
    const E_THRESHOLD_NOT_MET: u64 = 107;

    public fun unauthorized(): u64 { E_UNAUTHORIZED }
    public fun invalid_receipt(): u64 { E_INVALID_RECEIPT }
    public fun dry_run_forbidden(): u64 { E_DRY_RUN_FORBIDDEN }
    public fun sabbath_queued(): u64 { E_SABBATH_QUEUED }
    public fun insufficient_synapse(): u64 { E_INSUFFICIENT_SYNAPSE }
    public fun tier_gate_failed(): u64 { E_TIER_GATE_FAILED }
    public fun replay_detected(): u64 { E_REPLAY_DETECTED }
    public fun threshold_not_met(): u64 { E_THRESHOLD_NOT_MET }
}
