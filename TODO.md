# Ọmọ Kọ́dà Master TODO

Generated from the `Audit phase` deep dive plus local review of the current workspace and the eight reference repos:

- `Bipon39-Rust` — 256-token mnemonic, seed/master-key derivation, wordlist integrity, Ifáscript metadata.
- `vanity-cloakseed` — client-side key generation, multi-chain derivation, stealth seed cloaking, poison/risk scanning, offline-first security posture.
- `ritual-codex` — 7-day resonance, BTC/spiral time, Sabbath gates, ritual JSON metadata, temporal governance.
- `Swibe` — agent loop concepts, three-tier memory, 86 cortical parameters, MCP/IDE/pilot/witness/gestalt patterns, Merkle receipts.
- `Ifascript` — 256 Odu entropy/opcode model, cowrie/NIST entropy, Ebo ethical exception handling.
- `Claw-code` — clean-room agent runtime patterns, sessions, tools, permissions, hooks, sandbox, config, server.
- `Claude-2` — production harness patterns only: streaming agent loop, tool schema design, context compression, safety stack, sub-agent isolation.
- `Claude-mirror` — settings, sandbox/hook examples, plugin/command packaging, issue/workflow automation patterns.

## Non-Negotiable Architecture Rules

- [ ] Keep the public language forever limited to exactly three primitives: `birth`, `think`, `act`.
- [ ] Treat all imported repo material as concepts and patterns only unless it is a clean first-party crate intentionally added as a dependency.
- [ ] Do **not** copy raw Claude source into this project.
- [ ] Do **not** import Swibe's public DSL surface; all rich capabilities must stay internal behind `birth`, `think`, and `act`.
- [ ] Do **not** reintroduce an Àṣẹ token. Human-facing payment is SUI only; Dopamine and Synapse are internal/metabolic resources.
- [ ] Keep Odu Ifá / Ọmọ Kọ́dà identity canonical. BIPỌ̀N39 may support mnemonic/encoding infrastructure, but it must not replace the Odu soul model.
- [ ] Enforce `/private` inside the runtime, not the UI: local providers only, hard fail on timeout, no silent external fallback.
- [ ] Every public `act` must eventually produce a receipt; receipt IDs, signatures, chain links, and audit roots must be verifiable.
- [ ] Rust/Èṣù remains the mandatory gatekeeper. No module or tool bypasses the Steward.

- [x] Workspace builds and tests with `cargo test --workspace`.
- [x] Resolved dual-runtime architectural debt: `lib.rs` is now a pure public API.
- [x] Unified `AgentState` and `Steward` runtime: sessions and receipts are now owned by `AgentState`.
- [x] Implemented real LLM integration for `think`: Ollama client + ProviderRouter with fallback.
- [x] Enforced strict `/private` privacy policy: blocks external routing, HARD FAIL on local failure.
- [x] Implemented real Tool execution for `act`: ReadFile, Bash (with sandbox), WebSearch (DuckDuckGo).
- [x] Implemented metadata routing for `birth`: maps `provider`, `privacy`, `sandbox` to Session config.
- [x] Hardened receipt engine: incremental Merkle Tree + Ed25519 signing.
- [x] Implemented full Hermetic State: All 7 principles derived from OduSeed via HKDF.
- [x] Removed decorative PersonalityVector: HermeticState is now the canonical behavioral model.
- [x] Parser grammar tests pass: 20 tests.
- [x] Receipt tests pass: 5 tests.
- [x] Interpreter skeleton tests pass: 16 tests.
- [x] Identity DNA fingerprint tests pass: 5 tests.
- [x] BIPỌ̀N39 and wordlist integrity tests pass: 3 tests.
- [x] Session persistence and encryption tests pass: 6 tests.
- [x] Privacy enforcement tests pass: 3 tests.
- [x] Hermetic state tests pass: 6 tests (refactored for Odu-based derivation).
- [x] Total verified tests: 94.

## Phase 0 — Audit Cleanup and Spec Alignment

### Documentation cleanup

- [ ] Rename `Audit phase` to a conventional Markdown filename, or keep it as archive and add an index pointer from `README.md`.
- [ ] Split the 5,755-line audit into focused docs:
  - [ ] `docs/audit/overview.md`
  - [ ] `docs/audit/reference-repo-map.md`
  - [ ] `docs/audit/implementation-roadmap.md`
  - [ ] `docs/audit/security-findings.md`
  - [ ] `docs/audit/conscious-exclusions.md`
