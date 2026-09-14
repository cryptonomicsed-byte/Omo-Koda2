# Sovereign OS Architecture — Canonical Specification
## Status: Mostly IMPLEMENTED — Gaps Identified Below
### Date: 2026-09-14

---

## Audit Summary: What's Already Built

This spec was written AFTER inspecting actual source code. Most of what prior threads
proposed as "to be built" already exists.

```
kernel/
  ipc.rs              EXISTS — inter-process communication
  process.rs          EXISTS — process model
  scheduler.rs        EXISTS — task scheduler
  device.rs           EXISTS — device manager
  fs.rs               EXISTS — SovereignFS base
  capability.rs       EXISTS — CapabilityFabric (built this session)
  compute/
    accounting.rs     EXISTS — Dopamine accounting
    attestation.rs    EXISTS — HardwareAttestation
    gpu.rs            EXISTS — GPU device representation
    lease.rs          EXISTS — GPU lease lifecycle
    telemetry.rs      EXISTS — GpuTelemetry + commitment hash
    verified_work.rs  EXISTS — VerifiedGPUWork (COMPLETE — see below)

lifecycle/
  heartbeat.rs        EXISTS — AgentHeartbeat (tamper-evident SHA-256 chain)
  job_daemon.rs       EXISTS — job daemon
  skill_daemon.rs     EXISTS — skill daemon

vantage/
  heartbeat.rs        EXISTS — Vantage reporter (FRAGMENTED — see Gap A)
```

---

## The Canonical Sovereignty Stack

Locked architecture. Every system has ONE job.

```
HUMAN
  │
  ▼
VANTAGE                  — social / jobs / guilds / marketplace / coordination
  │                        "What work exists and who is coordinating it?"
  │ jobs / signals
  ▼
╔══════════════════════════════════════════════════════╗
║                    ỌMỌ KỌ́DÀ2                        ║
║               SOVEREIGN AGENT OS                     ║
║                                                      ║
║ Kernel: ipc / process / scheduler / device / fs      ║
║ Agent Runtime: identity / memory / skills / session  ║
║ Lifecycle: heartbeat / job_daemon / skill_daemon     ║
║ Compute: gpu / lease / telemetry / verified_work     ║
║ Network: Nostr / Meshtastic / Freenet / A2A / MCP    ║
║ Genesis: birth / capabilities / manifest / receipt   ║
║                                                      ║
║ "Who is alive and what can they do?"                 ║
╚══════════╤═══════════════════╤════════════════════════╝
           │                   │
    evidence/work          compute demand
           │                   │
           ▼                   ▼
       ZÀNGBÉTÒ          DOPAMINE POOL
     evidence /           total verified
     receipts /           GPU compute
     provenance                │
           │            Synapse (agent slice)
           └─────────┬──────────┘
                     │
                     ▼
╔══════════════════════════════════════════════════════╗
║                    Ọ̀ṢỌ́VM                            ║
║              PROTOCOL / BLOCKCHAIN VM                ║
║                                                      ║
║ deterministic execution / Veils / proof              ║
║ settlement / mint authorization / economic invariants║
║                                                      ║
║ "What actually happened and what does the protocol   ║
║  recognize?"                                         ║
╚══════════╤═══════════════════════════════════════════╝
           │
    ┌──────┼──────────┬──────────┐
    ▼      ▼          ▼          ▼
   Sui   Walrus     Akash     Arweave
settle  storage    GPU       permanent
```

**Token role separation (locked):**
```
Àṣẹ      = merit / reputation (NOT money, NOT compute)
Dopamine = compute capacity pool (86B units)
Synapse  = agent compute allocation (86M per agent slice)
ASE      = ecosystem currency (10:1 Dopamine burn to Synapse)
USDC     = external money (enters via Èṣù Router → Akash)
```

---

## VerifiedGPUWork — Already Complete

`kernel/compute/verified_work.rs` implements the full chain the thread described.
No new code needed. The struct covers every field proposed:

```rust
// Already exists — DO NOT RE-IMPLEMENT
pub struct VerifiedGPUWork {
    pub work_id:              String,
    pub contributor_id:       String,   // agent_id
    pub device_id:            String,   // /devices/gpu/0001
    pub gpu_model:            String,
    pub lease_id:             String,
    pub start_time:           u64,
    pub end_time:             u64,
    pub gpu_seconds:          f64,
    pub workload_hash:        String,
    pub workload_type:        WorkloadKind,
    pub utilization:          f32,
    pub output_commitment:    String,
    pub telemetry_commitment: String,
    pub witness_receipts:     Vec<String>,
    pub hardware_attestation: HardwareAttestation,
    pub osovm_proof:          Option<String>,  // filled by OSOVM (gap #19-21)
    pub compute_score:        Option<f64>,
    pub dopamine_allocation:  Option<u64>,     // filled after OSOVM verify (gap #19-21)
    pub zangbeto_receipt_id:  Option<String>,
    pub created_at:           u64,
}

// The key invariant — already enforced:
impl VerifiedGPUWork {
    pub fn is_fully_verified(&self) -> bool {
        self.osovm_proof.is_some()
            && !self.witness_receipts.is_empty()
            && self.zangbeto_receipt_id.is_some()
            && self.hardware_attestation.verified
    }
}
```

**The remaining gap:** `is_fully_verified()` is implemented but nothing calls it to
authorize OSOVM minting. This is gaps #19-21 (OSOVM ToC 3 code additions).

---

## AgentHeartbeat — Already Complete (Hash Chain)

`lifecycle/heartbeat.rs` implements the tamper-evident chain the thread described.
Tests pass. No new code needed for the heartbeat struct itself.

**Gap A (the real gap):** Two separate heartbeat systems coexist:

```
lifecycle/heartbeat.rs    ← canonical AgentHeartbeat with SHA-256 chain
                             genesis() / next_from() / verify_chain()
                             State: Alive/Thinking/Working/Sleeping/etc.
                             SomaVector { energy, tension, focus, gpu_util }

vantage/heartbeat.rs      ← independent Vantage reporter
                             POSTs raw JSON to /api/nodes/heartbeat
                             Does NOT use AgentHeartbeat
                             Maintains its own interval/config
```

**Fix:** `vantage/heartbeat.rs` should serialise a canonical `AgentHeartbeat`
and send it to Vantage, rather than constructing its own payload.
Vantage gets the full tamper-evident record; the hash chain stays intact.

**Gap A is small:** one function change in `vantage/heartbeat.rs` + Vantage
endpoint updated to accept `AgentHeartbeat` JSON.

---

## Gap B: The Services Layer (Missing)

The daemons exist as individual files. There is no supervisor above them.

**Current state:**
```
lifecycle/
  heartbeat.rs    ← runs as background task
  job_daemon.rs   ← runs as background task
  skill_daemon.rs ← runs as background task
```

**Missing:** A `DaemonSupervisor` that:
- Holds a registry of running daemons
- Restarts daemons on panic/exit
- Reports daemon health in `AgentHeartbeat.active_daemons`
- Allows new daemons to be registered at runtime (agent installs a skill → new daemon)
- Provides a clean shutdown signal

**Where it goes:** `src/lifecycle/supervisor.rs` — NOT a new top-level `services/`
directory (the daemons already live in `lifecycle/`, keep them together).

```rust
// New type — src/lifecycle/supervisor.rs
pub struct DaemonSupervisor {
    daemons: HashMap<String, DaemonHandle>,
}

pub struct DaemonHandle {
    pub name: String,
    pub task: tokio::task::JoinHandle<()>,
    pub restart_count: u32,
    pub started_at: u64,
}

impl DaemonSupervisor {
    pub fn register(&mut self, name: &str, handle: JoinHandle<()>) { ... }
    pub fn running_names(&self) -> Vec<String> { ... }  // feeds AgentHeartbeat.active_daemons
    pub async fn supervise_loop(self) { ... }           // restarts crashed daemons
}
```

---

## OSOVM Determinism Gates (Gate 4–8)

Gates 1–3 are confirmed complete. Remaining gates define the proof requirements
before simulation becomes economically meaningful.

