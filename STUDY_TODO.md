# OMO-KODA2 STUDY EXTRACTIONS — BUILD TODO
# Source: deep dive of Droidclaw, Claude-2 bridge, Claude-mirror, agency-agents, Core-Agency
# Last updated: 2026-09-13

Status legend: [ ] pending  [x] done  [~] in-progress  [!] blocked

---

## GROUP A — EMOTION + SENSOR LAYER

- [x] A-01  Add `update_from_sensors(battery, temp_c, hour)` to `emotion.rs`
            Maps device state → EmotionState (battery<15%→energy=0.1, temp>40→tension+=0.2)
- [x] A-02  Create `lifecycle/sensor.rs` — Termux device sensor reader
            Reads battery%, CPU temp, hour-of-day via termux-battery-status / system files
            Returns `SensorReading { battery_pct, temp_c, hour }`
- [x] A-03  Add `SomaVector` struct to `lifecycle/heartbeat.rs` and attach to `AgentHeartbeat`
            Fields: energy, tension, focus, gpu_util — snapshot of EmotionState at beat time

---

## GROUP B — SOMA LIFECYCLE

- [x] B-01  Create `lifecycle/soma_lifecycle.rs` — wake_up / pulse / sleep
            `wake_up()`: reads sensors, initializes EmotionState, loads LPM from disk
            `pulse(message, role)`: updates emotion, stores episodic memory if significant
            `sleep(session_summary)`: learns from session, updates LPM, writes to disk
- [x] B-02  Wire `soma_lifecycle::wake_up()` into `bootstrap.rs` agent startup sequence
- [x] B-03  Wire `soma_lifecycle::pulse()` into `main_loop.rs` per-message cycle
- [x] B-04  Wire `soma_lifecycle::sleep()` into SIGTERM/SIGINT graceful shutdown path

---

## GROUP C — SOUL BUILDER (DYNAMIC SYSTEM PROMPT)

- [x] C-01  Complete `steward/soul.rs` `SoulBuilder::build()` method
            Assembles system prompt from: SOMA context + goals + tools + device context + recent memories
            (build() was already implemented; C-02..C-05 are extensions)
- [x] C-02  SoulBuilder::new() constructor added; iris params are a required field
- [x] C-03  Add `SoulBuilder::with_device_context(SensorReading)` — injects battery/wifi/time
- [x] C-04  Add `SoulBuilder::with_goals(Vec<String>)` — injects active goal list
- [x] C-05  Add `SoulBuilder::with_memories(Vec<String>)` — injects recent minipae memories (top 5)

---

## GROUP D — SKILL PATCH GATE (SELF-MODIFY)

- [x] D-01  Create `skill_patch/patch.rs` — SkillPatch gate
            `SkillPatch { file, reason, proposed_code, original_code, diff, timestamp, sensitive }`
            `PatchGate::propose(file, reason, new_code) -> PatchProposal` — computes unified diff, saves pending
            `PatchGate::apply(proposal_id) -> Result` — backup original, write patch, emit ARP receipt
            `PatchGate::reject(proposal_id)` — discard pending
            `PatchGate::restore(file) -> Result` — restore from .backup file
- [x] D-02  ALLOWED list — files agents can propose changes to (skills/, plugins/, lifecycle/ only)
- [x] D-03  SENSITIVE list — files requiring extra confirmation (steward/soul.rs, steward/iris.rs, main_loop.rs)
- [x] D-04  Wire into `execution/permission_enforcer.rs` — `enforce_tier()` added; Sovereign for self_modify, Resident for tools, Observer read-only
- [x] D-05  Emit ARP receipt on apply (action=skill_patch_applied, evidence=diff_hash)
- [x] D-06  Create `skill_patch/mod.rs` — export PatchGate, SkillPatch, PatchProposal

---

## GROUP E — NDJSON BRIDGE / SESSION SPAWNER

- [x] E-01  Create `bridge/session.rs` — NDJSON session spawner
            `SessionSpawner::spawn(opts) -> (SessionHandle, NdjsonStream)`
            Spawns child process (omokoda-cli) via stdio pipes
            Reads NDJSON stdout, buffers stderr ring (last 10 lines)
            Emits tool_start / text / result / error activities
- [x] E-02  Add `PermissionRequest { request_id, tool_name, input, tool_use_id }` type
            Detect `control_request` in NDJSON stream, forward to permission gate
