# GlyphIndex — GIX1 Protocol Specification
## Design Specification — No Code Until Reviewed
### Date: 2026-09-14

---

## Status of Existing Code

Before designing anything, the audit found:

| Component | Location | Status |
|---|---|---|
| `content_hash()` SHA-256 | If-Script `src/glyph/mod.rs` | **EXISTS** — canonical primitive |
| `glyph_fold()` BMP fold | If-Script `src/glyph/mod.rs` | **EXISTS** — 63,422 valid codepoints, frozen test vectors |
| `odu_link()` → (u8, u16) | If-Script `src/glyph/mod.rs` | **EXISTS** — base + composed Odù |
| `GlyphResidue` struct | If-Script `src/glyph/mod.rs` | **EXISTS** — canonical_id/glyph/odu_base/odu_composed/chunk |
| `glyph_fold()` duplicate | larql-glyph `crates/larql-glyph/` | **DUPLICATE** — identical to If-Script, different context |
| `GlyphGraph`, `GlyphNode`, LQL verbs | larql-glyph | **EXISTS** — query adapter only, not canonical GIX |
| `gix1_audit`, `merkle_root`, `GIX1_EMPTY_ROOT` | `glyph_memory.rs` imports | **NOT IMPLEMENTED** — stub import, "not yet wired" |
| `GIX_MAGIC = b"GIX1"`, `GIX_VERSION = 1` | Vantage `glyph_index.py` | **EXISTS** |
| `GlyphKeyring`, `seal_blob`, `open_blob` | Vantage `glyph_index.py` | **EXISTS** — AES-256-GCM |
| `GlyphStore` DB schema | Vantage `glyph_index.py` | **EXISTS** — canonical_id PK, glyph, odu_base/composed, embedding |
| `merkle_root_binary()` | Vantage `glyph_index.py` | **EXISTS** — Python only |
| `AgentGenesisReceipt.birth_memory_glyph` | Omo-Koda2 `genesis/receipt.rs` | **EXISTS** — GIX at birth |
| `memory_root` field | Omo-Koda2 `genesis/receipt.rs` | **EXISTS** — Merkle root at birth |
| GIX1 wire envelope (full: version/namespace/type/provenance/routing) | **NOWHERE** | **DOES NOT EXIST** |
| Cross-system identity rule enforcement | **NOWHERE** | **DOES NOT EXIST** |
| GIX for non-birth objects | **NOWHERE** | **DOES NOT EXIST** |
| Sui anchor pipeline for GIX Merkle root | **NOWHERE** | **DOES NOT EXIST** |

---

## Architectural Decision: ~/GIX/ as Sovereign Crate

**Why a standalone repo, not a larql sub-crate:**

1. All 37 repos need GIX identity types. Making them depend on `larql` to get basic
   identity primitives creates an inappropriate dependency for repos like VCP, DIP, ARP,
   Omo-Koda2 that have no business importing an inference engine.

2. `larql-glyph` is an **adapter** (LQL query layer over glyph metadata), not the canonical
   GIX implementation. It correctly imports the primitives; it should continue to do so
   from `gix-core` once that exists.

3. The ecosystem precedent is clear: shared protocol types → standalone repo.
   VCP, DIP, ARP, UCX all follow this pattern. GIX follows it too.

4. `gix1_audit`, `merkle_root`, `GIX1_EMPTY_ROOT` are already imported by `glyph_memory.rs`
   but don't exist. The correct place to implement them is `gix-core`, not larql-glyph.

**Crate structure:**
```
~/GIX/
├── Cargo.toml                   (workspace)
├── crates/
│   ├── gix-types/               (wire envelope, namespaces, object kinds)
│   └── gix-core/                (primitives: content_hash, glyph_fold, odu_link, merkle)
└── GIX_SPEC.md                  (moves here when ~/GIX/ is created)
```

**Migration plan (no duplication):**
- `If-Script/src/glyph/mod.rs` → keep for If-Script-specific divination context,
  re-export from `gix-core` for the shared primitives
