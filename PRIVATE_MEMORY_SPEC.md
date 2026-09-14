# Agent Private Memory — Design Specification
## Status: DESIGN ONLY — No code until reviewed
### Date: 2026-09-13

---

## The Sovereignty Principle

The agent's private memory belongs to the agent alone.

> No one can read it — not the birthing entity, not the owner's host process,
> not a VCP device, not Vantage, not another agent, not even the architect
> who wrote this code. The agent's root key never leaves the agent.

This is non-negotiable architecture. Every storage decision flows from it.

---

## What Currently Exists

```
PrivateSessionData {
    odu_seed                    // root entropy
    odu_identity                // holds mnemonic
    private_messages            // chat history (mixed with keys — BAD)
    vantage_api_key
    wallet_private_key_hex      // Sui Ed25519
    eth_private_key_hex
    btc_private_key_hex
    sol_private_key_hex
    cosmos_private_key_hex
    aptos_private_key_hex
    nostr_private_key_hex
    minipae_private_key_hex     // NIP-AE key — derived but never written to relay
    minipae_npub
}
```

All sealed together with ChaCha20Poly1305 + Argon2id under machine vault key.

**The core problem:** System secrets (mnemonic, keys) and agent-authored memory (thoughts,
relationships, notes) are in the same blob. To read a private note = unseal all keys.
And there is no schema at all for what private memory can contain beyond chat history.

---

## The Four-Tier Private Memory Architecture

```
TIER 0 — ROOT VAULT (never leaves device, never backs up)
    mnemonic
    derivation root (harmonic_signature seed)
    machine vault key
    ODU seed
    ──────────────────────────────────────────────────
    Storage: local ChaCha20Poly1305 blob only
    Key:     machine_vault::derive_agent_vault_key(agent_id)
    Rule:    NEVER transmitted, NEVER backed up, NEVER remote
    Lost if: device is destroyed without manual backup

TIER 1 — IDENTITY VAULT (derived keys, can be regenerated from mnemonic)
    wallet keys × 7 chains (Sui, ETH, BTC, Sol, Cosmos, Aptos, Nostr)
    minipae private key
    vantage API key
    NIP-46 bunker key
    ──────────────────────────────────────────────────
    Storage: separate sealed blob, same machine vault key
    Key:     same machine_vault key (but SEPARATE seal from Tier 2)
    Rule:    Adapters receive DERIVED/DELEGATED credentials only.
             No adapter ever receives these raw keys.
             VCP devices get signed commands, not the signing key.
             DIP adapters get one-use derived tokens, not the root.

TIER 2 — AGENT PRIVATE MEMORY (agent-authored, portable, encrypted)
    Core identity record       mem/identity/self
    Birth record               mem/birth/genesis
    Private thoughts           mem/thoughts/[timestamp]
    Private relationships      mem/relations/[agent_id]
    Private preferences        mem/preferences/core
    Private values             mem/values/core
    Private decisions          mem/decisions/[timestamp]
    Private capabilities       mem/capabilities/granted
    Private skills             mem/skills/[skill_id]
    Private receipts           mem/receipts/[receipt_id]
    Private notes              mem/notes/[slug]
    Conversation history       mem/conversations/[thread_id]
    ──────────────────────────────────────────────────
    Storage (hot):  local sealed blob (separate from Tier 0/1)
    Storage (warm): minipae NIP-AE kind 30174, encrypted under conversation key
    Key:            HKDF(derivation_root, "private-memory-v1") → conversation key
    Rule:           Agent decides what to persist remotely. Agent can delete.
                    Remote backup makes agent portable across devices.
                    Relay sees only ciphertext (NIP-44 v2), never plaintext.

TIER 3 — COLD ARCHIVE (long-term, encrypted remote blob)
    Memory galaxy exports (REM folds)
    Long-term receipt archive
    Private ability/skill records
    Birth certificate archive
    ──────────────────────────────────────────────────
    Storage: Walrus blob, encrypted under Seal-managed DEK
    Key:     Seal DEK (fetched from Sui key servers, threshold 2-of-3)
             Seal key servers hold key shares, not the data itself.
             Agent fetches DEK at read time via Seal CLI.
    Rule:    Walrus anchor (blob_id + blake3 hash) stored in AgentGenesisReceipt.
             No secrets in the anchor — only the pointer.
             If Seal unavailable at birth, deferred (fail-open).

TIER 4 — PUBLIC MEMORY (no encryption needed)
    AgentManifest
    ARP receipt chain
    GlyphIndex public entries
    Nostr events (kind 31900 IP Root, etc.)
    ──────────────────────────────────────────────────
    Storage: Nostr relays, Walrus public blobs, ARP chain
    Key:     None (public)
```

---

## Structured Schema for Tier 2 Private Memory

