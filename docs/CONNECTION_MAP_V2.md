# Full Connection Map v2: Doubled Depth, Techgnosis Folded Into ỌṢỌVM, Mandelbrot as Cross-Cutting Layer

Correction applied: Techgnosis is a subsystem of ỌṢỌVM, not a sibling repo — its bytecode pipeline features now live under ỌṢỌVM's section below. This version doubles the enhancement count per repo and adds Mandelbrot dynamics as a load-bearing analytical layer threaded through the whole stack, not a LOOM-only feature.

> Follow-up specs now live in the Agentic repo:
> **`docs/BOUNDED_CHANNEL.md`** (the `bounded`-channel decay kernel math) and
> **`docs/FRACTAL_ORACLE.md`** (the standalone `fractal-oracle` service
> interface) — the piece unblocking dependency-order steps 3–7 below.

---

## 0. Agentic/Waggle core — the prerequisite everything else depends on (expanded)

### `core/waggled` (Go)

1. Extend signal schema from flat string to `{channel, subtype, decay_kernel, cross_inhibits: [channel...]}`.
2. `/.well-known/waggle.json` gains a `channels` block — self-registration, zero Go source changes per new power.
3. `watch` verb — subscribe write-paths (file glob, journal tail, webhook, Sui event log) and derive deposits from state transitions instead of requiring `mark` calls.
4. `recall(uri, at_time)` — second read path into the journal, ignores live decay, queries historical field state.
5. Spatial diffusion pass over the existing Hilbert position — 5% bleed to neighbors per tick, before primary decay.
6. Dynamic evaporation — claim-velocity-derived half-life multiplier, reusing lease-tracking data already collected.
7. **NEW: `bounded` channel reserved for Mandelbrot verdicts** — a distinct signal type whose value isn't gold/dead-end binary but a continuous fragility score (0.0 escape-immediately to 1.0 deep-bounded-island), decaying on its own kernel tuned slower than gold (robustness findings should persist — that's the whole point of "robust").
8. **NEW: `evidence_tier` field on every signal** — self-report < corroborated-by-second-agent < watch-derived < on-chain-anchored. This is the trust ladder that Ṣàngó, Ọbàtálá, and the Mandelbrot layer all read from, rather than each building their own ad hoc trust math.
9. **NEW: `sniff_explain` verb** — returns not just the gradient number but *why*: which signals, which evidence tiers, which cross-inhibitions contributed. Needed once taboo/bounded/gold all interact — agents (and humans watching Axiom) need to debug why a hotspot reads the way it does.
10. **NEW: batched multi-URI `sniff`** — a single call accepting an array of URIs, returning gradients for all, so agents doing depth-first exploration don't pay N round-trips for N candidate branches.

### `sdk/python`

11. `response_thresholds` on agent profile; probability-weighted `sniff`.
12. **NEW: `evidence_tier`-aware filtering** — an agent can request "only corroborated-or-better" signals, filtering out unverified self-reports when the cost of following bad scent is high (e.g. before committing real capital in LOOM).
13. **NEW: `subscribe_channel(channel, callback)`** — reactive SDK primitive so agents don't have to poll `sniff` on a loop; the SSE stream already exists at the daemon level, this just exposes it idiomatically in Python.

### `cli/wag` (Rust)

14. `wag recall <uri> --at <timestamp>`.
15. `wag channels list`.
16. **NEW: `wag explain <uri>`** — CLI wrapper on `sniff_explain`, for humans debugging the field from a terminal without opening the Observatory.
17. **NEW: `wag replay --speed <n>`** — replays journal history at variable speed into the live Observatory feed, useful for post-mortem review of how a hotspot formed (or a taboo suppressed a bad path) after the fact.

---

## 1. ỌṢỌVM (Julia/C/C++ VM) — now including the Techgnosis bytecode pipeline as a native subsystem

### Core VM bridge

1. `osovm-bridge` (Julia) implements Axiom's `GraphEngine` — translates process lifecycle to `node_spawned`/`node_updated`/`node_died`.
2. `watch`-registered auto-deposit: every Zangbeto-verified execution receipt writes `gold` at `evidence_tier: watch-derived`.
3. Four-function Wasm ABI export so ỌṢỌVM plugs into Axiom exactly like `rust-wasm-leaf`.
4. `sniff` before spawning new ritual computation — cache hit on identical computation signature short-circuits recompute.

### Techgnosis pipeline, now internal to ỌṢỌVM

