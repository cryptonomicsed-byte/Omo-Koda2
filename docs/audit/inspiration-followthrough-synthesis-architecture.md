# Inspiration Followthrough — synthesis.md / architecture.md / UNIFIED_ARCHITECTURE.md

**Audit date:** 2026-08-27 · **Auditor:** pi-p6 · **Repo:** Omo-Koda2
**Purpose:** Read the three reference docs, list every concrete integration /
inspiration item they name, verify each against `omokoda-core/src` + sibling
subdirs, flag what's stale vs. still worth pursuing, and produce a concrete
"still needs doing" list.

> **Headline finding:** the three docs describe a **Week 1–4, 35-test** build
> state. The shipped code is at **759 verified tests** and has already absorbed
> almost every repo the docs flag as "HIGH VALUE / Week 2 / Week 3 / Week 4".
> The docs are therefore mostly **stale as a build plan** and useful mainly as
> an **archaeological source map** — the genuinely-open items are the
> cross-repo / on-chain / orchestration gaps listed at the bottom.

---

## 1. The three docs, one line each

| Doc | What it is | State |
|---|---|---|
| `docs/synthesis.md` | Full conversation synthesis; PART 12 = 46-repo audit map, PART 14 = build sequence | stale build plan, valuable repo map |
| `docs/architecture.md` | "Complete Unified Architecture" + SOURCE REFERENCES table | stale (claims `Àṣẹ REMOVED`, 35/35 tests) |
| `docs/UNIFIED_ARCHITECTURE.md` | Canonical living doc (LIVE/BUILT/SPEC tags), critical path | most current, still accurate on the *gaps* |

---

## 2. Integration / inspiration items — verified status

Legend: ✅ done · 🟡 partial/misaligned · ❌ not started · 🔀 cross-repo.

