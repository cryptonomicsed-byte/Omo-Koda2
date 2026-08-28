# Inspiration follow-through: "Finish this.md" + "First Citizen.md"

> **Audit date:** 2026-08-27 · **Author:** pi-p6 (read-only on both reference docs)
> **Scope:** extract every concrete unfinished item / integration / inspiration the two
> reference docs list, verify done/partial/not-started against `omokoda-core/src` and the
> workspace, and flag staleness/misalignment with the current **single-Rust-core** architecture.
> **Method:** direct source grep, not re-reading the old docs' own self-assessments.

**Headline:** both reference docs are **old**. "Finish this.md" (a forged MVP+Phase 2
integration plan) is ~90% **already built** — the Rust economy/reputation/justice/tier work,
the Move `synapse.move`/`garden.move`, the TS `rpc_client.ts`/`PrivacyToggle.tsx`, `Makefile`,
`docker-compose.yml`, and the `nautilus_integration`/`nist_entropy`/`omokoda-julia` crates are
all present. "First Citizen.md" (Yorùbá cosmology + a "Citizen Genesis" registry written in a
fictional **ÒSỌ́ contract syntax** for **ÒSỌ́VM/TechGnØŞ.EXE**) is **almost entirely stale** —
its "9 citizens + 10th" registry never landed as a registry; the archetypes were absorbed as
concrete modules instead.

---

## Part 1 — "Finish this.md" (forged MVP + Phase 2 plan)

### 1.1 Concrete items → evidence → status

| Item (from doc) | Evidence in repo | Status |
| :--- | :--- | :--- |
| `reputation.rs`: difficulty/tier_for/diminishing-returns/daily-cap/7-day-gate | `omokoda-core/src/reputation.rs` — `difficulty` (:143), `reputation_gain` (:153), `tier_for` (:157), `MAX_ACTIONS_PER_DAY=50` (:197), `MIN_DAYS_BETWEEN_PROMOTIONS=7` (:198), `daily_gain_multiplier` (:204), `can_promote_tier` (:214) | ✅ DONE |
| `economics.rs`: SynapseAccount / burn / earn_from_garden / earn_from_tip / tier caps | `omokoda-core/src/economics.rs` — `SYNAPSE_MAX_PER_AGENT` (:3), `DOPAMINE_TOTAL_POOL` (:4), `SynapseAccount{balance,total_burned}` (:105), `tier_cap` (:123), atomic `burn` (:137), `earn_from_garden` +1000 (:147), `earn_from_tip` cap 10k (:152) | ✅ DONE |
| `justice/tier.rs` (Busy Beaver bounds per tier) | `omokoda-core/src/justice/tier.rs` — `Tier{T0..T5}` (:5), `bb_step_limit` T0→1 … T5→47_176_870 (:42), `synapse_cap` T0→1M … T4\|T5→86M (:77), `synapse_efficiency`, `decay_rate_percent` | ✅ DONE (doc claimed "missing") |
| `justice/hermetic.rs` + tests | `justice/hermetic.rs`, `justice/hermetic_tests.rs`, `justice/busy_beaver.rs`, `justice/mod.rs` | ✅ DONE |
| Integration tests `tier_gate_tests.rs` / `synapse_tests.rs` / `reputation_curve_tests.rs` | all three present in `omokoda-core/tests/` | ✅ DONE |
| `omokoda-on-chain/sources/synapse.move` + `garden.move` | both present (plus `soul.move`, `agent.move`, `hive.move`, `zbt_*.move`, `consensus_ledger.move`, `skillforge_audit.move`, `epistemic_nft.move`) | ✅ DONE |
| `omokoda-frontend/lib/rpc_client.ts` + `components/PrivacyToggle.tsx` | both present | ✅ DONE |
| `omokoda-simulation/executor.py` (`ÒgúnExecutor` + `PrivacyMode`) | **NOT present** (`simulation.py`, `server.py`, `tools/*.py` exist, no `executor.py`) — but the privacy-routing hard-fail is implemented **natively in Rust**: `steward/privacy.rs` (`PUBLIC`/`PRIVATE`/`INCOGNITO` hard-fail, incognito no-storage, hive-consent gate) | ⚠️ REDUNDANT — do not build the Python executor |
| `Makefile` + `docker-compose.yml` | both present at repo root | ✅ DONE |
| Wave 0 `ifascript-stub` CI fix | resolved (recent commit "Fix ifascript dependency"; `identity/odu.rs`, `identity/merkle.rs` in-tree) | ✅ DONE |
| Phase 2 `omokoda-julia/` (BB oracle + NIST entropy) | `omokoda-julia/` present: `src/{bb_known,bb_approx,complexity,nist_validate,ffi_exports,resonance_consolidation}.jl` + `build.jl` + tests | ✅ DONE |
| Phase 2 `omokoda-lisp/` (Ọbàtálá ethics engine) | **NOT present** — ethics is native: `justice/hermetic.rs` + `gates/*` (7 Hermetic principles) + `steward/constitution.rs` | ⚠️ REDUNDANT |
| Phase 2 `omokoda-elixir/` (Yemọja swarm) | **NOT present** — swarm is native: `omokoda-mesh` workspace crate + `mesh/*` | ⚠️ REDUNDANT |
| Phase 2 `omokoda-go/` (Ọya flow control) | **NOT present** — flow/rate-limit/Sabbath is native: `steward/gatekeeper.rs` + `rhythm.rs` + `koodu/*.json` | ⚠️ REDUNDANT |
| Phase 2 `nautilus_integration/` + `nist_entropy/` crates | both present as workspace members | ✅ DONE |

