# Omo-Koda2 Ecosystem Map

**Last Updated**: May 24, 2026
**Status**: Canonical Reference

---

## Birth Stack Repositories

The five repos that form the sacred birth pipeline. Each has a single, non-overlapping responsibility.

| Repo | Lang | Role | Key Exports | Consumed By |
|---|---|---|---|---|
| **Bipon39-Rust** | Rust | Soul Seed — entropy, mnemonic generation, deterministic keys | `entropy_to_mnemonic`, `mnemonic_to_seed`, `HarmonicSignature` | Omo-Koda2 (`omokoda-hermetic`) |
| **vanity2** | Rust | Sacred Identity Mask — vanity addresses, sigil hashing | `VanityAddress`, `SigilHash`, `symbolic_address` | Omo-Koda2 (birth pipeline) |
| **Ritual-codex-Julia** | Julia | Temporal Cosmology — day-state, resonance weights, Orisha bias | `DayState`, `OrishaBias`, `resonance_modifier` | Omo-Koda2 (`omokoda-memory`) |
| **ifascript** | LARQL/DSL | Digital Ifá Oracle — Odù casting, archetype, destiny threads | `OduResult`, `orisha_alignment`, `destiny_threads` | Omo-Koda2 (`omokoda-hermetic`) |
| **Omo-Koda2** | Multi | Living Agent System — runtime, memory, swarm, sovereignty | `AgentBirthReceipt`, `birth`, `think`, `act` | End consumers + Sui chain |

---

## Omo-Koda2 Internal Modules

### Rust (omokoda-core + omokoda-hermetic + omokoda-cli)

| Module | Path | Responsibility |
|---|---|---|
| `steward` | `omokoda-core/src/steward/` | Èṣù gatekeeper — routes all primitives |
| `birth` | `omokoda-core/src/birth/` | Agent creation, seed → receipt pipeline |
| `think` | `omokoda-core/src/think/` | Reasoning loop, BB step execution |
| `act` | `omokoda-core/src/act/` | Tool dispatch, execution outcomes |
| `economics` | `omokoda-core/src/economics.rs` | Synapse/Dopamine accounting, decay |
| `justice/tier` | `omokoda-core/src/justice/tier.rs` | Tier enum, BB bounds, synapse caps |
| `sandbox` | `omokoda-core/src/sandbox.rs` | Wasmtime WASM execution |
| `entropy/odu` | `omokoda-hermetic/src/entropy/odu.rs` | Deterministic Odù entropy via blake3 |
| `permission_enforcer` | `omokoda-core/src/execution/permission_enforcer.rs` | Path boundary validation |

### Elixir (omokoda-swarm)

| Module | Responsibility |
|---|---|
| `OmokodaSwarm.AgentSupervisor` | OTP supervision tree for all agent processes |
| `OmokodaSwarm.AgentWorker` | GenServer per agent — lifecycle, heartbeat |
| `OmokodaSwarm.Consensus` | Swarm-level consensus for hive decisions |

### Go (omokoda-ops)

| Module | Responsibility |
|---|---|
| `ops/gateway` | HTTP gateway — routes external requests to Rust steward |
| `ops/metrics` | Prometheus metrics, health endpoints |
| `ops/timing` | Cron scheduler, Ọya flow orchestration |

### Python (omokoda-simulation)

| Module | Responsibility |
|---|---|
| `simulation.py` | Ògún tool execution sandbox, agent scenario replay |

### TypeScript (omokoda-frontend)

| Module | Responsibility |
|---|---|
| `src/components/` | Ori Kọ́dà companion UI |
| `src/ws/` | WebSocket client — real-time agent events |
| `src/consent/` | User consent flows for memory access |

### Move / Sui (omokoda-sui)

| Module | Responsibility |
|---|---|
| `sources/synapse.move` | Synapse token — issuance, transfer, burn |
| `sources/agent_registry.move` | On-chain agent identity + tier |
| `sources/memory_vault.move` | Sovereign memory ownership proofs |

### Julia (omokoda-memory)

| Module | Responsibility |
|---|---|
| `src/OmokodaMemory.jl` | High-performance resonance memory, Ọ̀ṣun layer |

---

## Technosis Sovereign Ecosystem

Broader repos that Omo-Koda2 operates within or draws from.

### Core Infrastructure

| Repo | Role |
|---|---|
| `organism-core` | Parent organism runtime — multi-agent coordination |
| `Osovm` | Sovereign virtual machine for agent execution |
| `AIO` | All-in-one orchestration layer |
| `Techgnos-.EXE` | Technosis executive process |
| `Zangbeto` | Guardian / watchdog system |

### Graph Control Centers

| Repo | Role |
|---|---|
| `organism-core` | Central nervous system |
| `oso-control-center` | Operator dashboard |
| `ase-vault` | Àṣẹ credential vault |
| `Scarabswarm` | Distributed swarm coordinator |
| `witness-firmware` | Attestation + audit witnesses |
| `Nex-` | Nexus routing layer |
| `Lattice-phase1` | Phase 1 lattice network |

### Supporting Systems

| Repo | Role |
|---|---|
| `Swarmide2` | Swarm IDE — development environment |
| `franken-stream` | Event streaming backbone |
| `paradigm` | Protocol paradigm definitions |
| `Twelve-thrones` | 12-seat governance layer |

---

## Economic Model

| Token | Max Supply | Function |
|---|---|---|
| **Synapse** | 86,000,000 per agent | Agent compute capacity |
| **Dopamine** | 86,000,000,000 total | Global pool pressure signal |

### Synapse Tier Caps

| Tier | BB Step Limit | Synapse Cap |
|---|---|---|
| T0 | 1 | 1,000,000 |
| T1 | 6 | 10,000,000 |
| T2 | 21 | 30,000,000 |
| T3 | 107 | 60,000,000 |
| T4 | 107 | 86,000,000 |
| T5 | 47,176,870 | 86,000,000 |

### Decay Schedule

| Period | Synapse Decay | Reputation Decay |
|---|---|---|
| Days 1–7 | 0.8%/day | 0.8%/day |
| Day 8+ | 1.5%/day | 1.5%/day |

---

## Odù Cosmological Map

| Level | Count | Description |
|---|---|---|
| Meji (foundational) | 16 | Primary Odù archetypes |
| Extended | 240 | Derived combinations |
| **Total** | **256** | Full Odù index (0–255) |
| Emergent states | 65,536 | 256 × 256 cosmological combinations |