| # | Doc item (repo → extractable) | Status | Evidence in source |
|---|---|---|---|
| 1 | Oso-Aether → 3-primitive parser, WASM bridge, 86-DNA, tier ladder | ✅ | `parser.rs`, `identity/dna.rs`, `justice/tier.rs` |
| 2 | BIPON39 → 256 tokens, Odu index, argon2id, Merkle root | ✅ | `identity/bipon39.rs`, `identity/merkle.rs` |
| 3 | Omokoda → Twelfth Face | ✅ | `steward/twelfth_face.rs` |
| 4 | Omokoda → causal memory DAG | ✅ | `memory/dag.rs` |
| 5 | Omokoda → 11-lobe wisdom ensemble | ✅ | `omokoda-hermetic/src/wisdom/ensemble.rs` |
| 6 | Omokoda → Nautilus TEE | ✅ | `memory/tee.rs` |
| 7 | ritual-codex/Koodu → 7-day resonance, Sabbath | ✅ | `koodu/*.json`, `rhythm.rs` |
| 8 | IfáScript → 256 Odù → entropy opcodes | ✅ | `identity/odu.rs`, `divination.rs` |
| 9 | NarratorIDE → 8 personas → Wisdom archetypes | ✅ | `omokoda-hermetic/src/persona/engine.rs` |
| 10 | vanity-cloakseed → CloakSeed, duress, Poison Radar | ✅ | `identity/cloak.rs`, `duress.rs`, `safety.rs` |
| 11 | Twelve-thrones → consensus + epistemic NFT | ✅ | `omokoda-on-chain/sources/consensus_ledger.move`, `epistemic_nft.move` |
| 12 | Twelve-thrones → EpistemicSeverity | ✅ | `receipt/act_receipt.rs` |
| 13 | Zangbeto → Move guard/core/errors + bus | ✅ | `zbt_*.move` (4), `bus/zangbeto.rs` |
| 14 | Claw-code → tool registry, permission hooks, provider trait | ✅ | `tools/`, `permissions.rs`, `providers.rs` |
| 15 | Swibe → plugin hooks, receipt chain, provider fallback | ✅ | `plugins/`, `receipt/`, `providers/` |
| 16 | OpenClaw → 18 Sovereign capabilities | ✅ | `tools/sovereign.rs::sovereign_tool_list()` (tested ==18) |
| 17 | AIOS → Steward kernel design, scheduling | ✅ | `steward/`, `tasks/scheduler.rs` |
| 18 | Aider → coding/file-edit act backend | ✅ | `tools/file_ops.rs`, `tools/repl.rs` |
| 19 | Julia → BB verifier, Augury, Garden analytics, NIST | ✅ | `omokoda-julia/src/{bb_verifier,nist_validate,augury/*}.jl`, `omokoda-memory/src/garden_analytics.jl` |
| 20 | Busy Beaver → PoCW, difficulty compression | ✅ | `justice/busy_beaver.rs`, `omokoda-memory/src/busy_beaver.jl` |
| 21 | franken-stream → provider fallback/health | ✅ | `providers.rs` |
| 22 | LARQL → model-as-graph query | ✅ | `memory/larql_query.rs` |
| 23 | GlyphIndex → content-addressed memory | ✅ | `memory/glyph_memory.rs` |
| 24 | ZERO → weight-delta tool | ✅ | `tools/zero_tool.rs` |
| 25 | Walrus → blob memory | ✅ | `memory/walrus.rs`, `tools/walrus_tool.rs` |
| 26 | Nex- → graph execution | 🟡 | collapsed behind `act` by design (docs say "Archive") |
| 27 | Aether → job marketplace, witness-gated settlement | 🟡 | `garden.move`/`agent.move` exist; live job flow owned by Vantage |
| 28 | Techgnosis → @tithe/shrineSplit | 🟡 | 3.69% tithe live (`onchain.rs`); 50/25/15/10 shrine split not fully separated |
| 29 | Wallet masks → `elegbara_router.move` (8 sub-wallets) | 🟡 | `onchain.rs`/`bus/sango.rs` in Rust; **no `elegbara_router.move` contract** |
| 30 | NarratorIDE → 7 tones → Flow tone routing | 🟡 | personas done; **no `ToneEngine`/`DayResonance`/`TensionTracker`** in source despite README claiming them |
| 31 | Droidclaw → SOMA + IRIS | ✅ | `memory/soma.rs`, `steward/iris.rs` |
| 32 | Droidclaw → 24 phone tools / Kira social | ❌ | not present (mobile layer dropped) |
| 33 | Agent.TV → multi-agent orchestration, Pipecat voice | 🟡 | `mesh/`, `omokoda-swarm/`; voice not present |
| 34 | eternal-orisa-loom → content safety, tension tracking | 🟡 | `execution/risk_classifier.rs`, `gates/` |
| 35 | vibe-lang/vibe-coder → type-system / PDR / streaming | 🟡 | absorbed conceptually (`providers/streaming.rs`) |
| 36 | Warp → command-block UX | 🟡 | `omokoda-frontend` CommandForge (not audited here) |
| 37 | Tri-anchor receipts → Sui + **Arweave + Bitcoin OTS** | 🟡 | Sui anchor only (`receipt/`); **no arweave/opentimestamps** |
| 38 | The-Aether → audit before mainnet | ❌ | still open (pre-mainnet requirement) |
| 39 | TradingAgents → Tier-5 trading tool | ❌ | no `tools/trading.rs` (optional) |
| 40 | Immigration Office / Visas (World-ID gate) | ❌ | grep: no visa/immigration module (SPEC only) |
| 41 | Sentencing Engine (4-tier sanctions) | ❌ | SPEC only |
| 42 | organism-core bridges → live services :7777/:8787/:8001 | ❌ 🔀 | cross-repo; bridges still stubbed/simulated |
| 43 | Hive Mind v0 → Garden → LoRA fine-tune → Local provider | ❌ | LARQL+ZERO exist; the orchestration loop does not |
| 44 | sim→real mint v0 → VeilSim → Witness → mint | ❌ 🔀 | cross-repo with OSOVM; loop unwired |
| 45 | Axiom → live GraphEngine (off mock) | ❌ 🔀 | cross-repo; still on MockGraphEngine |
| 46 | World client Phase 1 (256×256 grid) | ❌ 🔀 | SPEC; OSOVM `world_tiles.jl` |
| 47 | Settlement token name + emission curve | ❌ ⏳ | owner-deferred decision |
| 48 | Nex: integrate into kernel or retire | ❌ ⏳ | open decision |
| 49 | Projects as first-class VM citizens | ❌ ⏳ | open decision (AIO thread) |

---

## 3. Stale / redundant / misaligned flags