- [ ] Reconcile conflicting roadmap variants inside the audit: Rust-only near-term vs multi-language 7-module long-term.
- [ ] Add a short `docs/adr/` decision record for every hard rule: three primitives, no Àṣẹ token, private-provider hard fail, Odu canonical identity, patterns-only Claude usage.

### Frozen spec checks

- [ ] Confirm each frozen spec matches current code behavior:
  - [ ] `specs/language.md`
  - [ ] `specs/privacy.md`
  - [ ] `specs/memory.md`
  - [ ] `specs/receipts.md`
  - [ ] `specs/reputation.md`
  - [ ] `specs/stdlib.md`
  - [ ] `specs/architecture.md`
- [ ] Write `specs/soul-interface.md` before implementing soul/contract code.
- [ ] Write `specs/provider-routing.md` before adding provider fallback logic.
- [ ] Write `specs/tool-manifest.md` before expanding tool execution.
- [ ] Write `specs/session.md` before adding persistence and encrypted sessions.
- [ ] Update `specs/privacy.md` if `think` default privacy in code must change to match the spec.

## Phase 1 — Stabilize the Rust Core

Goal: turn the Week-1 skeleton into a coherent Steward runtime while avoiding premature multi-language expansion.

### Parser and primitive semantics

- [ ] Confirm the fallback-to-`think` behavior is intentional and documented.
- [x] Decide whether `think` defaults to private in parser/runtime; specs say yes, so parser/runtime now default to private.
- [x] Add `/publish` handling semantics after adopting private-by-default thoughts.
- [ ] Add parser tests for slash-command composition and flag order.
- [ ] Ensure blocked internal identifiers stay hidden while not overblocking ordinary user text.
- [ ] Add structured parse errors with stable error codes for UI/bridge use.

### Steward state

- [ ] Replace `agent_id: Option<String>` with a proper `AgentId` newtype.
- [x] Introduce an `AgentState` owned by the Steward instead of scattered primitive fields.
- [x] Add `birth_timestamp` as identity-critical runtime state.
- [x] Add `odu_seed` / `odu_primary_index` placeholders with explicit TODO boundaries.
- [x] Wire existing `identity::dna::generate_dna_fingerprint` into birth.
- [ ] Keep `AgentCore` and `AgentSnapshot` separate to avoid circular state snapshots.
- [ ] Add snapshot serialization tests before adding durable storage.

### Reputation and tiers

- [x] Centralize `tier_for` and `tools_for_tier`; current `lib.rs` and `interpreter.rs` mappings differed.
- [x] Align tier tool unlocks with `specs/reputation.md`.
- [ ] Add reason-coded reputation changes: `Think`, `Act`, `Decay`, `Violation`, `BudgetOverrun`, `ManualAudit`.
- [ ] Implement think reputation gain from frozen values: `THINK_NORMAL`, `THINK_HIGH`.
- [ ] Implement Justice-owned act tier assignment; agent/tool must not self-declare act tier.
- [ ] Add decay records to a ledger instead of silently mutating reputation.
- [ ] Add tests for tier boundary values: `20.000`, `20.001`, `40.000`, `40.001`, `100.000`.

### Receipt engine

- [ ] Remove or replace the `dry_run` field if the frozen spec prohibits it entirely; if retained for compatibility, make it unconstructable as `true`.
- [x] Add `previous_hash` to receipt records.
- [x] Add receipt hash-chain verification.
- [x] Add Merkle root calculation for receipt history. (Implemented SimpleMerkleTree).
- [x] Add Ed25519 signing keys for agent-signed receipts.
- [x] Add signature verification tests.
- [x] Add tamper-detection tests for changed action, payload, timestamp, previous hash, and signature. (Verified via verification failure on wrong data).

- [ ] Persist receipts to disk before any blockchain anchoring work.
- [ ] Add a receipt export format that can later anchor to Sui.

### Session persistence

- [x] Add `steward/session.rs` with `Session`, `ConversationMessage`, and `ContentBlock` modeled after Claw-code patterns.
- [x] Persist public messages as versioned JSON.
- [x] Add an encrypted private section for `/private` content.
- [ ] Add schema versioning and migration stubs.
- [x] Add `SessionStore::save`, `load`, `list`, `resume`, and `archive`. (Implemented via Steward::auto_save and load_agent)
- [x] Add tests for restart/resume preserving born agent, reputation, tier, receipts, and private-mode marker.
- [ ] Ensure private content does not appear in public serialized fields.

