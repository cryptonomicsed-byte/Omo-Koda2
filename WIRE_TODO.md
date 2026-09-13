# OMO-KODA2 WIRE-IN — NATIVE EXTRACTION BUILD
# Source: Droidclaw, Claude-2, Core-Agency deep dive (2026-09-13)
# Strategy: take the best patterns, implement natively in Rust — no JS ports

Status legend: [ ] pending  [x] done  [~] in-progress  [!] blocked

---

## PHASE 1 — FOUNDATION (low effort, high ROI)

- [x] W-01  Create `src/util/neural_cache.rs` — hash-keyed response memoization
            FNV-1a hash of prompt string, 1hr TTL, max 100 entries, LRU eviction
            `NeuralCache::get(prompt) -> Option<String>`
            `NeuralCache::put(prompt, response)`
            `NeuralCache::prune()` — evict entries older than TTL or over capacity
            Inspired by: Core-Agency `services/neuralCache.ts` cyrb53 hash cache

- [x] W-02  Create `src/util/json_repair.rs` — auto-fix truncated LLM JSON
            `repair(s: &str) -> String` — balance `{` vs `}`, `[` vs `]`, close unterminated strings
            Handles: missing close braces, unclosed string at end, trailing commas
            Inspired by: Core-Agency `universalAiService.ts` repairJson()

- [x] W-03  Create `src/util/file_state_cache.rs` — SHA-256 + mtime change detection
            `FileStateCache` — HashMap<PathBuf, FileSnapshot>
            `FileSnapshot { hash: [u8;32], mtime: u64, size: u64 }`
            `has_changed(path) -> bool` — returns true if hash or mtime differs
            `update(path)` — re-hash and store new snapshot
            Inspired by: Claude-2 `src/utils/fileStateCache.ts`

- [x] W-04  Add `src/util/mod.rs` — exports neural_cache, json_repair, file_state_cache
- [x] W-05  Add `pub mod util;` to `lib.rs`

---

## PHASE 2 — MEMORY UPGRADES (medium effort, very high ROI)

- [x] W-06  Upgrade `src/memory/soma.rs` — full Droidclaw MemCell scoring
            Add to MemCell: `id: String`, `importance: f32`, `tags: Vec<String>`
            Replace `emotional_weight()` with full score():
              `recency_score(now) * 0.25 + importance * 0.35 + emotional_weight() * 0.25 + activation_boost() * 0.15`
            `recency_score(now)`: `1.0 / (1.0 + age_hours.ln_1p())`
            `activation_boost()`: `(activations as f32 * 0.05).min(0.3)`
            Add `MemCell::new_with_id(content, timestamp, importance) -> Self`

- [x] W-07  Upgrade `src/memory/soma.rs` — full Droidclaw LPM structure
            Replace current Lpm with: `identity: Vec<String>`, `patterns: Vec<String>`,
            `triggers: Vec<String>`, `needs: Vec<String>`, `foresight: Vec<String>`, `growth: Vec<String>`
            `max_8_dedup(vec: &mut Vec<String>, new: String)` — dedup + cap at 8
            `Lpm::update_identity(s)`, `update_patterns(s)`, etc.
            `Lpm::to_json() -> String`, `Lpm::from_json(s) -> Result<Lpm>`
            `Lpm::save(path)`, `Lpm::load(path)`

- [x] W-08  Add reconstructive recollection to soma.rs
            `top_memories(cells: &[MemCell], now: u64, n: usize) -> Vec<&MemCell>`
            Scores all cells, returns top-n by score — used by SoulBuilder::with_memories()

---

## PHASE 3 — BEHAVIORAL LEARNING LOOP (low effort, high ROI)

- [x] W-09  Upgrade `src/behavioral.rs` — add BehavioralMemory + learning loop
            Add `BehavioralMemory` struct:
              `do_more: Vec<String>`, `do_less: Vec<String>`, `user_traits: Vec<String>`,
              `core_rules: Vec<String>`, `session_count: u32`
            Add `BehavioralMemory::record_session(what_worked: Vec<String>, what_failed: Vec<String>)`
              Appends to do_more/do_less, dedup, cap at 20 each
            Add `BehavioralMemory::consolidate() -> Vec<String>`
              Every 5 sessions: return a stub `core_rules` update list (placeholder until LLM call wired)
            `BehavioralMemory::save(path)`, `BehavioralMemory::load(path)`
            Inspired by: Droidclaw behavioral learning loop

---

## PHASE 4 — INFERENCE ROUTER (medium effort, high ROI)

- [x] W-10  Create `src/inference/router.rs` — CoT / CoVe / Direct routing
            `InferenceStrategy` enum: Direct, ChainOfThought, ChainOfVerification
            `InferenceRouter::select(prompt: &str, emotion: &EmotionState) -> InferenceStrategy`
              Direct: short prompts < 30 chars, reflex profile
              CoT: technical/analysis keywords, deep profile
              CoVe: sensitive/irreversible decisions, tension > 0.6
            `InferenceRouter::describe(strategy) -> &str` — for logging
            Inspired by: Core-Agency InferenceRouter

- [x] W-11  Create `src/inference/mod.rs` — exports router
- [x] W-12  Add `pub mod inference;` to `lib.rs`

---

## PHASE 5 — SESSION MANAGEMENT (medium effort)

- [ ] W-13  Create `src/session_history.rs` — message dedup + pasted content refs
            `MessageHistory { messages: Vec<HistoryEntry>, max: usize }`
            `HistoryEntry { role, content, hash: u64, pasted: bool }`
            `push(role, content)` — dedup: if same hash as last, skip
            `inline_or_ref(content) -> String` — if < 1KB return inline; else `[Pasted text #N +M lines]`
            Trim to last 100 entries
            Inspired by: Claude-2 message history manager

- [x] W-14  Add auto-compact trigger to `src/compact.rs`
            Add `should_compact(used_tokens: u64, max_tokens: u64) -> bool` — true at 75%
            Add `CompactTrigger::check_and_flag(used, max) -> Option<&str>` — returns summary prompt if triggered
            Inspired by: Claude-2 auto-compact at 75% context limit

---

## PHASE 6 — TOOL EXECUTION LOG (low effort)

- [x] W-15  Create `src/tools/execution_log.rs` — per-call execution records
            `ToolExecution { tool: String, args_preview: String, result_preview: String, elapsed_ms: u64, at: u64 }`
            `ExecutionLog { entries: VecDeque<ToolExecution>, max: usize }` (default max=200)
            `log(tool, args, result, ms)` — args/result truncated to 500 chars
            `recent(n) -> Vec<&ToolExecution>`
            `stats() -> String` — "N calls, avg Xms, top tool: Y"
            Inspired by: Core-Agency tool registry execution logging

- [x] W-16  Add `pub mod execution_log;` to `src/tools/mod.rs`

---

## POST-BUILD VERIFICATION

- [ ] PV-01  rustfmt --edition 2021 --check all new files
- [ ] PV-02  Export all new modules from lib.rs
- [ ] PV-03  Unit tests: neural_cache (3), json_repair (4), file_state_cache (3),
             mem_cell scoring (3), lpm persistence (2), behavioral_memory (3),
             inference_router (3), execution_log (2)
- [ ] PV-04  Commit to Omo-Koda2