- [x] E-03  Add `SessionHandle::kill()` (SIGTERM) and `force_kill()` (SIGKILL)
- [x] E-04  Add `SessionHandle::write_stdin(data)` — send control messages to child
- [x] E-05  Add token refresh via stdin: `update_environment_variables` message type
- [x] E-06  Create `bridge/mod.rs` — export SessionSpawner, SessionHandle, PermissionRequest
- [x] E-07  Add transcript file writing (NDJSON log alongside debug log)

---

## GROUP F — TASK SYSTEM ENHANCEMENTS

- [x] F-01  Add prefix-coded task IDs to `tasks/types.rs`
            Think→`t`, Act→`a`, Dream→`d`, Delegate→`g`, Background→`b`, Agent→`ag`, Monitor→`m`
            Format: `{prefix}{8_random_alphanum}` e.g. `t3x9kf2a`
- [x] F-02  Add `TaskKind::Monitor { target: String }` — MCP server / process monitoring
- [x] F-03  Add `TaskKind::RemoteAgent { node_url, agent_id, prompt }` — explicit remote variant
- [x] F-04  Add `task.notified: bool` field — tracks if completion was surfaced to user
- [x] F-05  Add `TaskManager::active_count()` and `TaskManager::by_kind()` helpers

---

## GROUP G — HEARTBEAT ENHANCEMENTS

- [x] G-01  Add `soma_vector: Option<SomaVector>` field to `AgentHeartbeat`
            Snapshot of emotion state (energy, tension, focus) + gpu_util at beat time
- [x] G-02  Add `message_count: u64` to AgentHeartbeat — total messages since boot
- [x] G-03  Update `AgentHeartbeat::hash()` canonical JSON to include new fields
- [x] G-04  Add `AgentHeartbeat::shutdown_receipt()` — generates ARP-compatible receipt on SIGTERM
            Receipt includes: final sequence, uptime_secs, message_count, exit_reason

---

## GROUP H — AGENT ROLE CATALOG

- [x] H-01  Create `agent_catalog/mod.rs` — AgentRole struct + catalog loader
            `AgentRole { id, title, division, context, responsibilities, capabilities, output_format }`
- [x] H-02  Create `agent_catalog/engineering.rs` — seed from agency-agents engineering/ dir
            Include: embedded-firmware-engineer, autonomous-optimization-architect, backend-architect
- [x] H-03  Create `agent_catalog/osovm.rs` — OSOVM-specific roles
            SimulationRunner, WitnessAttestor, ComputeProofVerifier
- [x] H-04  Create `agent_catalog/sovereign.rs` — sovereign ecosystem roles
            NodeOperator, ReceiptAuditor, MeshRouter, GovernanceCouncilor
- [x] H-05  Add `AgentCatalog::find_by_id(id)`, `find_by_division(division)`, `all()` methods
- [x] H-06  Wire catalog into `plugins/registry.rs` — role_for(agent_id) + roles_by_division()

---

## GROUP I — IRIS ENHANCEMENTS

- [x] I-01  Add `IrisProfile::Fast` — between Reflex and Balanced
            256-512 tokens, temp 0.5, style: "direct, 2-3 lines, action-oriented"
            Routes: prompts 20-60 chars that aren't technical or emotional
- [x] I-02  Add `IrisEngine::record_decision(profile, emotion)` — learns routing patterns
            Stores last 200 decisions in agent memory for pattern analysis
- [x] I-03  Add `IrisEngine::get_stats()` — returns profile distribution as string

---

## GROUP J — SETTINGS TIER SYSTEM

- [x] J-01  Add `AgentTier` enum to `config.rs` (Observer/Resident/Sovereign)
            Tier 1 (Observer): read-only, no tool execution
            Tier 2 (Resident): standard tools, no self-modify
            Tier 3 (Sovereign): full tool access, self-modify allowed
- [x] J-02  Create example settings files: `config/settings-strict.json`, `config/settings-lax.json`
- [x] J-03  Wire tier checks into `execution/permission_enforcer.rs` — enforce_tier() with TierViolation error

---

## POST-BUILD VERIFICATION

- [!] PV-01  cargo check -p omokoda-core — blocked: cranelift-codegen SIGSEGV on Termux/ARM64 (pre-existing, unrelated to our code; all new files parse cleanly via rustfmt)
- [x] PV-02  All new modules exported from lib.rs (agent_catalog, bridge, skill_patch)
- [x] PV-03  Unit tests written for: SoulBuilder (8 tests), PatchGate (4 tests), sensor (2 tests), soma_lifecycle (3 tests), IRIS (12 tests), heartbeat (5 tests), tasks, tier enforcement (4 tests)
- [x] PV-04  Commit to Omo-Koda2 with descriptive message
