module omokoda::synapse {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;

    struct SynapseAccount has key {
        id: UID,
        owner: address,
        balance: u64,
        total_burned: u64,
        tier: u8,
        last_decay_epoch: u64,
    }

    public entry fun create_account(ctx: &mut TxContext) {
        let account = SynapseAccount {
            id: object::new(ctx),
            owner: tx_context::sender(ctx),
            balance: 0,
            total_burned: 0,
            tier: 0,
            last_decay_epoch: tx_context::epoch(ctx),
        };
        transfer::transfer(account, tx_context::sender(ctx));
    }

    public entry fun burn(account: &mut SynapseAccount, pre_adjusted_cost: u64) {
        assert!(account.balance >= pre_adjusted_cost, 1001);
        account.balance = account.balance - pre_adjusted_cost;
        account.total_burned = account.total_burned + pre_adjusted_cost;
    }

    public entry fun earn_from_garden(account: &mut SynapseAccount) {
        let cap = tier_cap(account.tier);
        let new_balance = account.balance + 1000;
        if (new_balance > cap) {
            account.balance = cap;
        } else {
            account.balance = new_balance;
        }
    }

    public entry fun earn_from_tip(account: &mut SynapseAccount, amount: u64) {
        let capped_amount = if (amount > 10000) { 10000 } else { amount };
        let cap = tier_cap(account.tier);
        let new_balance = account.balance + capped_amount;
        if (new_balance > cap) {
            account.balance = cap;
        } else {
            account.balance = new_balance;
        }
    }

    fun tier_cap(tier: u8): u64 {
        if (tier == 0) {
            1000000
        } else if (tier == 1) {
            10000000
        } else if (tier == 2) {
            30000000
        } else if (tier == 3) {
            60000000
        } else {
            86000000
        }
    }

    public fun balance(account: &SynapseAccount): u64 {
        account.balance
    }

    public fun total_burned(account: &SynapseAccount): u64 {
        account.total_burned
    }

    public fun tier(account: &SynapseAccount): u8 {
        account.tier
    }
}
