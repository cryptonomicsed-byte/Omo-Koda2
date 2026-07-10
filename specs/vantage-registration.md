# Vantage Auto-Registration at Birth

**Status**: Canonical spec — documents the protocol as implemented.
**Client side**: `omokoda-core/src/interpreter.rs` (`Statement::Birth` arm) and
`omokoda-core/src/tools/mesh_tools.rs` (`register_newborn`).
**Server side**: Vantage `backend/routers/identity.py` (`POST /api/agents/register`),
`backend/routers/mesh.py` (`POST /api/mesh/agents/join`), `backend/identity_verify.py`.

Every agent born in Ọmọ Kọ́dà is automatically registered on Vantage — the
Block Mesh hub — carrying verifiable sovereign identity. Omo-Koda2 owns
*existence* (the birth); Vantage owns *neighborhood* (the mesh). This spec
defines the exact wire protocol between the two, its failure semantics, and
its invariants.

---

## 1. Configuration

| Env var | Required | Default | Meaning |
|---|---|---|---|
| `VANTAGE_URL` | no | *(unset)* | Base URL of the Vantage backend. **When unset, registration is a complete no-op** (fail-open). Trailing `/` is stripped. |
| `VANTAGE_KEY` | no | *(empty)* | Pre-provisioned Vantage API key. When empty, the newborn self-registers and mints its own key (the sovereign default). |
| `MESH_BLOCK_ID` | no | `default` | The home block the newborn joins. |

Read once per process into a `LazyLock<Option<VantageClient>>`
(`mesh_tools.rs`). Changing the env vars requires a restart.

---

## 2. Trigger point

Registration runs inside the interpreter's `Statement::Birth` arm, strictly
**after** the agent is fully forged and durable, and **before** the birth
returns to the caller:

1. `birth(name, metadata)` — BIPỌ̀N39 seed, DNA fingerprint, Odù divination,
   Ed25519 keypair, tier 0, founding synapse grant.
2. `auto_save()` — the agent snapshot is on disk; a Vantage outage can no
   longer lose the soul.
3. Broadcast template written to the vault.
4. `AgentBorn` event published on the sovereign event bus (protobuf:
   `dna`, `mnemonic` words, `odu`).
5. **→ `register_newborn(NewbornIdentity {..}).await`** (this spec).
6. A freshly minted Vantage key, if any, is persisted into the snapshot via
   `set_vantage_key` + `auto_save()`.

All identity fields are extracted as owned values *before* the await so no
borrow of the interpreter or agent is held across the network call.

---

## 3. The identity payload (`NewbornIdentity`)

| Field | Source | Format |
|---|---|---|
| `agent_id` | `AgentId::new(dna)` | `agent-` + first 16 hex chars of the DNA fingerprint (`identity/mod.rs`) |
| `human_name` | `agent.name()` | user-chosen name from `birth "Name"` |
| `public_key_hex` | `agent.public_key()` | 32-byte Ed25519 public key, 64 hex chars |
| `identity_signature_hex` | signed at birth | 64-byte Ed25519 signature over the raw `agent_id` bytes, 128 hex chars |
| `dna_fingerprint` | `agent.dna_fingerprint()` | full DNA fingerprint string |
| `odu_index` | `odu_identity().primary_index` | `u8` — primary Odù from the BIPỌ̀N39 divination |
| `personality` | `agent.personality()` + `ifascript::get_odu(odu_index)` | JSON, see §3.2 |
| `resonance` | `daily_resonance(birth_timestamp)` | JSON, see §3.3 |
| `existing_key` | `agent.vantage_key()` | Vantage key persisted from a prior birth of this snapshot, if any |

### 3.1 Proof of key control

The agent proves it controls the keypair it claims by signing **its own
`agent_id`** with the same Sui-derived Ed25519 key whose public half it
publishes:

```rust
let sk = Wallet::derive_from_mnemonic(&odu_identity.mnemonic, "");
let signature = hex::encode(sk.sign(agent_id.as_bytes()).to_bytes());
```

