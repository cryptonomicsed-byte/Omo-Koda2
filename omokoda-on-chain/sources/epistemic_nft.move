module omokoda::epistemic_nft {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use omokoda::consensus_ledger;

    /// An NFT minted when wisdom ensemble reaches SEVERE disagreement.
    /// Permanent artifact of an epistemically contested decision.
    struct EpistemicNft has key, store {
        id: UID,
        agent_id: vector<u8>,
        question_hash: vector<u8>,
        disagreement_score: u64,
        model_count: u64,
        timestamp: u64,
    }

    const E_NOT_SEVERE: u64 = 1;

    entry fun mint_on_severe_disagreement(
        agent_id: vector<u8>,
        question_hash: vector<u8>,
        disagreement_score: u64,
        model_count: u64,
        timestamp: u64,
        ctx: &mut TxContext,
    ) {
        let nft = EpistemicNft {
            id: object::new(ctx),
            agent_id,
            question_hash,
            disagreement_score,
            model_count,
            timestamp,
        };
        transfer::transfer(nft, tx_context::sender(ctx));
    }

    public fun agent_id(nft: &EpistemicNft): &vector<u8> { &nft.agent_id }
    public fun disagreement_score(nft: &EpistemicNft): u64 { nft.disagreement_score }
}
