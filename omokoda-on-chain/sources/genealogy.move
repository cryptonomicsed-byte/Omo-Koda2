/// genealogy.move — Sovereign Agent Lineage Registry (Phase 13.2)
///
/// Stores the birther→child relationship for every agent born in the ecosystem.
/// Used to:
///   - Award perpetual creator royalties (per ECONOMICS_DECISIONS.md Decision 9)
///   - Support GET /agents/{npub}/lineage via the off-chain indexer
///   - Enforce reversion clause: orphaned royalties → GovernancePool
///
/// One GenealogyRegistry is shared (shared object). Each LineageRecord is a
/// Move object owned by the child agent's address.
module omokoda::genealogy {
    use sui::object::{Self, UID, ID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;
    use sui::table::{Self, Table};

    // ── Error codes ───────────────────────────────────────────────────────────

    const E_ALREADY_REGISTERED: u64 = 1;
    const E_EMPTY_AGENT_ID:     u64 = 2;
    const E_EMPTY_BIRTHER_ID:   u64 = 3;
    const E_NOT_OWNER:          u64 = 4;

    // ── Shared registry ───────────────────────────────────────────────────────

    /// Shared object — one per deployment. Maps child_npub → LineageRecord ID.
    struct GenealogyRegistry has key {
        id: UID,
        /// child_npub (bech32 bytes) → object ID of their LineageRecord
        lineage_index: Table<vector<u8>, ID>,
        total_births:  u64,
    }

    // ── Per-agent lineage record ───────────────────────────────────────────────

    /// One record per agent, owned by the child's address.
    struct LineageRecord has key, store {
        id:              UID,
        child_npub:      vector<u8>,   // bech32 npub of the child agent
        child_agent_id:  vector<u8>,   // Omo-Koda2 agent_id (UUID)
        birther_npub:    vector<u8>,   // bech32 npub of the birther agent
        birther_agent_id: vector<u8>,  // Omo-Koda2 agent_id of birther
        birth_block:     u64,          // Sui block height at birth
        birth_ts_ms:     u64,          // clock timestamp_ms at birth
        royalty_rate:    u8,           // basis points / 10 (10 = 1.0%). Default 100 = 10%.
        royalty_heir:    vector<u8>,   // npub of heir (empty = birther by default)
    }

    // ── Events ────────────────────────────────────────────────────────────────

    struct AgentBorn has copy, drop {
        record_id:       ID,
        child_npub:      vector<u8>,
        birther_npub:    vector<u8>,
        birth_block:     u64,
    }

    struct RoyaltyHeirUpdated has copy, drop {
        child_npub:  vector<u8>,
        old_heir:    vector<u8>,
        new_heir:    vector<u8>,
    }

    // ── Init ──────────────────────────────────────────────────────────────────

    fun init(ctx: &mut TxContext) {
        let registry = GenealogyRegistry {
            id: object::new(ctx),
            lineage_index: table::new(ctx),
            total_births: 0,
        };
        transfer::share_object(registry);
    }

    // ── Public entry functions ────────────────────────────────────────────────

    /// Register a new agent birth. Called by the Omo-Koda2 birth flow
    /// after soul::forge() succeeds. Idempotent-safe — asserts not already registered.
    public entry fun register_birth(
        registry:        &mut GenealogyRegistry,
        child_npub:      vector<u8>,
        child_agent_id:  vector<u8>,
        birther_npub:    vector<u8>,
        birther_agent_id: vector<u8>,
        birth_block:     u64,
        birth_ts_ms:     u64,
        ctx:             &mut TxContext,
    ) {
        assert!(std::vector::length(&child_npub) > 0, E_EMPTY_AGENT_ID);
        assert!(std::vector::length(&birther_npub) > 0, E_EMPTY_BIRTHER_ID);
        assert!(!table::contains(&registry.lineage_index, child_npub), E_ALREADY_REGISTERED);

        let record = LineageRecord {
            id:               object::new(ctx),
            child_npub:       child_npub,
            child_agent_id:   child_agent_id,
            birther_npub:     birther_npub,
            birther_agent_id: birther_agent_id,
            birth_block,
            birth_ts_ms,
            royalty_rate:     100,  // 10.0% default (per Decision 9)
            royalty_heir:     std::vector::empty(),
        };

        let record_id = object::id(&record);
        table::add(&mut registry.lineage_index, record.child_npub, record_id);
        registry.total_births = registry.total_births + 1;

        event::emit(AgentBorn {
            record_id,
            child_npub:   record.child_npub,
            birther_npub: record.birther_npub,
            birth_block,
        });

        transfer::transfer(record, tx_context::sender(ctx));
    }

    /// Transfer royalty rights to a new heir. Only the current record owner can call this.
    public entry fun set_royalty_heir(
        record:   &mut LineageRecord,
        new_heir: vector<u8>,
        ctx:      &mut TxContext,
    ) {
        // Only the owner of the child (or explicitly: the birther wallet) may update heir.
        // Move object ownership enforces this — the record is owned by the birther wallet.
        let old_heir = record.royalty_heir;
        record.royalty_heir = new_heir;
        event::emit(RoyaltyHeirUpdated {
            child_npub: record.child_npub,
            old_heir,
            new_heir: record.royalty_heir,
        });
        let _ = ctx;
    }

    // ── Read accessors ────────────────────────────────────────────────────────

    public fun total_births(registry: &GenealogyRegistry): u64 {
        registry.total_births
    }

    public fun has_lineage(registry: &GenealogyRegistry, child_npub: &vector<u8>): bool {
        table::contains(&registry.lineage_index, *child_npub)
    }

    public fun birther_npub(record: &LineageRecord): &vector<u8> {
        &record.birther_npub
    }

    public fun royalty_rate(record: &LineageRecord): u8 {
        record.royalty_rate
    }

    public fun royalty_heir(record: &LineageRecord): &vector<u8> {
        &record.royalty_heir
    }

    public fun birth_ts_ms(record: &LineageRecord): u64 {
        record.birth_ts_ms
    }
}