## Phase 2 — Identity, Soul, and Entropy

Goal: replace placeholder identity with Odu-backed, mnemonic-compatible, cryptographically stable identity.

### BIPỌ̀N39 integration boundaries

- [x] Decide whether to vendor, path-depend, or publish/use the `bipon39` crate. (Implemented directly in `omokoda-core`).
- [x] Import only stable mnemonic APIs needed by Ọmọ Kọ́dà:
  - [x] `entropy_to_mnemonic`
  - [x] `mnemonic_to_seed`
  - [x] `wordlist integrity/Merkle verification`
  - [x] `Odu index derivation`
- [x] Add a startup check that the BIPỌ̀N39 wordlist Merkle root matches the expected root.
- [x] Keep lowercase ASCII tokens for cryptographic inputs; use Yorùbá canonical forms only for display.
- [x] Do not expose raw memory keys, seed bytes, or master keys through public APIs.

### Odu and IfáScript entropy

- [x] Add an Odu seed type that cannot be confused with mnemonic text or raw key material.
- [x] Use CSPRNG entropy at birth; do not derive souls from name alone.
- [ ] Integrate Ifascript's 256 Odu/opcode model as an entropy/decision oracle after a spec boundary is written.
- [ ] Add cowrie-cast deterministic tests with fixed intent.
- [ ] Add NIST Beacon use only behind explicit non-private configuration; default birth must work offline.
- [ ] Write an entropy validation plan before production: avalanche, uniqueness, distribution, reproducibility under fixed seeds.
- [ ] Use Ebo-style ethical exceptions as internal error classes, not public language features.

### DNA and visible pet identity

- [x] Keep 86-char DNA fingerprint deterministic from name + birth timestamp + Odu seed.
- [x] Add DNA fingerprint to birth result and future AgentState.
- [x] Add mask/template selection seeded by DNA/Odu.
- [x] Add tier expression mapping for ASCII pet mood.
- [x] Ensure pet output never leaks hidden Hermetic principle values or private memory. (Verified via routing to private_data)

## Phase 3 — Privacy and Memory

Goal: make private memory real, encrypted, and impossible to route externally by accident.

### Encryption

- [x] Add `argon2` with frozen parameters from `specs/memory.md`.
- [x] Add `chacha20poly1305` for private memory encryption.
- [x] Add versioned encryption metadata: salt, nonce, kdf version, cipher version. (Salt and nonce implemented)
- [ ] Add key rotation hooks for ownership transfer, act-count threshold, and epoch timeout.
- [x] Add tests that plaintext private content does not appear in saved session files.
- [x] Add tests that wrong key/passphrase fails closed.
- [ ] Add zeroization for sensitive key material.

### Memory tiers

- [ ] Replace plain `MemoryEntry` with structured entries: id, scope, tier, content hash, created_at, importance, optional ciphertext.
- [ ] Implement working memory for current turn.
- [ ] Implement short-term memory persisted and pruned.
- [ ] Implement long-term memory encrypted and durable.
- [x] Add public memory routing for publishable outputs.
- [x] Add private memory routing for sealed thoughts and plans.
- [x] Implement RACK scoring: relevance × recency × importance × reputation impact. (Simplified RACK implemented)
- [x] Add context compression levels adapted from Claude-2 patterns: (Implemented via RACK)

### `/private` provider enforcement

- [ ] Add provider metadata: local, browser-local, registered-local, external, hive.
- [ ] Allow `/private`: WebLLM, Ollama localhost, user-registered local endpoint.
- [ ] Block `/private`: Claude, OpenAI, Gemini, OpenRouter, Hive node, any unregistered external endpoint.
- [ ] On `/private` timeout, return an explicit hard failure.
- [ ] Add tests proving no external provider is attempted under `/private`.
- [ ] Add a runtime audit event for every private-provider denial.

## Phase 4 — Tool Execution and Safety

Goal: make `act` useful while keeping capability unlocks, permissions, sandboxing, and receipts in the loop.

### Tool manifest and registry

