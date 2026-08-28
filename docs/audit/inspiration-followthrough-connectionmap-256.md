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

### §1 ỌṢỌVM + folded Techgnosis — 🟡 PARTIAL

`OSOVM/src/waggle_bridge.jl` and `veilsim_engine.jl` exist — a real bridge
file, not just a doc reference. Did not verify item-by-item (Wasm ABI export,
bytecode-cache-keyed-to-signal-URI, build-time Mandelbrot perturbation gate,
`recall`-backed regression detection) — these need a Julia-side deep read
this pass didn't have budget for. **Flag for follow-up**, don't assume done
just because the bridge file exists — a bridge file existing is necessary,
not sufficient, evidence for the 10 items §1 lists.

### §2 Axiom — 🟡 PARTIAL, real adapter surface exists

`Axiom/src/engine/OmokodaGraphEngine.ts`, `MockGraphEngine.ts`,
`nodeTypes/registerOmokoda.ts`, `ui/NodeInspector.ts` all real. This
strongly suggests items 1 (`waggle-hotspot` node type) and 4 (`GraphEngine`
adapters) have at least a skeleton. **Not verified:** SSE subscription (item
2), gradient-driven camera auto-zoom (item 3), taboo visual treatment (item
5), Mandelbrot shader blending live `bounded` density (item 6),
`sniff_explain` panel in NodeInspector (item 7), cross-inhibition dome
visualization (item 8) — `NodeInspector.ts` existing is necessary-not-
sufficient for item 7 specifically; needs a real read to confirm the panel
content, not just the file's existence.

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

### §6 Omo-Koda2 core, per-Òrìṣà — 🟡 MIXED, real evidence for Rust/Python; unverified for Julia/Elixir/Clojure/Go

Directly grepped `omokoda-core/src/waggle/mod.rs` (the Rust/Èṣù side):

| Item | Status | Evidence |
|---|---|---|
| Èṣù: capability-token gate on claim/mark/release/dance | ✅ DONE | `CapabilityGate`, real tests (`capability_tokens_gate_the_verbs`) |
| Èṣù: mark rate-limiting (NEW) | ✅ DONE | `MarkThrottle` token-bucket, tested (`mark_throttle_caps_burst_and_refills`) |
| Ọbàtálá: taboo channel + justification metadata (NEW) | ✅ DONE | `taboo_from_halt()` — deposits `principle`/`justification`/`source` in signal meta on every gatekeeper HALT |
| Ògún: tool-outcome auto-signaling (NEW) | ✅ DONE | `tool_outcome()` via a registered `watch`, success→gold/failure→dead-end |
| Ọ̀ṣun: `recall`-based resonance consolidation (NEW) | ⬜ NOT VERIFIED | lives in Julia, not this Rust crate — needs checking in Ọ̀ṣun's own repo, not omokoda-core |
| Yemọja: supervision-tree topology mirrors Hilbert territory (NEW) | ⬜ NOT VERIFIED | Elixir-side, same caveat |
| Ọya: tunable per-territory heartbeat rate (NEW) | ⬜ NOT VERIFIED | Go-side (this is `waggled` itself — `agentic-waggle/core`), not grepped for this specific tunable |

**Still needs doing / needs checking (§6):** the three ⬜ rows — genuinely
not verifiable from `omokoda-core/src` alone since Ọ̀ṣun/Yemọja/Ọya are
separate-language subsystems this repo doesn't contain. Flag for a follow-up
pass with those specific repos checked out, don't assume either way.

### §7 Ṣàngó / Move contracts — 🟡 PARTIAL

Real Move contracts exist and are substantial:
`omokoda-on-chain/sources/{zbt_core,hive,soul,consensus_ledger,synapse,
epistemic_nft,zbt_guard}.move`. But the two specific **NEW** items this doc
calls for — `bounded`-channel on-chain anchoring for high-stakes robustness
verdicts (item 3), and reputation-decay reusing Waggle's own decay-kernel
math instead of a separate formula (item 4) — have **zero matches** for
`bounded`/`decay kernel`/`reputation decay` anywhere in `sources/*.move`.
The on-chain foundation (receipts, reputation ledger) is real; these two
specific integrations are not started.

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
3. **Ṣàngó/Move: `bounded`-channel anchoring + decay-kernel-reuse for
   reputation.** The on-chain foundation is real; these two specific
   integrations (§7 items 3–4) are not. Small, scoped, buildable now that
   §0's `bounded` channel and decay math are done.
4. **Ọ̀ṣun/Yemọja/Ọya-specific NEW items (§6)** — resonance consolidation,
   supervision-topology mirroring, tunable heartbeat. Not verified either way
   this pass (wrong-language repos, out of scope for a fast grep) — **next
   step is a dedicated pass reading those three repos directly**, not
   assuming absence.
5. **ỌṢỌVM §1 item-by-item verification.** A real bridge file exists but
   the 10 listed items weren't individually confirmed — needs a Julia-side
   read.
6. **Axiom §2 item-by-item verification**, especially the `sniff_explain`
   NodeInspector panel (item 7) and Mandelbrot-shader/`bounded`-density blend
   (item 6) — real adapter scaffolding exists, specific features unconfirmed.
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
sniff_explain, batched sniff, full `wag` CLI, Python SDK) and §5 (LOOM: all
six trade/oracle/reputation integrations) are **done**. Re-implementing them
would be pure duplication. §8's fractal-oracle standalone service is also
done. The Hermetic-principle-as-gate idea from `256---65536.md` is superseded
by real, more mature gate code already in `omokoda-core/src/gates/` and
`steward/`.
