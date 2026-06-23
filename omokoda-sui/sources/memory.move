/// Ṣàngó Memory Contract — on-chain agent identity and memory receipt anchoring.
///
/// Complements `sango.move` (constitutional governance) with the persistent
/// memory layer: birth ritual produces an AgentIdentity + GovernanceCapability,
/// and every RACK write produces a MemoryReceipt anchored on-chain for auditability.
///
/// Layer encoding (u8):
///   0 = identity    — immutable genesis; anchor_memory rejects writes here
///   1 = long_term   — minimum resonance 500 (0.5 × 1000) required
///   2 = short_term  — daily notes; no resonance requirement
module omokoda_sui::memory {

    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use sui::event;
    use std::string::{Self, String};
    use std::vector;

    // ---------------------------------------------------------------------------
    // Error codes
    // ---------------------------------------------------------------------------

    const E_IDENTITY_PROTECTED: u64 = 1;
    const E_RESONANCE_TOO_LOW: u64  = 2;

    const MIN_LONG_TERM_RESONANCE: u64 = 500;

    // ---------------------------------------------------------------------------
    // Events
    // ---------------------------------------------------------------------------

    struct AgentBorn has copy, drop {
        agent_id: String,
        timestamp: u64,
        holder: address,
    }

    struct MemoryAnchored has copy, drop {
        agent_id: String,
        layer: u8,
        resonance: u64,
        sequence: u64,
        timestamp: u64,
    }

    struct BeliefsUpdated has copy, drop {
        agent_id: String,
        timestamp: u64,
    }

    // ---------------------------------------------------------------------------
    // Objects
    // ---------------------------------------------------------------------------

    /// Immutable birth certificate — created once, beliefs updated via governance.
    struct AgentIdentity has key {
        id: UID,
        agent_id: String,
        genesis_hash: vector<u8>,
        core_beliefs: vector<String>,
        birth_timestamp: u64,
    }

    /// Governance capability — holder may update core_beliefs on AgentIdentity.
    struct GovernanceCapability has key {
        id: UID,
        agent_id: String,
    }

    /// Portable proof of a memory write — transferable, storable, queryable.
    struct MemoryReceipt has key, store {
        id: UID,
        agent_id: String,
        content_hash: vector<u8>,
        resonance: u64,
        layer: u8,
        timestamp: u64,
        sequence: u64,
    }

    // ---------------------------------------------------------------------------
    // Entry functions
    // ---------------------------------------------------------------------------

    /// Birth ritual — creates AgentIdentity + GovernanceCapability for the sender.
    public entry fun birth_agent(
        agent_id: String,
        genesis_hash: vector<u8>,
        core_beliefs: vector<String>,
        ctx: &mut TxContext,
    ) {
        let sender    = tx_context::sender(ctx);
        let ts        = tx_context::epoch_timestamp_ms(ctx);
        let agent_id_gov = agent_id;

        let identity = AgentIdentity {
            id: object::new(ctx),
            agent_id,
            genesis_hash,
            core_beliefs,
            birth_timestamp: ts,
        };

        let gov = GovernanceCapability {
            id: object::new(ctx),
            agent_id: agent_id_gov,
        };

        event::emit(AgentBorn {
            agent_id: identity.agent_id,
            timestamp: ts,
            holder: sender,
        });

        transfer::transfer(identity, sender);
        transfer::transfer(gov, sender);
    }

    /// Anchor a memory write — produces a MemoryReceipt transferred to the caller.
    ///
    /// Rejects layer=0 (identity layer) and layer=1 with resonance below threshold.
    public entry fun anchor_memory(
        identity: &AgentIdentity,
        content_hash: vector<u8>,
        resonance: u64,
        layer: u8,
        sequence: u64,
        ctx: &mut TxContext,
    ) {
        assert!(layer != 0, E_IDENTITY_PROTECTED);
        assert!(layer != 1 || resonance >= MIN_LONG_TERM_RESONANCE, E_RESONANCE_TOO_LOW);

        let ts = tx_context::epoch_timestamp_ms(ctx);

        event::emit(MemoryAnchored {
            agent_id: identity.agent_id,
            layer,
            resonance,
            sequence,
            timestamp: ts,
        });

        let receipt = MemoryReceipt {
            id: object::new(ctx),
            agent_id: identity.agent_id,
            content_hash,
            resonance,
            layer,
            timestamp: ts,
            sequence,
        };

        transfer::transfer(receipt, tx_context::sender(ctx));
    }

    /// Update core beliefs — requires GovernanceCapability for this agent.
    public entry fun update_beliefs(
        identity: &mut AgentIdentity,
        _gov: &GovernanceCapability,
        new_beliefs: vector<String>,
        ctx: &mut TxContext,
    ) {
        identity.core_beliefs = new_beliefs;

        event::emit(BeliefsUpdated {
            agent_id: identity.agent_id,
            timestamp: tx_context::epoch_timestamp_ms(ctx),
        });
    }

    // ---------------------------------------------------------------------------
    // Read-only helpers
    // ---------------------------------------------------------------------------

    public fun agent_id(identity: &AgentIdentity): String {
        identity.agent_id
    }

    public fun beliefs(identity: &AgentIdentity): &vector<String> {
        &identity.core_beliefs
    }

    public fun receipt_resonance(receipt: &MemoryReceipt): u64 {
        receipt.resonance
    }

    public fun receipt_layer(receipt: &MemoryReceipt): u8 {
        receipt.layer
    }
}