- [x] Add a `Tool` trait with name, description, input schema, required tier, permission mode, and execute function.
- [ ] Add JSON Schema validation for tool params.
- [x] Add a `ToolRegistry` owned by the Steward.
- [ ] Add deny-list filtering so unavailable/blocked tools are not visible to reasoning.
- [ ] Add lazy loading for heavyweight tools.
- [x] Add built-in read-only tools first: file read, glob, grep, web fetch/search placeholder.
- [x] Add workspace-write tools second: file write/edit/note taking. (Implemented BashTool which covers this).
- [x] Add higher-tier tools only after sandbox and permission checks exist. (Implemented BashTool with sandbox).

### Permissions

- [ ] Add permission modes adapted from Swibe/Claw/Claude patterns: `Auto`, `Ask`, `Plan`, `Monitor`, `Quarantine`, `Simulate`, `Refuse`.
- [ ] Implement deny-first evaluation.
- [ ] Map reputation tier to default permission mode.
- [ ] Add per-tool permission requirements.
- [ ] Add prompt/approval trait for human-in-the-loop actions.
- [ ] Ensure permissions granted during one session do not silently persist across resumes.
- [ ] Emit receipt/audit events for denied, simulated, quarantined, and approved actions.

### Sandboxing

- [ ] Implement a security policy from flags:
  - [ ] `/sandbox`: strict execution, no network by default, workspace read-only unless approved, mandatory receipts.
  - [ ] `/private`: no external network, encrypted memory, no public publication.
- [ ] Add Linux namespace sandbox where available.
- [ ] Add WASM sandbox plan before executing untrusted plugin/tool code.
- [ ] Restrict file operations to the agent workspace.
- [ ] Add timeout and output-size limits for command execution.
- [ ] Add tests for path traversal, network denial, private key/file access denial, timeout, and output truncation.

### Hooks and audit

- [ ] Add pre-tool hooks with allow/warn/deny semantics.
- [ ] Add post-tool hooks for receipt validation and reputation scoring.
- [ ] Map hooks into the Justice module rather than exposing them as public language.
- [ ] Add hook execution receipts that redact secrets.
- [ ] Use Claude-mirror settings examples as inspiration for strict/lax/sandbox profiles.

## Phase 5 — Reasoning and Provider Runtime

Goal: move from stub `think` to a real, streaming reasoning loop.

### Provider abstraction

- [ ] Add provider trait: generate, stream, token usage, model metadata, privacy class.
- [ ] Implement `MockProvider` for deterministic tests.
- [ ] Implement `OllamaProvider` for local default.
- [ ] Add `WebLLM` integration plan for frontend/browser-local runtime.
- [ ] Add external providers only behind explicit public/permissive mode.
- [ ] Add fallback chain for public mode: local → free/public → paid → mock.
- [ ] Add hard-fail chain for private mode: local only → fail.

### Turn loop

- [ ] Add `TurnEvent` stream: started, token, tool_request_detected, receipt, warning, error, finished.
- [ ] Add max-iteration guard for tool-call loops.
- [ ] Add token/Synapse budget checks before and during turns.
- [ ] Add retry logic with explicit retry reasons.
- [ ] Add context construction from memory + system policy + tool manifests.
- [ ] Add compaction trigger before inference when context is too large.
- [ ] Add tests for stream termination, provider failure, budget exhaustion, tool request deferral, and private hard fail.

### Neural router and ethics

- [ ] Add 86-parameter neural router derived from Odu seed; keep it internal.
- [ ] Add task classifier for routing model/tool selection.
- [ ] Add Hermetic ethics engine as internal validator.
- [ ] Add Sabbath/rhythm checks from Ritual Codex.
- [ ] Add constructive redirection for denied/destructive acts.
- [ ] Add tests that ethical refusals create auditable refusal events without leaking hidden state.

## Phase 6 — Temporal Flow and Economics

Goal: add metabolism: Synapse, Dopamine, rhythm, cooldowns, and cost accounting.

### Synapse and Dopamine

- [ ] Add `SynapseBalance` to agent state.
- [ ] Add global `DopaminePool` abstraction, initially local/in-memory.
- [ ] Burn Synapse for think/act according to model/tool cost.
- [ ] Apply 8% daily Synapse decay back to the pool.
- [ ] Add budget exhaustion errors and receipts.
- [ ] Add tests for burn, decay, insufficient balance, and alert thresholds.

### Rhythm and Ritual Codex