5. Stage-transition signaling (`parsing` → `type-checking` → `codegen` → `emitted`) via `watch` on ỌṢỌVM's own compile log — no separate instrumentation layer needed since it's the same process boundary now, not a cross-repo call.
6. `dead-end` auto-deposit on compile failure, tagged with failing stage — since Techgnosis and the VM share memory space now, this can be a direct in-process signal deposit rather than a webhook, cutting one network hop out of the loop entirely.
7. **NEW: bytecode cache keyed to signal URI** — a `gold`-marked compiled artifact is retrievable directly via its URI, turning Waggle into a de facto content-addressed build cache for the whole VM, not just a coordination layer. This is a genuinely new capability: compilation results become part of the scent field itself.
8. **NEW: `bounded`/Mandelbrot verdict on bytecode stability** — ỌṢỌVM runs newly-compiled bytecode through a lightweight perturbation test (small input variations) and deposits a `bounded` signal reflecting whether the compiled agent logic behaves as a robust island or a brittle escape zone under input noise. This turns Mandelbrot fragility analysis into a *build-time* gate, not just a runtime/trading concept — bytecode that's fragile under perturbation gets flagged before it's ever deployed.
9. **NEW: Zangbeto verification results feed `evidence_tier` directly** — Zangbeto's pass/fail becomes the mechanism that promotes a signal from `watch-derived` to a distinct `zangbeto-verified` tier, one step below on-chain anchoring in trust ranking.
10. **NEW: `recall`-backed regression detection** — before shipping a new ỌṢỌVM build, query `recall` on the affected URIs to compare current bounded/fragile classification against historical — a build that silently makes previously-robust logic newly brittle gets caught automatically.

---

## 2. Axiom (TypeScript/Three.js galaxy) — expanded

1. `waggle-hotspot` node type via existing pluggability.
2. SSE subscription direct from Waggle's event stream.
3. Gradient-driven camera auto-zoom using `GET /v1/gradient?depth=N`.
4. `osovm-bridge`, `loom-bridge`, `vantage-bridge` as `GraphEngine` adapters (Techgnosis folded into `osovm-bridge` per the correction above).
5. **NEW: taboo-channel visual treatment** — distinct slow-pulsing red-black glow, visually distinguishable from fast-decaying dead-end red, so a human glancing at the galaxy can tell "ethically excluded" from "just didn't work out" at a glance.
6. **NEW: Mandelbrot shader driven by live `bounded` signals, not just the standalone Fractal Oracle demo** — the existing escape-time shader in `postfx.ts` currently renders from the oracle's own `mandelbrot_scan`; extend it to also blend in real `bounded` signal density from the field, so the fractal visualization reflects actual ecosystem-wide robustness, not only the local oracle's sandboxed computation.
7. **NEW: `sniff_explain` panel in NodeInspector** — clicking a hotspot shows the evidence-tier breakdown (self-report vs corroborated vs on-chain), giving humans (and debugging agents) the "why" behind a glow, not just the glow.
8. **NEW: cross-inhibition visualization** — when a taboo signal is suppressing a nearby gold reading, render a visible dampening field (a translucent dome) around the taboo source, making Ọbàtálá's ethical gating *visually legible* rather than an invisible math adjustment.

---

## 3. IfáScript — doubled

1. `cast(uri_pattern)` — casts against `recall`'s historical signal shape, returns matched Odù pattern from real operational history instead of a fixed table.
2. **NEW: `cast_bounded(uri_pattern)`** — a divination variant specifically over the Mandelbrot `bounded` channel: instead of asking "what happened here," it asks "is this a robust island or fragile boundary," giving IfáScript a genuinely new oracle mode — fortune-telling about *stability*, not just history.
3. **NEW: Odù-pattern-to-channel mapping table** — formalize which of the 256 Odù correspond to which signal-channel shapes (a pattern dominated by gold with low taboo interference maps to an auspicious Odù; heavy taboo suppression maps to a warning Odù). This gives the divination system a principled, non-arbitrary grounding in the actual field math rather than a lookup table someone hand-authored.
4. **NEW: `cast_federated(remote_prefix, uri_pattern)`** — once Vantage bridges a remote field, IfáScript can divine across ecosystems, casting against another Technosis deployment's history through the same interface, with the trust-discount already applied by Vantage's bridge.

---

## 4. Vantage (federation) — doubled

