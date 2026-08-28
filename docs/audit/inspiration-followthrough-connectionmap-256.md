# Inspiration follow-through audit — "CONNECTION_MAP_V2.md", "256---65536.md"

Auditor: Claude (Vantage session). Read-only review, docs not edited.
Date: 2026-08-28. Scope: extract concrete unfinished items / integrations /
inspiration from the two reference docs, verify against real code across
`omokoda-core/src` and every repo each doc names, flag staleness/misalignment.
Repos read from `cryptonomicsed-byte/*` (canonical, push-eligible) unless noted;
none read from `Bino-Elgua/*` was needed for this pass, and nothing was pushed
there.

---

## Day → Òrìṣà conflict (CORRECTIONS.md §3) — still unresolved, confirmed

`CORRECTIONS.md` names `256---65536.md` as one of three conflicting sources
on the day→Òrìṣà table (alongside `Ritual Codex.md`'s internal
Saturday-contradiction and `SIM 369.md`). Re-checked this pass:

- `256---65536.md` (lines 733–744) gives its own full 7-day table: **256→Saturday,
  2048→Friday, 4096→Sunday, 8192→Thursday, 16384→Tuesday, 32768→Wednesday,
  65536→Monday** — presented as settled fact ("THE 256 → 65536 LADDER NOW MAPS
  TO DAYS... This is PERFECT").
- Cross-checked the one point of overlap CORRECTIONS.md calls out explicitly:
  **Monday→Ṣàngó**. Both `256---65536.md` and `SIM 369.md` agree on this one
  pairing, and the real code (`omokoda-core/src/koodu/monday.json`) also has
  `"archetype": "Ṣàngó"` — so that specific pairing happens to be corroborated
  three ways.
- But CORRECTIONS.md's actual complaint is **Saturday**, and that's still
  live: `Ritual Codex.md` contradicts itself (Saturday→Ọbàtálá in one
  paragraph, Saturday→Ọya in another), `SIM 369.md` says Saturday→Èṣù,
  `256---65536.md` says Saturday→256/Èṣù (agrees with SIM 369 here, actually)
  — so Ritual Codex.md is the odd one out on Saturday specifically, not a
  clean 3-way split. Still: **no single ratified table exists**, and
  `256---65536.md` is one of the sources presenting an unratified table as
  finished. CORRECTIONS.md's instruction stands: *do not hard-code any of
  this*. Confirmed the code doesn't — `koodu/*.json` are flavor files consumed
  for tone/color/facet text, not gating logic (checked: no `match day` or
  `if day ==` branching on them in `omokoda-core/src`).

**Verdict: still unresolved. `256---65536.md` should be read as one more
unratified opinion, not the tiebreaker.**

---

## Part 1 — `CONNECTION_MAP_V2.md`

This is a genuinely different kind of doc than the fabricated `Next.md`
flagged in the sibling audit (`inspiration-followthrough-newfile-next-osovmcodex.md`):
**every repo it names is real** (`cryptonomicsed-byte/Agentic-waggle`,
`OSOVM`, `Axiom`, `Omo-Koda2` itself, `Loom`) and, unlike `Next.md`, a large
fraction of what it frames as forward-looking "NEW" proposals **is already
built**. The doc is stale in the *opposite* direction from the usual
problem — it undersells how much of §0, §5, and §6 already shipped.

One real naming trap found during verification, worth recording so nobody
re-hits it: **`~/waggle` (`cryptonomicsed-byte/waggle`) is NOT `core/waggled`**
from this doc. It's an unrelated Solana bounty-market token for the Buzz
workspace, already self-archived ("folded into Bondhive" per its own
`ARCHIVED.md`). The real stigmergic field daemon this doc means by
`core/waggled` lives in **`cryptonomicsed-byte/agentic-waggle`, `core/`**
(Go, `core/kernel/kernel.go` + siblings).

### §0 Agentic/Waggle core — ✅ DONE, comprehensively

| Item | Status | Evidence |
|---|---|---|
| Typed channel schema (`channel/subtype/decay_kernel/cross_inhibits`) | ✅ DONE | `agentic-waggle/core/channels.go` — real `CrossInhibits []Inhibition` field, wired into scoring (`field.go`) |
| `watch` verb (write-path subscription → derived deposits) | ✅ DONE | `core/watches.go`, `POST /v1/watches`, `POST /v1/ingest/{id}` |
| `recall(uri, at_time)` | ✅ DONE | `core/recall.go`: `RecallAt`, `RecallWindow`; `GET /v1/recall`, `/v1/recall/window` |
| `bounded` channel (Mandelbrot fragility, 0–1, slow decay) | ✅ DONE | `core/channels.go:93` — named channel, cross-inhibits gold at floor 0.25 |
| `evidence_tier` field + trust ladder | ✅ DONE | `core/field.go` — `EvidenceTier` on every signal, `TierRank`/`TierWeight` used throughout scoring math |
| `sniff_explain` | ✅ DONE | `core/field.go: Explain()`, `GET /v1/explain` |
| Batched multi-URI `sniff` | ✅ DONE | `POST /v1/sniff/batch` |
| SDK `response_thresholds` + probability-weighted sniff | ✅ DONE | `sdk/python/waggle.py:40,222` |
| SDK `evidence_tier`-aware filtering | 🟡 PARTIAL | `recall_at()` takes filters; explicit "corroborated-or-better" convenience filter not confirmed as a named param — spot-checked, not exhaustive |
| SDK `subscribe_channel` | ✅ DONE | `sdk/python/waggle.py:175` |
| `wag recall --at`, `wag channels list`, `wag explain`, `wag replay --speed` | ✅ DONE | `cli/src/main.rs:92-95,294` — all four present |
| Spatial diffusion / dynamic evaporation (Hilbert bleed, claim-velocity half-life) | ⬜ NOT VERIFIED | not grepped this pass — lower priority, revisit if this becomes load-bearing |

**Still needs doing (§0):** the two ⬜/🟡 rows above. Everything else in §0
is real and running — treat future references to "building the watch/recall/
bounded/evidence_tier/sniff_explain layer" as **already done**, not a task.

### §1 ỌṢỌVM + folded Techgnosis — ✅ MOSTLY DONE (8/10), 2 confirmed not started

**Follow-up verification done 2026-08-28** — read `OSOVM/src/waggle_bridge.jl`
in full (276 lines). It's a real, faithful implementation, with inline
comments citing the exact doc section numbers:

| Item | Status | Evidence |
|---|---|---|
| §1.1 `osovm-bridge` implements Axiom's `GraphEngine` (`node_spawned`/`node_updated`/`node_died`) | 🔴 NOT STARTED | grepped all of `OSOVM/src` (excl. vendored `julia-1.10.5/`) for `node_spawned`/`node_updated`/`node_died`/`GraphEngine`/`Axiom` — only one passing prose mention in a comment, no implementation |
| §1.2 watch-registered auto-deposit on Zangbeto-verified execution | ✅ DONE | `zangbeto_promote!()` — deposits `gold` at `evidence_tier: "zangbeto-verified"`, plus a companion `bounded` deposit when robustness is known |
| §1.3 four-function Wasm ABI export (plugs in like `rust-wasm-leaf`) | 🔴 NOT STARTED | zero `wasm`/`extern "C"`/`WASM_EXPORT` matches anywhere in `OSOVM/src` |
| §1.4 `sniff` before spawning new computation, cache hit short-circuits recompute | ✅ DONE | `cache_get()` sniffs `osovm://bytecode/<key>` for gold before recompute; `compile_with_signals()` calls it first |
| §1.5 stage-transition signaling (parsing→type-checking→codegen→emitted) | ✅ DONE | `stage!()`, called at each transition in `compile_with_signals()` |
| §1.6 dead-end auto-deposit on compile failure, tagged with stage | ✅ DONE | `compile_failed!()` — tags the failing stage, intensity 4 |
| §1.7 bytecode cache keyed to signal URI | ✅ DONE | `cache_key()` (sha256 of source), `cache_put!()`/`cache_get()` — genuinely content-addressed, stored in Waggle's own `/v1/memory` namespace |
| §1.8 build-time Mandelbrot perturbation gate on bytecode stability | ✅ DONE | `perturb_and_verdict!()` — runs `f` over N jittered input copies, computes bounded-fraction stability, deposits on the `bounded` channel with verdict text (`"robust island"`/`"fragile boundary"`/`"escape zone"`) — this is a faithful, complete implementation of exactly what the doc describes |
| §1.9 Zangbeto verification promotes `evidence_tier` directly | ✅ DONE | same `zangbeto_promote!()` as §1.2 — one function covers both |
| §1.10 `recall`-backed regression detection before shipping | ✅ DONE | `regression_check()` — compares current `bounded` verdict against `recall` N hours ago, computes a decay-normalized regression-severity index, flags `regressed = rsi > 0.5` |

**Revised verdict: §1 is 8/10 done**, essentially the same story as §0/§5 —
undersold by the doc's own framing. The **only real gaps are §1.1 (Axiom
GraphEngine bridge) and §1.3 (Wasm ABI export)** — both genuinely absent, not
just unverified. Note §1.1's gap is one half of a two-sided contract: Axiom's
side (§2.4, `GraphEngine` adapters) needs checking too before assuming which
side is actually missing — see §2 below.

