module omokoda::skillforge_audit {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;

    /// Ṣàngó's immutable record of one SkillForge audit decision — the
    /// on-chain leg of the Audit stage. Content-addressed like GlyphIndex
    /// (hashes, never the raw name/URL) so the receipt is a durable proof
    /// a review happened without duplicating the skill manifest on-chain.
    struct AuditReceipt has key, store {
        id: UID,
        skill_name_hash: vector<u8>,  // sha256(forged skill name)
        source_url_hash: vector<u8>,  // sha256(source GitHub URL)
        risk_score: u64,
        requires_review: bool,
        approved: bool,               // final registration outcome
        timestamp: u64,
    }

    entry fun record(
        skill_name_hash: vector<u8>,
        source_url_hash: vector<u8>,
        risk_score: u64,
        requires_review: bool,
        approved: bool,
        timestamp: u64,
        ctx: &mut TxContext,
    ) {
        let receipt = AuditReceipt {
            id: object::new(ctx),
            skill_name_hash,
            source_url_hash,
            risk_score,
            requires_review,
            approved,
            timestamp,
        };
        transfer::transfer(receipt, tx_context::sender(ctx));
    }

    public fun risk_score(r: &AuditReceipt): u64 { r.risk_score }
    public fun approved(r: &AuditReceipt): bool { r.approved }
    public fun requires_review(r: &AuditReceipt): bool { r.requires_review }
}
