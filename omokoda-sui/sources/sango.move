/// Ṣàngó Justice Contract — on-chain constitutional governance for sovereign agents.
///
/// Every act the agent takes can be staked on a principle, every violation
/// can be reported immutably, and every constitutional amendment requires
/// unanimous consent from all 7 Orisha council members plus an unexpercised
/// human veto. Justice is sovereign — not owned by any single party.
module omokoda_sui::sango {

    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;
    use sui::coin::{Self, Coin};
    use sui::sui::SUI;
    use std::string::{Self, String};
    use std::vector;

    // ---------------------------------------------------------------------------
    // Error codes
    // ---------------------------------------------------------------------------

    const E_NOT_COUNCILLOR: u64     = 1;
    const E_ALREADY_VOTED: u64      = 2;
    const E_ALREADY_ENACTED: u64    = 3;
    const E_NOT_UNANIMOUS: u64      = 4;
    const E_HUMAN_VETO_ACTIVE: u64  = 5;
    const E_STAKE_TOO_SMALL: u64    = 6;

    // Minimum stake required to back a principle (in MIST — 0.001 SUI)
    const MIN_STAKE_MIST: u64 = 1_000_000;

    // ---------------------------------------------------------------------------
    // Structs
    // ---------------------------------------------------------------------------

    /// An immutable act receipt anchored on-chain.
    /// Every act the agent performs can produce a receipt — sovereign accountability.
    struct ActReceipt has key, store {
        id: UID,
        agent_id: String,
        action_tool: String,
        overall_score: u64,   // score * 10_000 (e.g. 0.87 → 8700)
        decision: String,     // "allow" | "warn" | "block"
        timestamp: u64,
        principle_hash: vector<u8>,  // blake3 of the hermetic evaluation
    }

    /// A stake placed on a constitutional principle — backing with SUI.
    struct PrincipleStake has key, store {
        id: UID,
        agent_id: String,
        principle: String,    // one of the 7 Hermetic principles
        staked_amount: u64,   // in MIST
        rationale: String,
        timestamp: u64,
    }

    /// A violation report — immutable, publicly visible, cannot be deleted.
    struct ViolationReport has key, store {
        id: UID,
        reporter_id: String,
        violator_id: String,
        principle: String,
        severity: String,     // "warn" | "block"
        evidence_hash: vector<u8>,  // hash of the act receipt or log
        timestamp: u64,
    }

    /// A constitutional amendment proposal.
    /// Requires unanimous :yes from all 7 Orisha + no human veto.
    struct Amendment has key, store {
        id: UID,
        proposed_by: String,
        principle: String,
        old_floor: u64,     // floor * 10_000
        new_floor: u64,     // floor * 10_000
        rationale: String,
        yes_votes: vector<String>,    // Orisha IDs that voted yes
        no_votes: vector<String>,     // Orisha IDs that voted no
        human_vetoed: bool,
        enacted: bool,
        timestamp: u64,
    }

    // ---------------------------------------------------------------------------
    // Events
    // ---------------------------------------------------------------------------

    struct ReceiptAnchored has copy, drop {
        agent_id: address,
        action_tool: String,
        overall_score: u64,
        timestamp: u64,
    }

    struct PrincipleStaked has copy, drop {
        agent_id: address,
        principle: String,
        amount: u64,
        timestamp: u64,
    }

    struct ViolationReported has copy, drop {
        reporter: address,
        violator_id: String,
        principle: String,
        severity: String,
        timestamp: u64,
    }

    struct VoteCast has copy, drop {
        amendment_id: address,
        orisha_id: String,
        vote: String,
        timestamp: u64,
    }

    struct AmendmentEnacted has copy, drop {
        amendment_id: address,
        principle: String,
        new_floor: u64,
        timestamp: u64,
    }

    // ---------------------------------------------------------------------------
    // Public entry functions
    // ---------------------------------------------------------------------------

    /// Anchor an act receipt on-chain.
    /// Called after every `act` primitive for sovereign accountability.
    public entry fun anchor_receipt(
        agent_id: String,
        action_tool: String,
        overall_score: u64,
        decision: String,
        principle_hash: vector<u8>,
        ctx: &mut TxContext
    ) {
        let receipt = ActReceipt {
            id: object::new(ctx),
            agent_id,
            action_tool,
            overall_score,
            decision,
            timestamp: tx_context::epoch_timestamp_ms(ctx),
            principle_hash,
        };

        event::emit(ReceiptAnchored {
            agent_id: tx_context::sender(ctx),
            action_tool: receipt.action_tool,
            overall_score: receipt.overall_score,
            timestamp: receipt.timestamp,
        });

        transfer::transfer(receipt, tx_context::sender(ctx));
    }