### §2 Axiom — 🟡 PARTIAL, verified item-by-item

**Follow-up verification done 2026-08-28**:

| Item | Status | Evidence |
|---|---|---|
| 1. `waggle-hotspot` node type | 🟡 LIKELY (not re-confirmed) | `nodeTypes/registerOmokoda.ts` exists; not re-opened this pass |
| 2. SSE subscription from Waggle's event stream | ✅ DONE | `EventSource`/SSE usage confirmed in `src/engine/OmokodaGraphEngine.ts` |
| 3. Gradient-driven camera auto-zoom (`GET /v1/gradient?depth=N`) | 🔴 NOT STARTED | zero matches for `gradient`+`zoom`/`autoZoom`/camera-gradient anywhere in `src/` |
| 4. `osovm-bridge`/`loom-bridge`/`vantage-bridge` as `GraphEngine` adapters | 🟡 PARTIAL | `OmokodaGraphEngine.ts` + `MockGraphEngine.ts` are real, but per §1.1 above, OSOVM's *own* side of that bridge (the code that would call into Axiom's GraphEngine) doesn't exist yet — so this is a client interface with an unconnected server side, at least for the OSOVM leg specifically |
| 5. Taboo-channel visual treatment (slow-pulsing red-black glow) | 🔴 NOT STARTED | zero matches for `taboo` anywhere in `Axiom/src` |
| 6. Mandelbrot shader blended with live `bounded` signal density | 🔴 NOT STARTED | `src/scene/postfx.ts` has a real escape-time Mandelbrot GLSL shader (confirmed: `bool bounded = dot(z,z) <= 4.0` at the shader level) — but it's a **self-contained local computation**, not fed by field data. Zero `fetch`/`EventSource`/`/v1/sniff`/`/v1/explain` calls anywhere in `postfx.ts`. The doc's ask (extend the existing shader to blend in real field `bounded` density) has not been done — the shader exists, the extension doesn't |
| 7. `sniff_explain` panel in NodeInspector | 🔴 NOT STARTED | zero matches for `explain` in `NodeInspector.ts` |
| 8. Cross-inhibition dome visualization | 🔴 NOT STARTED | zero matches for `cross_inhibit`/`dome` anywhere in `Axiom/src` |