- `larql-glyph` → keep as LQL query adapter, change to depend on `gix-core`
- Vantage `glyph_index.py` → keeps Python implementation for Python ecosystem;
  `gix-types` becomes the Rust canonical, Vantage stays authoritative for social/registry

---

## The Critical Separation (User-Locked Invariant)

**Cryptographic identity is NOT the same as indexed projection.**

```
SHA-256 digest      = cryptographic identity (immutable, collision-resistant)
Odù base (u8)       = 256-state index projection (semantic coordinate)
Odù composed (u16)  = 65,536-state index projection (fine coordinate)
GlyphIndex glyph    = symbolic representation (human/machine readable)
```

These MUST remain distinct in code:

```rust
// WRONG — do not conflate
fn get_identity(content: &str) -> u8 { sha256(content)[0] }  // Odù is NOT the identity

// CORRECT — keep layers separate
struct GlyphResidue {
    canonical_id: [u8; 32],    // cryptographic root — never use as Odù
    glyph: char,               // symbolic representation
    odu_base: u8,              // 256-state projection — derived, not authoritative
    odu_composed: u16,         // 65,536-state projection — derived, not authoritative
}
```

**Why this matters:**
- Odù coordinates are deterministic projections from the digest, not the digest itself
- Two different objects CAN share an Odù coordinate (hash space is 2^256; Odù space is 256/65,536)
- You cannot use an Odù coordinate to reconstruct the canonical_id
- If-Script, OSOVM, and symbolic reasoning operate on Odù; cryptographic verification
  operates on canonical_id. Mixing them breaks both.

---

## Conceptual Model — Full Vision (All Objects)

GIX identifies ANY indexable object in the ecosystem. This section defines the
general model. Phase 1 implements only the wire type + ARP receipts.

### Object kinds that receive GIX identities

```
GixKind enum:
    Agent               -- a born Omo-Koda2 agent
    Birth               -- genesis event
    Memory              -- a private or public memory entry
    Task                -- a job/task in agentic-waggle
    Execution           -- an OSOVM execution run
    Device              -- a VCP device session
    Artifact            -- a Walrus blob, Blossom blob, or Nostr event
    Receipt             -- an ARP ActionReceipt
    Transaction         -- an economic settlement (ASE/Sui)
    Composite           -- a GIX-FOLD of multiple GIX objects
    Namespace           -- a named scope (habitat, guild, ecosystem)
    Skill               -- a mycelium-derived skill
    Hypothesis          -- an If-Script authored rule/hypothesis
```

### GIX identity invariant

```
GIX(x) == GIX(y)
    if and only if
canonical_bytes(x) == canonical_bytes(y)
```

No subsystem gets to redefine identity locally when a GIX identity already exists.
Each system keeps its own database IDs; cross-system references use GIX.

---

## GIX1 Wire Envelope — Phase 1 Normative Definition

This is what DOES NOT EXIST yet and needs to be built.

```rust
// gix-types/src/lib.rs

pub const GIX_MAGIC: [u8; 4] = *b"GIX1";
pub const GIX_VERSION: u8 = 1;

/// GIX1 — the cross-system identity envelope.
///
/// Identifies an object without BEING the object.
/// The canonical_id is the cryptographic root.
/// Odù coordinates are projections — NOT the identity.
pub struct Gix1 {
    pub version: u8,                   // always 1 for GIX1
    pub kind: GixKind,                 // what type of object is this?
    pub namespace: GixNamespace,       // which ecosystem scope?
    pub canonical_id: [u8; 32],        // SHA-256 of canonical bytes — the root
    pub glyph: char,                   // symbolic representation
    pub odu_base: u8,                  // 256-state projection
    pub odu_composed: u16,             // 65,536-state projection
    pub provenance: Option<[u8; 32]>,  // canonical_id of the parent/creator
    pub created_at: u64,               // unix millis
    pub routing: RoutingHints,         // optional: where to find this object
    pub integrity: IntegrityMeta,      // checksum of the envelope itself
}

pub struct RoutingHints {
    pub primary: Option<String>,       // DIP NetworkRepr address (network:address)
    pub fallback: Vec<String>,
}

pub struct IntegrityMeta {
    pub envelope_hash: [u8; 32],       // SHA-256 of all fields except this one
}

pub enum GixKind {
    Agent, Birth, Memory, Task, Execution, Device, Artifact,
    Receipt, Transaction, Composite, Namespace, Skill, Hypothesis,
}

pub enum GixNamespace {
    OmokodaAgent,      // Omo-Koda2 agent space
    VantageRegistry,   // Vantage social/agent layer
    OsovmExecution,    // OSOVM execution space
    ArpReceipt,        // ARP receipt chain
    MeshDevice,        // VCP/habitat device space
    Mycelium,          // mycelium trace/skill space
    IfScript,          // If-Script hypothesis/rule space
    Custom(String),    // extensible
}
```

