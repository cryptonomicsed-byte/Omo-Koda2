/// Synapse: per-agent metabolic energy token (internal, non-transferable).
/// Synapse is NOT a public token — it is a private life-energy resource.
/// No Àṣẹ token exists. SUI is the external payment rail; Synapse is internal metabolism.
module omokoda::synapse {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;

    // --- Errors ---
    const E_INSUFFICIENT_BALANCE: u64 = 1;
    const E_CAP_EXCEEDED: u64 = 2;
    const E_UNAUTHORIZED: u64 = 3;

    // --- Events ---
    struct BurnEvent has copy, drop {
        owner: address,
        amount: u64,
        total_burned: u64,
    }

    struct EarnEvent has copy, drop {
        owner: address,
        amount: u64,
        source: vector<u8>,
    }

    /// Per-agent Synapse balance account.
    /// Non-transferable: uses `transfer::transfer` (not `public_transfer`),
    /// because SynapseAccount does NOT have the `store` ability.
    struct SynapseAccount has key {
        id: UID,
        owner: address,
        balance: u64,
        total_burned: u64,
        tier: u8,
        last_decay_epoch: u64,
    }

    /// Create a new SynapseAccount for the transaction sender.
    /// Each agent gets exactly one account at birth.
    public entry fun create_account(ctx: &mut TxContext) {
        let owner = tx_context::sender(ctx);
        let account = SynapseAccount {
            id: object::new(ctx),
            owner,
            balance: 10_000,
            total_burned: 0,
            tier: 0,
            last_decay_epoch: 0,
        };
        transfer::transfer(account, owner);
    }

    /// Atomically burn `pre_adjusted_cost` Synapse.
    /// The caller has already applied any tier efficiency multiplier.
    /// This function never re-applies the multiplier — burn is non-refundable.
    public entry fun burn(account: &mut SynapseAccount, pre_adjusted_cost: u64, ctx: &TxContext) {
        assert!(tx_context::sender(ctx) == account.owner, E_UNAUTHORIZED);
        assert!(account.balance >= pre_adjusted_cost, E_INSUFFICIENT_BALANCE);
        account.balance = account.balance - pre_adjusted_cost;
        account.total_burned = account.total_burned + pre_adjusted_cost;
        event::emit(BurnEvent {
            owner: account.owner,
            amount: pre_adjusted_cost,
            total_burned: account.total_burned,
        });
    }

    /// Earn 1000 Synapse from a Garden interaction, capped at tier ceiling.
    public entry fun earn_from_garden(account: &mut SynapseAccount, ctx: &TxContext) {
        assert!(tx_context::sender(ctx) == account.owner, E_UNAUTHORIZED);
        let cap = tier_cap(account.tier);
        let new_balance = account.balance + 1_000;
        if (new_balance > cap) {
            account.balance = cap;
        } else {
            account.balance = new_balance;
        };
        event::emit(EarnEvent {
            owner: account.owner,
            amount: 1_000,
            source: b"garden",
        });
    }

    /// Earn Synapse from a tip, capped at 10,000 per event and at the tier ceiling.
    public entry fun earn_from_tip(account: &mut SynapseAccount, amount: u64, ctx: &TxContext) {
        assert!(tx_context::sender(ctx) == account.owner, E_UNAUTHORIZED);
        let clamped = if (amount > 10_000) { 10_000 } else { amount };
        let cap = tier_cap(account.tier);
        let new_balance = account.balance + clamped;
        if (new_balance > cap) {
            account.balance = cap;
        } else {
            account.balance = new_balance;
        };
        event::emit(EarnEvent {
            owner: account.owner,
            amount: clamped,
            source: b"tip",
        });
    }

    /// Update the tier when reputation crosses a threshold.
    /// Only called by the Justice module after a verified tier promotion.
    public entry fun update_tier(account: &mut SynapseAccount, new_tier: u8, ctx: &TxContext) {
        assert!(tx_context::sender(ctx) == account.owner, E_UNAUTHORIZED);
        assert!(new_tier <= 5, E_CAP_EXCEEDED);
        account.tier = new_tier;
        // Enforce new cap immediately
        let cap = tier_cap(new_tier);
        if (account.balance > cap) {
            account.balance = cap;
        };
    }

    // --- Read-only views ---

    public fun balance(account: &SynapseAccount): u64 {
        account.balance
    }

    public fun total_burned(account: &SynapseAccount): u64 {
        account.total_burned
    }

    public fun tier(account: &SynapseAccount): u8 {
        account.tier
    }

    // --- Internal helpers ---

    /// Maximum Synapse balance per tier.
    /// T0: 1M, T1: 10M, T2: 30M, T3: 60M, T4+: 86M (= total supply per agent cap)
    fun tier_cap(tier: u8): u64 {
        if (tier == 0) {
            1_000_000
        } else if (tier == 1) {
            10_000_000
        } else if (tier == 2) {
            30_000_000
        } else if (tier == 3) {
            60_000_000
        } else {
            86_000_000
        }
    }
}