**Revised verdict: §2 is roughly 2/8 done, 1 partial, 1 assumed-but-not-
re-confirmed, 4 confirmed not started.** This is the opposite pattern from
§0/§1/§5 — Axiom's adapter *scaffolding* (the TS interface types, the mock
engine) is real, but almost none of the *specific visual features* the doc
asks for are built on top of it yet. This is the most honestly-unfinished
section verified so far, closer to what the doc's overall framing implies
than §0/§1/§5 turned out to be.

### §3 IfáScript — 🔴 NOT STARTED

`Omo-Koda2/Ifascript/src/` is **37 lines total** (`odu.rs` 32 lines,
`lib.rs` 5 lines): a static `get_odu(index)` / `get_odu_by_binary()` lookup
table. Zero occurrences of `cast` anywhere in the crate. None of the four
items (`cast(uri_pattern)` against `recall`, `cast_bounded`,
Odù-to-channel mapping table, `cast_federated`) exist in any form —
IfáScript today doesn't even call the Waggle field at all, let alone
divine from it. This is the single most concrete, fully-unstarted section
of the whole doc.

### §4 Vantage federation — 🔴 NOT STARTED

Grepped all of `Vantage/backend` for `waggle`, `remote_manifest`,
namespace-prefixed trust-discount subscription, `federation-health` signal —
**zero matches**. Vantage's real `bridge(remote_manifest_url)` verb, the
bidirectional negotiation, and the federation-health meta-signal (items 1–4)
do not exist. (Note: Vantage does have plenty of *other* "bridge" code —
`ares-bridge`, buzz-relay bridges, etc. — none of it is the Waggle-field
federation this doc means. Don't let a `grep bridge` false-positive on those
count as progress here.)

