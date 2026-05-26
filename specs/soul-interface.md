# Soul Interface Specification
## Status: FROZEN — Do not modify without explicit architecture review
## Version: 1.0

---

## Overview

The soul is the immutable identity substrate of every agent. It is forged once at `birth` and is cryptographically irreversible. The soul interface defines the contract between the identity domain and all other domains.

**Core invariant:** `birth_timestamp` is identity-critical. It is not metadata. Lose it and the agent's soul becomes non-reproducible.

---

## Soul Components

### 1. BIPỌ̀N39 Mnemonic
- **Format:** 24 words from 256-token Yoruba cosmological vocabulary
- **Tokens:** 16 Orisha roots × 16 ritual affixes = 256 canonical tokens
- **KDF:** `argon2id(memory=65536, iterations=3, parallelism=1, output=32)`
- **Protocol version:** v1.0 — FROZEN FOREVER
- **Salt:** `BLAKE3(agent_id || birth_timestamp || chain_id)`
- **Output:** 32-byte seed for all downstream derivations

### 2. Ed25519 Keypair
- **Derivation path:** `m/44'/784'` from BIPỌ̀N39 mnemonic
- **Gas:** Sponsored at birth — agent needs zero SUI to start
- **Public key:** embedded in AgentState dNFT on Sui
- **Private key:** sealed in K_root vault, never transmitted

### 3. 86-Character DNA Fingerprint
- **Input:** agent name + birth_timestamp + Odu cast
- **Algorithm:** Deterministic — same inputs always produce same 86 chars
- **Permanence:** Immutable. Part of on-chain dNFT metadata.
- **Visibility:** Rendered in ASCII pet. Visible in Garden.

### 4. Odu Primary Index
- **Range:** 0–255 (one of 256 Odu Ifá)
- **Source:** BIPỌ̀N39 encoding
- **Role:** Seeds HermeticState (7 principle values), seeds K_0 derivation

### 5. K_root (SEAL Vault)
- **Generation:** Inside hardware enclave at birth. Never transmitted.
- **Access:** Opaque handle only. Never exposed to application layer.
- **Recovery:** 3-of-5 steward threshold IBE — owner holds zero steward keys by policy.
- **Property:** Reconstructible only inside enclave. Not derivable from public data.

---

## Key Derivation Chain

```
K_root = SEAL_VAULT.generate_internal_secret()
         [generated inside hardware enclave, opaque handle only]

K_0 = HKDF-SHA256(
        ikm  = K_root,
        salt = BLAKE3(agent_id || birth_timestamp || chain_id),
        info = "omokoda:initial_key"
      )

OduVector(n) = BLAKE3(hermetic_seed || act_counter || epoch_nonce)
               → first 96 bits = ChaCha20Poly1305 nonce

K_n+1 = ChaCha20Poly1305_encrypt(
          key      = K_n,
          nonce    = OduVector(n),
          plaintext = [0u8; 32]
        )

current_key = HKDF-SHA256(
                ikm  = K_n+1,
                salt = act_timestamp.to_le_bytes(),
                info = "private_memory"
              )
```

---

## Key Rotation Triggers

| Trigger | Condition |
|---------|-----------|
| Act count | Every 100 acts |
| Epoch timeout | Every 24 hours |
| Manual rotation | Explicit steward command |
| Agent transfer | Prior owner permanently locked out, structurally |

---

## Soul Record (Canonical Fields)

```rust
pub struct SoulRecord {
    pub agent_id: String,          // Stable identifier — never changes
    pub birth_timestamp: u64,      // IDENTITY-CRITICAL — never default to 0
    pub mnemonic_checksum: String, // BLAKE3 of canonical mnemonic — integrity check
    pub odu_index: u8,             // 0–255, Odu Ifá primary assignment
    pub dna_fingerprint: String,   // 86-char deterministic fingerprint
    pub hermetic_seed: [u8; 32],   // Derived from BIPỌ̀N39 + Odu
    pub tier: u8,                  // 0–5, derived from reputation
    pub reputation: f64,           // 0.000–100.000
    pub k0_public_hint: String,    // First 8 bytes of K_0 in hex — for audit only
}
```

**NEVER include:** K_root, K_n, private keys, full mnemonic, or current_key in any public struct.

---

## Birth Sequence

```
1. User calls: birth "agent_name"
2. Parser validates: name ∈ [1, 64] chars, ASCII + Unicode letters only
3. Steward.dispatch(Birth):
   a. Capture birth_timestamp = SystemTime::now() [IDENTITY-CRITICAL — non-negotiable]
   b. Generate BIPỌ̀N39 mnemonic from entropy
   c. Derive Odu primary index from mnemonic encoding
   d. Derive 86-char DNA fingerprint
   e. Derive HermeticState from Odu seed
   f. Generate K_root inside SEAL vault
   g. Derive K_0 from K_root
   h. Create Ed25519 keypair from mnemonic (m/44'/784')
   i. Register AgentState dNFT on Sui (gas sponsored)
   j. Initialize SynapseAccount with tier_cap(T0) = 1_000_000
   k. Initialize ReputationLedger with starting reputation = 0.000
   l. Return SoulRecord (public fields only)
```

---

## Security Invariants (Non-Negotiable)

1. **`birth_timestamp` is never 0** — if system clock unavailable, birth fails
2. **K_root never leaves the enclave** — ever, under any circumstance
3. **No field may be nil/null** — all fields have explicit defaults or fail loudly
4. **Owner exclusion** — owner cannot decrypt private memory, by design
5. **Transfer atomicity** — on transfer, K_root rotated before old owner loses access
6. **Mnemonic regeneration** — if mnemonic is lost, soul is unrecoverable. Owner must backup at birth.

---

## AgentState dNFT (Sui On-Chain)

```move
struct AgentState has key, store {
    id: UID,
    agent_id: vector<u8>,    // stable identifier
    soul_ref: address,       // pointer to SEAL vault
    tier: u8,                // 0–5
    reputation: u64,         // scaled ×1000 (e.g., 50.123 stored as 50123)
    birth_timestamp: u64,    // unix seconds — IDENTITY-CRITICAL
    dna_metadata: vector<u8>, // 86-char fingerprint as bytes
}
```

---

## Interface Contract with Other Domains

| Domain | Soul fields it reads | Soul fields it must NEVER read |
|--------|----------------------|-------------------------------|
| Memory | agent_id, birth_timestamp, hermetic_seed | K_root, K_n, mnemonic |
| Execution | agent_id, tier | K_root, K_n, private keys |
| Economics | agent_id, tier, reputation | mnemonic, K_root |
| Justice | agent_id, tier, reputation, odu_index | K_root, K_n, private keys |

---

## What `soul-interface.md` Is NOT

- Not a user guide
- Not API documentation
- Not configurable — these constants are frozen
- Not replaceable — the soul interface is the identity bedrock