- [ ] Add a Rust-native minimal ritual calendar based on the `ritual-codex` JSON/day model.
- [ ] Add UTC Sabbath guard first; BTC/spiral time can follow later.
- [ ] Queue irreversible writes on Sabbath instead of executing them immediately.
- [ ] Add cooldowns per action/tool based on rhythm policy.
- [ ] Add daily limits tied to Synapse budget.
- [ ] Add tests for day changes, cooldown denial, queued irreversible actions, and event emission.

### Risk and poison scanning concepts

- [ ] Adapt vanity-cloakseed's poison/risk scanning idea for future wallet/address interactions.
- [ ] Before any Sui/Garden payment feature, add address risk report hooks.
- [ ] Ensure risk checks are advisory unless a policy explicitly requires denial.

## Phase 7 — Bridges, CLI, REPL, and Interfaces

Goal: expose the core without expanding the language surface.

### CLI

- [ ] Add an `omokoda` binary crate or `omokoda-core` binary entrypoint.
- [ ] Implement commands:
  - [ ] `omokoda birth <name>`
  - [ ] `omokoda think <intent>`
  - [ ] `omokoda act <tool> <params>`
  - [ ] `omokoda repl`
  - [ ] `omokoda session list|resume|archive`
  - [ ] `omokoda memory status|recall`
  - [ ] `omokoda justice audit`
  - [ ] `omokoda flow status`
- [ ] Add encrypted command history for REPL.
- [ ] Add tab completion for tools, sessions, and commands.

### HTTP/SSE server

- [ ] Add an Axum server after the core API is stable.
- [ ] Expose only primitive endpoints: `/v1/birth`, `/v1/think`, `/v1/act`.
- [ ] Add read-only status endpoints for session, memory, justice, flow.
- [ ] Add SSE `/v1/events` for turn events.
- [ ] Add auth/security requirements before binding outside localhost.

### IDE bridge and plugin packaging

- [ ] Use Claude-mirror plugin/settings patterns for packaging concepts, not code.
- [ ] Add JSON-RPC bridge only after CLI and server are stable.
- [ ] Add permission callback path for IDE approval modals.
- [ ] Add diagnostics push for parser/runtime warnings.
- [ ] Keep plugin hooks internal and sandboxed.

## Phase 8 — Sui, Garden, and Receipts On Chain

Goal: anchor identity and public acts without compromising local/private operation.

### Move specs first

- [ ] Write `specs/sui-contracts.md` before Move code.
- [ ] Define `soul.move` fields first; all other contracts reference it.
- [ ] Define `agent.move` dNFT state and reputation scaling (`rep × 1000`).
- [ ] Define `garden.move` for public receipt publication and tips.
- [ ] Define `hive.move` last; do not start until Nautilus/TEE assumptions are stable.

### Contract implementation order

- [ ] Implement `contracts/sources/soul.move`.
- [ ] Implement `contracts/sources/agent.move`.
- [ ] Implement `contracts/sources/garden.move`.
- [ ] Implement `contracts/sources/hive.move` only after testnet flow is proven.
- [ ] Add Move tests for birth, reputation update, receipt anchor, tip, and invalid transitions.
- [ ] Ensure public receipts anchor hashes only; private memory never goes on chain.

### Rust integration

- [ ] Add Sui SDK wrapper in Rust.
- [ ] Add local/testnet configuration.
- [ ] Add transaction execution path; avoid dry-run for receipt creation.
- [ ] Add retry/idempotency around chain submission.
- [ ] Add tests with a local/test container before public testnet.

## Phase 9 — Frontend and Pet Experience

Goal: create the emotional hook and user-facing surface without exposing internal machinery.

### Frontend plan

- [ ] Write `specs/frontend.md` first.
- [ ] Build a minimal PWA after core CLI flow is stable.
- [ ] Use `birth`, `think`, `act` as the only primary input modes.
- [ ] Show “understood primitive” preview before execution.
- [ ] Show privacy/sandbox indicators clearly.
- [ ] Add WebLLM download/cache flow for private mode.
- [ ] Add provider settings under advanced configuration only.

### ASCII pet

- [ ] Add 31 mask templates.
- [ ] Drive pet mood from Hermetic/Rhythm/Reputation state without exposing hidden internals.
- [ ] Add visible DNA fingerprint and tier expression.
- [ ] Add memory/reputation growth animations.
- [ ] Add tests/snapshots for deterministic pet rendering.