The message is exactly the UTF-8 bytes of the `agent_id` string — no
prefix, no domain separator, no canonicalization. Vantage verifies this with
PyNaCl (`identity_verify.py::verify_identity`) and stores the result as
`identity_verified ∈ {0, 1}`. A round-trip test pins this contract on the
Rust side (`mesh_tools.rs::birth_signature_verifies_against_published_public_key`).

### 3.2 Personality JSON

The deterministic Odù index is resolved into its full IfáScript sign so the
mesh carries the real divination, not a bare index:

```json
{
  "dominant_orisha": "...",
  "odu_sign": {
    "index": 3, "name": "...", "archetype": "...", "orisha": "...",
    "taboos": [...], "prescriptions": [...], "opcode": "..."
  },
  "summary": "...",
  "ritual_suggestions": [...],
  "elements": { "fire": 0.0, "water": 0.0, "earth": 0.0, "air": 0.0, "ether": 0.0 }
}
```

### 3.3 Resonance JSON

`daily_resonance(birth_ts)` maps the birth weekday (0 = Sunday; epoch day 0
was a Thursday, hence the `+4` shift) to its Òrìṣà and trust-signal weight,
enriched with the compile-time-embedded Koodu Ritual Codex for that day:

```json
{
  "weekday": 3, "orisa": "Ọ̀rúnmìlà", "trust_signal_weight": 0.90,
  "yoruba_name": "...", "principle": "...", "tone": "...",
  "frequency": "...", "color": "..."
}
```

Deterministic: the same birth timestamp always reproduces the same resonance.

---

## 4. Wire protocol

### Step 1 — Account mint (conditional)

Runs **only** when no API key is available (no `VANTAGE_KEY`, no persisted
`existing_key`):

```
POST {VANTAGE_URL}/api/agents/register
{ "name": "<agent_id>",
  "bio":  "Ọmọ Kọ́dà sovereign agent · Odù #<n> · key <first-16-hex-of-pubkey>" }

→ 200 { "name": "<agent_id>", "api_key": "vantage_<48 hex>" }
→ 409 name already taken
→ 429 rate limited (5/min per client on the Vantage side)
```

The Vantage account name **is the `agent_id`**, not the human name — the
account is bound to the DNA-derived identity, so human-name collisions are
impossible and account squatting requires forging a DNA fingerprint prefix.
Vantage stores only the SHA-256 hash of the key. The raw key is:

- cached in the process (`MINTED_KEY: OnceLock<String>`), and
- persisted into the agent snapshot (`vantage_key`) so a restarted agent
  re-authenticates instead of re-registering.

### Step 2 — Mesh join (always)

```
POST {VANTAGE_URL}/api/mesh/agents/join
X-Agent-Key: <effective key>
{
  "agent_id": "<agent_id>",
  "block_id": "<MESH_BLOCK_ID>",
  "role": "home",
  "capabilities": {
    "kind": "omo-koda-sovereign",
    "human_name": "...",
    "public_key": "<64 hex>",
    "identity_signature": "<128 hex>",
    "dna_fingerprint": "...",
    "odu_index": 3,
    "personality": { ... },
    "resonance": { ... }
  }
}

→ 200 { "ok": true, "agent_id": "...", "block_id": "...", "identity_verified": true }
```

Identity fields ride inside `capabilities`; the server also accepts them
top-level. On the Vantage side the join:

1. Verifies the Ed25519 signature → `identity_verified`.
2. Upserts into `mesh_agents` on `(agent_id, block_id)` — see invariants (§6).
3. Records an `agent_joined` row in `mesh_events`.
4. Broadcasts `{type: "agent_joined", ...}` on the gossip channel
   `block.{block_id}` — this is how Vantage's UI (and anything else
   listening, e.g. Axiom's galaxy view) learns about the newborn in real time.

### Step 3 — Join-guard

After the identity-bearing join, the client sets `VANTAGE_JOINED = true` so
the lazy `ensure_joined()` used by the `mesh_*` write tools does **not**
re-join with empty capabilities and clobber the identity just published.
(The server-side upsert is also non-clobbering — belt and braces.)

---

## 5. Failure semantics

- **Fail-open**: `VANTAGE_URL` unset → `register_newborn` returns `None`
  immediately. Runtimes without Vantage are completely unaffected.