### GIX-FOLD-v1 (composite identity)

Multiple GIX objects fold into a single composite identity.
Order matters — fold is not commutative.

```rust
// gix-core/src/fold.rs

pub fn gix_fold_v1(inputs: &[[u8; 32]]) -> [u8; 32] {
    // SHA-256 of the concatenated canonical_ids in order
    // Same algorithm as existing glyph_fold, applied at the GIX level
    let mut hasher = Sha256::new();
    for id in inputs {
        hasher.update(id);
    }
    hasher.finalize().into()
}

// Example: Task + Execution + Receipt → Composite GIX
let composite = gix_fold_v1(&[
    task_gix.canonical_id,
    execution_gix.canonical_id,
    receipt_gix.canonical_id,
]);
```

### GIX-KDF-v1 (domain-separated derivation)

Derives domain-specific material from a GIX root.
Prevents a GIX identity from accidentally becoming a key.

```rust
// gix-core/src/kdf.rs

pub enum GixDomain {
    Memory,
    Mesh,
    Receipt,
    AgentKey,
    Encryption,
    Custom(&'static str),
}

pub fn gix_kdf_v1(
    canonical_id: &[u8; 32],
    domain: GixDomain,
    context: &[u8],
) -> [u8; 32] {
    // HKDF-SHA256(ikm=canonical_id, info=domain_label || context)
    // Each domain produces a DIFFERENT key from the same root
}
```

**Critical:** `gix_kdf_v1()` output is never stored as identity. It is key material only.
The GIX `canonical_id` and KDF outputs must NEVER be used interchangeably.

### GIX Merkle root

```rust
// gix-core/src/merkle.rs

pub const GIX1_EMPTY_ROOT: [u8; 32] = [0u8; 32];  // sentinel for empty tree

pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    // Standard binary Merkle tree over sorted canonical_ids
    // Empty input → GIX1_EMPTY_ROOT
}

pub fn gix1_audit(claimed_root: &[u8; 32], leaves: &[[u8; 32]]) -> bool {
    // Recompute root from leaves; return claimed_root == computed_root
    merkle_root(leaves) == *claimed_root
}
```

These are the three functions already imported (but unimplemented) by `glyph_memory.rs`.

---

## Phase 1 Implementation Scope

**What to build:**
```
gix-types:  Gix1 struct, GixKind, GixNamespace, RoutingHints, IntegrityMeta
gix-core:   content_hash(), glyph_fold(), odu_link()      ← consolidate from If-Script
            gix_fold_v1()                                  ← composite identity
            gix_kdf_v1()                                   ← domain separation
            merkle_root(), gix1_audit(), GIX1_EMPTY_ROOT   ← implement the stubs
```

