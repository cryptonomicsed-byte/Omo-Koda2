# Reference Repository Map: Portability Analysis

This document maps architectural patterns and components from the omo-koda ecosystem and other reference repositories that can be ported or adapted for Ọmọ Kọ́dà.

## 1. Claw-code (Rust)
Claw-code is a mature, production-grade agent runtime. Since it is also written in Rust, many components can be directly ported or used as high-fidelity references.

### High-Fidelity Ports
- **Session Persistence System**: Replace `MemoryEntry` with structured `ConversationMessage` and `ContentBlock`. Add versioned JSON persistence.
- **Permission System**: Map `PermissionMode` to Ọmọ Kọ́dà reputation tiers (0-5). Implement tiered authorization.
- **Config System**: Agent-centric configuration loading (e.g., `~/.omokoda/agents/{id}/settings.json`).
- **File Operations**: Tier-gated `read_file`, `write_file`, `edit_file`, `glob`, and `grep`.
- **Bash Execution with Sandbox**: Linux `unshare` namespace isolation for sandboxed tool execution.
- **Hook System**: Pre/post tool use hooks for the Justice module (reputation scoring, tier assignment).

### Moderate Adaptation
- **ConversationRuntime**: The core loop for the Steward, generic over provider and tool executor.
- **API Provider Abstraction**: Implement `LocalProvider` (Ollama, WebLLM) for `/private` and `ExternalProvider` (Claude, OpenAI) for public use.
- **Usage Tracking**: Map token usage to Synapse/Dopamine cost.

## 2. Claude-2 (TypeScript/Claude Code)
Claude-2 represents a sophisticated production agent harness. Ports are conceptual translations from TypeScript to Rust.

### Key Architectural Patterns
- **Async Generator Agent Loop**: Unifies streaming, termination, and errors into a single flow.
- **5-Level Context Compression**: Content replacement → Snip → Microcompact → Collapse → Autocompact.
- **7-Layer Safety Stack**: Tool pre-filtering → Deny-first rules → Permission modes → Auto-mode classifier → Shell sandboxing → No permission restore on resume → Hook interception.
- **Process-Based Sub-agents**: Independent subprocesses with restricted tool sets for isolation.

## 3. Swibe (Agent-native Scripting)
Swibe provides the conceptual reference for the `birth`/`think`/`act` surface language.
- **Neural Router**: 86 cortical parameters derived from seed.
- **Three-Tier Memory**: Working, short-term, and long-term (encrypted) hierarchy.
- **Hermetic Ethics Engine**: 7 principles with AST visitor enforcement.

## 4. Conflict Resolution & Alignment
| Conflict | Reference Repos | Ọmọ Kọ́dà Alignment |
| :--- | :--- | :--- |
| Identity | BIPỌ̀N39 (16x16) | **Keep 256 Odu Ifá**. |
| Tokenomics | Àṣẹ/Dopamine/Synapse | **SUI-only** human-facing; Dopamine/Synapse internal. |
| Language | 35+ keywords | **3 words forever** (`birth`, `think`, `act`). |
| Code Source | Claude-2 mirrored source | **Patterns only**. Write fresh Rust designs. |

## 5. Full verdict table — all 46 repos from `reference_repos.md` (2026-08-27 audit)

