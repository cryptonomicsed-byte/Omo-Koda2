/// Sui-native port of OSOVM's GENESIS_FLAW_TOKEN invariant
/// (`src/opcodes.jl` 0x2b, `op_genesis_flaw` in `src/vm_core.jl`): a single,
/// once-ever entitlement mintable only at genesis. OSOVM's Julia VM enforces
/// this with a runtime `genesis_flaw_used` boolean flag checked per-call.
/// Sui Move has a stronger, idiomatic equivalent: a one-time capability
/// object created exactly once in `init()` and consumed (destroyed) by the
/// mint call -- the type system makes a second mint structurally
/// impossible, not just runtime-guarded.
///
/// Deliberately does NOT mint a fictional "Àṣẹ" token -- per synapse.move's
/// own canon ("No Àṣẹ token exists. SUI is the external payment rail;
/// Synapse is internal metabolism"), this module mints a standalone
/// soulbound marker object instead, matching OSOVM's real precedent of
/// 1440 soulbound entitlement tokens (`src/flaw_tokens.jl`) rather than
/// inventing new token economics here.
module omokoda::genesis_flaw {
    use sui::object::{Self, UID, ID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;

    /// Consumed (destroyed) by the one and only mint call. Created once in
    /// `init`, so a second mint is a compile-time-impossible operation, not
    /// a runtime check that could be bypassed by a bug.
    struct GenesisFlawCap has key { id: UID }

    /// Soulbound (no `store` ability -- cannot be transferred after mint).
    /// Matches OSOVM's real "1440 soulbound entitlement tokens" precedent.
    struct GenesisFlawToken has key {
        id: UID,
        recipient: address,
        minted_at_epoch: u64,
    }

    struct GenesisFlawMinted has copy, drop {
        token_id: ID,
        recipient: address,
        minted_at_epoch: u64,
    }

    /// Module initializer -- runs exactly once, at publish. Whoever
    /// publishes the package receives the one-time cap.
    fun init(ctx: &mut TxContext) {
        transfer::transfer(
            GenesisFlawCap { id: object::new(ctx) },
            tx_context::sender(ctx),
        );
    }

    /// The one and only genesis mint. `cap` is consumed by value -- there
    /// is no code path that can call this twice, by construction.
    public entry fun mint(cap: GenesisFlawCap, recipient: address, ctx: &mut TxContext) {
        let GenesisFlawCap { id } = cap;
        object::delete(id);

        let epoch = tx_context::epoch(ctx);
        let token = GenesisFlawToken {
            id: object::new(ctx),
            recipient,
            minted_at_epoch: epoch,
        };
        event::emit(GenesisFlawMinted {
            token_id: object::id(&token),
            recipient,
            minted_at_epoch: epoch,
        });
        transfer::transfer(token, recipient);
    }

    public fun recipient(token: &GenesisFlawToken): address { token.recipient }
    public fun minted_at_epoch(token: &GenesisFlawToken): u64 { token.minted_at_epoch }

    #[test_only]
    public fun test_init(ctx: &mut TxContext) { init(ctx) }

    #[test]
    fun test_genesis_mint_once() {
        use sui::test_scenario;

        let publisher = @0xA11CE;
        let recipient = @0xB0B;

        let scenario = test_scenario::begin(publisher);
        {
            test_init(test_scenario::ctx(&mut scenario));
        };

        test_scenario::next_tx(&mut scenario, publisher);
        {
            let cap = test_scenario::take_from_sender<GenesisFlawCap>(&scenario);
            mint(cap, recipient, test_scenario::ctx(&mut scenario));
        };

        test_scenario::next_tx(&mut scenario, recipient);
        {
            let token = test_scenario::take_from_sender<GenesisFlawToken>(&scenario);
            assert!(recipient(&token) == recipient, 0);
            test_scenario::return_to_sender(&scenario, token);
        };

        test_scenario::end(scenario);
    }
}
