module omokoda::consensus_ledger {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;

    /// Epistemic severity from multi-model wisdom ensemble disagreement.
    const SEVERITY_UNANIMOUS: u8 = 0;
    const SEVERITY_STRONG: u8 = 1;
    const SEVERITY_MODERATE: u8 = 2;
    const SEVERITY_SEVERE: u8 = 3;

    /// A recorded consensus decision from the Wisdom ensemble.
    struct ConsensusRecord has key, store {
        id: UID,
        agent_id: vector<u8>,
        question_hash: vector<u8>,
        weighted_score: u64,
        model_count: u64,
        epistemic_severity: u8,
        timestamp: u64,
    }

    const E_INVALID_SEVERITY: u64 = 1;
    const E_ZERO_MODELS: u64 = 2;

    entry fun record(
        agent_id: vector<u8>,
        question_hash: vector<u8>,
        weighted_score: u64,
        model_count: u64,
        epistemic_severity: u8,
        timestamp: u64,
        ctx: &mut TxContext,
    ) {
        assert!(model_count > 0, E_ZERO_MODELS);
        assert!(epistemic_severity <= SEVERITY_SEVERE, E_INVALID_SEVERITY);

        let record = ConsensusRecord {
            id: object::new(ctx),
            agent_id,
            question_hash,
            weighted_score,
            model_count,
            epistemic_severity,
            timestamp,
        };
        transfer::transfer(record, tx_context::sender(ctx));
    }

    public fun is_severe(record: &ConsensusRecord): bool {
        record.epistemic_severity == SEVERITY_SEVERE
    }
    public fun weighted_score(record: &ConsensusRecord): u64 { record.weighted_score }
    public fun epistemic_severity(record: &ConsensusRecord): u8 { record.epistemic_severity }
    public fun severity_severe(): u8 { SEVERITY_SEVERE }
}