1. **"Week 1–4 build sequence" + "35/35 tests, Steward is next"** (synthesis.md, architecture.md) — stale; shipped code is 759 tests with Steward/interpreter/identity/memory/tools all complete.
2. **`Àṣẹ REMOVED — does not exist`** (architecture.md) — **contradicts** UNIFIED_ARCHITECTURE.md and the live `identity/ase.rs` + OSOVM tokenomics; superseded by the "settlement token, reframed" decision.
3. **Directory drift** — docs say `omokoda/contracts/sources/` and `omokoda-core/src/flow/`; actual is `omokoda-on-chain/sources/` and `steward/iris.rs` (IRIS lives in steward, not flow).
4. **Two architecture docs** — `specs/architecture.md` (frozen) vs `docs/architecture.md` (synthesis) vs `docs/UNIFIED_ARCHITECTURE.md` (canonical); only the last is current.
5. **Droidclaw "24 phone tools / Kira social network"** listed as HIGH VALUE — never pursued, and correctly so: the shipped architecture is a single Rust core, no mobile layer. The "HIGH VALUE" verdict is now moot.
6. **`The-Aether` "role unknown — MUST audit"** — still open and still a real pre-mainnet item (the only un-audited reference repo).
7. **ToneEngine/DayResonance/TensionTracker** — README lists them as "Completed & Verified" but they are not in `omokoda-core/src`; either folded under another name or the README overstates. Worth a reconcile.

---

## 4. Still needs doing (concrete, actionable, prioritized)

**A. On-chain / identity gaps (in this repo)**
1. **Tri-anchor receipts** — add Arweave blob + Bitcoin OpenTimestamps to `receipt/act_receipt.rs` (Sui anchor already exists). Concrete file: `receipt/mod.rs`.
2. **Wallet masks as Move** — promote the Rust `elegbara` routing (`onchain.rs`, `bus/sango.rs`) into a real `elegbara_router.move` with 8 isolated sub-wallets, or document why Rust-side routing is sufficient.
3. **shrineSplit 50/25/15/10** — ensure the TechGnØŞ.EXE offering split is separated from the 3.69% AIO tithe (currently both seem folded into the single tithe path).
4. **Reconcile ToneEngine/DayResonance/TensionTracker** — find or restore, or delete the README claim.

**B. SPEC-only systems (design before code)**
5. **Immigration Office / Visas** — World-ID-bound citizenship gate in front of `birth` (anti-Sybil spine for hive training + mint). No code yet.
6. **Sentencing Engine** — 4-tier sanctions (Notice→Probation→Suspension→Revocation) feeding Zàngbétò verdicts. No code yet.
7. **The-Aether audit** — pre-mainnet requirement; determine if it contains anything beyond `Aether`.

**C. Cross-repo wiring (the "two bodies don't touch" gap)**
8. **organism-core bridges → live services** — turn the simulated bridges into real calls to :7777/:8787/:8001. This is the single highest-leverage item in UNIFIED_ARCHITECTURE's critical path.
9. **sim→real mint v0** — wire VeilSim commitment → Witness-firmware attestation → Sui mint (Twelve-thrones jury + Zàngbétò judge).
10. **Axiom → live GraphEngine** — replace the mock with real data from :7777.
11. **Hive Mind v0** — the LARQL+ZERO pieces exist; build the orchestration: aggregate Garden → LoRA fine-tune → serve as Local provider.

**D. Owner-deferred decisions (unblock, don't build)**
12. Settlement token name + emission curve (reconcile `ase.move` cap vs infinite-asymptotic).
13. Nex: integrate into kernel reasoning or formally retire.
14. Projects as first-class VM citizens (decide before OSOVM VM coding).

---

## 5. Bottom line

The docs' inspiration has been **~90% consumed** — the shipped Rust core already contains SOMA, IRIS, Twelfth Face, the causal DAG, the 11-lobe ensemble, persona engine, Cloakseed, Poison Radar, BIPON39, LARQL, GlyphIndex, ZERO, Walrus, Busy Beaver, the 18 Sovereign tools, and the full Move contract set. What remains is **not more repo-mining** — it's (a) the few on-chain/identity gaps, (b) the SPEC-only citizenship/enforcement systems, and (c) the cross-repo wiring that joins the live body to the on-chain half. That cross-repo wiring (`organism-core`, `sim→real mint`, `Axiom`, `Hive Mind`) is the real remaining work.