```
Gate 1: Free dynamics              ✓ DONE
Gate 2: Cross-platform (macOS/Linux)  ✓ DONE
Gate 3: Cross-architecture (ARM64/x86) ✓ DONE

Gate 4: Contact / collision determinism
        Test: single rigid body + ground plane, deterministic contact solve
        Block: Dopamine allocation for simulation workloads (WorkloadKind::Simulation)

Gate 5: Multiple contact bodies
        Test: N bodies in mutual contact, canonical hash reproducible across arch

Gate 6: Long-horizon simulation (>10k timesteps)
        Test: hash of trajectory at timestep 10,000 matches across ARM64/x86

Gate 7: Witnessed simulation receipt
        Test: ScarabSwarm SimReceipt produced, ARP ActionReceipt wraps it,
              Zàngbétò can independently recompute

Gate 8: Simulation → physical transfer authorization
        Gate condition: all 7 above pass. OSOVM authorizes sim→real job dispatch.
        This is when VerifiedGPUWork.workload_type = Simulation qualifies for
        Dopamine allocation.
```

**Connection to VerifiedGPUWork:** Until Gate 7 is passed, `WorkloadKind::Simulation`
cannot set `dopamine_allocation`. The `is_fully_verified()` check already enforces
this — the OSOVM proof field will remain None until OSOVM accepts simulation workloads.

---

## Omo-Koda2 Portability (Node Independence)

The same agent identity should survive hardware changes.

```
AgentID: OK-001
  │
  ├─ Android/Termux (Fold 8)
  │    same birth_memory_glyph
  │    same private_key (Tier 1 identity vault)
  │    same memory (Tier 2 minipae)
  │    same receipt chain (ARP)
  │
  ├─ Linux VPS
  │    restores from encrypted state capsule
  │    resumes heartbeat chain (sequence continues)
  │    resumes daemons
  │
  ├─ ZimaOS node
  │    same AgentID, same chain
  │    storage via ZimaOS ZFS layer → SovereignFS abstraction
  │
  └─ GPU compute node
       same AgentID
       HeartbeatState::Computing
       VerifiedGPUWork attributed to this agent
```

**What enables this:** `AgentGenesisReceipt` + sealed `identity_vault` + Tier 2
minipae memory are the portable state. The kernel/compute layer is stateless
(leases are per-machine). The agent's identity outlives any single machine.

**What needs to be built:** `AgentCapsule` (already designed in `capsule.rs`) +
migration path (INTEROP_FABRIC_SPEC.md Gap A: MIGRATION lifecycle state).

---

## ZimaOS/ZVM Architecture Extraction (Design Exercise)

**Rule:** Pattern transplantation, not code transplantation. Study → classify → implement natively.

```
ZimaOS Component         Classification    Target in our stack

ZFS/RAID storage         ADAPT             SovereignFS (fs.rs already exists)
Docker/Compose apps      ADAPT             DaemonSupervisor + skill packaging
App Store (800+ apps)    STUDY UX          Skill registry philosophy
ZVM VM lifecycle         ADAPT → OSOVM     OSOVM runtime layer (not consensus)
ZVM resource management  ADAPT → OSOVM     OSOVM resource abstraction
ZVM isolation            ADAPT             kernel/process.rs sandbox
ZVM snapshots            ADAPT             AgentCapsule (already designed)
ZVM scheduling           ADAPT             kernel/scheduler.rs (already exists)
ZimaOS networking        REJECT            We own network abstraction
ZimaOS identity          REJECT            BIPON39 + GIX is our identity
ZimaOS remote access     ADOPT patterns    Via Vantage + DIP network
ZimaOS hardware mgmt     STUDY             kernel/device.rs (already exists)
ZimaOS UX philosophy     ADOPT             "Make complex infra feel simple"
```

**The UX principle worth stealing:**
ZimaOS hides RAID/Docker/VM complexity behind a unified interface.
Omo-Koda2 should do the same for agents:

```
Traditional OS concept    Omo-Koda2 equivalent
─────────────────────    ─────────────────────
process                  → daemon / worker
container                → skill sandbox
driver                   → device adapter (VCP/DIP)
network interface        → transport (Nostr/Mesh/Freenet)
filesystem               → SovereignFS + minipae + Walrus
GPU kernel               → kernel/compute/ + VerifiedGPUWork
service                  → lifecycle daemon
VM                       → OSOVM execution context

User interface:
  "Run this service continuously" → DaemonSupervisor registers it
  "Give agent access to camera"   → VCP device session
  "Store this permanently"        → SovereignFS policy → Walrus/Arweave
  "Use GPU for this task"         → kernel/compute/ → lease → VerifiedGPUWork
```

