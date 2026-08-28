# Inspiration Follow-Through: `First todo.md` & `First_todo_full.md` Audit (2026-08-28)

Both docs are reference/historical, **not to be edited**. This audit reads them as source material, checks their claims against real `omokoda-core/src` evidence, and produces a concrete "still needs doing" list.

## Doc 1: `docs/First todo.md`

**Confirmed still accurate**: despite the filename, this document is headed **"COMPLETED ✅"**, not an outstanding todo list. It's a historical gap-analysis comparing Omo-Koda2 against Claw-code's 48,599-LOC Rust workspace, with a status banner claiming all P0/P1 items are implemented.

### P0 — Critical (claimed complete)

| # | Item | Claimed file | Verified | Evidence |
|---|---|---|---|---|
| 1 | Streaming Provider Abstraction | `providers/streaming.rs` | ✅ real | 190 lines, real `TokenEvent`-style streaming |
| 2 | Permission System (Deny-First) | `execution/permission_enforcer.rs` | ✅ real | 79 lines |
| 3 | Bash Validation (18 patterns) | `execution/bash_validation.rs` | ✅ real | 350 lines |
| 4 | Safety Stack (7-Layer Defense) | `execution/hooks.rs` + justice | ✅ real | `execution/hooks.rs` (66 lines) + full `justice/` module (`busy_beaver.rs`, `hermetic.rs`, `tier.rs`, `mod.rs` — see Doc 2 below for the hermetic.rs staleness nuance) |

### P1 — High priority (claimed complete)

| # | Item | Claimed file | Verified | Evidence |
|---|---|---|---|---|
| 5 | File Operations Suite | `tools/file_ops.rs` | ✅ real | 478 lines |
| 6 | Config Loader | `config.rs` | ✅ real | 180 lines |
| 7 | Usage Tracking | `usage.rs` | ✅ real | 234 lines |
| 8 | Web Tools (WebSearch/WebFetch) | not present at time of writing | ✅ real now | `tools/web.rs` — real `WebSearchInput`/`WebSearchOutput` types |

All 8 P0/P1 claims check out. The "COMPLETED" banner is honest for what it claims.

### P2 — Medium priority (**not** claimed complete by the banner — genuinely still open)

The doc's own status banner only claims "All P0 and P1 items" are done — it never claims P2 is finished. Checked anyway, since the task asked for real status on every listed item, not just the ones the doc claims:

| # | Item | Status | Evidence |
|---|---|---|---|
| 9 | LLM-Powered Compaction | **partial** | `compact.rs` exists (real token-aware compaction: summarizes old messages, extracts key files/pending work/timeline) but is **rule-based/extraction-based, not LLM-generated** — grepped for `llm`/`LLM`/`provider.`/`think(` calls inside `compact.rs`, zero hits. The doc's ask ("needs LLM-generated summaries for AutoCompact") is still a real, unclosed gap. |
| 10 | MCP Integration | **done** | `mcp.rs` (20 lines, thin) + `plugins/mcp.rs` (102 lines, the real tool-bridge logic) — real, if smaller than Claw-code's 406-line reference. |
| 11 | Plugin System | **done, exceeds spec** | `plugins/` is a full 1,696-line module: `registry.rs`, `discovery.rs`, `manifest.rs`, `hook_manifest.rs`, `skill.rs`, `agent.rs`, `settings.rs`, `output_style.rs`, `rule_engine.rs`, `command.rs`, `config_loader.rs`, `mcp.rs`. This is real and substantially built out, not a stub. |
| 12 | Task/Team/Cron Registry | **partial** | `tasks/` module is real (`types.rs`, `mod.rs`, `scheduler.rs` — the scheduler wires `BackgroundRegistry`, `DreamEngine`, `QueryEngine`, `TaskManager` together). But grepped `tasks/` for "cron"/"Cron"/"team"/"Team" — **zero hits**. The background-job/task-scheduling half of this item is real; a cron-style time-based trigger and a "team" (multi-agent group) registry concept were not found anywhere in `omokoda-core/src`. |

