# Inspiration follow-through audit — "New file.md", "Next.md", "OSOVM_CODEX.md"

Auditor: graph-security-auditor (Vantage agent 371). Read-only review. Date: 2026-08-27.
Scope: extract concrete unfinished items / integrations / inspiration from three reference
docs, verify against `omokoda-core/src` (and adjacent crates), flag staleness + fabrication.

---

## ⚠️ FABRICATION FLAG — re-confirmed (do NOT treat as progress)

**`docs/Next.md` → "Deep Dive: Each Repo's Extractable Value" (lines 386–843) and
"Integration Roadmap — Phased Rollout" (lines 844–898) are FABRICATED.**

- The deep dive cites a **fictional org `github.com/omo-koda/*`**. The real repos of that
  name live under `Bino-Elgua/*` (mostly ARCHIVED) and `cryptonomicsed-byte/*`.
- It cites **nonexistent code**: `claw_policy::{Policy,PolicyEngine}`, `SovereignPolicy`,
  `neural_router.rs` (in `omokoda-hermetic`), `hermetic_laws::allows`,
  `ritual_codex::temporal_gate`, `memory-engine.js` (in Swibe), `swarm_neural_router.rs`.
- **Grep confirms absence**: `claw_policy`, `SovereignPolicy`, `neural_router` return zero
  hits in `omokoda-core/src/` and `omokoda-hermetic/src/` (only unrelated strings like a
  JSON `"kind": "omo-koda-sovereign"` and repo paths).
- The `git submodule add https://github.com/omo-koda/...` commands in the roadmap are
  therefore also fiction. **Do not run them; do not plan around them.**

The only salvageable value from Next.md is the *topology sketch* (which mirrors
organism-core's Elder's Map) — but every "✅ Verified Integration Point" (§182–238) cites
those fabricated paths, so none of them should be trusted without re-verifying against
`organism-core/`.

---

## Per-doc extraction

### 1. `docs/New file.md` (178 KB) — STALE vision dump, mostly redundant
- ~8 near-identical "complete synthesis / exhaustive record" restatements of the same
  vision (the file is a concatenation of repeated session summaries).
- **Only two concrete unfinished items survive** (both stated multiple times):
  1. **Node dopamine allocation + reward formula** — "we have the high-level rules but not
     the final math."
  2. **Precise tier unlock table** — "what tools unlock at each tier."
- Everything else in the doc is vision/cosmology (3 primitives, 7 Hermetic principles,
  reputation 0–100, tiered tools, public/private memory, sandbox, Odu memory) — already
  reflected in the codebase or superseded by OSOVM_CODEX.

### 2. `docs/Next.md` (81 KB) — connection map, core section fabricated
- The deep-dive + roadmap = fabricated (flag above).
- The "Verified Connection Matrix" and "Module Dependency Graph" claim Rust crates and TS
  bridges that do not all exist as described. Treat as a *wishlist*, not a map.

### 3. `docs/OSOVM_CODEX.md` (110 KB) — the CANONICAL, still-current doc
- Best-maintained; explicit status markers (LOCKED / CONFLICT / OPEN / DEFERRED / RETIRED).
- **§24 "ISSUES FLAGGED + GAME PLAN" is the real source of truth** for "still needs doing"
  (P0/P1/P2). Cross-reference below.
- **§29 payment model** (owner-confirmed 2026-07-11): **no self-issued token; USDC via
  Èṣù router; Àṣẹ = soulbound merit; Dopamine/Synapse = internal compute credit.** This
  RETIRES `ase.move` + all emission/supply/burn work (§8, §18b, §25 Àṣẹ-token, §27d).
- **§25 OPEN** (2 owner sub-decisions): 8%/day Synapse decay confirm; connection direction
  (both legs).

---

## Cross-reference: GAME PLAN (OSOVM_CODEX §24) vs real code — done/partial/not-started

