module omokoda::garden {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::coin::{Self, Coin};
    use sui::sui::SUI;
    use sui::transfer;
    use sui::event;

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
