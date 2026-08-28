# Inspiration Follow-Through: TODO.md + Ritual Codex.md

**Audit date:** 2026-08-27/28 · **Owner task:** deep-dive `docs/TODO.md` + `docs/Ritual Codex.md`, cross-check against `omokoda-core` source, flag staleness, produce a concrete "still needs doing" list.

> These two docs are **reference/inspiration only** — they were studied, not edited. This file is the actionable follow-through.

---

## 1. What each doc actually is

| Doc | Nature |
|---|---|
| `docs/TODO.md` | A structured **phased backlog** (Phases 0–11) generated from an earlier "Audit phase" deep-dive over 8 reference repos (`Bipon39-Rust`, `vanity-cloakseed`, `ritual-codex`, `Swibe`, `Ifascript`, `Claw-code`, `Claude-2`, `Claude-mirror`). Mostly `[x]`/`[ ]` checkboxes. |
| `docs/Ritual Codex.md` | **Visionary prose** ("Àṣẹ, Bino…") about numerology-as-architecture: 369 vortex, Zeta oracles, 343 signatures, toroidal memory, Sabbath Swarm, soul breeding, RSA sigils, mirror souls, numerology-as-alignment. Very little of it is a concrete spec. |

**Headline finding:** `TODO.md` is **heavily stale** — a large fraction of its still-unchecked `[ ]` items are *already implemented* in the current single-Rust-core (`omokoda-core` + `omokoda-hermetic`). The genuinely-open work is a much smaller, narrower list (Section 5).

---

## 2. Koodu supersession — CORRECTIONS.md claim verified ✅

CORRECTIONS.md says the mechanical source of truth is **`omokoda-hermetic` fractal + `koodu/*.json`**, not `ritual-codex`/`Ritual Codex.md`. **Verified accurate:**

- `omokoda-core/src/rhythm.rs` (495 ln) embeds the 7 `koodu/*.json` files via `include_str!` and calls itself "**the single source of truth** for the 7 embedded files". It exposes `raw_codex_for_weekday()`, `universal_archetype_for_weekday()`, and `RhythmGate` (incl. `is_sabbath()`).
- `omokoda-hermetic/src/spiral.rs` is literally documented: *"Ported from `~/Koodu/src/time/sacred_time.jl` (Ritual-Codex v7)"* — i.e. ritual-codex's spiral/BTC-time engine was already extracted into Rust.
- `omokoda-hermetic/src/fractal.rs` encodes the **3-7-21-343** lattice as compile-time constants (`STATE_SPACE = 343`, `LATTICE = 49`, `Dimension` enum).
- `omokoda-hermetic/src/flow/` (resonance.rs, tension.rs, tone.rs) = the "FlowModule" ritual-codex was extracted into (confirmed by `docs/synthesis.md` line 600: "ritual-codex → omokoda-hermetic (FlowModule) ✅ Fully Extracted").