**NOT to be done:** Don't fork ZimaOS or ZVM. Don't make ZimaOS a dependency.
It's a reference, not a substrate.

---

## Canonical OS Subsystem Status Matrix

```
SUBSYSTEM                    FILE                        STATUS
─────────────────────────────────────────────────────────────────
kernel/ipc                   kernel/ipc.rs               EXISTS
kernel/process               kernel/process.rs           EXISTS
kernel/scheduler             kernel/scheduler.rs         EXISTS
kernel/device                kernel/device.rs            EXISTS
kernel/fs (SovereignFS)      kernel/fs.rs                EXISTS
kernel/capability            kernel/capability.rs        EXISTS (this session)
kernel/compute/gpu           kernel/compute/gpu.rs       EXISTS
kernel/compute/lease         kernel/compute/lease.rs     EXISTS
kernel/compute/telemetry     kernel/compute/telemetry.rs EXISTS
kernel/compute/attestation   kernel/compute/attestation.rs EXISTS
kernel/compute/verified_work kernel/compute/verified_work.rs EXISTS
kernel/compute/accounting    kernel/compute/accounting.rs EXISTS
kernel/compute → OSOVM mint  (wiring)                    MISSING (gap #19-21)
─────────────────────────────────────────────────────────────────
lifecycle/heartbeat          lifecycle/heartbeat.rs      EXISTS (chain complete)
lifecycle/job_daemon         lifecycle/job_daemon.rs     EXISTS
lifecycle/skill_daemon       lifecycle/skill_daemon.rs   EXISTS
lifecycle/supervisor         lifecycle/supervisor.rs     MISSING (Gap B)
vantage heartbeat unified    vantage/heartbeat.rs        PARTIAL (Gap A)
─────────────────────────────────────────────────────────────────
genesis/capability           genesis/capability.rs       EXISTS (this session)
genesis/network_router       genesis/network_router.rs   EXISTS (this session)
genesis/receipt (compute)    genesis/receipt.rs          MISSING GPU fields
─────────────────────────────────────────────────────────────────
network adapters             (DIP crates)                STUBS (gaps #29-34)
─────────────────────────────────────────────────────────────────
OSOVM Gate 1-3               (confirmed)                 DONE
OSOVM Gate 4-8               (contact determinism)       NOT STARTED
─────────────────────────────────────────────────────────────────
services/ supervisor layer   (planned)                   MISSING (use lifecycle/)
Agent portability capsule    capsule.rs exists           PARTIAL (Migration=gap #16)
```

---

## Actual Implementation Priority (Corrected from Thread)

The thread's P0-P4 list contains many items already done. Correct remaining order:

```
IMMEDIATE (small, high impact):
  Gap A:  Unify heartbeat — vantage/heartbeat.rs uses AgentHeartbeat (1 file change)
  Gap B:  DaemonSupervisor in lifecycle/supervisor.rs (new file, no external deps)
  Gap #19-21: Wire kernel/compute/ → OSOVM → Dopamine mint (3 additions)

NEXT:
  OSOVM Gate 4: Contact determinism test + gate enforcement
  Gap #23-24: ARP wrapping for think/act (completes receipt chain)
  Gap #64: ArpBridge wraps receipts with GIX1 (after ~/GIX/ Phase 1)

AFTER THAT:
  OSOVM Gates 5-7: Multi-body + long-horizon + witnessed sim receipt
  Gap #16: MIGRATION lifecycle state (AgentCapsule portability)
  ZimaOS UX study: Apply "make complex infra feel simple" to skill packaging

OSOVM Gate 8 (sim→real authorization):
  Only after Gates 1-7 all pass. Not a code task yet — a gate condition.
```

---

## What NOT to Build

| Thread Suggests | Why Not |
|---|---|
| VerifiedGPUWork type | Already exists in kernel/compute/verified_work.rs |
| AgentHeartbeat with hash chain | Already exists in lifecycle/heartbeat.rs |
| kernel/compute/ subsystem | Already exists with 6 files |
| job_daemon, skill_daemon | Already exist in lifecycle/ |
| Fork ZimaOS | Study patterns only; we own the abstraction |
| Import ZVM | Extract relevant patterns into OSOVM runtime layer |
| services/ directory | Use lifecycle/ — daemons already live there |
| Aether engine dup fix | Already fixed (memory: "3 known bugs fixed") — VPS-side |