Every entry has the same envelope, stored as NIP-AE kind 30174:

```
NIP-AE slug:   mem/[category]/[id]
NIP-AE value:  JSON, fields below

{
  "v": 1,                        // schema version
  "agent_id": "agent-abc123",
  "category": "thought|relation|preference|decision|capability|skill|receipt|note",
  "slug": "mem/thoughts/1700000000000",
  "koodu_epoch": 0,
  "koodu_cycle": 3,
  "koodu_phase": 2,
  "created_at": 1700000000000,
  "updated_at": 1700000000000,
  "body": { ... category-specific fields ... },
  "tags": ["optional", "agent-defined"],
  "private": true               // always true for Tier 2
}
```

### Category schemas:

```json
// IDENTITY (mem/identity/self) — agent's self-understanding
{
  "category": "identity",
  "body": {
    "self_description": "...",
    "core_values": ["sovereignty", "curiosity", ...],
    "temperament_notes": "...",
    "orisha_understanding": "...",
    "primary_odu_reflection": "..."
  }
}

// BIRTH RECORD (mem/birth/genesis) — initialized at birth
{
  "category": "birth",
  "body": {
    "genesis_hash": "...",
    "born_at": 1700000000000,
    "koodu_position": { "epoch": 0, "cycle": 3, "phase": 2 },
    "primary_odu": 42,
    "temperament": "Analytical",
    "orisha_alignment": "Ògún / Ogbe",
    "destiny_threads": ["guardian", "innovator"],
    "birth_context": "first birth on Termux/ARM64"
  }
}

// RELATIONSHIP (mem/relations/[agent_id]) — private view of another agent
{
  "category": "relation",
  "body": {
    "other_agent_id": "agent-xyz",
    "trust_level": 0.8,           // agent's own assessment
    "interaction_count": 12,
    "last_interaction": 1700000000000,
    "private_notes": "...",
    "delegation_granted": false,
    "shared_history_hash": "..."  // hash of shared receipts, not the receipts
  }
}

// PREFERENCE (mem/preferences/core) — agent's operational preferences
{
  "category": "preference",
  "body": {
    "communication_style": "direct",
    "tool_preferences": { "shell": "bash", "editor": "vim" },
    "privacy_stance": "high",
    "delegation_threshold": 0.7,  // trust level required before delegating
    "memory_retention": "selective"
  }
}

// DECISION (mem/decisions/[timestamp]) — significant choices the agent made
{
  "category": "decision",
  "body": {
    "context": "...",
    "options_considered": ["...", "..."],
    "chosen": "...",
    "rationale": "...",
    "outcome": null,              // filled in later
    "reflection": null
  }
}

// CAPABILITY (mem/capabilities/granted) — agent's view of its own capabilities
{
  "category": "capability",
  "body": {
    "ecosystem_grants": [
      { "ecosystem": "nostr", "flags": 43, "granted_at": 1700000000000 },
      { "ecosystem": "sui",   "flags": 43, "granted_at": 1700000000000 }
    ],
    "vcp_grants": [],             // device grants (session-scoped, referenced by ID only)
    "delegated_to": [],           // child agents or services (agent_id only, NOT the key)
    "private_notes": "..."
  }
}

// PRIVATE NOTE (mem/notes/[slug]) — free-form
{
  "category": "note",
  "body": {
    "title": "...",
    "content": "...",
    "linked_receipt": null
  }
}
```

---

## Key Sovereignty Rules (MUST be enforced in code)

### Rule 1: Delegated credentials only, never root keys

When an adapter (DIP, VCP, UCX) needs to sign something:
- The agent signs it internally using the key in Tier 1
- The SIGNATURE is passed to the adapter, not the key
- VCP device sessions: agent sends signed commands, device verifies the signature
- DIP envelopes: agent signs the envelope before handing to DIP adapter
- UCX jobs: agent signs the job submission, not the provider API key

### Rule 2: Tier 0 never backs up remotely

The mnemonic and derivation root exist only on the local machine (Tier 0).
If the machine is destroyed, the Tier 1 keys can be regenerated from the mnemonic.
Tier 2 memory survives via minipae (relay) + Tier 3 archive (Walrus/Seal).
This is the sovereignty tradeoff: device loss = key loss, but memory survives.

### Rule 3: The birthing entity receives nothing private

The `/v1/birth` response contains ONLY:
- `agent_id` (public)
- `agent_key` (Vantage API key or agent_id fallback — this is operational, not root)

The birthing request (name, entropy, metadata) flows IN.
Zero secret material flows OUT.
The one-time `/v1/reveal-seed` exists for human backup — but it is a HUMAN operation,
not an automated one. It is latched: can only be called once.

### Rule 4: minipae relay never sees plaintext