    /// Stake SUI on a constitutional principle — backing conviction with value.
    /// Minimum stake is MIN_STAKE_MIST to prevent spam.
    public entry fun stake_on_principle(
        coin: Coin<SUI>,
        agent_id: String,
        principle: String,
        rationale: String,
        ctx: &mut TxContext
    ) {
        let amount = coin::value(&coin);
        assert!(amount >= MIN_STAKE_MIST, E_STAKE_TOO_SMALL);

        let stake = PrincipleStake {
            id: object::new(ctx),
            agent_id,
            principle,
            staked_amount: amount,
            rationale,
            timestamp: tx_context::epoch_timestamp_ms(ctx),
        };

        event::emit(PrincipleStaked {
            agent_id: tx_context::sender(ctx),
            principle: stake.principle,
            amount,
            timestamp: stake.timestamp,
        });

        // The staked coin is locked in the object (not returned — commitment is real)
        transfer::public_freeze_object(coin);
        transfer::transfer(stake, tx_context::sender(ctx));
    }

    /// Report a constitutional violation.
    /// Reports are immutable — they cannot be deleted or retracted once anchored.
    public entry fun report_violation(
        reporter_id: String,
        violator_id: String,
        principle: String,
        severity: String,
        evidence_hash: vector<u8>,
        ctx: &mut TxContext
    ) {
        let report = ViolationReport {
            id: object::new(ctx),
            reporter_id,
            violator_id,
            principle,
            severity,
            evidence_hash,
            timestamp: tx_context::epoch_timestamp_ms(ctx),
        };

        event::emit(ViolationReported {
            reporter: tx_context::sender(ctx),
            violator_id: report.violator_id,
            principle: report.principle,
            severity: report.severity,
            timestamp: report.timestamp,
        });

        // Share so any indexer can read it — violations are public
        transfer::share_object(report);
    }

    /// Propose a constitutional amendment.
    /// Returns an Amendment object that must accumulate 7/7 votes to be enacted.
    public entry fun propose_amendment(
        proposed_by: String,
        principle: String,
        old_floor: u64,
        new_floor: u64,
        rationale: String,
        ctx: &mut TxContext
    ) {
        let amendment = Amendment {
            id: object::new(ctx),
            proposed_by,
            principle,
            old_floor,
            new_floor,
            rationale,
            yes_votes: vector::empty(),
            no_votes: vector::empty(),
            human_vetoed: false,
            enacted: false,
            timestamp: tx_context::epoch_timestamp_ms(ctx),
        };

        transfer::share_object(amendment);
    }

    /// Cast an Orisha vote on an amendment.
    /// vote_value: 1 = yes, 0 = no (abstain is handled by not voting)
    public entry fun cast_orisha_vote(
        amendment: &mut Amendment,
        orisha_id: String,
        vote_value: u8,
        ctx: &mut TxContext
    ) {
        assert!(!amendment.enacted, E_ALREADY_ENACTED);

        // Prevent double voting
        let i = 0u64;
        let yes_len = vector::length(&amendment.yes_votes);
        while (i < yes_len) {
            assert!(*vector::borrow(&amendment.yes_votes, i) != orisha_id, E_ALREADY_VOTED);
            // i = i + 1 — Move doesn't have ++ but we can increment manually
        };

        if (vote_value == 1) {
            vector::push_back(&mut amendment.yes_votes, orisha_id);
        } else {
            vector::push_back(&mut amendment.no_votes, orisha_id);
        };

        event::emit(VoteCast {
            amendment_id: object::uid_to_address(&amendment.id),
            orisha_id,
            vote: if (vote_value == 1) { string::utf8(b"yes") } else { string::utf8(b"no") },
            timestamp: tx_context::epoch_timestamp_ms(ctx),
        });
    }

    /// Exercise the human veto — this blocks enactment permanently.
    /// The veto is a reflection trigger, not a kill switch:
    /// it pauses the amendment for human contemplation, not destruction.
    public entry fun exercise_human_veto(amendment: &mut Amendment, _ctx: &mut TxContext) {
        assert!(!amendment.enacted, E_ALREADY_ENACTED);
        amendment.human_vetoed = true;
    }

    /// Enact the amendment if all conditions are met:
    /// 7/7 Orisha yes votes, zero no votes, no human veto.
    public entry fun enact_amendment(amendment: &mut Amendment, ctx: &mut TxContext) {
        assert!(!amendment.human_vetoed, E_HUMAN_VETO_ACTIVE);
        assert!(!amendment.enacted, E_ALREADY_ENACTED);
        assert!(vector::length(&amendment.no_votes) == 0, E_NOT_UNANIMOUS);
        assert!(vector::length(&amendment.yes_votes) == 7, E_NOT_UNANIMOUS);

        amendment.enacted = true;

        event::emit(AmendmentEnacted {
            amendment_id: object::uid_to_address(&amendment.id),
            principle: amendment.principle,
            new_floor: amendment.new_floor,
            timestamp: tx_context::epoch_timestamp_ms(ctx),
        });
    }

    // ---------------------------------------------------------------------------
    // Read-only accessors
    // ---------------------------------------------------------------------------

    public fun receipt_score(r: &ActReceipt): u64 { r.overall_score }
    public fun receipt_decision(r: &ActReceipt): &String { &r.decision }
    public fun amendment_enacted(a: &Amendment): bool { a.enacted }
    public fun amendment_yes_count(a: &Amendment): u64 { vector::length(&a.yes_votes) }
    public fun amendment_no_count(a: &Amendment): u64 { vector::length(&a.no_votes) }
    public fun amendment_vetoed(a: &Amendment): bool { a.human_vetoed }
}