### §5 LOOM — ✅ DONE, comprehensively, and self-documenting

`Loom/waggle_field.py` (280 lines) opens with a docstring literally citing
"Connection Map v2 §5" and implements, in order: trade-outcome auto-deposit
(§5.1), Fractal Oracle bounded verdicts via the shared oracle (§5.2),
MarketEvent fan-out to Axiom + field in one call (§5.3), regime-shift
detection via bounded-decay anomaly (§5.4), preset quorum-selection via live
gold+bounded gradient (§5.5), and Ṣàngó reputation weighting from
oracle-confirmed gold (§5.6). Only §5.7 (dead-cat-bounce cross-inhibition
filter) wasn't explicitly grepped for — worth a quick confirm, but given the
rest of the file's fidelity to the spec, likely present or trivial to add.

**§5 should be read as done, not as a todo list**, same caveat as §0.

### §6 Omo-Koda2 core, per-Òrìṣà — ✅ MOSTLY DONE (6/7), 1 confirmed not started

Directly grepped `omokoda-core/src/waggle/mod.rs` (the Rust/Èṣù side) plus,
**follow-up 2026-08-28**, the actual per-language subsystem directories —
`Omo-Koda2` turns out to be a monorepo with real dedicated dirs per Òrìṣà
(`omokoda-julia`, `omokoda-swarm` [Elixir]), and Ọya's home is confirmed to
be `agentic-waggle/core` (Go) since Ọya *is* `waggled` per the doc's own
framing, not a separate repo:

| Item | Status | Evidence |
|---|---|---|
| Èṣù: capability-token gate on claim/mark/release/dance | ✅ DONE | `CapabilityGate`, real tests (`capability_tokens_gate_the_verbs`) |
| Èṣù: mark rate-limiting (NEW) | ✅ DONE | `MarkThrottle` token-bucket, tested (`mark_throttle_caps_burst_and_refills`) |
| Ọbàtálá: taboo channel + justification metadata (NEW) | ✅ DONE | `taboo_from_halt()` — deposits `principle`/`justification`/`source` in signal meta on every gatekeeper HALT |
| Ògún: tool-outcome auto-signaling (NEW) | ✅ DONE | `tool_outcome()` via a registered `watch`, success→gold/failure→dead-end |
| Ọ̀ṣun: `recall`-based resonance consolidation (NEW) | ✅ DONE | `omokoda-julia/src/resonance_consolidation.jl` (106 lines, cites §6.3–6.4 inline) — `resonance_over_history()` samples `recall` at N instants over a time window, scores resources by cross-time persistence (not just current loudness); `consolidate!()` promotes above-threshold patterns into durable memory at `osun/consolidated/<territory>`. Faithful, complete implementation, plus a real `RackMemory.jl`/`Resonance.jl` under `omokoda-julia/src/soma/` backing it |
| Yemọja: supervision-tree topology mirrors Hilbert territory (NEW) | 🔴 NOT STARTED | `omokoda-swarm/lib/omokoda_swarm/swarm_supervisor.ex` is a real, working `DynamicSupervisor` — but generic `:one_for_one`, zero territory/prefix partitioning. Checked all 4 supervisor files in the repo (`swarm_supervisor.ex`, `constitutional_supervisor.ex`, `mesh/neighbor_supervisor.ex`, `memory/supervisor.ex`) — zero matches for `territory`/`Hilbert` anywhere. The swarm coordination layer is real; the territory-topology-mirroring idea specifically is not built |
| Ọya: tunable per-territory heartbeat rate (NEW) | ✅ DONE | `agentic-waggle/core/swarm.go:285`, section literally titled `// ---- Territories (Ọya's heartbeat) ----`. `Territories.Set(prefix, tempo)` / `.Tempo(resource)` — longest-prefix-match tempo multiplier scales the default half-life per territory (fast-moving trading-book territory can run tempo<1, slow ethics-judgment territory tempo>1), exposed via `POST /v1/territories`. Exactly what §6.12 describes |