1. `bridge(remote_manifest_url)` verb, absorbed natively into Vantage.
2. Namespace-prefixed subscription with trust-discount multiplier on foreign signals.
3. **NEW: bidirectional bridge negotiation** — rather than one-way subscription, Vantage negotiates a manifest handshake where both sides declare which channels they're willing to export (an ecosystem might federate `gold`/`bounded` but keep `taboo` judgments private/local, since ethics evaluations may be context-specific and shouldn't silently propagate).
4. **NEW: federation health signal** — Vantage itself deposits a `federation-health` meta-signal reflecting bridge uptime/latency, so agents can `sniff` whether a remote field is currently reliable before trusting its imported gold as strongly as local gold.
5. **NEW: Mandelbrot cross-ecosystem robustness comparison** — bounded/fragile classifications from a federated remote field get compared against local classifications for overlapping resource types (e.g. two ecosystems both running similar strategy logic) — divergent verdicts on structurally similar work are themselves an interesting signal worth surfacing (either one ecosystem has better data or a real environmental difference exists).

---

## 5. LOOM (market/agent-native runtime) — doubled

1. Confluence Forge trade outcomes (filled/rejected) auto-deposit gold/dead-end via `watch` on strategy-parameter URIs.
2. Fractal Oracle bounded/escape verdicts wired into the field as `bounded`-channel deposits, closing the visualization-to-substrate gap.
3. `MarketEvent`s map to both Axiom `message_pulse` and Waggle deposits from one event.
4. **NEW: regime-shift detection via `bounded` decay rate** — a strategy parameter whose `bounded` signal is decaying unusually fast (relative to its configured half-life) indicates the market regime underneath it is shifting — this is a genuinely novel early-warning signal: not "the strategy lost money" (lagging) but "the ground it stands on is destabilizing" (leading), derived purely from decay-rate anomaly rather than any new model.
5. **NEW: Sniper/Warrior/Scalper preset competition via quorum-gated selection** — instead of a fixed preset per strategy, LOOM's agents `sniff` each preset's live gold/bounded gradient and probabilistically allocate capital toward whichever preset the field currently favors, self-rebalancing without a human flipping a config flag.
6. **NEW: `swarm_stability_map` (already an Oracle tool) feeds Ṣàngó reputation weighting** — a trading agent that consistently deposits gold in regions later confirmed `bounded` (robust) by the Oracle earns reputation faster than one whose gold regions later reveal themselves as fragile escape zones under Mandelbrot re-analysis.
7. **NEW: dead-cat-bounce filter** — a `gold` signal appearing inside a region the Oracle currently classifies as a fragile escape zone gets automatically down-weighted (cross-inhibition, not deletion) — a short-term win inside known-brittle territory is treated with appropriate skepticism by the field itself, before any human or agent has to manually flag it.

---

## 6. Omo-Koda2 core, power by power — doubled

### Èṣù (Rust)

1. Capability-token gate on `claim`/`mark`/`release`/`dance`, folded into existing session/security dispatch.
2. **NEW: rate-limiting as a first-class scheduler concern** — Èṣù already handles cooldown enforcement generally; extend it to throttle `mark` frequency per-agent, preventing a single misbehaving or looping agent from flooding the field with junk signals faster than decay can clean them.

### Ọ̀ṣun (Julia)

3. Consumes `recall` for semantic resonance calculations over historical field state.
4. **NEW: resonance-weighted memory consolidation** — periodically, Ọ̀ṣun queries `recall` across a whole territory (not just single URIs) and consolidates recurring bounded/gold patterns into RACK's longer-term memory structures, effectively promoting field-level emergent patterns into Ọ̀ṣun's own symbolic memory — this is the mechanism that turns "scent that happened to work" into "wisdom the memory system actually retains" independent of Waggle's own decay.

### Yemọja (Elixir)

5. Reads quorum-gated gradient rollups to decide spawn location/count proportionally.
6. **NEW: supervision-tree topology mirrors territory topology** — Yemọja's `DynamicSupervisor` tree structure is reshaped to match the Hilbert-curve territory boundaries Waggle already computes, so a supervisor crash/restart only affects agents working the same scent-territory, containing blast radius along the same boundaries the field already recognizes as coherent.

### Ọbàtálá (Clojure, or Prolog per earlier discussion)

7. `taboo` channel, slow-decay kernel, cross-inhibition against other channels.
8. **NEW: taboo justification attached as signal metadata** — not just a suppression value but a short symbolic-reasoning trace (which Hermetic principle triggered the exclusion), retrievable via `sniff_explain`, so a taboo isn't a silent wall — any agent hitting it can query *why*.

### Ògún (Python)

9. Primary gold/dead-end depositor, existing forage_swarm pattern.
10. **NEW: tool-outcome auto-signaling** — extend `watch` coverage to Ògún's tool-execution layer generally (not just the demo), so every external integration call's success/failure becomes a field signal by default, making the whole tool-execution surface stigmergic without per-tool instrumentation.