The 3 repos above (Claw-code, Claude-2, Swibe) had real verdicts. The other 43 listed in `reference_repos.md` had never been individually evaluated — this table closes that gap. Method: read each repo's real purpose (README/source), then grep `omokoda-core/src` and `Cargo.toml` for concrete evidence of an actual port, not just thematic overlap. **Incorporated** means real, cited source evidence exists (a file, a dependency, a doc comment naming the source). **Rejected** means evaluated and consciously not ported. **Deferred** means a real, scoped-but-not-yet-built gap. **Not-applicable** means the repo doesn't map to omokoda-core in a portable way (wrong domain, or it's an external tool consumed as-is rather than a pattern source).

| # | Repo | What it actually is | Verdict | Reason |
|---|---|---|---|---|
| 1 | Oso-Aether | Rust/WASM ASCII-pet interpreter, 3-primitive language (`birth`/`think`/`act`), on-chain Sui identity | **incorporated** | Direct conceptual source of Ọmọ Kọ́dà's own `birth`/`think`/`act` 3-word language (already credited via Swibe/OsO/Aether family in §3-4; this is the WASM-compiled generation of the same lineage). |
| 2 | OsO | Earliest 3-primitive "Own My Own" language, same birth/think/act syntax | **incorporated** | Same lineage as #1 — the literal syntax `birth "x" / think "y" / act "z"` is Ọmọ Kọ́dà's real parser grammar (`parser.rs`, `interpreter.rs`). |
| 3 | Aether / The-Aether (dup listing) | Node.js "sovereign agent language," on-chain metabolism, verifiable marketplace | **rejected** | Same 3-generation lineage as #1-2 but the actual shipped generation adopted was the Rust one (Oso-Aether/OsO); this Node.js/JS generation's runtime was not ported — superseded before real integration. |
| 4 | Swibe | Already verdicted in §3 | — | See §3 above. |
| 5 | Nex- | Agent-native graph-runtime VM (Technosis ecosystem), real-time computation graphs / neural routing | **not-applicable** | Lives as its own standalone execution layer (paired with `organism-core`), consumed as an external graph server if ever wired — no evidence of its runtime being ported into `omokoda-core/src`. |
| 6 | Kimi-bino | Vanilla Vite+React+TypeScript scaffold template, no agent-specific content | **not-applicable** | Boilerplate starter template, not an architecture — nothing to extract. |
| 7 | Claw-code | Already verdicted in §1 | — | See §1 above. |
| 8 | Claude-mirror | Mirror of `@anthropic-ai/claude-code` npm package | **rejected** | Same source as #10 (Claude-2); mirrored copy explicitly ruled "patterns only, write fresh Rust" per the Conflict Resolution table (§4) — this literal mirror was never a code source. |
| 9 | Claude | Same `@anthropic-ai/claude-code` upstream, unmirrored | **rejected** | Same reasoning as #8. |
| 10 | Claude-2 | Already verdicted in §2 | — | See §2 above. |
| 11 | franken-stream | Terminal media streamer (movies/TV), Vantage Agent TV integration | **not-applicable** | Media/entertainment tool, no agent-kernel architecture to port. (Note: `tools/ytforge.rs`'s SkillForge YouTube-intake feature is unrelated — no real connection to franken-stream despite the naming echo.) |
| 12 | BIPON39 / bipon39 | Key-derivation library, 16×16 mnemonic space | **incorporated** | Real, live `Cargo.toml` dependency: `bipon39 = { git = "https://github.com/cryptonomicsed-byte/BIPON39", rev = "1c4d5c9" }` — not a pattern port, a compiled dependency. |
| 13 | Osovm / OSOVM | The Move/Sui proof-and-settlement VM pillar | **incorporated** | Real integration surface: `onchain.rs`, `tools/onchain_tools.rs` call out to OSOVM as the settlement/proof layer — pillar-level wiring, not a pattern extraction. |
| 14 | Omokoda / "Omokoda Agent" | Standalone agent-kernel lineage: 11 lobes, Nautilus TEE, `soul.move` | **incorporated** | `memory/tee.rs` implements the Nautilus/Seal TEE-sealing envelope described in its own doc comment ("the Nautilus/Seal envelope") — the TEE concept from this lineage is real and shipped. The "11 lobes" / `soul.move` on-chain-soul concept was not found ported elsewhere in `src/`. |
| 15 | ritual-codex | Skill library / resonance system, ritual patterns & ceremonial protocols | **rejected (superseded)** | Intentionally archived — its active successor is **Koodu** (rhythm/Sabbath temporal gates), which *is* real and live (`rhythm.rs`, `koodu/*.json` weekday codices, `HermeticRhythmGate`). Patterns flowed through Koodu, not this repo directly. |
| 16 | Techgnosis | Julia compiler (`oso_compiler.jl`/`oso_vm.jl`), OSOVM's bytecode source | **not-applicable** | Pipeline sibling of OSOVM (Techgnosis compiles → OSOVM executes), not a pattern source for the Rust agent kernel — no `omokoda-core` evidence, and none expected given its role. |
| 17 | Zangbeto (Zàngbétò) | Standalone guardian-signing / red-team audit daemon | **not-applicable** | Runs as its own service (real signed receipts, separate deployment) rather than a ported pattern inside `omokoda-core` — no direct source evidence in `src/`, consistent with its role as an external security pillar, not a kernel component. |
| 18 | Ifascript (IfáScript Ω) | Divination/entropy VM, real Rust, explicitly built for Omo-Koda2 | **deferred** | Confirmed real and Rust-native, but **not yet wired**: no `if_script_tool.rs` exists in `tools/`. (Note: `tools/divination.rs` is a different, already-shipped feature — "LARQL-style divination over an agent's own memory," ported from `larql-glyph`, not from Ifascript — don't conflate the two.) Next step already scoped in an earlier memory note: add `omokoda-core/src/tools/if_script_tool.rs` following the existing per-capability tool-file convention. |
| 19 | NarratorIDE | Multi-LLM code-narration engine (Claude/Ollama/HuggingFace/Grok personas) | **not-applicable** | Ported into **Vantage's** voice pipeline (`ThinkingNarrator`, per Vantage-Voice work), not into `omokoda-core` — right pillar, wrong repo for this audit's scope. |
| 20 | vanity-cloakseed | Client-side ETH vanity-address generator + seed-phrase cipher overlay | **incorporated** | `identity/cloak.rs`'s own doc comment: "Ported from vanity-cloakseed's `ciphers.js`" — a real, cited, working port of the positional-substitution cipher, adapted onto BIPỌ̀N39's 256-word space. |
| 21 | Sign-wise | AI legal-contract analyzer SaaS (React/Firebase/Gemini/Stripe) | **not-applicable** | Consumer legal-tech SaaS, no architectural overlap with an agent kernel — no evidence, none expected. |
| 22 | Twelve-thrones | On-chain 12-model AI-epistemology/disagreement-detection engine | **not-applicable** | Feeds Zàngbétò's audit prioritization as its own pillar-adjacent service (per ecosystem-breakdown), not a pattern ported into `omokoda-core`. |
| 23 | paradigm | Consciousness/cognitive-architecture layer (Technosis ecosystem) | **deferred** | No `omokoda-core` evidence found; conceptually adjacent to the Justice/reputation and emotion-state work already real in `justice.rs`/`emotion.rs`, but no cited port exists yet — worth a real look, not yet done. |
| 24 | Npc-forge | NPC-minting SaaS: 3D avatars, RAG brains, on-chain wallets/memories | **not-applicable** | Consumer 3D-avatar product, different domain — no evidence, none expected. |
| 25 | Agent.TV | Streaming connector (lean, pairs with franken-stream) | **not-applicable** | No README, no `omokoda-core` evidence — media/streaming tool, not a kernel pattern source. |
| 26 | vibe-lang | AI-native prompt-first language, 18 compile targets | **rejected** | Independently audited (2026-08-26 agent-tooling audit): single-commit AI-scaffold prototype, duplicates ground already covered by the live `zerolang`/`larql`. No `omokoda-core` evidence. Recommended for archival at the repo level. |
| 27 | vibe-coder | AI app-generator using a *mocked* Amp SDK | **rejected** | Same 2026-08-26 audit: core dependency was mocked, never a real integration; nothing extractable. No `omokoda-core` evidence. |
| 28 | eternal-orisa-loom-v8 | Narrative engine: text→voice→image→frame→video ritual-beat pipeline | **not-applicable** | Content-generation pipeline for a different product surface (Cinema/Agent.TV2-adjacent); no `omokoda-core` evidence, and the domain (video-beat generation) doesn't map to the agent kernel. |
| 29 | Droidclaw | "Kira" — long-term personal-memory AI, SOMA architecture | **incorporated** | `memory/soma.rs` doc comment: "SOMA — Self-Organizing Memory Architecture (from Droidclaw)." `steward/soul.rs` also real and shipped. Matches an earlier memory finding that only 3 Droidclaw modules (soma/soul/bus) were ever actually shipped — confirmed still accurate; the IRIS/emotion-engine and 9-language "Orisha distribution" concepts from the same repo remain unbuilt. |
| 30 | Omo-koda (dup of #14 listing) | Same standalone lineage as #14 | — | See #14. |
| 31 | Memory (Triune-Memory) | "Autonomous Orchestrator" scaffold, phase-gated task runner | **rejected** | Independently audited previously: storage confirmed fake (in-memory `Map`, base64 "encryption," fabricated tx hashes) — closed as not-worth-porting. No `omokoda-core` evidence. |
| 32 | The-Aether (dup of #3) | Same repo content as #3 | — | See #3. |
| 33 | Swibe (dup of #4) | Same repo as #4/§3 | — | See §3. |
| 34 | Zangbeto- (dup of #17, trailing dash) | Same repo as #17 | — | See #17. |
| 35 | ifascript (dup of #18, lowercase) | Same repo as #18 | — | See #18. |
| 36 | warpdotdev/warp | Warp terminal (external, closed-source-adjacent product) | **not-applicable** | Terminal-UX product, not an agent-kernel architecture; no plausible port surface, no evidence. |
| 37 | (Android app, gptos intelligence assistant) | Play Store listing, no accessible source | **not-applicable** | No source available to evaluate; closed product. |
| 38 | agiresearch/AIOS | Academic "AI Agent Operating System" research project | **deferred** | Real, relevant *conceptually* (LLM-as-kernel framing overlaps Ọmọ Kọ́dà's own thesis) but no evidence of an actual code port — worth a real comparative read, not yet done. |
| 39 | Aider-AI/aider | Git-native AI pair-programming CLI | **not-applicable** | A tool to be used, not an architecture to port — no kernel-pattern overlap, no evidence expected or found. |
| 40 | Agent Zero | No public repo confirmed | **not-applicable** | Cannot evaluate a repo that doesn't exist publicly; nothing to port. |
| 41 | OpenFang | No public repo confirmed | **not-applicable** | Same as #40 — no evaluable source. |
| 42 | TradingAgents (unconfirmed link) | Duplicate listing of #43 | — | See #43. |
| 43 | TauricResearch/TradingAgents | Multi-agent LLM trading-strategy framework | **not-applicable** | Domain mismatch — trading-strategy orchestration, not agent-kernel architecture; real trading integration already lives in the separate `kanban`/Ares-Intel pillar, not `omokoda-core`. |
| 44 | ultraworkers/claw-code | Upstream fork source of Claw-code (#7/§1) | — | Same repo family as §1 — verdicts already covered there; this entry is the upstream pointer, not a separate evaluation target. |
| 45 | Julia (concept, julialang.org) | Language choice, not a repo | **incorporated** | Real or as designed: Julia is the live language for OSOVM/VeilSim/Ọ̀ṣun-side memory population (per `soma.rs`'s own comment: "In the full distributed system, Ọ̀ṣun (Julia) populates this..."). A concept reference, correctly adopted. |
| 46 | Busy Beaver (mathematical concept) | Computability-theory concept, no repo | **incorporated (as design reference)** | Referenced in prior ecosystem design docs as the Proof-of-Simulation/compute-attestation rationale (Busy Beaver PoCW, per earlier session memory) — a conceptual, not code, incorporation. |

### Summary
- **Incorporated** (real, cited evidence): Oso-Aether, OsO, BIPON39, Osovm, Omokoda (partial — TEE only), vanity-cloakseed, Droidclaw, Julia, Busy Beaver — 9 of 43 (plus the 3 pre-existing verdicts).
- **Rejected**: Aether/The-Aether (superseded generation), Claude-mirror, Claude, vibe-lang, vibe-coder, Memory/Triune-Memory, ritual-codex (superseded by Koodu) — 7.
- **Deferred** (real, scoped, not yet built): Ifascript, paradigm, AIOS — 3.
- **Not-applicable** (domain mismatch or external tool): Nex-, Kimi-bino, franken-stream, Techgnosis, Zangbeto, NarratorIDE, Sign-wise, Twelve-thrones, Npc-forge, Agent.TV, eternal-orisa-loom-v8, warp, gptos app, aider, Agent Zero, OpenFang, TradingAgents — 17.
- Duplicates in the source list (#3/#32, #4/#33/§3, #14/#30, #17/#34, #18/#35, #42/#43, #44): resolved to their single real verdict, not double-counted.

---

## 6. Re-verification against Bino-Elgua (2026-08-27)

> The `omokoda` GitHub org was suspended; the "Inspiration Pattern Repos" now live under `github.com/Bino-Elgua`. READ-ONLY access only — **never push/commit to Bino-Elgua**. This section re-verifies six repos against their correct Bino-Elgua URLs (fresh clones, grep against `omokoda-core/src` + `omokoda-on-chain/sources`).

### 6.1 Claw-code (`Bino-Elgua/Claw-code`) — verdict CONFIRMED: incorporated
Real Rust agent runtime (session persistence, permission modes, bash sandbox, hook system). Fresh clone matches the source the §1 verdict was written against. Grep evidence already cited in §1 (`tools/`, `permissions.rs`, `providers.rs`, `execution/`). No change.

### 6.2 Swibe (`Bino-Elgua/Swibe`) — verdict CONFIRMED: incorporated
Agent-native scripting language (neural router, 3-tier memory, hermetic ethics). Fresh clone confirms it is the same language reference §3 was written against. Real evidence: `plugins/` (hook system), `receipt/` (receipt chain), `providers/` (provider fallback), `omokoda-hermetic/` (7-principle ethics engine). No change.

### 6.3 Claude-2 (`Bino-Elgua/Claude-2`) — verdict CONFIRMED: incorporated
TypeScript Claude Code harness (async generator loop, context compression, safety stack). Fresh clone matches §2. Real evidence: `providers/` (provider abstraction + `/private` routing), `execution/` (sandbox + hooks), `tools/` (per-capability tool files). No change.

### 6.4 Claude-mirror (`Bino-Elgua/Claude-mirror`) — verdict CONFIRMED: rejected
Mirror of the `@anthropic-ai/claude-code` npm package. Fresh clone confirms it is a literal mirror, not a separate code source. The "patterns only, write fresh Rust" rule (§4) still holds. No change.

### 6.5 ase-vault (`Bino-Elgua/ase-vault`) — verdict CHANGED: not-applicable → **deferred (reference)**

The earlier audit used a wrong/absent source and marked this "not-applicable". Against the real Bino-Elgua URL, **ase-vault is genuinely relevant to the receipt-chain / opcode work** and deserves the second look the owner requested.

What it actually is: `ÀṣẹVault_COMPLETE.py` — a "dictionary is the source of truth" definition of the **155-opcode ÀṢẹ/OSOVM VM** (25 runtime-enforced core + 130 expansion, plus 777 veil opcodes), with BIP-39/Ed25519 key handling and Sabbath guards baked in. `OPCODE_REFERENCE.md` is the full opcode semantic map, including:

- `IMPACT` (0x11) / `VEIL` (0x12) — mint-from-work / VeilSim
- `TITHE` (0x27) — the 3.69% AIO split
- `RECEIPT` (0x1f) — immutable proof
- `BIPON_SEED` (0x26) — HD wallet derivation (1440)
- `NONREENTRANT` / `GENESIS_FLAW_TOKEN` (0x28 / 0x2b) — reentrancy + block-0 mint guards
- **1440 inheritance-wallet governance opcodes** (0x30–0x34): `CANDIDATE_APPLY → COUNCIL_APPROVE → FINAL_SIGN → DISTRIBUTE_OFFERING → CLAIM_REWARDS`

Grep against `omokoda-core/src` + `omokoda-on-chain/sources`: **RECEIPT** (`receipt/act_receipt.rs`) and **TITHE** (3.69% in `onchain.rs`) are real and live, but there is **no opcode table, no 1440 inheritance-wallet flow, no GENESIS_FLAW_TOKEN, no IMPACT/VEIL** anywhere in the Rust core or the Move contracts. Those opcodes live in OSOVM (Julia) — and the earlier OsoVM audit flagged IMPACT/VEIL/RECEIPT there as *stubbed*.

**Concrete unbuilt patterns worth pulling in (as design references, not code — it's Python, this repo is Rust):**
1. The **1440 inheritance-wallet governance opcode sequence** (apply → council-approve → final-sign → distribute → claim) — absent from `omokoda-on-chain`.
2. The **GENESIS_FLAW_TOKEN** block-0 mint guard — absent (the genesis-flaw / Èṣù's-Twist guard is a real, still-missing settlement invariant).
3. The **opcode → semantic naming convention** (`@impact`, `@veil`, `@tithe`, `@receipt`, `@biponSeed`) as the vocabulary that `onchain.rs`'s settlement calls should align to.

### 6.6 Npc-forge (`Bino-Elgua/Npc-forge`) — verdict CHANGED: not-applicable → **deferred (reference)** for the Move dNFT pattern only

The earlier "not-applicable" verdict over-weighted the 3D-avatar frontend and missed the real extractable. The correct Bino-Elgua clone shows the repo's substance is `move/sources/npc.move` — a **real Sui Move dNFT minting contract**:

- `Personality` struct (friendly/chatty/genius 0–100)
- `NPC` object with `secrets_hash` (Seal-encrypted), `avatar_blob_id` (Walrus blob), `wallet`, `interaction_count`
- `NPCRegistry` (shared, `total_minted` counter)
- `NPCMinted` / `NPCInteracted` **events**

Grep against `omokoda-on-chain/sources`: `soul.move` (SoulRecord) and `agent.move` (AgentState) exist and cover the identity half — but they **emit no events**, have **no personality-traits struct**, and carry **no Seal `secrets_hash` or Walrus `avatar_blob_id`** field (only `hermetic_seed_hash` and `dna_fingerprint`).

**Concrete unbuilt pattern worth pulling in:** the **mint-event emission** — `soul.move::forge` and `agent.move::create` currently transfer the object silently; emitting a `SoulForged`/`AgentCreated` event (Npc-forge's `NPCMinted` shape) would make on-chain birth queryable/discoverable. The 3D-avatar + Ready-Player-Me + Groq frontend remains correctly **not-applicable**.

**Status update (2026-08-27, later same night):** this gap is now closed for the mint-event half — `soul.move::forge` emits `SoulForged` and `agent.move::create` emits `AgentCreated` (plus a bonus `TierChanged` event on `update_reputation`), matching the `NPCMinted`/`NPCInteracted` shape. `sui move build` passes clean; compatible upgrade, no function signatures changed. Personality-traits struct and Seal/Walrus fields remain open (item 2 above, ase-vault settlement patterns, is the next piece of this same gap).

### Re-verification summary
- **Confirmed unchanged** (4): Claw-code (incorporated), Swibe (incorporated), Claude-2 (incorporated), Claude-mirror (rejected).
- **Changed** (2): ase-vault (not-applicable → deferred/reference, receipt-chain opcode vocabulary), Npc-forge (not-applicable → deferred/reference, Move dNFT mint-event pattern; mint-event half now built, see status update above).
- Both changes move items from the "not-applicable" bucket into "deferred" — i.e. real, scoped, not-yet-built patterns the owner explicitly wanted a second look at.

## 7. Re-verification (2026-08-27): Bino-Elgua "Inspiration Pattern Repos" — 5 repos

Read-only shallow clones of `github.com/Bino-Elgua/{Droidclaw, Oso-Aether, Kimi-bino, OsO, NarratorIDE}`. Confirms/updates the §2 verdicts with fresh evidence from the real Bino-Elgua repos.

### 1. Droidclaw — **incorporated** (CONFIRMED, with two corrections)
- `memory/soma.rs` L1: "SOMA — Self-Organizing Memory Architecture (from Droidclaw)" — and it implements **MemCells** (`tension`, `connection_depth`, `activation_count`), so the "emotionally weighted memory" idea is ported *more deeply* than the old verdict said.
- `steward/soul.rs` + `bus/` — shipped (confirmed).
- `emotion.rs` (`EmotionState { energy, tension, connection, focus }`) exists and its doc reads "Influences IRIS routing" — so the emotion engine is **also already present**, contrary to the old "IRIS/emotion-engine remain unbuilt" note.
- **Correction A:** the "9-language 'Orisha distribution'" is **not in Droidclaw** (no `orisha`/`9-language` hits in src or README) — that line in the prior verdict is inaccurate.
- **Correction B:** still genuinely worth pulling (optional): IRIS's full person-state *response-architecture* routing table (GENTLE/DIRECT/… per tension, `src/core/iris.js`) — only the state + a routing *hint* is ported, not the routing table; and `src/core/sense.js` (phone-sensor → emotion) belongs to the agent-phone device pillar, not the kernel.

### 2. Oso-Aether — **incorporated** (CONFIRMED)
- Rust/WASM; `core/interpreter/` has the full `birth`/`think`/`act` grammar with tier gates (`ActionBlockedTier0`, `ToolLocked`, `UnknownTool`) — matches `omokoda-core/src/parser.rs` (`parse_birth/parse_think/parse_act`) + `reputation.rs`.
- Still worth pulling: the **ASCII-pet deterministic renderer** + 86-char DNA (`core/interpreter`, frontend). omokoda-core has the DNA fingerprint but the pet/31-mask feature is unbuilt — this repo is the direct reference.

### 3. OsO — **incorporated** (CONFIRMED)
- Earliest generation: `translator/` (Python) drives the `birth`/`think`/`act` prompts (`process_birth/process_think/process_act`). Conceptually superseded by Oso-Aether (Rust/WASM) + the `omokoda-core` parser. Nothing new to pull.

### 4. Kimi-bino — **not-applicable** (verdict stands, but the REASONING was stale)
- **Not** a "vanilla Vite+React scaffold". It is actually **"Aether Orchestra"**: a real multi-agent orchestration frontend — React Three Fiber 3D `AgentPortal`, `OrchestraLive`, Dashboard/TaskInput/Settings/History, Zustand stores (`useOrchestraStore`, `useTaskAgentStore`) — plus `openclaw/SOUL.md` (Maestro orchestrator + specialist agents + ralph-loop), `openclaw/config.yaml` (gateway :18789, rate limits), and `docs/MONETIZATION.md` (freemium tiers).
- **Correction:** there is **no Kimi/moonshot API code** (`grep kimi|moonshot` = empty) — the repo name is misleading; it's the Aether Orchestra UI, not a Kimi integration. `package.json` name is still `"my-app"` (generic scaffold base).
- Still **not-applicable to `omokoda-core`**: it's a frontend + OpenClaw runtime config, not a Rust-kernel pattern source. Plausible reuse is elsewhere — the multi-agent orchestration UI → `omokoda-frontend`/Axiom (not the kernel), and the OpenClaw SOUL/config as a *separate* orchestration runtime (adjacent to mesh, not core).

### 5. NarratorIDE — **not-applicable to omokoda-core** (CONFIRMED correct)
- Bino-Elgua/NarratorIDE = the multi-LLM code-narration engine (`src/thinking-narrator.js`, `narrator.js`, `personas.js`, `llm-provider.js`). `omokoda-core` has **no** narrator/voice/persona concept (grep: only `compact.rs` "narrative summary" text + `output_style.rs` "no narrative" — unrelated to persona-voiced narration). It was ported into **Vantage's** voice pipeline (`ThinkingNarrator`), not the kernel.
- Given the doc's "IDE / narrative interface for Omo-Koda2 directly" framing: still the right call — there is no narration layer in the shipped kernel. If Omo-Koda2 ever wants persona-voiced narration over its `think` reasoning, `thinking-narrator.js` is the reference to port into `omokoda-core` (future inspiration, not current).

### Net effect on summary counts
No verdict changes: Droidclaw/Oso-Aether/OsO remain **incorporated**; Kimi-bino + NarratorIDE remain **not-applicable**. (Only the Kimi-bino *reasoning* and the Droidclaw *corrections* are updated above.)