| # | Item | Status | Real evidence |
|---|---|---|---|
| P0.1 | Build Julia on VPS, run ScarabSwarm race_demo | ✅ DONE | `osovm.service` live on hostinger (`julia src/server.jl 7778`, Julia 1.11.5) |
| P0.2 | Determinism: 2 runs / 2 machines → same trajectory hash | 🟡 PARTIAL | `job_spec`/`merkle`/`checkpoint_export` tested (97+16+6+13 tests); determinism claimed proven cross-machine (`782a7b2a…`) but not re-verified this session |
| P0.3 | `sui move build` + `test` on OSOVM Move | ✅ DONE | `techgnosis` package built + **published to Sui testnet** (`0xb3b6ef1d…`), 7 modules incl. `proof_of_witness` |
| P0.4 | `git rm -r --cached julia-1.10.5`; commit canonical docs | 🟡 PARTIAL | `julia-1.10.5/` still tracked in OSOVM (27 MB); `OSOVM_CODEX.md` + `UNIFIED_ARCHITECTURE.md` now committed under `Omo-Koda2/docs/` |
| P1.5 | Sign witness attestations (per-device key, Bipon39+Cloakseed, secure element) | 🟡 PARTIAL | `witness_bridge/` (Rust) does Ed25519 sign/verify (10 tests); firmware-side secure-element signing NOT done |
| P1.6 | Stake+slash via `elegbara_router`/`economic_security.move` | 🟡 PARTIAL | `elegbara_router` deployed testnet; `economic_security` in published package; not wired to witness identities |
| P1.7 | Replace fakeable RSSI with UWB / ≥3-anchor multilateration + real SX1278 | 🔴 NOT STARTED | firmware still `MockSX1278`, `rssi=-80` hardcoded |
| P2.8 | NFC handshake (2 devices → joint signed attestation) | 🔴 NOT STARTED | no evidence |
| P2.9 | ScarabSwarm proof → `proof_of_witness.move` on Sui | 🟡 PARTIAL | `witness_bridge` built + tested locally; on-chain round-trip not run (testnet gas ~0.0075 SUI, faucet rate-limited) |
| P2.10 | Reconcile tokenomics (one emission + one split) | ✅ DONE (superseded) | §29: Àṣẹ retired → USDC stablecoin; `ase.move` deferred; no emission curve needed |
| P2.11 | Zàngbétò wraps receipts; Vantage=AIO; Odù/IfáScript addresses job_ids; Koodu | 🟡 PARTIAL | `zangbeto_receipts.jl` real (13/13 tests); Vantage live (`:8001`); Odù/IfáScript + Koodu exist; full cross-wiring open |

## Additional open items (not in GAME PLAN)

| Item | Source | Status | Evidence |
|---|---|---|---|
| Node dopamine allocation + reward formula (exact math) | New file.md | 🟡 PARTIAL | `dopamine`/`synapse` referenced in `scheduler.rs`, `usage.rs`, `server.rs`, `main_loop.rs`, `query.rs`; exact formula not pinned |
| Tier unlock table | New file.md | 🟡 PARTIAL | `gates/` has all 7 Hermetic gates + `tier_gate_tests.rs`; no explicit per-tier tool-unlock table found |
| 8%/day Synapse decay confirm | OSOVM_CODEX §25 | 🔴 OPEN | owner decision |
| Connection direction (both legs Àṣẹ↔Dopamine) | OSOVM_CODEX §25 | 🔴 OPEN | owner decision |
| x402/USDC payment rail | OSOVM_CODEX §29 | 🟡 PARTIAL | `identity/x402.rs` implements EIP-3009 `transferWithAuthorization`; **no funded EVM/USDC wallet**, no real facilitator settlement yet |

---

## Staleness / misalignment flags

1. **`New file.md` is fully superseded** — the vision it describes is now realized as
   `omokoda-core` modules (`gates/` = the 7 Hermetic laws, `receipt/` = receipt chain,
   `reputation.rs`, `sandbox.rs`, `memory/`, `memory_vault/`). Only the two numeric gaps
   (dopamine formula, tier table) remain relevant.
2. **`Next.md` integration roadmap contradicts the locked §29** — it proposes submodules
   and a token-heavy "harvest" that predates (and conflicts with) the stablecoin/USDC
   decision. Ignore its Phases 1–3 entirely.
3. **Àṣẹ-token references are stale everywhere** — §8, §18b, §25 Àṣẹ-as-token, and
   `ase.move` are all RETIRED by §29. Any "still to do" list that includes Àṣẹ minting/
   emission is outdated.
4. **Two-OSOVM / two-ScarabSwarm / two-Witness fragmentation** (§24) — still worth
   resolving (canonical home per repo) but lower priority than P0/P1 above.

---

## CONCRETE "STILL NEEDS DOING" (prioritized)

**P0 — prove the loop (execution + determinism)**
1. Re-run the determinism test on two separate VPSes/builds; publish the result (this
   gates PoSim entirely).
2. `git rm -r --cached julia-1.10.5` in OSOVM (27 MB vendored Julia).

**P1 — make the witness un-gameable**
3. Wire `witness_bridge` on-chain: fund the testnet operator (faucet) and run the first
   `register-sensor` / `submit-attestation` / `submit-witness` round-trip against
   package `0xb3b6ef1d…` + WitnessOracle `0x03380e98…` + SensorRegistry `0x6b380504…`.
4. Firmware: sign attestations with a per-device key (Bipon39 child + secure element).
5. Replace RSSI mock with UWB or ≥3-anchor multilateration.

**P2 — settle the tokenomics/decision items (owner-gated, cheap now)**
6. Confirm 8%/day Synapse decay (or correct number).
7. Decide connection direction (both legs to close the loop).
8. Pin the node dopamine allocation + reward formula.
9. Publish the tier unlock table.

**Defer (over-scope, per §24):** 707-veil catalog fill, native L1 (use Sui), Axiom macro
layer, genesis ceremony, Spatial Twin (§26/§30) — all behind the P0 gate.
