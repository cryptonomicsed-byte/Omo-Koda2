/// Omo-Koda Sui Move Contract
/// Implements decentralized identity and reputation management on Sui blockchain
/// This contract manages ODU (Oracle Data Units) and agent interactions

module omokoda_sui::odu {

    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::table::{Self, Table};
    use std::string::{String};

    /// ODU Identity structure
    struct ODUIdentity has key, store {
        id: UID,
        /// Unique identifier derived from DNA fingerprint
        dna_fingerprint: vector<u8>,
        /// Agent name
        name: String,
        /// Primary Odu index (0-255)
        primary_odu: u8,
        /// Birth timestamp
        birth_timestamp: u64,
        /// Current reputation score
        reputation: u64,
        /// Current tier level
        tier: u8,
        /// Owner of this identity
        owner: address,
    }

    /// Reputation change record
    struct ReputationRecord has store {
        timestamp: u64,
        old_reputation: u64,
        new_reputation: u64,
        reason: String,
        change_amount: u64,
    }

    /// Global registry for ODU identities
    struct ODURegistry has key {
        id: UID,
        /// Map of DNA fingerprints to ODU identities
        identities: Table<vector<u8>, address>,
        /// Total number of registered identities
        total_count: u64,
    }

    /// Error codes
    const E_ALREADY_REGISTERED: u64 = 1;
    const E_NOT_OWNER: u64 = 2;
    const E_INVALID_REPUTATION: u64 = 3;
    const E_INVALID_TIER: u64 = 4;

    /// Initialize the ODU registry (called once during deployment)
    fun init(ctx: &mut TxContext) {
        let registry = ODURegistry {
            id: object::new(ctx),
            identities: table::new(ctx),
            total_count: 0,
        };
        transfer::share_object(registry);
    }

    /// Create a new ODU identity
    public fun create_identity(
        dna_fingerprint: vector<u8>,
        name: String,
        primary_odu: u8,
        birth_timestamp: u64,
        ctx: &mut TxContext
    ): ODUIdentity {
        let identity = ODUIdentity {
            id: object::new(ctx),
            dna_fingerprint,
            name,
            primary_odu,
            birth_timestamp,
            reputation: 0,
            tier: 0,
            owner: tx_context::sender(ctx),
        };
        identity
    }

    /// Register an ODU identity in the global registry
    public fun register_identity(
        registry: &mut ODURegistry,
        identity: ODUIdentity,
        ctx: &mut TxContext
    ) {
        let fingerprint = identity.dna_fingerprint;
        assert!(!table::contains(&registry.identities, fingerprint), E_ALREADY_REGISTERED);

        let identity_addr = object::id_address(&identity);
        table::add(&mut registry.identities, fingerprint, identity_addr);
        registry.total_count = registry.total_count + 1;

        transfer::transfer(identity, tx_context::sender(ctx));
    }

    /// Update reputation score (only by authorized oracles)
    public fun update_reputation(
        identity: &mut ODUIdentity,
        new_reputation: u64,
        reason: String,
        _ctx: &mut TxContext
    ) {
        // In production, this would require oracle authorization
        identity.reputation = new_reputation;
    }

    /// Get identity information
    public fun get_identity_info(identity: &ODUIdentity): (String, u8, u64, u8, address) {
        (
            identity.name,
            identity.primary_odu,
            identity.birth_timestamp,
            identity.reputation,
            identity.tier,
            identity.owner
        )
    }

    /// Publicly anchor a receipt Merkle root to an identity
    public entry fun anchor_receipt(
        identity: &mut ODUIdentity,
        merkle_root: vector<u8>,
        ctx: &mut TxContext
    ) {
        // Here we could store the root in a table or emit an event
        // For anchoring, emitting an event is sufficient for indexers
        event::emit(ReceiptAnchored {
            identity: object::uid_to_address(&identity.id),
            merkle_root,
            timestamp: tx_context::epoch_timestamp_ms(ctx),
        });
    }

    struct ReceiptAnchored has copy, drop {
        identity: address,
        merkle_root: vector<u8>,
        timestamp: u64,
    }