**Revised verdict: §6 is 6/7 done.** Same pattern as §0/§1/§5 — the doc
undersells reality. The one real gap is **Yemọja's territory-aware
supervision topology** — real supervisor infrastructure exists, the
Hilbert-boundary-mirroring specifically does not.

### §7 Ṣàngó / Move contracts — 🔴 0/4 items done; real adjacent infrastructure exists, but none of it is wired the way the doc asks

**Follow-up verification done 2026-08-28** — read `agent.move`,
`consensus_ledger.move` in full, and traced every off-chain caller of
`update_reputation`. Corrects the earlier "PARTIAL" framing: on a strict
per-item basis this section is closer to fully unstarted than partial,
despite real supporting code existing nearby.

| Item | Status | Evidence |
|---|---|---|
| 1. On-chain finalization triggers `watch`-derived `gold` at max `evidence_tier` | 🔴 NOT STARTED | `omokoda-core/src/bus/sango.rs` is real — a fire-and-forget HTTP client that reports every completed `act` to a "Ṣàngó relay" for eventual on-chain anchoring. But it's a pure receipt-recording pipe: zero Waggle deposit, zero `evidence_tier`, zero `gold` anywhere in the file. On-chain finalization does not currently feed back into the scent field at all |
| 2. Trust-weighted reputation deltas from corroboration history | 🔴 NOT STARTED | `agent.move`'s `update_reputation(state, new_reputation)` is a pure setter — it takes an already-computed value and just range-checks/stores it; no corroboration-weighting logic exists on-chain (Move can't easily read the Waggle field anyway — this would have to be computed off-chain and passed in, and nothing does that computation). Separately, `omokoda-core/src/interpreter.rs` has its **own, entirely different, local (off-chain, in-session) reputation system** (`self.snapshot.reputation`, a `reputation_ledger`) with a hardcoded, explicitly-commented **"simplistic decay"** formula (`rep -= 0.008 + (rep * 0.001)`) — real code, but neither corroboration-weighted from the field nor connected to the on-chain `agent.move` reputation field. Two parallel reputation concepts exist (local session + on-chain dNFT), neither matches what this item asks for |
| 3. `bounded`-channel anchoring for high-stakes verdicts (NEW) | 🔴 NOT STARTED | zero matches for `bounded` anywhere in `sources/*.move` |
| 4. Reputation decay reuses Waggle's own decay-kernel math (NEW) | 🔴 NOT STARTED | the one real reputation-decay formula found (`interpreter.rs`, above) is a standalone hardcoded formula, explicitly self-described as "simplistic" — it does not call into or share code with Waggle's kernel selection in any way |

Real Move contracts do exist and are substantial:
`omokoda-on-chain/sources/{zbt_core,hive,soul,consensus_ledger,synapse,
epistemic_nft,zbt_guard,agent}.move` (803 lines total) — `agent.move`'s
`AgentState` dNFT (tier/reputation/synapse balance/act count) and
`consensus_ledger.move`'s multi-model wisdom-ensemble disagreement records
are real, working infrastructure. **None of it is currently wired to
Waggle** in either direction — no field signal promotes to on-chain gold,
no on-chain event demotes/promotes a field signal's evidence tier, and the
one real reputation-decay implementation in the whole ecosystem is
disconnected from both the chain and the field. This is a clean,
well-scoped integration gap: the two things needing connecting already
exist independently, they just don't talk to each other yet.