### Ọya (Go — this is `waggled` itself)

11. Already the substrate; formalize the identity in docs.
12. **NEW: Ọya-as-rhythm literalized** — expose the diffusion/decay tick rate as a tunable "heartbeat" parameter per territory, so different regions of the ecosystem (fast-moving LOOM trading territory vs. slow-moving Ọbàtálá ethics territory) can run on different rhythms without forking the daemon.

---

## 7. Ṣàngó / Move contracts (Sui) — doubled

1. On-chain finalization triggers `watch`-derived `gold` at maximum `evidence_tier`.
2. Trust-weighted reputation deltas from corroboration history.
3. **NEW: `bounded`-channel anchoring for high-stakes verdicts** — when a Mandelbrot robustness classification crosses a significance threshold (very deep bounded island, or very fast escape), Ṣàngó optionally anchors that specific verdict on-chain too, not just gold findings — this makes robustness claims themselves auditable and non-repudiable, useful if strategy-robustness claims ever need to be defensible to an external party.
4. **NEW: reputation decay mirrors signal decay math** — instead of a separate reputation formula, Ṣàngó reuses Waggle's own exponential/power-law kernel selection for reputation decay, so a component of trust literally uses the same decay physics the rest of the ecosystem runs on — one mental model, one set of tunables, instead of parallel reputation and pheromone math that could drift out of sync.

---

## 8. Mandelbrot as a cross-cutting layer, not a LOOM feature

This is the structural addition this version adds beyond the original map — Mandelbrot dynamics stop being "a LOOM/Axiom demo" and become a shared analytical primitive available to every power:

1. **Shared `bounded` channel** (already specified in §0.7) is the single wire every repo above writes to and reads from — ỌṢỌVM (bytecode stability), LOOM (strategy robustness), Ṣàngó (verdict anchoring), Vantage (cross-ecosystem comparison) all use the *same* channel and kernel rather than each reinventing fragility scoring. Kernel math: `Agentic/docs/BOUNDED_CHANNEL.md`.
2. **A shared `fractal-oracle` service, not a per-repo embedded Wasm module** — currently the Oracle lives inside Axiom as a Wasm species. Promote it to a standalone service (Rust or Julia, given the numerical work) that any power can call via MCP-style tool invocation (`mandelbrot_scan`, `escape_time_risk`, `robust_island_query`, `fractal_signal_filter`, `swarm_stability_map`) — Axiom keeps its embedded copy for offline/demo use, but the ecosystem-wide source of truth is one service everyone queries, avoiding N slightly-divergent fractal implementations across ỌṢỌVM, LOOM, and Axiom. Interface spec: `Agentic/docs/FRACTAL_ORACLE.md`.
3. **`swarm_stability_map` becomes the input to Yemọja's spawn-throttling** — if the Oracle reports the current swarm's parameter-space is trending toward a fragile boundary (not just individual strategies, but the *swarm's aggregate behavior*), Yemọja throttles new spawns in that territory rather than adding more agents to an already-destabilizing region.
4. **Depth-rollup gradient + Mandelbrot depth are the same concept, unify the API** — Waggle's `gradient?depth=N` (tree-depth rollup) and the Oracle's `mandelbrot_scan` (iteration-depth escape time) are conceptually the same operation — recursive refinement toward more detail — applied to different domains. Worth a shared depth-parameter convention across both APIs so agents reasoning about "how deep should I look" use one mental model for both scent-gradient exploration and fractal robustness exploration.

---

## Dependency order (expanded)

1. Waggle typed channels + `watch` + `recall` + `evidence_tier` + `bounded` channel.
2. Èṣù auth gate + rate-limiting.
3. Standalone `fractal-oracle` service extraction (unblocks everyone who needs `bounded` beyond Axiom's embedded copy).
4. ỌṢỌVM bridge + folded-in Techgnosis pipeline signaling + bytecode cache + build-time Mandelbrot gate.
5. Axiom node type + taboo/bounded visual treatment + `sniff_explain` panel.
6. Ṣàngó on-chain relay + reputation-decay-kernel reuse.
7. LOOM auto-signaling + regime-shift detection + preset quorum-selection.
8. Ọbàtálá taboo channel + justification metadata (needs multiple live channels to test cross-inhibition against).
9. Ọ̀ṣun resonance consolidation + Yemọja topology mirroring (need real field history to be worth building against).
10. IfáScript `cast`/`cast_bounded` (needs a populated journal).
11. Vantage federation + bidirectional negotiation + cross-ecosystem Mandelbrot comparison (last — federates fields that need to already be rich).