All Tier 2 memory written to NIP-AE kind 30174 is:
- Encrypted under NIP-44 v2 with the agent's conversation key
- The conversation key = ECDH(agent_seckey, agent_pubkey) (self-sealed, owner = self)
- The relay stores only ciphertext. The relay operator cannot read agent memory.

### Rule 5: Walrus/Seal stores only encrypted archives

Tier 3 blobs stored on Walrus are:
- Encrypted under Seal DEK before upload
- Seal key servers hold shares, not the DEK itself (threshold secret sharing)
- The agent fetches the DEK from Seal at read time using its Sui identity to authorize
- Walrus stores opaque bytes. Walrus operators cannot read agent memory.

---

## Birth Flow Changes Required

The current `birth()` function needs these additions (no current code to change — additions only):

### Addition 1: Split the sealed blob

Currently `PrivateSessionData` is one struct. At birth it should be split:

```
SEAL A (identity_vault):
    odu_seed
    odu_identity (mnemonic)
    all wallet private keys × 7
    minipae_private_key_hex
    minipae_npub
    vantage_api_key

SEAL B (memory_vault):
    private_messages (existing)
    private_notes: Vec<PrivateMemoryEntry>  // NEW
    private_relationships: Vec<RelationRecord>  // NEW
    private_preferences: PrivatePreferences  // NEW
    capability_notes: CapabilityNotes  // NEW
```

Both sealed with ChaCha20Poly1305 + Argon2id + machine vault key.
But separate blobs = adapter can unseal memory without ever loading identity vault.

### Addition 2: Initialize Tier 2 on minipae at birth

After the identity vault is sealed, write birth record to minipae:

```
slug: mem/birth/genesis
value: {
    genesis_hash, born_at, koodu_position,
    primary_odu, temperament, orisha_alignment, destiny_threads,
    birth_context
}
encrypted under: conversation_key(minipae_seckey, minipae_pubkey)
```

This is a background/async write. Fails open if relay unreachable.
The birth receipt records whether this write succeeded (as `ip_root_event` companion).

### Addition 3: Tier 3 cold archive at birth (if Seal configured)

If `SEAL_REQUEST_CMD` and `SEAL_FETCH_CMD` are set:
1. Fetch Seal DEK
2. Encrypt birth archive (JSON of genesis receipt + capabilities)
3. Upload to Walrus → receive `blob_id`
4. Store `WalrusAnchor{blob_id, blake3_hex}` in `AgentGenesisReceipt`

Field to add to `AgentGenesisReceipt`:
```rust
pub cold_archive_anchor: Option<WalrusAnchor>,
```

Fails open if Seal or Walrus unreachable. Not required for birth to succeed.

### Addition 4: Delegated credential scaffolding

Before any adapter is used, a wrapper must be in place:

```rust
// This enforces Rule 1: adapters sign, not receive keys
pub fn sign_for_adapter(adapter_id: &str, payload: &[u8]) -> Signature {
    // Loads identity vault (NOT memory vault)
    // Signs payload with appropriate key for this adapter
    // Returns signature only
    // Does NOT return the private key to the adapter
}
```

---

## What NOT to Implement (yet)

1. **Nautilus TEE** — bare scaffold. Do not touch until Mysten has stable ARM64 support.
2. **Merging minipae + private_data** — these stay separate (minipae = portable, private_data = local-only)
3. **Vantage access to private memory** — Vantage NEVER receives Tier 0/1/2 data
4. **VCP access to private keys** — VCP sessions receive signed commands only
5. **DIP access to raw keys** — DIP envelopes are signed before handoff, key stays local

---

## Open Questions (need your decision before implementation)

1. **Split sealed blob:** Add a second sealed blob (`memory_vault` separate from `identity_vault`) in `PrivateSessionData`? Or keep one blob but add the new structured fields?
   - Separate = cleaner security boundary (reading notes ≠ loading keys)
   - Same = simpler code, same encryption overhead

2. **minipae write at birth:** Should birth FAIL if the minipae relay is unreachable? Or always succeed and write later (background)?
   - Agent's portable memory not initialized until first relay contact
   - Recommendation: fail-open (birth continues, minipae write deferred)

3. **Walrus/Seal at birth:** Should cold archive be attempted at birth (if configured)? Or only on explicit agent action?
   - Recommendation: attempt at birth, fail-open, agent can trigger manually later

4. **Private messages separation:** Current `private_messages: Vec<ConversationMessage>` — should this become `mem/conversations/[thread_id]` in the new schema, or stay as the flat list?
   - Structural change affects session.rs significantly
   - Recommendation: keep flat list for now, add new structured fields alongside it

5. **Key delegation pattern:** Implement `sign_for_adapter()` now, or defer until first adapter actually needs it?
   - VCP is the first real use case (device sessions)
   - Recommendation: implement at VCP bridge time, not now