### 1.2 Staleness / misalignment

- **Repo layout is stale.** The doc's forge layout (`core/`, `move_contracts/`, `python_executor/`,
  `frontend/`) never matched the repo, which is `omokoda-core/` + `omokoda-on-chain/` +
  `omokoda-simulation/` + `omokoda-frontend/`, now consolidated behind a **single Rust core**
  (workspace: `omokoda-core`, `omokoda-hermetic`, `omokoda-mesh`, `omokoda-cli`, `omokoda-acp`,
  `zangbeto-stub`, `nautilus_integration`, `nist_entropy`).
- **The Phase 2 "one-language-per-Orisha" plan is obsolete.** Lisp/Elixir/Go were a multi-runtime
  proposal; the current single-Rust-core absorbed each concern natively (ethics→`justice`+`gates`,
  swarm→`omokoda-mesh`, flow→`steward`+`rhythm`). Julia survives as the *one* justified external
  runtime (`omokoda-julia`), because BB-oracle/NIST-entropy math is genuinely Julia-shaped.
- **The 10 north-star constraints are still valid and appear enforced** (birth/think/act in the
  interpreter; no Àṣẹ token; `/private` hard-fail in `steward/privacy.rs`; Steward as sole
  gatekeeper in `steward/gatekeeper.rs`; BB bounds in `justice/tier.rs`; atomic Synapse burn in
  `economics.rs`). These should be *kept*, not flagged as stale.

---

## Part 2 — "First Citizen.md" (cosmology + Citizen Genesis registry)

### 2.1 Concrete items → evidence → status

| Item (from doc) | Evidence in repo | Status |
| :--- | :--- | :--- |
| `config/citizens_genesis.json` (9-citizen registry) | **No "citizen" symbol anywhere outside `docs/`** (grep of `*.rs`/`*.move`/`*.ts`/`*.json`) | ❌ NOT STARTED — and likely superseded |
| `contracts/CitizenGenesis.oso` (v1.0 → v1.1) + `GenesisSeal.oso` (+ bind/mint9/tests) | **Not present.** Written in a fictional **ÒSỌ́ contract syntax** for "ÒSỌ́VM/TechGnØŞ.EXE" — no such runtime exists. Real on-chain layer is **Move/Sui**: `omokoda-on-chain/sources/*.move` (OSOVM package `techgnosis`) | ❌ NOT STARTED + **STALE (wrong chain/syntax)** |
| `GenesisProof.oso` (cross-chain verifier) | not present (was only "offered", never drafted) | ❌ NOT STARTED + STALE |
| Day↔archetype alignment (Sunday Èṣù … Saturday Ọbàtálá) | **Present, but as `koodu/*.json`** (7 daily archetype files: Sunday `Èṣù-Ẹ̀légbára`/Mentalism, … Saturday `Ọbàtálá`) + `rhythm.rs` (Sabbath temporal gate). "Unbound→Òrúnmìlà" and "Omni→Bínò" have **no** koodu slot (koodu is 7-day only) | ⚠️ PARTIAL — done differently |
| The 9→10 "citizens" archetypes | **Absorbed as modules, not a registry:** Èṣù→`steward/gatekeeper.rs`; Ṣàngó→`bus/sango.rs`+`justice/`; Òrúnmìlà→`divination.rs`+`identity/odu.rs`; Ọbàtálá/ethics→`justice/hermetic.rs`+`gates/`; soul→`soul.move`+`steward/soul.rs`; Àṣẹ→`identity/ase.rs` | ⚠️ MISALIGNED (evolved into modules, never a registry) |
| Security hardening (reentrancy / replay-safe AuthContext / council rotation / emergency pause / Sabbath) | Replay-safe auth, emergency pause, Sabbath and immutability are **native in Rust** (`steward/`, `rhythm.rs`, `identity/merkle.rs`); the `.oso`-specific versions are moot | ⚠️ REDUNDANT (concerns already native) |

