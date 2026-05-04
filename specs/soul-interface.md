# Soul Interface Specification (v1.0 — FROZEN)

## Purpose

Defines the soul.move object fields and entry functions.
This spec must be written before any identity or memory code is built.
All other contracts (agent.move, garden.move, hive.move) reference soul.move.

## soul.move Object

```
SoulState {
  id:                    UID,
  agent_id:              vector<u8>,   // unique agent identifier
  odu_seed_commitment:   vector<u8>,   // BLAKE3(K_root || odu_index) — public anchor
  hermetic_seed:         vector<u8>,   // birth seed for HermeticState derivation
  birth_timestamp:       u64,          // identity-critical — never 0, never changed
  dna_fingerprint:       vector<u8>,   // 86-char fingerprint as bytes
}
```

## Entry Functions

```
commit_odu_seed(odu_seed: vector<u8>, hermetic_seed: vector<u8>, ctx: &mut TxContext)
  → creates SoulState object
  → emits SoulForged event
  → birth_timestamp = tx_context::epoch(ctx)
  → MUST NOT be called twice for the same agent_id

transfer_soul(soul: SoulState, new_owner: address, ctx: &mut TxContext)
  → transfers ownership
  → emits SoulTransferred event
  → does NOT rotate K_root (that happens in SEAL vault, not on-chain)
```

## Events

```
SoulForged {
  agent_id:    vector<u8>,
  commitment:  vector<u8>,   // odu_seed_commitment
  timestamp:   u64,
}

SoulTransferred {
  agent_id:    vector<u8>,
  new_owner:   address,
  timestamp:   u64,
}
```

## Invariants

- birth_timestamp is set once at creation and never modified
- odu_seed_commitment is a one-way binding — K_root cannot be derived from it
- hermetic_seed is stored on-chain for auditability and reproducibility
- dry_run is prohibited on all transactions involving soul objects
- soul.move must be deployed before agent.move, garden.move, or hive.move

## What soul.move Does NOT Do

- Does NOT hold K_root (that lives inside SEAL vault enclave)
- Does NOT manage memory (that is Walrus + SEAL)
- Does NOT track reputation or tier (that is agent.move)
- Does NOT handle payments (that is garden.move)