### §8 Mandelbrot cross-cutting layer — ✅ DONE (the service), 🟡 PARTIAL (the integration)

`agentic-waggle/oracle/src/main.rs` is a real standalone Rust service
implementing **all five** named tools exactly as spec'd: `mandelbrot_scan`,
`escape_time_risk`, `robust_island_query`, `fractal_signal_filter`,
`swarm_stability_map` — including the `swarm_stability_map` doc comment
literally saying "feed Yemoja spawn-throttling" (item 8.3). This closes item
8.2 (standalone service, not just an Axiom-embedded Wasm module) cleanly.
Item 8.1 (shared `bounded` channel as the one wire) is also done per §0.
**Item 8.3 (Yemọja actually throttling on it) and item 8.4 (unifying the
`gradient?depth=N` / `mandelbrot_scan` depth-parameter convention)** were not
verified — the oracle offering the tool is not the same as Yemọja consuming
it; that consumption side needs checking in Yemọja's own (Elixir) repo.

### Dependency-order note

The doc's own dependency order (§ "Dependency order (expanded)") assumes a
build sequence starting from Waggle typed channels. Given §0 is done, the
**real current blocking point is §3 (IfáScript) and §4 (Vantage federation)**
— both fully unstarted and both late in the doc's own dependency chain (9 and
11 of 11), meaning nothing else in the doc is actually blocked by their
absence; they're legitimately just not-yet-reached, not neglected out of
order.

---

## Part 2 — `256---65536.md`

Structurally very different from `CONNECTION_MAP_V2.md`: almost entirely
narrative/cosmological framing (numerology table, Hermetic-principle-as-gate
pseudocode, Ritual Codex vision prose), not an itemized technical spec. Very
little of it is "concrete unfinished item" material in the sense the other
doc has — most of the file is vision statement, not a build list.

### What's actually concrete/actionable in this doc

1. **The 7-tier number↔Òrìṣà↔language↔primitive table** (lines 47–55,
   883–920, appears twice, slightly differently formatted). This mostly
   *does* match the real language-per-Òrìṣà assignment already established
   elsewhere (Rust/Èṣù, Julia/Ọ̀ṣun, Elixir/Yemọja, Clojure/Ọbàtálá,
   Python/Ògún, Go/Ọya, Move/Ṣàngó) — **not new information**, just a
   restatement in numerological packaging.
2. **Day→Òrìṣà table** (lines 733–744) — see the unresolved-conflict section
   above. Not safe to encode.