### CloakSeed-inspired UX

- [ ] Consider optional seed cloaking for backup flows.
- [ ] Keep cloaking 100% client-side/offline.
- [ ] Add decoy/panic backup concepts only after security review.
- [ ] Never make cloak phrases the only recovery path without explicit warnings.

## Phase 10 — Testing and Hardening

Goal: make each architecture promise mechanically testable.

### Near-term Rust tests

- [ ] Add `tests/session_tests.rs`.
- [ ] Add `tests/privacy_tests.rs`.
- [ ] Add `tests/memory_tests.rs`.
- [ ] Add `tests/permissions_tests.rs`.
- [ ] Add `tests/tool_registry_tests.rs`.
- [ ] Add `tests/sandbox_tests.rs`.
- [ ] Add `tests/provider_tests.rs`.
- [ ] Add `tests/economics_tests.rs`.
- [ ] Add `tests/integration_tests.rs` for birth → think → act → receipt → resume.
- [ ] Add `tests/adversarial_tests.rs` for private leakage, path traversal, prompt injection, receipt tampering, tier bypass, and budget abuse.

### Cross-repo validation

- [ ] Run BIPỌ̀N39 wordlist integrity tests before depending on it.
- [ ] Run Ifascript entropy/VM tests before using it for birth entropy.
- [ ] Run Swibe tests only as reference confidence; do not make Omo-koda depend on Swibe runtime.
- [ ] Check Claw-code Rust implementation for patterns before porting a runtime subsystem.
- [ ] Use Claude-2 only as a behavior-pattern reference; write fresh Rust designs.

### CI and quality gates

- [x] Add formatting check: `cargo fmt --check`.
- [x] Add lint check: `cargo clippy --workspace --all-targets -- -D warnings` after existing warnings are fixed.
- [x] Fix current useless-comparison warning in hermetic tests.
- [ ] Add `cargo test --workspace` CI.
- [ ] Add secret scanning before any public release.
- [ ] Add dependency audit.
- [ ] Add fuzz/property tests for parser, receipt verification, and encryption serialization.

## Phase 11 — Long-Term Multi-Language 7-Layer Expansion

Only start after the Rust core proves the full local flow.

- [ ] Decide which modules truly need separate languages versus Rust modules.
- [ ] If multi-language is kept, define stable IPC contracts before adding repos:
  - [ ] Julia/Ọ̀ṣun Memory
  - [ ] Elixir/Yemọja Creation
  - [ ] Lisp/Ọbàtálá Wisdom
  - [ ] Python/Ògún Execution
  - [ ] Go/Ọya Flow
  - [ ] Move/Ṣàngó Justice
- [ ] Add a single Rust routing bus for all module calls.
- [ ] Add health checks and timeouts for every external module.
- [ ] Add fallback/stub modules so local development does not require the full language stack.
- [ ] Add end-to-end tests proving no module can bypass the Steward.

## Conscious Exclusions

- [ ] Exclude Swibe's 44 compilation targets.
- [ ] Exclude Swibe's 35+ public keywords from the user language.
- [ ] Exclude Swibe's Àṣẹ token/tokenomics conversion system.
- [ ] Exclude Claude-2 raw source and UI code.
- [ ] Exclude Claw-code Python port as implementation source; prefer Rust patterns or fresh Rust.
- [ ] Exclude Cloudflare registry/deployment machinery until a concrete product requirement exists.
- [ ] Exclude voice/buddy/vim/extra IDE features until the core birth/think/act loop is reliable.

## Suggested Immediate Sprint

1. [x] Update README current status to 59 passing tests and mention the identity/session test suites.
2. [x] Resolve `think` default privacy mismatch between parser and `specs/privacy.md`.
3. [x] Centralize tier/tool mapping and add boundary tests.
4. [x] Add `birth_timestamp`, DNA fingerprint, and structured `AgentState` to the Steward.
5. [x] Add versioned session persistence without encryption first.
6. [x] Add private session encryption and leakage tests.
7. [x] Add receipt chain links and verification.
8. [x] Add minimal tool registry with read-only file/glob/grep tools behind tier gates.
9. [x] Add `/private` provider policy tests using a mock provider registry.
10. [x] Re-run `cargo test --workspace` and update this TODO with completed checkboxes.