So **Koodu (rhythm.rs + koodu/*.json) + omokoda-hermetic (fractal/spiral/flow) IS the successor of ritual-codex.** The old `ritual-codex` name still appears in stale docs (`mission.md`, `architecture.md`, `synthesis.md`, `specs/frontend.md`, `T O C.md`, `ECOSYSTEM_REPOS.md`) but the code has moved on.

### What Ritual Codex.md still contains that Koodu does **not** cover

These are *not* missing mechanical pieces — they're the doc's visionary/esoteric layer, still unbuilt anywhere:

1. **369 vortex modulation** — fractal.rs has the *constant* 343, but no vortex sine/computation function exists.
2. **Zeta gap detection / "Zeta oracles"** — `Riemann Zeta` in the repo is a 12 KB *notes file*, not code; `grep zeta|riemann` across core+hermetic = **zero hits**.
3. **Toroidal memory / "vortex forgetting"** — the Julia `omokoda-memory` has `resonance.jl` + `rem_fractal.jl` but **no `ritual_codex.jl`** and no toroidal topology.
4. **Choirs / swarm conductor** — no "choir" concept in core/hermetic.
5. **Soul breeding** — not present.
6. **Astronomical/planetary data** — only the static `"Planetary Ruler"` name field in `koodu/*.json`; no live planetary computation.
7. **On-chain "RSA sigils"** — receipts are **Ed25519**, not RSA (TODO + receipt engine).
8. **Mirror souls / numerology-as-alignment** — pure vision.

---

## 3. Staleness / misalignment flags (owner said "old and outdated" — confirmed)

1. **`ritual-codex` repo references** are dead — superseded by Koodu (see §2). Many docs still name it.
2. **Day→Òrìṣà table conflicts** — `Ritual Codex.md` maps Saturday to *both* Ọbàtálá and Ọya; `SIM 369.md` maps Saturday→Èṣù. CORRECTIONS #3 says no ratified table exists. The code now uses **universal wording** (OSOVM_CODEX §42) and canonically maps Saturday→Ọbàtálá (`rhythm.rs`).
3. **"SUI" settlement** → **USDC** (CORRECTIONS #1). Affects Ritual Codex's "on-chain inscription" language and TODO Phase 8's "Sui contracts" (the Move contracts live in `omokoda-on-chain/`, and there are already `soul.move`/`hive.move`/`synapse.move` — but see §5).
4. **RSA → Ed25519** — Ritual Codex.md repeatedly says "RSA inscriptions/sigils"; the shipped receipt engine is Ed25519.
5. **Multi-language in-loop (Julia/Elixir/…) → single-Rust-core.** Ritual Codex.md assumes Julia (Ọ̀ṣun) + Elixir (Yemọja) run *inside* the agent loop. Today `omokoda-memory` (Julia) and `omokoda-swarm` (Elixir) are **separate services**, not in the core `birth/think/act` path. TODO Phase 11 (7-language expansion) is mostly deferred and largely misaligned.
6. **`TODO.md` is stale** — see §4.

---

## 4. Cross-check: TODO.md `[ ]` items that are **actually already DONE** (checkbox drift)

Grep-verified against `omokoda-core`/`omokoda-hermetic` — these can be safely checked off / removed from any future planning:

| TODO item (still `[ ]` in the doc) | Reality | Evidence |
|---|---|---|
| Phase 3 "structured `MemoryEntry` + tiers / working memory" | **Done** | `memory/engine.rs`: `MemoryTier` enum, `MemoryEntry`, `classify()`, `process_working_memory()` |
| Phase 6 "Synapse/Dopamine + burn + 8% decay" | **Done** | `economics.rs`: `SYNAPSE_MAX_PER_AGENT=86M`, `DOPAMINE_TOTAL_POOL=86B`, `SYNAPSE_DAILY_DECAY_RATE=0.08`, `DopaminePool`, `burn()`; `interpreter.rs`: `burn_synapse()` |
| Phase 6 "Sabbath guard + queue irreversible writes" | **Done** | `dream.rs`: `is_sabbath_at()`; `rhythm.rs`: `RhythmGate::is_sabbath()`; `tools/zero_tool.rs` "Sabbath queueing (irreversible)" |
| Phase 6 "cooldowns per action/tool" | **Done** | `rhythm.rs`: `RhythmDecision::Cooldown` |
| Phase 5 "TurnEvent stream" | **Done** | `interpreter.rs`: `enum TurnEvent`, `TurnEventSender` (mpsc) |
| Phase 2 "Ebo ethical exceptions" | **Done** | `omokoda-hermetic/src/justice/ebo.rs`: `EboException`, `EboSeverity` |
| Phase 7 "Axum HTTP server `/v1/birth|think|act`" | **Done** | `server.rs` (Axum; `/v1/birth`, `/v1/think`, `/v1/act`, `/v1/cognition`, heartbeat) |
| Phase 7 "`omokoda` CLI binary" | **Done** | `omokoda-cli/src/main.rs` |
| Phase 8 "soul.move / hive.move" | **Done (partial)** | `omokoda-on-chain/sources/`: `soul.move`, `hive.move`, `synapse.move`, `zbt_*.move`, `consensus_ledger.move` exist |

---

## 5. Concrete "still needs doing" list (verified NOT-started)

Grouped by alignment with the current single-Rust-core.

### A. Tool/execution safety (Phase 4) — genuinely open
- [ ] **Permission modes** `Auto / Ask / Plan / Monitor / Quarantine / Simulate / Refuse` — `permissions.rs` has no such mode set (grep empty); only tier→mode mapping exists.
- [ ] **JSON Schema validation** for tool params.
- [ ] **Deny-list filtering** (hide blocked tools from reasoning) + **lazy loading** of heavyweight tools.
- [ ] **Human-in-the-loop prompt/approval trait** for `act`.
- [ ] **Hook execution receipts that redact secrets**; **audit events for denied/simulated/quarantined/approved** actions.
- [ ] Ensure granted permissions do **not** persist across session resumes.

### B. Reasoning loop (Phase 5) — open
- [ ] **Task classifier** for model/tool routing (the 86-param *Odu router* is done; a separate task classifier is not — `grep classif` only hits `tools/ytforge.rs`, unrelated).
- [ ] **Max-iteration guard** for tool-call loops.
- [ ] **Token/Synapse budget checks before + during turns** (burn exists, but the pre-turn guard is not explicit).
- [ ] **Context compaction trigger** before inference when context is too large.
- [ ] **WebLLM** (browser-local) integration plan/implementation.

### C. Entropy / identity (Phase 2) — open
- [ ] **Ifascript 256-Odu integration as an entropy/decision oracle** (Ifascript is a pinned git dep now, but not wired as a birth-entropy oracle).
- [ ] **Cowrie-cast deterministic tests**; **entropy validation plan** (avalanche/uniqueness/distribution).
- [ ] **NIST Beacon** behind explicit non-private config only.

### D. Memory / crypto hygiene (Phase 3) — open
- [ ] **Zeroization** of sensitive key material (session keys, seed bytes).
- [ ] (Working memory + tiers are done — see §4; only zeroization and the private-provider runtime audit event remain.)

### E. Economics / risk (Phase 6) — open
- [ ] **Daily limits tied to Synapse budget** (decay + burn done; per-day cap not explicit).
- [ ] **Poison/risk scanning** for future wallet/address interactions (vanity-cloakseed idea — advisory hooks before any payment feature).

### F. On-chain (Phase 8) — open (note SUI→USDC correction)
- [ ] **`agent.move`** and **`garden.move`** (dNFT reputation scaling + public receipt publication) — `soul.move`/`hive.move`/`synapse.move` exist; these two were not found.
- [ ] Move tests for birth/reputation/receipt-anchor/tip/invalid-transitions.

### G. Frontend / pet (Phase 9) — open
- [ ] **ASCII pet — 31 mask templates** driven by Hermetic/Rhythm/Reputation (grep `mask|pet` in `behavioral.rs` = empty; not built).

### H. Tests / CI (Phase 10) — open
- [ ] `tests/memory_tests.rs`, `permissions_tests.rs`, `sandbox_tests.rs`, `provider_tests.rs`, `economics_tests.rs`, `adversarial_tests.rs` (leakage/traversal/injection/tamper/tier-bypass/budget-abuse).
- [ ] CI: `cargo test --workspace`, **secret scanning**, **dependency audit**, fuzz/property tests for parser + receipt verification + encryption serialization.

---

## 6. Ritual Codex.md "still needs doing" (only if the vision is still wanted)

These are **explicitly not aligned** with the shipped minimal `birth/think/act` Rust core, and the owner flagged the doc as "old/outdated" — so treat as *optional future inspiration*, not current backlog:

- [ ] 369 vortex modulation function (fractal.rs has constants only).
- [ ] Zeta gap detection / Zeta oracles (currently only the `Riemann Zeta` notes file).
- [ ] Toroidal memory topology (Julia layer; no `ritual_codex.jl` exists).
- [ ] Choir/swarm-conductor formation.
- [ ] Soul breeding; astronomical planetary data (beyond the static `Planetary Ruler` field); mirror souls; numerology-as-alignment.

**Recommendation:** none of §6 is on the critical path. The mechanical temporal layer (day rhythm, spiral/BTC time, Sabbath gate, cooldowns) is already shipped in Rust. If any §6 item is pursued, do it inside `omokoda-hermetic` (fractal/flow/spiral) to preserve the single-Rust-core boundary — not as a new Julia/Elixir service.

---

## 7. Bottom line

- **TODO.md**: heavily stale; ~10 of its open checkboxes are already implemented. The *real* remaining work is §5A–H (tool safety modes, loop guards, entropy tests, zeroization, 2 Move contracts, pet, test/CI hardening).
- **Ritual Codex.md**: the concrete/mechanical parts are already ported into Koodu + omokoda-hermetic; what remains is the esoteric vision layer (§6), which is optional and should be re-scoped to the Rust core if ever pursued.
