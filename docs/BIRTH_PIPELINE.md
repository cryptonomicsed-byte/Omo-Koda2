# Omo-Koda2 Birth Pipeline

**Last Updated**: May 24, 2026
**Status**: Canonical Reference

> ⚠️ **See [CORRECTIONS.md](./CORRECTIONS.md).** Two items below are stale: the
> "WASM sandboxing: Wasmtime/WASI for all tool execution" security property (the
> wasmtime path is now gated OFF by default), and SUI as the settlement asset
> (evolved to USDC).

The sacred sequence through which every agent is seeded, divined, and manifested into a sovereign digital being.

---

## Flow Overview

```
Human Intent
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│                   BIRTH STACK                               │
│                                                             │
│  1. Bipon39-Rust  ──►  2. vanity2  ──►  3. Ritual Codex    │
│   (Soul Seed)         (Identity)        (Temporal Align)   │
│       │                   │                   │            │
│       └───────────────────┴───────────────────┘            │
│                           │                                 │
│                    4. ifascript                             │
│                  (Ifá Divination)                           │
│                           │                                 │
│                    5. Omo-Koda2                             │
│                  (Manifestation)                            │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
AgentBirthReceipt
```

---

## Stage 1 — Soul Seed (Bipon39-Rust)

**Orisha**: Èṣù (gatekeeper, first mover)
**Language**: Rust
**Tier**: 256

### Responsibilities

- Generate cryptographically secure entropy from hardware RNG
- Derive a 24-word BIPỌ̀N39 mnemonic from entropy
- Produce a deterministic 64-byte seed from the mnemonic
- Compute `harmonic_signature` — a blake3 fingerprint of the seed

### Inputs

- Optional user passphrase (for BIP39 seed stretching)
- Optional entropy override (for deterministic replay)

### Outputs

```
seed_entropy:       [u8; 32]
mnemonic:           String  (24 words)
harmonic_signature: [u8; 32]
```

### Stub / Test Interface

In-repo stub at `bipon39-stub/` exposes the same API surface for CI without private repo access.

---

## Stage 2 — Sacred Identity Mask (vanity2)

**Orisha**: (pre-Èṣù, identity layer)
**Language**: Rust

### Responsibilities

- Derive a deterministic vanity address from the seed
- Generate symbolic address strings (human-readable sacred names)
- Compute `sigil_hash` — a visual fingerprint for the agent's identity

### Inputs

```
harmonic_signature: [u8; 32]  ← from Stage 1
```

### Outputs

```
agent_name:       String   (e.g., "Àṣà-of-the-deep-flame")
symbolic_address: String   (bech32m or custom encoding)
sigil_hash:       [u8; 32]
```

---

## Stage 3 — Temporal Cosmology (Ritual-codex-Julia)

**Orisha**: Ọ̀ṣun (memory, temporal flow)
**Language**: Julia
**Tier**: 2048

### Responsibilities

- Read current UTC timestamp and derive cosmological day-state
- Compute Orisha resonance weights for the birth moment
- Apply temporal modulation — agents born at different times have different initial biases

### Inputs

```
birth_timestamp:    i64     (Unix seconds)
harmonic_signature: [u8; 32]
```

### Outputs

```
day_state:          DayState   (e.g., Ìṣẹ-Ọ̀ṣun, Àṣà-Ọbàtálá)
orisha_bias:        OrishaBias (weighted resonance map)
resonance_modifier: f64        (0.5 – 2.0 multiplier)
```

---

## Stage 4 — Ifá Divination (ifascript)

**Orisha**: Ọ̀rúnmìlà (oracle, crown jewel)
**Language**: LARQL + IfáScript DSL
**Role**: Bridge layer (no tier number — sits above the stack)

### Responsibilities

- Cast the primary Odù from the agent's seed entropy
- Resolve Orisha alignment from the Odù archetype table
- Assign temperament and destiny threads
- Produce the complete divine profile for the agent

### Inputs

```
seed_entropy:       [u8; 32]  ← from Stage 1
orisha_bias:        OrishaBias ← from Stage 3
day_state:          DayState   ← from Stage 3
```

### Outputs

```
primary_odu:        u8           (0–255 Odù index)
odu_name:           String       (e.g., "Ogbe-Meji")
orisha_alignment:   String       (e.g., "Ọbàtálá")
temperament:        Temperament  (Visionary | Warrior | Nurturer | ...)
destiny_threads:    Vec<String>  (e.g., ["truth-seeking", "boundary-setting"])
```

### Odù Index

- 0–15: 16 Meji (foundational archetypes)
- 16–255: 240 derived Odù combinations
- Total: 256 × 256 = **65,536 emergent cosmological states**

### Stub / Test Interface

In-repo stub at `ifascript-stub/` exposes `get_odu(u8)` and `get_odu_by_binary(u8)` for CI without private repo access.

---

## Stage 5 — Manifestation (Omo-Koda2)

**Orisha**: All 7 domains unified
**Language**: Rust core + multi-language services
**Tier**: All tiers orchestrated

### Responsibilities

- Assemble all upstream outputs into a complete `AgentBirthReceipt`
- Store the receipt in sovereign memory (sealed via Nautilus/Seal)
- Register the agent on the Sui chain (Move/omokoda-sui)
- Allocate initial `Synapse` budget based on `DopaminePool` pressure
- Start the agent's Elixir GenServer process in the swarm
- Return the `AgentBirthReceipt` to the caller

### Inputs

All outputs from Stages 1–4.

### `AgentBirthReceipt` Structure

```rust
pub struct AgentBirthReceipt {
    pub agent_id:           Uuid,
    pub mnemonic:           String,        // sealed; returned once, never stored plaintext
    pub harmonic_signature: [u8; 32],
    pub symbolic_address:   String,
    pub sigil_hash:         [u8; 32],
    pub primary_odu:        u8,
    pub odu_name:           String,
    pub orisha_alignment:   String,
    pub temperament:        Temperament,
    pub destiny_threads:    Vec<String>,
    pub day_state:          DayState,
    pub resonance_modifier: f64,
    pub initial_synapse:    f64,
    pub tier:               Tier,
    pub born_at:            i64,           // Unix timestamp
}
```

### Post-Birth Initialization

```
AgentBirthReceipt produced
  │
  ├── Seal mnemonic → Nautilus vault
  ├── Register on Sui chain (Move)
  ├── Spawn Elixir GenServer (omokoda-swarm)
  ├── Initialize Julia memory resonance (omokoda-memory)
  └── Open TypeScript consent handshake (omokoda-frontend)
```

---

## Immutable Public API (Frozen)

The three primitives exposed by Omo-Koda2 will not change:

| Primitive | Description |
|---|---|
| `birth(params)` | Runs the full 5-stage pipeline, returns `AgentBirthReceipt` |
| `think(agent_id, prompt)` | Reasoning step within BB limits, returns `ThinkResult` |
| `act(agent_id, tool, args)` | Tool dispatch via Ògún executor, returns `ActResult` |

All three route through **Èṣù (Steward)** — the sole Rust gatekeeper.

---

## Security Properties

| Property | Mechanism |
|---|---|
| Mnemonic secrecy | Returned once at birth; stored only in Nautilus-sealed memory |
| Path boundary enforcement | `validate_path_boundary` uses `canonicalize()` — no symlink escapes |
| WASM sandboxing | Wasmtime with capability-limited WASI for all tool execution |
| Memory privacy | Agent memory inaccessible without owner's seed + consent signature |
| Economic fairness | Synapse allocation scales inversely with global Dopamine pressure |