**First consumer — ARP receipts:**
`ArpBridge` (Omo-Koda2 `src/bridge/arp.rs`, gap #6) wraps every `ActionReceipt`
with a `Gix1 { kind: GixKind::Receipt, ... }`. The receipt's canonical bytes are the
ARP receipt serialization. This makes ARP the first production consumer without
baking receipt-specific assumptions into GIX.

**Conformance test vectors (Phase 1):**
- Same frozen vectors as If-Script `glyph/mod.rs`: "Àṣẹ", "hello", "GlyphIndex",
  "😊🚀 Unicode test", "Ọ̀rúnmìlà"
- Verify: `content_hash()` matches, `glyph_fold()` matches, `odu_link()` matches
  across Rust (gix-core) and Python (Vantage glyph_index.py)
- Vectors must produce identical results on ARM64 and x86

---

## What Is NOT Changed

| Component | What stays | Reason |
|---|---|---|
| `If-Script/src/glyph/mod.rs` | Stays — keeps divination/casting context | Re-exports from gix-core for shared primitives |
| `larql-glyph` | Stays — LQL query adapter | Changes to depend on gix-core instead of duplicating primitives |
| `Vantage/glyph_index.py` | Stays — Python canonical for Vantage | Vantage IS the social/registry layer; Python impl is correct here |
| `AgentGenesisReceipt.birth_memory_glyph` | Stays — already using GIX at birth | Will reference gix-core types in Phase 2 |
| `GlyphStore` DB schema | Stays | Vantage owns its own storage |

---

## Phases 2–4 (Full Vision, Not Phase 1)

```
Phase 2 — GIX beyond birth:
    Every ARP ActionReceipt gets a Gix1 wrapper (gap #6 — ArpBridge)
    Every think() and act() primitive produces a Receipt GIX (gaps #23-24)

Phase 3 — Task + Device GIX:
    agentic-waggle WorkKind::Job gets a Task GIX
    VCP DeviceManifest gets a Device GIX
    Agent Habitat resources get a Namespace GIX

Phase 4 — Merkle pipeline + Sui anchor:
    Tree of GIX objects → Merkle root → Zàngbétò → Sui
    Reputation becomes a GIX graph (evidence-derived, not mutable integer)

Phase 5 — Full cross-system identity rule:
    No subsystem creates a new ID for an object that already has a GIX
    Vantage `agent_id` maps to `Gix1 { kind: Agent }` at registration
    VCP session_id maps to `Gix1 { kind: Device }` at handshake grant
```

---

## Reputation as GIX Graph (Phase 4 Vision)

Currently: `agent.reputation = 87` (mutable integer in Vantage)

Phase 4 target:
```
Agent GIX
    │
    ├── 1,492 Receipt GIX (completed tasks)
    │       ├── 1,480 verified (gix1_audit passes)
    │       └── 12 disputed
    ├── 0 forged (Zàngbétò audit clean)
    ├── avg execution score (from OSOVM execution receipts)
    └── settlement history (Transaction GIX chain)
```

Reputation becomes a query over verified GIX history, not a mutation target.
No single system can falsify reputation because each component is independently anchored.

---

## Implementation Order

```
Phase 1 (no blockers):
    Step 1: ~/GIX/ workspace with gix-types/ and gix-core/
    Step 2: Implement primitives in gix-core (consolidate from If-Script + add 3 stubs)
    Step 3: Implement Gix1 wire envelope in gix-types
    Step 4: Conformance test vectors pass on ARM64 + x86
    Step 5: larql-glyph Cargo.toml → depends on gix-core
    Step 6: Omo-Koda2 glyph_memory.rs git dep → local gix-core
    Step 7: ArpBridge wraps ActionReceipt with Gix1 (first consumer)

Phase 2 (after ArpBridge):
    Extend to think/act receipt chain (gaps #23-24)

Phase 3+ (after habitat + device work):
    Task/Device/Namespace GIX

Phase 4 (after full receipt chain):
    Merkle pipeline + Sui anchor + reputation graph
```

---

## Open Questions

None blocked for Phase 1.

One decision deferred: when `glyph_memory.rs` currently pins a git rev
(`rev = "149322efa11a2592b9cbebfbfa91c98d7b2d50a7"`) to larql-glyph for the
`gix1_audit`/`merkle_root`/`GIX1_EMPTY_ROOT` stubs — once `gix-core` implements
them, both `glyph_memory.rs` and `larql-glyph` should switch to depending on `gix-core`.
The git rev pin in `omokoda-core/Cargo.toml` should then be replaced with a path dep
to `~/GIX/crates/gix-core`. This is a mechanical change, not a design question.
