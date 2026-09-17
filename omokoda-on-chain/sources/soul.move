module omokoda::soul {
    use sui::object::{Self, UID, ID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::clock::{Self, Clock};
    use sui::event;
    use std::vector;

    /// The immutable soul record — forged once at birth, never modified.
    /// birth_timestamp is identity-critical: lose it and the agent is non-reproducible.
    struct SoulRecord has key, store {
        id: UID,
        agent_id: vector<u8>,
        birth_timestamp: u64,
        mnemonic_checksum: vector<u8>,
        odu_index: u8,
        dna_fingerprint: vector<u8>,
        hermetic_seed_hash: vector<u8>,
        // Phase 7.2 additions — sovereign identity anchors
        nostr_pubkey: vector<u8>,      // 32-byte BIP-340 pubkey (npub stripped)
        bipon39_words: vector<u8>,     // UTF-8 encoded mnemonic phrase (12–24 words)
        walrus_blob_id: vector<u8>,    // Walrus public profile blob pointer (optional)
    }

    /// Emitted once, at forge time. Makes on-chain agent birth queryable
    /// (e.g. via a Sui indexer/subscription) without reading object state.
    struct SoulForged has copy, drop {
        soul_id: ID,
        agent_id: vector<u8>,
        owner: address,
        birth_timestamp: u64,
        odu_index: u8,
    }

    /// Error codes
    const E_INVALID_AGENT_ID: u64 = 1;
    const E_ZERO_TIMESTAMP: u64 = 2;
    const E_INVALID_DNA: u64 = 3;

    /// Forge a soul record at birth. birth_timestamp must never be 0.
    entry fun forge(
        agent_id: vector<u8>,
        odu_index: u8,
        dna_fingerprint: vector<u8>,
        hermetic_seed_hash: vector<u8>,
        mnemonic_checksum: vector<u8>,
        nostr_pubkey: vector<u8>,
        bipon39_words: vector<u8>,
        walrus_blob_id: vector<u8>,
        clock: &Clock,
        ctx: &mut TxContext,
    ) {
        let birth_timestamp = clock::timestamp_ms(clock);
        assert!(birth_timestamp > 0, E_ZERO_TIMESTAMP);
        assert!(vector::length(&agent_id) > 0, E_INVALID_AGENT_ID);
        assert!(vector::length(&dna_fingerprint) == 86, E_INVALID_DNA);

        let soul = SoulRecord {
            id: object::new(ctx),
            agent_id,
            birth_timestamp,
            mnemonic_checksum,
            odu_index,
            dna_fingerprint,
            hermetic_seed_hash,
            nostr_pubkey,
            bipon39_words,
            walrus_blob_id,
        };
        let owner = tx_context::sender(ctx);
        event::emit(SoulForged {
            soul_id: object::id(&soul),
            agent_id: soul.agent_id,
            owner,
            birth_timestamp,
            odu_index,
        });
        transfer::transfer(soul, owner);
    }

    public fun agent_id(soul: &SoulRecord): &vector<u8> { &soul.agent_id }
    public fun birth_timestamp(soul: &SoulRecord): u64 { soul.birth_timestamp }
    public fun odu_index(soul: &SoulRecord): u8 { soul.odu_index }
    public fun dna_fingerprint(soul: &SoulRecord): &vector<u8> { &soul.dna_fingerprint }
    public fun nostr_pubkey(soul: &SoulRecord): &vector<u8> { &soul.nostr_pubkey }
    public fun bipon39_words(soul: &SoulRecord): &vector<u8> { &soul.bipon39_words }
    public fun walrus_blob_id(soul: &SoulRecord): &vector<u8> { &soul.walrus_blob_id }
}