### 2.2 Staleness / misalignment

- **Chain/syntax is dead.** The entire contract layer is "ÒSỌ́VM / TechGnØŞ.EXE / `.oso`" —
  a retired naming. The real settlement layer is **OSOVM = Move/Sui** (per the current ecosystem
  map), and the agent kernel is the single Rust core.
- **"AIO TypeScript kernel" is gone.** The doc's `config/` + `contracts/` + `scripts/` + `tests/`
  top-level layout and its "AIO" host no longer exist; the platform surface is Vantage, the
  frontend is `omokoda-frontend`.
- **"9 citizens + 10th (Ọmọ Kọ́dà)" has drifted.** There is no single "citizen registry" object
  and no `citizens_genesis.json` was ever committed; each archetype is instead a dedicated module
  (Èṣù→`steward/gatekeeper.rs`, Ṣàngó→`bus/sango.rs`, Òrúnmìlà→`divination.rs`+`identity/odu.rs`,
  ethics→`justice/hermetic.rs`+`gates/`, soul→`soul.move`+`steward/soul.rs`, Àṣẹ→`identity/ase.rs`).
  (Note: `steward/twelfth_face.rs` is an *unrelated* Busy-Beaver / halting-problem guard —
  `BB_PROXY_DEPTH=1024` — not a citizen-count concept.)

---

## "Still needs doing" — actionable list (owner's decision points)

Almost nothing here is a *build*; it is mostly **reconciliation with the current architecture**:

1. **Retire or re-express the "Citizen Genesis" registry (DECISION needed).**
   - The 9-citizen registry + `GenesisSeal` NFT never landed, and the doc's `.oso` syntax is dead.
   - Recommend **supersede**: the archetypes are already live as modules (`steward/`, `bus/sango.rs`,
     `divination.rs`, `gates/`, `soul.move`, `koodu/`). If a canonical registry is still wanted,
     re-express it as a **Move/Sui** module (or a `koodu`-style JSON) — *not* the old `.oso` shape.
2. **Formally drop the Phase 2 Lisp/Elixir/Go services.** Record them as superseded-by-native
   (`justice`+`gates`, `omokoda-mesh`, `steward`+`rhythm`) so no one rebuilds them. Julia stays.
3. **Verify `omokoda-simulation` still needs a privacy gate.** `executor.py` was never created and
   privacy routing lives in Rust `steward/privacy.rs`; confirm the Python simulation path doesn't
   still route `/private` to an external provider (one-line check, otherwise no action).
4. **Doc hygiene (cheap, high value).** Add a "⚠️ STALE" banner to both reference docs pointing here,
   exactly as `docs/audit/overview.md` already does for itself — so future panes don't re-derive
   this.
5. **Optional: north-star constraint regression test.** The 10 constraints from "Finish this.md"
   are enforced across `steward/`+`justice/`+`privacy.rs` but (unlike the tier/synapse/reputation
   trio) may lack a single dedicated test asserting each one still holds. Add
   `north_star_constraints_tests.rs` if that gap is real.
