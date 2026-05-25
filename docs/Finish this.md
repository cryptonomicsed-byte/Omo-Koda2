Integration Plan: MVP + Phase 2 Full Forge (10 Components)
Context
The user submitted a production-ready forged package ("Ògún forges for you") containing Rust economy improvements, a new Move contract, a Python executor, TypeScript RPC/UI components, and integration tests. The goal is to integrate these into the existing Omo-Koda2 repository without breaking the 410+ existing passing tests, while respecting all north-star constraints from architecture.md, mission.md, synthesis.md, TODO.md, etc.
Two forge packages received:
MVP Core (Rust economy, Move synapse contract, Python executor, TypeScript RPC/UI)
Phase 2 (Julia BB Oracle, Lisp Ethics Engine, Elixir Swarm, Go Flow, Nautilus TEE, NIST Entropy)
The MVP forged code uses a different directory layout (core/, move_contracts/, python_executor/, frontend/) that must be mapped to the existing repo structure (omokoda-core/, omokoda-on-chain/, omokoda-simulation/, omokoda-frontend/). Phase 2 components use new top-level directories and must integrate with the Rust core via gRPC and FFI.
Branch: claude/omokoda-mvp-phase2-complete-IkptI
North-Star Constraints (Must Not Violate)
Three primitives only: birth, think, act — frozen, no additions
No Àṣẹ token — never implement
/private = hard fail, never silently reroute to public providers
dry_run structurally prohibited on all receipts
birth_timestamp = immutable, identity-critical, never default to 0
WASM bridge = exactly 6 functions (no seventh)
Steward (Èṣù) = mandatory gatekeeper — no module bypasses it
Busy Beaver bounds: T0→1, T1→6, T2→21, T3/T4→107, T5→47,176,870
Synapse burn is atomic and non-refundable; pre-adjusted cost passed to burn(), no re-apply of tier multiplier
Reputation formula: difficulty = 1.0 / (1.0 + (rep / 25.0)) — existing formula is correct, just needs safeguards added
Existing State (Key Findings)
omokoda-core/src/reputation.rs (186 lines)
✅ Already has correct difficulty(rep) formula
✅ Already has tier_for(rep) with thresholds at 20/40/60/80/100
✅ T5 threshold already at 100.0
❌ Missing: daily action cap (50), diminishing returns (0.995^n), 7-day tier promotion gate
omokoda-core/src/economics.rs (123 lines)
✅ SYNAPSE_MAX_PER_AGENT = 86_000_000
✅ DOPAMINE_TOTAL_POOL = 86_000_000_000
✅ Two-phase decay math present
❌ Missing: SynapseAccount struct, earn_from_garden() (should return 1000 not 100), earn_from_tip() capped at 10,000, tier-based synapse caps
omokoda-core/src/justice/ (existing)
✅ hermetic.rs, hermetic_tests.rs, mod.rs all exist
❌ Missing: tier.rs with Busy Beaver bounds
omokoda-on-chain/sources/ (existing)
✅ garden.move exists (keep as-is)
❌ Missing: synapse.move
omokoda-frontend/
✅ lib/api.ts, components/AgentChat.tsx, components/AgentDashboard.tsx, etc.
❌ Missing: lib/rpc_client.ts, components/PrivacyToggle.tsx
Tests
21 integration test files in omokoda-core/tests/
❌ Missing: tier_gate_tests.rs, synapse_tests.rs, reputation_curve_tests.rs
Makefile
❌ No Makefile at repo root — needs to be created
Implementation Plan
A. Rust Core — omokoda-core/src/
A1. Enhance reputation.rs (MODIFY — additive only)
Add to the existing module:
Rust
Augment reputation_gain() to accept actions_today: u32 and apply the diminishing returns multiplier. Gate tier promotion with can_promote_tier(). Cap actions_today at MAX_ACTIONS_PER_DAY — further gains return 0.0 if at cap.
No changes to existing difficulty(), tier_for(), tool_allowed(), mode_for_tier(), or the ReputationLedger struct — those are correct.
A2. Enhance economics.rs (MODIFY — additive only)
Add SynapseAccount struct:
Rust
Tier caps (synapse_cap):
T0: 1_000_000
T1: 10_000_000
T2: 30_000_000
T3: 60_000_000
T4/T5: 86_000_000
burn() is atomic: check balance ≥ cost, subtract, add to total_burned. Return Err("insufficient_synapse") if balance < cost — never partial burn.
A3. Create justice/tier.rs (NEW FILE)
Rust
Add pub mod tier; to justice/mod.rs. Export Tier from lib.rs if needed by integration tests.
B. Move Contracts — omokoda-on-chain/sources/
B1. Create synapse.move (NEW FILE)
Key rules:
Use transfer::transfer not transfer::public_transfer for non-store objects
Use if/else chain not match (Move doesn't have match)
Entry functions cannot return values
burn() takes pre-adjusted cost — no re-apply of tier multiplier inside
Structure:
Move
Do NOT modify garden.move.
C. Python Executor — omokoda-simulation/
C1. Create omokoda-simulation/executor.py (NEW FILE)
Add ÒgúnExecutor class alongside the existing simulation.py:
Python
Privacy routing rule (from architecture.md): PUBLIC can use any provider; PRIVATE/INCOGNITO must hard-fail with a descriptive error if routed to an external provider. Never silently reroute.
D. TypeScript Frontend — omokoda-frontend/
D1. Create lib/rpc_client.ts (NEW FILE)
Typescript
D2. Create components/PrivacyToggle.tsx (NEW FILE)
Tsx
E. Integration Tests — omokoda-core/tests/
E1. Create tier_gate_tests.rs
Test BB step limits per tier (T0→1, T1→6, T2→21, T3→107, T5→47_176_870)
Test synapse_cap per tier
Test 7-day promotion gate: promotion blocked if < 7 days since last
Test daily action cap: 51st action in a day returns 0.0 gain
E2. Create synapse_tests.rs
Test atomic burn: balance decremented exactly by cost
Test burn failure: insufficient balance returns Err, balance unchanged
Test earn_from_garden: always adds exactly 1000
Test earn_from_tip: amount > 10,000 clamped to 10,000
Test tier_cap enforcement: balance never exceeds cap
E3. Create reputation_curve_tests.rs
Test difficulty at rep=0: should be 1.0
Test difficulty at rep=25: should be 0.5
Test difficulty at rep=50: should be ~0.333
Test diminishing returns: daily_gain_multiplier(0) = 1.0, daily_gain_multiplier(50) ≈ 0.778
Test T5 threshold is exactly 100.0
F. Makefile (NEW FILE at repo root)
Makefile
G. docker-compose.yml (NEW FILE at repo root) — Lower Priority
Simple dev environment: Rust builder image + Node.js for frontend + Python for executor. Deferred until after core features are verified.
Phase 2 Components
All Phase 2 components are NEW directories — none conflict with existing structure.
H. omokoda-julia/ — Ọ̀ṢUN: BB Oracle + NIST Entropy (NEW DIRECTORY)
Julia package compiled to a shared library via PackageCompiler. Rust loads it via libloading at runtime. Three C-callable FFI exports:
calculate_bbu_c — heuristic BBU score (1.0–47.1) from AST analysis
validate_entropy_c — NIST SP 800-22 battery (frequency, runs, longest-run tests)
check_bb_bound_c — verify estimated steps within tier's BB bound
Key files to create:
omokoda-julia/Project.toml — Julia package manifest
omokoda-julia/src/bb_known.jl — BB(1-5) known values with citations
omokoda-julia/src/bb_approx.jl — conservative bounds for n>5
omokoda-julia/src/complexity.jl — BBU calculation with AST analysis + entropy-based non-determinism
omokoda-julia/ext/nist/dieharder_jl.jl — NIST SP 800-22 test implementations
omokoda-julia/src/ffi_exports.jl — C ABI wrappers for Rust FFI
omokoda-julia/build.jl — PackageCompiler build script
Integration point: omokoda-core gets a new ffi/julia.rs module that loads the shared library. Tests guarded by SKIP_JULIA_TESTS env var for CI without Julia installed.
I. omokoda-lisp/ — ỌBÀTÁLÁ: Ethics Engine (NEW DIRECTORY)
SBCL (Steel Bank Common Lisp) process loaded via CFFI. Implements the 7 Hermetic principles as symbolic logic evaluators for intent gating.
Key files to create:
omokoda-lisp/ethics.lisp — hermetic principle evaluators + consent check + Seal policy validator
omokoda-lisp/consent_rules.lisp — privacy/consent symbolic logic
omokoda-lisp/policy_ast.lisp — Seal policy S-expression representation
omokoda-lisp/rust_ffi.lisp — CFFI bindings exposing export_evaluate_intent, export_check_consent
omokoda-lisp/sbcl_init.lisp — SBCL startup, thread pool, FFI entry point
omokoda-lisp/tests/ethics_tests.lisp — test cases
Integration point: Rust calls the Lisp FFI before any think or act execution. All 7 hermetic principles must pass; denial returns structured reason list. Complements the existing justice/hermetic.rs scoring (which scores; this gates).
J. omokoda-elixir/ — YEMỌJA: Swarm Coordination (NEW DIRECTORY)
New Elixir OTP application with full production structure. The existing omokoda-swarm/ (basic OTP stub) remains unchanged — this new directory is the production-grade swarm layer. Eventually the two can be merged; for now they coexist.
Key files to create:
omokoda-elixir/mix.exs — Mix project file
omokoda-elixir/lib/yemoja.ex — OTP Application, DynamicSupervisor, Registry, gRPC server startup
omokoda-elixir/lib/agent_worker.ex — GenServer per agent: handles think/act/memory_checkpoint messages, public memory contributions, swarm coordination
omokoda-elixir/lib/hive_aggregator.ex — aggregates public memory across agents (PubSub pattern)
omokoda-elixir/lib/profile_sync.ex — cross-agent profile handoff
omokoda-elixir/proto/swarm.proto — gRPC service: SpawnAgent, SendMessage, QueryPublicMemory, SubscribeSwarmUpdates
omokoda-elixir/test/yemoja_test.exs — OTP supervision tests
Integration: Elixir gRPC server on port 50051. Rust core connects via tonic. Public memory aggregation — memory that agents mark as public flows here; private memory never touches Elixir (stays Rust-only per the sovereignty model).
K. omokoda-go/ — ỌYA: Networking & Flow Control (NEW DIRECTORY)
Go service distinct from the existing omokoda-ops/ (health/metrics). ỌYA handles real-time rate limiting, rhythm enforcement (Sabbath gate), and streaming flow updates. Go was chosen for its concurrency primitives (goroutines, channels).
Key files to create:
omokoda-go/go.mod — Go module file
omokoda-go/cmd/oya/main.go — gRPC server entry, graceful shutdown, Prometheus metrics on :9090
omokoda-go/internal/flow/service.go — EnforceFlow (rate limit + Sabbath check), StreamUpdates (heartbeat channel), BroadcastUpdate
omokoda-go/internal/ratelimit/limiter.go — tier-based token bucket: T0→1/5, T1→2/10, T2→5/20, T3→10/40, T4→20/80, T5→50/200 (req/sec, burst)
omokoda-go/proto/oya.proto — flow service definitions
omokoda-go/deploy/k8s/oya-deployment.yaml — K8s deployment manifest
Sabbath enforcement: UTC Sunday 00:00–01:00 = no actions allowed (matches ritual-codex governance constraints from north-star docs). Port: 50052.
L. nautilus_integration/ — TEE: Private Memory Encryption (NEW RUST CRATE)
New Rust crate added to workspace. Wraps the Nautilus SDK (Mysten Labs TEE for Sui) for private memory sealing. The existing memory/ module's private routes are the caller.
Key files to create:
nautilus_integration/Cargo.toml — workspace member crate
nautilus_integration/src/attestation.rs — verify TEE quote, derive seal key via HKDF(enclave_id + code_measurement), AES-GCM seal/unseal
nautilus_integration/src/handshake.rs — InitRequest/Response, StoreRequest, RetrieveRequest state machine with 5-minute session timeout
nautilus_integration/src/sealed_memory.rs — high-level memory sealing API
nautilus_integration/nautilus-sdk/ — vendored SDK stub (replace with git submodule)
nautilus_integration/tests/attestation_test.rs — seal/unseal roundtrip test
Security invariants: code measurement must match expected value (pinned in config); session nonces prevent replay; keys never leave TEE boundary; all sessions expire in 5 minutes.
Add nautilus_integration to workspace Cargo.toml members.
M. nist_entropy/ — NIST SP 800-22 Validation (NEW RUST CRATE)
New Rust crate added to workspace. FFI bindings to dieharder C library for statistical entropy validation. Used during birth ceremony and Odu key generation.
Key files to create:
nist_entropy/Cargo.toml — workspace member crate with bindgen build dependency
nist_entropy/src/dieharder_ffi.rs — unsafe FFI to dieharder: frequency_test, runs_test, longest_run_ones_test
nist_entropy/src/validator.rs — validate_entropy_seed(seed, min_bits) — runs battery at 99% confidence, checks SHA3 avalanche
nist_entropy/src/report.rs — NistReport / TestResult with serde serialization
nist_entropy/build.rs — links dieharder static library, runs bindgen
nist_entropy/ext/dieharder/ — vendored dieharder source (or git submodule instruction)
nist_entropy/tests/nist_validation_test.rs — valid entropy passes; all-zeros fails; avalanche check
Add nist_entropy to workspace Cargo.toml members.
N. Phase 2 Integration Tests
omokoda-core/tests/ additions (guarded by feature flags where external tools needed):
tier_gate_tests.rs — BB bounds, synapse caps, 7-day promotion gate, daily action cap (from MVP plan)
synapse_tests.rs — atomic burn, earn_from_garden, earn_from_tip, tier_cap (from MVP plan)
reputation_curve_tests.rs — difficulty formula, diminishing returns, T5=100.0 (from MVP plan)
New Phase 2: julia_bb_oracle_tests.rs — guarded by SKIP_JULIA_TESTS
New Phase 2: nautilus_tee_tests.rs — seal/unseal roundtrip (no hardware needed, uses test key)
New Phase 2: nist_entropy_tests.rs — valid entropy passes, weak entropy fails
Execution Order
Wave 0 — CI Prerequisite (fix ifascript dependency before any Rust work)
The CI currently checks out omo-koda/ifascript unconditionally. That repo is public and checks out fine, but its odu.rs is missing ODU_TABLE, causing a compile failure. The working local stub is at /home/user/Ifascript/ (3,085 lines across vm.rs, odu.rs, ebo.rs, entropy.rs — all correct).
Fix: Create ifascript-stub/ at the repo root (mirroring bipon39-stub/ pattern) as a self-contained copy of the working local stub. Update .github/workflows/ci.yml to replace the unconditional Checkout Ifascript step with continue-on-error: true + a fallback cp -r Omo-Koda2/ifascript-stub/. Ifascript/ step.
Files to create:
ifascript-stub/Cargo.toml — name = "ifascript", same deps as /home/user/Ifascript/Cargo.toml (lazy_static, rand, rand_chacha, sha2)
ifascript-stub/src/lib.rs — copy of working stub lib.rs (re-exports vm, odu, entropy, ebo modules)
ifascript-stub/src/vm.rs, odu.rs, ebo.rs, entropy.rs — copy from /home/user/Ifascript/src/
CI yaml change in the rust job (after the bipon39 fallback step):
Yaml
Wave 1 — Rust Core (must pass cargo test --workspace before proceeding)
justice/tier.rs + justice/mod.rs update (Tier enum with BB bounds)
reputation.rs enhancements (daily cap, diminishing returns, 7-day gate)
economics.rs enhancements (SynapseAccount struct, earn_from_garden, earn_from_tip)
Integration tests: tier_gate_tests.rs, synapse_tests.rs, reputation_curve_tests.rs
Add nautilus_integration/ and nist_entropy/ as workspace crates
NIST entropy tests (pure Rust, no dieharder needed for basic avalanche test)
Wave 2 — Smart Contracts + Python + TypeScript
7. omokoda-on-chain/sources/synapse.move (new Move contract)
8. omokoda-simulation/executor.py (Python executor with privacy routing)
9. omokoda-frontend/lib/rpc_client.ts (TypeScript RPC client)
10. omokoda-frontend/components/PrivacyToggle.tsx (UI component)
Wave 3 — New Language Services
11. omokoda-julia/ — Julia package with BB oracle + NIST + FFI exports
12. omokoda-lisp/ — SBCL ethics engine with CFFI
13. omokoda-elixir/ — Yemọja OTP swarm with gRPC
14. omokoda-go/ — ỌYA flow control with rate limiting
Wave 4 — Integration + Build
15. Phase 2 integration tests (julia_bb_oracle, nautilus_tee, nist_entropy)
16. Makefile (cross-language build targets including julia, lisp, elixir, go)
17. docker-compose.yml (optional dev environment)
Verification
Rust Core (Wave 1)
cargo test --workspace — all 410+ existing tests still pass + new tests pass
assert_eq!(difficulty(0.0), 1.0) and assert_eq!(difficulty(25.0), 0.5) — formula unchanged
SynapseAccount::burn() returns Err on insufficient balance, balance unchanged (never partial)
assert_eq!(Tier::T5.bb_step_limit(), 47_176_870) — BB bounds correct
nist_entropy basic tests pass without dieharder (avalanche check is pure Rust)
Smart Contracts (Wave 2)
6. cd omokoda-on-chain && sui move build — compiles (no match expressions, transfer::transfer not public_transfer)
7. omokoda-frontend: npm run build — no TypeScript type errors
8. PrivacyToggle: private mode hard-fails with user-visible error when non-local provider attempted
Language Services (Wave 3)
9. cd omokoda-julia && julia --project=. build.jl — builds lib/libomokoda_julia.so
10. julia --project=. test/runtests.jl — BB oracle + NIST tests pass
11. cd omokoda-elixir && mix test — Yemọja OTP supervision tests pass
12. cd omokoda-go && go test ./... — flow control + rate limiter tests pass
13. SKIP_JULIA_TESTS=1 cargo test --workspace — Rust tests still pass without Julia installed
Full Integration (Wave 4)
14. make all — cross-language build succeeds end-to-end
15. Verify Sabbath enforcement: Sunday 00:00 UTC → flow service returns rhythm_constraint
16. Verify Lisp ethics engine: hermetic violation returns structured reason list (not panic)

pub const MAX_ACTIONS_PER_DAY: u32 = 50;
pub const MIN_DAYS_BETWEEN_PROMOTIONS: u64 = 7;
pub const DIMINISHING_RETURNS_BASE: f64 = 0.995;

pub fn daily_gain_multiplier(actions_today: u32) -> f64 {
    DIMINISHING_RETURNS_BASE.powi(actions_today as i32)
}

pub fn can_promote_tier(last_promotion: Option<DateTime<Utc>>) -> bool {
    match last_promotion {
        None => true,
        Some(ts) => {
            let days = (Utc::now() - ts).num_days() as u64;
            days >= MIN_DAYS_BETWEEN_PROMOTIONS
        }
    }
}

pub struct SynapseAccount {
    pub balance: u64,
    pub total_burned: u64,
    pub tier: u8,
    pub last_decay_epoch: u64,
}

impl SynapseAccount {
    pub fn burn(&mut self, pre_adjusted_cost: u64) -> Result<(), &'static str> { ... }
    pub fn earn_from_garden(&mut self) { self.balance = (self.balance + 1000).min(self.tier_cap()); }
    pub fn earn_from_tip(&mut self, amount: u64) { self.balance = (self.balance + amount.min(10_000)).min(self.tier_cap()); }
    pub fn tier_cap(&self) -> u64 { ... } // per-tier caps below
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier { T0 = 0, T1 = 1, T2 = 2, T3 = 3, T4 = 4, T5 = 5 }

impl Tier {
    pub fn bb_step_limit(self) -> u64 {
        match self {
            Tier::T0 => 1,
            Tier::T1 => 6,
            Tier::T2 => 21,
            Tier::T3 | Tier::T4 => 107,
            Tier::T5 => 47_176_870,
        }
    }

    pub fn synapse_efficiency(self) -> f64 {
        match self { T0→1.0, T1→0.95, T2→0.90, T3→0.85, T4→0.80, T5→0.75 }
    }

    pub fn decay_rate_percent(self) -> f64 {
        match self { T0→12.0, T1→10.0, T2→8.0, T3→7.0, T4→5.0, T5→4.0 }
    }

    pub fn synapse_cap(self) -> u64 {
        match self { T0→1_000_000, T1→10_000_000, T2→30_000_000, T3→60_000_000, T4|T5→86_000_000 }
    }
}

module omokoda::synapse {
    struct SynapseAccount has key { id: UID, owner: address, balance: u64, total_burned: u64, tier: u8, last_decay_epoch: u64 }

    public entry fun create_account(ctx: &mut TxContext) { ... transfer::transfer(account, sender) }
    public entry fun burn(account: &mut SynapseAccount, pre_adjusted_cost: u64) { ... }
    public entry fun earn_from_garden(account: &mut SynapseAccount) { ... } // +1000, capped at tier_cap
    public entry fun earn_from_tip(account: &mut SynapseAccount, amount: u64) { ... } // capped at 10_000
    fun tier_cap(tier: u8): u64 { if (tier == 0) { 1_000_000 } else if (tier == 1) { 10_000_000 } else ... }
}

class PrivacyMode(Enum):
    PUBLIC = "public"
    PRIVATE = "private"
    INCOGNITO = "incognito"

class ÒgúnExecutor:
    # /private → hard fail if provider not WebLLM or Ollama
    # INCOGNITO → same as PRIVATE but no logging
    # execute_tool() → checks tier_allowed, computes synapse_cost, routes to correct runtime
    # _tee_execute_stub() → placeholder for TEE integration

export class RustRpcClient {
    private ws: WebSocket
    private pending: Map<string, { resolve, reject, timer }>

    birth(params): Promise<BirthReceipt>
    think(prompt, options): Promise<ThoughtReceipt>
    act(tool, args, options): Promise<ActReceipt>
    setPrivacyMode(mode: 'public'|'private'|'incognito'): Promise<void>
    forget(): Promise<void>
    private sendRequest(method, params, timeoutMs = 30_000)
}


// Three mode buttons: public / private / incognito
// Private/incognito: show ConsentDialog before switching
// Forget My Data: confirmation dialog → calls rpc_client.forget()
// Display current mode badge


.PHONY: all build test check move python frontend

all: build test

build:
    cargo build --workspace

test:
    cargo test --workspace
    cd omokoda-frontend && npm test --if-present
    cd omokoda-simulation && python -m pytest --if-present

check:
    cargo check --workspace
    cargo clippy --workspace -- -D warnings

move:
    # Sui Move build (requires sui CLI)
    cd omokoda-on-chain && sui move build

python:
    cd omokoda-simulation && python -m pytest

frontend:
    cd omokoda-frontend && npm run build


- name: Checkout Ifascript
  id: checkout_ifascript
  uses: actions/checkout@v4
  continue-on-error: true
  with:
    repository: omo-koda/ifascript
    path: Ifascript
- name: Fallback to vendored Ifascript stub
  if: steps.checkout_ifascript.outcome == 'failure'
  run: cp -r Omo-Koda2/ifascript-stub/. Ifascript/
