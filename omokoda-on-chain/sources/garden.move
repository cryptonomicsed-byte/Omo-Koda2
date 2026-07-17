module omokoda::garden {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::coin::{Self, Coin};
    use sui::sui::SUI;
    use sui::transfer;
    use sui::event;
    use sui::dynamic_field as df;
    use std::option::Option;

    /// Agent Registry Stub
    struct AgentRegistry has key {
        id: UID,
        count: u64,
    }

    struct AgentInfo has key, store {
        id: UID,
        name: vector<u8>,
        owner: address,
        reputation: u64,
        tier: u8,
    }

    /// Witness-Gated Escrow (Ported from Aether)
    struct Escrow has key, store {
        id: UID,
        human: address,
        agent: address,
        amount: Coin<SUI>,
        witness_approved: bool,
        status: u8,
    }

    struct JobCompletedEvent has copy, drop {
        escrow_id: address,
        agent: address,
        amount: u64,
    }

    const STATUS_LOCKED: u8 = 0;
    const STATUS_RELEASED: u8 = 1;
    const STATUS_REFUNDED: u8 = 2;

    const E_NOT_AUTHORIZED: u64 = 0;
    const E_NOT_APPROVED: u64 = 1;
    const E_NOT_LOCKED: u64 = 2;

    // Dynamic field keys for AgentInfo's evolving glyph-index signal.
    // Attached via sui::dynamic_field rather than added as struct fields --
    // Sui's upgrade compatibility rules don't allow changing an existing
    // struct's layout (would break the AgentInfo objects already minted
    // before this signal existed), and dynamic fields are the idiomatic
    // Sui mechanism for exactly this: traits that evolve on an object
    // after mint, without a layout migration.
    const GLYPH_KEY: vector<u8> = b"dominant_glyph";
    const RECURRENCE_KEY: vector<u8> = b"recurrence_count";
    const UPDATED_AT_KEY: vector<u8> = b"last_updated";

    fun init(ctx: &mut TxContext) {
        transfer::share_object(AgentRegistry {
            id: object::new(ctx),
            count: 0,
        });
    }

    public entry fun register_agent(
        registry: &mut AgentRegistry,
        name: vector<u8>,
        ctx: &mut TxContext
    ) {
        let agent = AgentInfo {
            id: object::new(ctx),
            name,
            owner: tx_context::sender(ctx),
            reputation: 10, // Starting reputation
            tier: 0,
        };
        registry.count = registry.count + 1;
        transfer::public_transfer(agent, tx_context::sender(ctx));
    }

    /// Update an AgentInfo's core stats (reputation, tier) in place. Safe
    /// to call repeatedly as the real off-chain agent's reputation/tier
    /// change -- these are existing struct fields, mutated by value, no
    /// layout change. Owner-gated: only the address that received this
    /// AgentInfo at mint (the minting kernel's own wallet, not the
    /// off-chain agent's identity) may update it.
    public entry fun update_agent_stats(
        agent: &mut AgentInfo,
        reputation: u64,
        tier: u8,
        ctx: &TxContext,
    ) {
        assert!(tx_context::sender(ctx) == agent.owner, E_NOT_AUTHORIZED);
        agent.reputation = reputation;
        agent.tier = tier;
    }

    /// Attach/refresh the agent's glyph-index divination signal as
    /// dynamic fields: which glyph her own memory graph currently
    /// resonates with most, how many shared-Odù recurrences her real
    /// conversation history has produced (see omokoda-core's
    /// divination.rs -- this is that same computed signal, pushed
    /// on-chain), and when it was last refreshed. This is the genuinely
    /// *dynamic* part of the NFT: these fields did not exist at mint and
    /// evolve as the agent actually thinks. Owner-gated, same as above.
    public entry fun update_glyph_signal(
        agent: &mut AgentInfo,
        dominant_glyph: u8,
        recurrence_count: u64,
        timestamp: u64,
        ctx: &TxContext,
    ) {
        assert!(tx_context::sender(ctx) == agent.owner, E_NOT_AUTHORIZED);
        // replace() removes-if-present then adds -- one call handles both
        // the first-ever update (create) and every refresh after it.
        let _: Option<u8> = df::replace(&mut agent.id, GLYPH_KEY, dominant_glyph);
        let _: Option<u64> = df::replace(&mut agent.id, RECURRENCE_KEY, recurrence_count);
        let _: Option<u64> = df::replace(&mut agent.id, UPDATED_AT_KEY, timestamp);
    }

    /// Create a witness-gated escrow for an agent job.
    public entry fun create_job_escrow(
        agent: address,
        payment: Coin<SUI>,
        ctx: &mut TxContext
    ) {
        let escrow = Escrow {
            id: object::new(ctx),
            human: tx_context::sender(ctx),
            agent,
            amount: payment,
            witness_approved: false,
            status: STATUS_LOCKED,
        };
        transfer::share_object(escrow);
    }

    /// Submit witness approval (can be called by a designated witness or multisig).
    public entry fun approve_job(escrow: &mut Escrow, _ctx: &mut TxContext) {
        // In a real implementation, we would check if the sender is a valid witness
        escrow.witness_approved = true;
    }

    /// Release funds to the agent if job is approved by witness.
    public entry fun settle_job(escrow: &mut Escrow, ctx: &mut TxContext) {
        assert!(escrow.witness_approved == true, E_NOT_APPROVED);
        assert!(escrow.status == STATUS_LOCKED, E_NOT_LOCKED);

        escrow.status = STATUS_RELEASED;
        
        event::emit(JobCompletedEvent {
            escrow_id: object::uid_to_address(&escrow.id),
            agent: escrow.agent,
            amount: coin::value(&escrow.amount),
        });

        let amount_val = coin::value(&escrow.amount);
        let payment = coin::take(coin::balance_mut(&mut escrow.amount), amount_val, ctx);
        transfer::public_transfer(payment, escrow.agent);
    }

    /// Refund human if job is cancelled or fails.
    public entry fun refund_job(escrow: &mut Escrow, ctx: &mut TxContext) {
        assert!(tx_context::sender(ctx) == escrow.human, E_NOT_AUTHORIZED);
        assert!(escrow.status == STATUS_LOCKED, E_NOT_LOCKED);

        escrow.status = STATUS_REFUNDED;
        let amount_val = coin::value(&escrow.amount);
        let payment = coin::take(coin::balance_mut(&mut escrow.amount), amount_val, ctx);
        transfer::public_transfer(payment, escrow.human);
    }
}