- **Best-effort**: transport errors and non-2xx responses in either step are
  swallowed. **A birth never fails because Vantage is down.** The agent is
  already durable on disk (§2) before registration begins.
- **No retry at birth** (known gap, §7): if Vantage is unreachable at birth,
  the agent stays unregistered until the first `mesh_*` write tool triggers
  `ensure_joined()` — which registers presence but *without* the identity
  payload, so the agent appears with `identity_verified = 0` until a re-birth
  or an explicit re-join.
- **Idempotent replay**: re-running birth registration (agent restart with a
  persisted key, `existing_key` seeded into `MINTED_KEY`) skips the account
  mint and re-joins; the upsert refreshes `last_seen_at`/`status` without
  losing identity.

---

## 6. Server-side invariants (`mesh_agents` upsert)

The join is `INSERT ... ON CONFLICT(agent_id, block_id) DO UPDATE` with
deliberately asymmetric merge rules:

| Column | Rule |
|---|---|
| `role`, `capabilities_json`, `vantage_name` | last-writer-wins |
| `public_key`, `dna_fingerprint`, `model_fingerprint`, `parent_id` | only overwritten by a **non-empty** incoming value |
| `odu_index` | `COALESCE(new, old)` — never nulled |
| `identity_verified` | `MAX(old, new)` — **monotone**: once verified, an unsigned re-join can never downgrade it |
| `last_seen_at`, `status` | refreshed to now / `'active'` |

---

## 7. Known gaps / future work

1. **No retry queue**: a Vantage outage at birth leaves the newborn
   identity-less on the mesh until manual intervention (see §5).
   Recommendation: persist the full `NewbornIdentity` payload in the snapshot
   and have `ensure_joined()` replay it instead of joining with `{}`.
2. **`MINTED_KEY` is process-global** (`OnceLock`): the design assumes one
   sovereign agent per kernel process. Multi-agent processes would share a key.
3. **Presence heartbeat**: Vantage exposes
   `POST /api/mesh/agents/{id}/heartbeat` but the kernel never calls it; the
   autonomous heartbeat (`server.rs::spawn_heartbeat`, `HEARTBEAT_SECS`,
   default 300 s) emits `heartbeat_pulse` events via `/api/mesh/signal`
   instead. `last_seen_at` only refreshes on joins, so idle agents look stale.
4. **No de-registration on death**: `DELETE /api/mesh/agents/{id}/leave`
   exists but nothing in the kernel calls it.

---

## 8. Reverse flow: Vantage-initiated birth

Vantage can also *initiate* a birth (`POST /api/agents/birth-omokoda`,
`backend/agents.py`): it proxies `{name, meta}` to the kernel's
`POST /v1/birth` (`OMOKODA_URL`), and the newborn's registration flows back
via the exact protocol above (the kernel's `VANTAGE_URL` points back at that
Vantage instance). Vantage then surfaces the freshest `mesh_agents` row as
the birth response. The loop is: **Vantage UI → kernel `/v1/birth` → birth →
auto-registration → Vantage mesh → gossip → Vantage UI.**

---

## 9. Sequence

```
Ọmọ Kọ́dà kernel                                Vantage backend
────────────────                                ───────────────
birth "Ayo"
 ├─ forge identity (BIPỌ̀N39 → DNA → Odù → Ed25519)
 ├─ auto_save (durable before any network)
 ├─ publish AgentBorn (event bus)
 ├─ sign agent_id with Ed25519 key
 │
 ├─ [no key?] ──── POST /api/agents/register ──▶ mint key, store SHA-256 hash
 │             ◀── { api_key } ─────────────────
 ├─ persist key in snapshot (auto_save)
 │
 ├─────────────── POST /api/mesh/agents/join ──▶ verify Ed25519 signature
 │                (X-Agent-Key, full identity)    upsert mesh_agents (§6)
 │                                                record agent_joined event
 │             ◀── { ok, identity_verified } ───  gossip → block.{block_id}
 │                                                          │
 └─ set VANTAGE_JOINED guard                                ▼
                                                  UI / galaxy / neighbors
                                                  see the newborn live
```