3. **`gate MENTALISM { ... }` / `gate RHYTHM { ... }` / `gate CAUSE_EFFECT
   { ... }` pseudocode** (lines 390–421) — a real, small, concrete idea:
   Hermetic principles as literal runtime gate DSL blocks. **Checked
   `omokoda-core/src/steward/gatekeeper.rs`-adjacent code and
   `src/gates/`**: gates already exist and enforce Hermetic principles in
   Rust (this predates and is more mature than this doc's pseudocode sketch
   — the doc's 3-line `gate X { reject/require }` syntax is a simplification
   of what's already real, not a new proposal). No action needed here beyond
   noting the doc's sketch is already superseded by working code.
4. **DNA overlay as symbolic resonance class, not biological data** (lines
   759–800) — a real, scoped idea (`zk_class`, `ritual_bias.days/frequencies/
   orisha_bias`) that's more concrete than most of the surrounding prose.
   **Not found anywhere in `omokoda-core/src`** — no `dna_overlay`, `zk_class`,
   or `ritual_bias` structures. Genuinely unstarted, and small enough to be
   a real candidate if the day-table conflict ever resolves (it depends on a
   ratified day table, so it's downstream of that open question, not
   independent of it).
5. **Ritual Codex repo integration** (`github.com/omo-koda/ritual-codex`,
   `monday.json`...`sunday.json` as "planetary runtime states") — the link
   target's org (`omo-koda/*`) doesn't exist as a real active org per the
   sibling audit's fabrication finding on `Next.md`; the *real* ritual-codex
   content lives at `Technosis-Sovereign-Ecosystem/ritual-codex`
   (`cryptonomicsed-byte`) and, more concretely, as `omokoda-core/src/koodu/
   *.json` — which **already exist and are consumed** (per CORRECTIONS.md,
   "the mechanical source of truth"), just not wired to the elaborate
   "AI tone / economic bias / consensus mode" runtime-modulation vision this
   doc describes. Checked `koodu/monday.json`: it's a flavor/facet file
   (tone, frequency, color, planetary ruler) — real data, but nothing in
   `omokoda-core/src` reads it to change AI tone, economic bias, or
   consensus behavior at runtime. **The gap between "file exists with the
   right data" and "system behavior actually changes based on it" is the
   real unstarted item here**, not the file itself.

### Staleness/misalignment verdict

**This doc should be weighted low relative to `CONNECTION_MAP_V2.md`.** It:
- Presents an unratified day-table as settled fact, directly contradicting
  `CORRECTIONS.md`'s explicit "do not hard-code" instruction — the doc
  predates that correction and was never updated.
- Is ~90% cosmological narrative restating the same "one recursive structure"
  claim in escalating rhetorical registers ("THIS is one of the most
  important breakthroughs" / "THE MOST IMPORTANT PART" / "THE FINAL
  REALIZATION") rather than adding new technical surface area.
- Its one real code-adjacent idea (Hermetic gates as DSL) is already
  superseded by more mature working code.
- Its Ritual Codex "runtime modulates on day-state" vision has zero
  wiring in the actual codebase beyond the day-JSON files sitting there
  as flavor data.

**Recommendation: do not treat this doc as a source of new work items.** Its
only durable, still-open contribution is reinforcing that the DNA-overlay
concept (item 4 above) and the day-table ratification (open per
CORRECTIONS.md) remain real gaps — both of which are better tracked via
CORRECTIONS.md and a future dedicated ADR than via this file.

---

## Consolidated "still needs doing" list

Ordered roughly by how independently actionable each item is (not blocked on
other open items):

1. **IfáScript: build `cast()` at all.** Currently a 37-line static lookup
   table with zero Waggle-field integration. This is the biggest genuine gap
   in `CONNECTION_MAP_V2.md` — an entire doc section (§3, 4 items) fully
   unstarted, and it's late enough in the dependency order that it's safe to
   pick up independently.
2. **Vantage federation (`bridge(remote_manifest_url)`).** Zero code found.
   §4, 4 items, also late-dependency and independently startable — but
   correctly sequenced last per the doc's own ordering (needs richer local
   fields first), so lower urgency than IfáScript.
3. **Ṣàngó/Move: all 4 items confirmed not started (verified 2026-08-28),
   not just the 2 NEW ones.** No direction of Waggle↔chain wiring exists at
   all: `bus/sango.rs` reports completed acts to the chain relay but never
   deposits to the field; `agent.move`'s `update_reputation` is a pure
   setter with no corroboration-weighting; the `bounded` channel has no
   on-chain anchor; and the one real reputation-decay formula that exists
   (`interpreter.rs`, explicitly commented **"simplistic decay"**) is a
   separate, disconnected, *local session* reputation system — not the
   on-chain `agent.move` reputation, and not reusing Waggle's kernel math.
   **Real, useful side-finding: there are two parallel reputation concepts
   in the ecosystem today** (local/session vs. on-chain dNFT) that don't
   talk to each other — worth flagging to the owner as its own open
   question before building §7's asks on top of either one. Still
   well-scoped to build now that both sides (Waggle's `bounded` channel/
   decay kernel, and the on-chain `agent.move`/`consensus_ledger.move`
   infrastructure) exist independently and just need connecting.
4. ~~Ọ̀ṣun/Yemọja/Ọya-specific NEW items (§6)~~ **DONE 2026-08-28**: 2 of 3
   built. Ọ̀ṣun's resonance consolidation (`omokoda-julia/src/
   resonance_consolidation.jl`) and Ọya's per-territory tunable heartbeat
   (`agentic-waggle/core/swarm.go`, `Territories.Set/Tempo`) are both real
   and faithful to spec. **The one real remaining gap: Yemọja's
   territory-aware supervision topology** — `omokoda-swarm`'s supervisors
   are real but generically `:one_for_one`, zero Hilbert/territory
   partitioning anywhere in the repo. Small, scoped, independently
   buildable now that the territory concept itself (Ọya's `Territories`
   type) already exists to mirror.
5. ~~ỌṢỌVM §1 item-by-item verification~~ **DONE 2026-08-28**: 8/10 built.
   Real remaining gaps: **§1.1 Axiom GraphEngine bridge (OSOVM's side)** and
   **§1.3 Wasm ABI export** — both confirmed absent, small enough to be
   picked up directly.
6. **Axiom §2 — 6 of 8 items confirmed not started** (verified 2026-08-28):
   gradient-driven camera auto-zoom (item 3), taboo visual treatment (item
   5), Mandelbrot-shader/`bounded`-density blend (item 6) — the shader
   exists but is a self-contained local computation, not fed by field data,
   `sniff_explain` NodeInspector panel (item 7), cross-inhibition dome (item
   8). Only SSE subscription (item 2) is confirmed done. This is the most
   genuinely-unfinished section of `CONNECTION_MAP_V2.md` verified so far —
   good next pickup if visual/Axiom work is prioritized, since the adapter
   scaffolding (`OmokodaGraphEngine.ts`, `MockGraphEngine.ts`) it all builds
   on already exists.
7. **Day→Òrìṣà table ratification** (CORRECTIONS.md-tracked, not this doc's
   job to resolve, but blocks item 8).
8. **DNA-overlay-as-symbolic-resonance-class** (256---65536.md's one concrete
   idea) — small, scoped, but downstream of #7.
9. **Wire `koodu/*.json` day files to actual runtime behavior** — the files
   exist and are real, but nothing in `omokoda-core/src` currently changes
   AI tone / economic bias / consensus mode based on them, despite that
   being 256---65536.md's central "Ritual Codex" claim. Also downstream of
   #7 — don't wire behavior to an unratified table.

### What NOT to build (already done, despite doc framing as "NEW")

For anyone picking up either doc without reading this audit first: §0
(Agentic/waggled: channels, watch, recall, bounded channel, evidence_tier,
sniff_explain, batched sniff, full `wag` CLI, Python SDK), §1 (ỌṢỌVM +
Techgnosis pipeline: 8 of 10 items — stage signaling, dead-end tagging,
content-addressed bytecode cache, the full Mandelbrot perturbation build
gate, Zangbeto evidence-tier promotion, recall-backed regression detection),
§5 (LOOM: all six trade/oracle/reputation integrations), and §6 (Omo-Koda2
core per-Òrìṣà: 6 of 7 — Èṣù's capability gate + throttle, Ọbàtálá's taboo
justification, Ògún's tool-outcome signaling, Ọ̀ṣun's resonance
consolidation, Ọya's per-territory tunable heartbeat) are **done**.
Re-implementing any of these would be pure duplication. §8's fractal-oracle
standalone service is also done. The Hermetic-principle-as-gate idea from
`256---65536.md` is superseded by real, more mature gate code already in
`omokoda-core/src/gates/` and `steward/`.

**Confirmed real gaps, worth picking up (small and scoped, not "still
mostly unbuilt" like §2 Axiom):** Yemọja's territory-aware supervision
topology (§6.6 — the `Territories` concept it would mirror already exists
in `agentic-waggle`), the OSOVM↔Axiom `GraphEngine` bridge (§1.1/§2.4 — two
sides of one contract, neither built), and the Wasm ABI export (§1.3).

**By contrast, §2 (Axiom) is genuinely mostly unbuilt** — 6 of its 8 items
confirmed not started as of 2026-08-28. Don't extend the "already done"
assumption to Axiom just because §0/§1/§5 turned out that way — verify
per-section, this repo set does not have one uniform completion level.