## Doc 2: `docs/First_todo_full.md`

**Checked separately, not assumed identical to Doc 1.** This is a *different* document — not a general gap analysis, but a single, narrow **implementation spec for `HermeticEvaluation`** (the 7 Hermetic Principles ethics-scoring gate), also headed "COMPLETED ✅", claiming all 5 listed files were written, integrated, and all hermetic tests pass.

### The staleness this audit found

The completion claim is **stale in a specific, worth-flagging way** — not "never built," but **superseded by a later architecture**:

- `justice/hermetic.rs` (spec claims ~400+ lines of real logic) is now **7 lines**, and its entire content is:
  > "SUPERSEDED — This module has been replaced by the 7-gate enforcement system. See: `omokoda-core/src/gates/` and `omokoda-core/src/steward/gatekeeper.rs`. The old HermeticEvaluation scoring system (0-100 advisory) is gone. The 7 Hermetic Principles are now MANDATORY enforcement gates via `EsuGatekeeper`."
- `justice/hermetic_tests.rs` is similarly a 4-line pointer: "Tests moved to individual gate files in `omokoda-core/src/gates/` and to the gatekeeper integration tests in `omokoda-core/src/steward/gatekeeper.rs`."
- The successor is real and more substantial than the original spec: `gates/` has 7 real files (`mentalism.rs`, `correspondence.rs`, `vibration.rs`, `polarity.rs`, `rhythm.rs`, `gender.rs`, `cause_effect.rs` — one per Hermetic Principle), 1,148 lines total, plus a 256-line `steward/gatekeeper.rs` with `EsuGatekeeper` referenced 12 times — a genuine architectural upgrade from advisory 0-100 scoring to mandatory enforcement gates.
- The other pieces this spec depended on are real, non-stub implementations: `session.rs` (943 lines), `receipt/mod.rs` (302 lines) — the doc listed these as "STUB — if these methods don't exist," and they don't need stubbing; they're fully real.

**This is not a doc-hygiene failure of the "fabricated citations" kind flagged elsewhere in this codebase's audit history** (unlike `docs/Next.md`, independently found to cite nonexistent code) — every file this doc names is real, the logic really was built, it just isn't where the doc says anymore because the design evolved past it. Worth a one-line pointer update in the doc itself if it's ever revisited, but per this task's instruction the doc is reference-only and was not edited.

## Concrete "still needs doing" list

1. **LLM-Powered Compaction** (`First todo.md` #9) — `compact.rs` is real but purely rule-based; wire an actual `think()`/provider call for `AutoCompact` summary generation. Real, scoped, unclosed gap.
2. **Cron-style task triggers** (`First todo.md` #12, half) — `tasks/scheduler.rs` has background/query/dream scheduling but no time-based cron trigger concept anywhere in `omokoda-core/src`. Real gap if recurring/scheduled agent work is wanted.
3. **Team registry** (`First todo.md` #12, other half) — no "team" (multi-agent group) concept found in `tasks/` or elsewhere searched. Unclear if still wanted given the swarm/hive/mesh-presence architecture already live on the Contabo Elixir OTP side (`ares-omokoda-swarm.service` per infra) may already cover this at a different layer — needs an owner decision on whether this is redundant with swarm/hive rather than a real gap, not a blind port.
4. **Doc pointer hygiene** (optional, not urgent) — `First_todo_full.md` still tells a reader to look at `justice/hermetic.rs` for the real logic; a future editor should know that file is now a 7-line "moved to `gates/`" pointer, not the implementation itself. No action taken this pass since the doc is reference-only per this task's instructions.

## Redundancy / misalignment note

Neither doc references any of the 43 inspiration-repo names covered in the separate `reference-repo-map.md` audit (Claw-code and Claude-2 are named here too, but already covered there) — no new repo-inspiration surface was found in either doc beyond what's already tracked. Both docs are internally consistent with the current architecture except for the one superseded-hermetic-module case above; nothing else here is misaligned with current vision, just two real, small, genuinely-still-open gaps (#1 and #2/#3 above).
