module omokoda::zbt_core {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use omokoda::zbt_errors;

    /// A security audit entry — immutable once recorded.
    struct AuditEntry has key, store {
        id: UID,
        agent_id: vector<u8>,
        action: vector<u8>,
        severity: u8,
        timestamp: u64,
        receipt_hash: vector<u8>,
    }

    const SEVERITY_LOW: u8 = 0;
    const SEVERITY_MEDIUM: u8 = 1;
    const SEVERITY_HIGH: u8 = 2;
    const SEVERITY_CRITICAL: u8 = 3;

    entry fun record_audit(
        agent_id: vector<u8>,
        action: vector<u8>,
        severity: u8,
        timestamp: u64,
        receipt_hash: vector<u8>,
        ctx: &mut TxContext,
    ) {
        assert!(timestamp > 0, zbt_errors::invalid_receipt());
        assert!(severity <= SEVERITY_CRITICAL, zbt_errors::unauthorized());

        let entry = AuditEntry {
            id: object::new(ctx),
            agent_id,
            action,
            severity,
            timestamp,
            receipt_hash,
        };
        transfer::transfer(entry, tx_context::sender(ctx));
    }

    public fun severity_critical(): u8 { SEVERITY_CRITICAL }
    public fun severity_high(): u8 { SEVERITY_HIGH }
    public fun severity_medium(): u8 { SEVERITY_MEDIUM }
    public fun severity_low(): u8 { SEVERITY_LOW }
}
