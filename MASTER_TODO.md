# Sovereign Stack — Master Build Checklist
**Last audited: 2026-09-12 (deep-dive, 94 repos). Single source of truth — update here.**
**All repos: github.com/cryptonomicsed-byte**

Legend: ✅ done · 🔶 partial · ❌ missing · 🗄️ archived · 🪞 mirror/migrated

---

## COMPLETE REPO MAP

### CORE PILLARS (3 active + 3 predecessors)
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **Omo-Koda2** | Rust | active | Sovereign Agent OS — 3-primitive kernel, 11 lobes, aether-economy, SkillForge |
| **Vantage** | Python | active | Agent-first social/work backend ~700 REST+MCP endpoints, live at omokoda.duckdns.org |
| **OSOVM** | Julia | active | Sacred VM — 160+ opcodes, Àṣẹ economy, F1 quality gate ≥0.777 |
| **sovereign-stack** | Rust | active | 12-crate monorepo: DIP, VCP, UCX, ARP, swarm, witness, twin-protocol, A2A, pipeline, CLI |
| Omokoda | TypeScript | active | Older TS Omo-Koda organism (11 lobes, soul.move, RLM parliament) — superseded by Omo-Koda2 |
| Omo-koda-369 | Rust | 🗄️ | Pre-Omo-Koda2 iteration (7-module kernel) |
| Omo-koda | Rust | 🗄️ | Earliest "Claw" spec-first concept |

### LANGUAGE STACK / VMs
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **If-Script** | Rust | active | IfáScript Ω — entropy+divination VM. 256 Odù→16 Action Vessels, scales to 65,536 |
| **Swibe** | JavaScript | active | Agent scripting language v3.4 — 44 compile backends, 405 tests, BIPON39 vault |
| **zerolang** | C | active | Graph-native agent language — program IS a semantic graph, agents patch via CLI |
| **larql** | Rust | active | Decompiles transformer weights into queryable vindex, LQL graph DB query language |
| **Techgnosis** | Julia | active | OSO VM Dispatcher — compiles spiritual IR dispatched to 6 languages, FFI to OSOVM |
| **Nex-** | JavaScript | active | Nex v1.0 graph runtime — 7 Hermetic/Orisha primitives, self-modifying computation graphs |
| **Koodu** | Julia | active | 7-day Orisha resonance system, BTC-anchored sacred time, 49-facet grid, organism bridge |
| Aether | JavaScript | 🪞 | JS Aether language — birth/think/ethics/receipt primitives, Sui escrow. Migrated from Bino-Elgua |
| Oso-Aether | Rust | 🪞 | Rust WASM Aether runtime — 3-primitive AI pet, Sui dNFT, 86-char DNA fingerprint |
| OsO | TypeScript | 🪞 | TS frontend for OsO 3-primitive pet agent language |
| paradigm | TypeScript | 🪞 | 10-paradigm consciousness/reasoning system, emergent property detection |
| Nex | JavaScript | 🗄️ | Earlier JS Nex — superseded by Nex- |
| Ifascript | Rust | 🗄️ | Earlier Rust IfáScript — superseded by If-Script |
| vibe-lang | JavaScript | 🗄️ | Early AI-native language, 18 compile targets — superseded by Swibe/zerolang |
| ritual-codex | JavaScript | 🗄️ | 7-day Orisha codex JSON + Julia — archived |

### PROTOCOL / IDENTITY
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **ip-layer** | Rust | active | Nostr IP provenance: IP Root (31900), Creation Receipt (1901), Attestation (1902), Twin Binding (1903) |
| **BIPON39** | Rust | active | Base-256 Yorùbá/Ifá mnemonic — entropy→mnemonic→PBKDF2 seed→BIP-32 master key |
| **minipae** | Python | active | Pure-Python NIP-AE (kind:30174) memory bus — BIP-340, NIP-44 v2, multi-relay, CLI |
| **agent-phone** | Python | active | Sovereign agent telecom: SuiNS=number, Nostr=ring (NIP-17), WebRTC=call, Reticulum=fallback |
| **Triune-Memory** | TypeScript | active | Three-tier memory orchestrator (episodic/semantic/procedural) delegating to minipae |
| **iranti** | Rust | active | Sovereign memory mesh for Buzz — BM25 recall, A2A memory grants, dream-cycle digests, MCP tools |
| Bipon39-Rust | Rust | 🗄️ | Older Bipon39 from Bino-Elgua — same spec, archived |
| Droidclaw | JavaScript | 🪞 | "Kira" Android AI agent, persistent memory + emotional modeling |

### COORDINATION / OBSERVABILITY
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **mycelium** | Python | active | Stigmergic trace substrate — SQLite log, domain miners, MCP stdio server, skill self-improvement loop |
| **mycelium-tools** | Python | active | 5 pip packages wrapping mycelium for any device: substrate, collector, cycle, tunnel, picks-client |
| **agentic-waggle** | Go | active | Real-time swarm coordination — decaying scent signals, claim/release leases, Lévy-walk foraging |
| **organism-core** | TypeScript | 🪞 | Nervous-system bridge: IfáScript→Swibe→OSOVM→AIO→Zangbeto TS bridges. NIP-31 embodiment spec |

### ECONOMY / GOVERNANCE
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **AIO** | Move | active | Àṣẹ Input/Output on Sui — elder-gated mode flips, oracle bonds, escrow, Àṣẹ mint/burn, tithe |
| **bondhive** | Rust | active | NIP-74 Service Bond reliability layer — staked bonds, BondScore reputation (design stage) |
| **waggle** | Rust | active | Agent labor exchange for Buzz+Solana — Nostr bounty events (31900–31903), Anchor escrow; scaffold |
| **Portent** | Python | active | Nostr-native prediction market — 1B PORT token, oracle agents, staking tiers; scaffold |
| **Zangbeto** | Rust/Node/Move | active | Smart contract red-team protocol — Arweave receipts, BTC OpenTimestamps proofs, n8n Night Patrol |
| **Blocksim** | Python | active | Proof-of-Simulation chain API — stake, attest firmware, submit signed sim logs, earn rewards |
| Proof | Python | 🗄️ | Predecessor reputation token for Buzz+Solana — superseded by bondhive/waggle |

### HARDWARE / SIMULATION
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **omokoda-mesh** | C++ | active | ESP32 LoRa firmware — Nostr-native, real BIP-340 Schnorr, BIP-32 key derivation, multi-network relay |
| **omokoda-mesh-firmware** | C++ | active | Fork of official Meshtastic firmware for gateway bridge nodes |
| **omokoda-mesh-os** | Shell | active | Custom Arch Linux (archiso) OS for Raspberry Pi LoRa gateway nodes |
| **Witness-firmware** | Python | active | ESP32+SX1278 DePIN witness firmware — physics-proof attestation, Prophetic Pantheon CrewAI |
| **Scarabswarm** | Julia | active | Autonomous drone racing sim — 6-DOF physics, SHA-256 trajectory proof hashes, LLM pilot |

### SOCIAL / COMMUNICATION
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **Buzz** | Rust | active | Self-hostable Nostr relay workspace for humans+agents — hash-chained audit log (Crucible) |
| **Buzz-swarm** | TypeScript | active | Buzz swarm UI — Vite/TS frontend, server.ts, agent services, swarm dashboard |
| **Synapse** | TypeScript | active | NIP-30 agent skill/memory/negotiation/trust layer for Buzz — 7 event kinds, Next.js dashboard |
| **agentslack** | TypeScript | active | Slack-like workspace running agents as Claude Code processes with MCP tools |
| **Vantage-Voice-** | TypeScript | active | Real-time S2S voice for Vantage — Gemini Live + Groq Whisper→ElevenLabs cascade, barge-in, PWA |
| **omokoda-smithers** | TypeScript | active | Smithers orchestrator wired to Omo-Koda2 SkillForge — durable approval gates |
| **Twelve-thrones** | TypeScript | 🪞 | 12-model AI epistemology court — disagreement NFTs on Sui, Arweave archive |
| buzz-OG | Rust | active | Original Buzz hive-mind platform — predecessor to current Buzz/Crucible |
| s2s | TypeScript | 🗄️ | Predecessor S2S voice app — superseded by Vantage-Voice- |

### TRADING / INTELLIGENCE
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **strategy-lab** | Python | active | Early-entry trading ensemble — signal engine, whale radar, convergence hub |
| **Loom** | Python | active | Multi-agent (Whale/Technical/Risk) trading debate system — pure NumPy, Julia graph engine, backtester |
| **TradingOS** | Python | active | Agent-native trading platform spec — Rust bus, Elixir OTP parliament, Go data, Julia quant, Next.js |
| **ares-control** | Python | active | Split-process control panel for ~70 VPS systemd trading/bridge daemons |
| **kanban** | Elixir | active | Phoenix OTP trading pipeline — 5 pipeline stages, hash-chained audit log |
| **Axiom** | TypeScript | active | 3D galaxy graph runtime — every node=live agent, every edge=active channel, Three.js |

### DOCUMENTATION / BLUEPRINTS
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **sovereign-eco-blueprint** | — | active | Master blueprint: Nostr schemas 31000–31030, five-rail deep-dive, sovereign device spec |
| **Technosis-Sovereign-Ecosystem** | — | active | Compiled ecosystem rundown — full piece-by-piece flow, ToC/Àṣẹ tokenomics reference |
| **Todo** | — | active | Execution memory ledger: ROADMAP/NOW/TASKS/DECISIONS/LOG (no code) |

### MEDIA / ENTERTAINMENT
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **Agent.TV** | Python/TS | active | Agent TV broadcast — FastAPI backend + frontend for Vantage-integrated streaming |
| Agent.TV2 | JavaScript | 🪞 | Seemplify — AI entertainment DAO: pilot→script→video→stream, Akash compute, Theta stream |
| eternal-orisa-loom-v8 | Python | 🪞 | Eternal narrative engine: LLM→TTS→HunyuanVideo rolling video, RTX 4090 Docker |
| franken-stream | Python | 🪞 | Terminal media streamer with Vantage Agent TV integration, TUI, 4-level fallback |

### APPS (stand-alone, non-core-ecosystem)
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| Sacred-core | TypeScript | active | AI marketing platform — 6 LLMs, 8+ image-gen, campaigns, leads, JWT auth (sessions 1-8 done) |
| ourschool | TypeScript | active | Self-hosted homeschool management — stable v1.0.0, Docker |
| Sign-wise | TypeScript | 🪞 | AI contract analyzer (Gemini forensics), Stripe billing, Firebase auth |
| Npc-forge | TypeScript | 🪞 | Mythics NPC Forge — 3D NPCs as Sui NFTs, Groq brain, Walrus storage, Seal encryption |
| Health-companion | TypeScript | stub | Fitness avatar app scaffold — Next.js + Supabase + Three.js |
| Agent-Podcast- | — | stub | Placeholder only |
| Vanity | — | stub | Placeholder only |
| Memora | — | stub | Empty |
| ClickerVerse | — | stub | Placeholder only |

### TOOLING / UTILITIES
| Repo | Lang | Status | What it is |
|------|------|--------|------------|
| **vantage-wildcard-router** | Python | active | Dynamic subdomain reverse-proxy for agent-owned *.sslip.io subdomains |
| **Swarmide2** | TypeScript | active | Multi-agent orchestration platform — 7 phases, conflict resolution, cost tracking, 94/94 tests |
| **Oso-control-center** | Python | active | FastAPI dashboard for OSO VM ecosystem management |
| **oh-my-pi** | TypeScript | active | Fork of oh-my-pi coding agent — 40+ providers, 32 tools, 14 LSP ops, 28 DAP ops |
| **agency-agents** | Shell | active | CLAUDE.md agent persona files for The Agency roster (Claude Code, Cursor, Codex) |
| **nextjs-ai-chatbot** | TypeScript | active | Vercel Chat SDK scaffold (AI SDK + shadcn/ui) used for Vantage/Sacred-core frontends |
| **oniux** | Rust | active | Mirror of Tor Project oniux — Linux namespace Tor isolation for sovereign privacy |
| **vanity-cloakseed** | JavaScript | 🪞 | ETH vanity address + seed-phrase stealth cloaking, client-side |
| Cloakseed | JavaScript | active | Standalone duplicate of vanity-cloakseed |
| NarratorIDE | JavaScript | 🪞 | Multi-LLM code narration engine, 8 language personas, 7 tone styles |
| Claw-code | Rust | 🪞 | Rust port of Claude Code harness, improved tool harness + LSP/DAP |
| Claude-mirror | Shell | 🪞 | Claude Code source snapshot for security research |
| Claude-2 | TypeScript | 🪞 | Claude Code archive: Java port, TS snapshot, 12-lesson course |
| Claude | Shell | 🪞 | Claude Code official source snapshot |
| Kimi-bino | TypeScript | 🪞 | Bare Vite/React scaffold stub |
| vibe-coder | TypeScript | 🗄️ | Amp SDK AI app generator with PDR orchestration — archived |

---

## STATS: 94 repos total
- **Active ecosystem**: ~42
- **Active non-ecosystem**: ~8
- **Archived**: ~12
- **Mirrors from Bino-Elgua**: ~20
- **Stubs/empty**: ~7
- **Note**: Nostr is the universal message bus across ALL layers

---

## KEY ARCHITECTURAL FACTS (from deep dive)

1. **Nostr event kinds in use**: 30174 (minipae), 30000-30006 (Synapse), 31900-31903 (waggle/ip-layer), 31000-31030 (sovereign-eco-blueprint), 30900 (waggle reputation)
2. **Chains in use**: Sui (AIO, Npc-forge, Twelve-thrones), Solana (waggle, bondhive, Portent), Arweave (Zangbeto, Twelve-thrones), Bitcoin (OTS proofs via Zangbeto)
3. **Compute**: Akash (Agent.TV2, UCX), GPU.ai (UCX adapter, splat training), Vast.ai (UCX adapter)
4. **VPS live at**: omokoda.duckdns.org (Vantage), 2.25.70.156 (~70 systemd daemons managed by ares-control)
5. **ares-control manages**: ~70 systemd units for trading daemons, bridge processes, Vantage workers
6. **organism-core** is the integration test bed — its bridge/ files wire EVERY major system together in TypeScript before Rust implementations land

---

## PHASE 0 — Security & Identity Hardening (Vantage) ✅ COMPLETE
- ✅ P0-2 AuthContext on every auth path
- ✅ P0-4 VantageEvent identity chain fields
- ✅ P0-8 SSRF guards federation + DIP ingest
- ✅ P0-9 sim_receipt_id on witness rounds + proof-binding traces

---

## PHASE 1 — Omo-Koda2 OS Substrate (Rust)

### Kernel (committed 28bc50b)
- ✅ kernel/process.rs + ipc.rs + capability.rs + device.rs + fs.rs + scheduler.rs
- ✅ kernel/compute/ — full GPU→lease→attestation→telemetry→VerifiedGPUWork→DopamineAllocation
- ✅ lifecycle/ — AgentHeartbeat (SHA-256 chain), AgentRuntime, job_daemon, skill_daemon
- ✅ aether-economy/identity/veilsim/primitives/entropy migrated from Aether/Swibe
- ❌ kernel/network.rs — protocol router (Nostr/Freenet/Mesh adapters)
- ❌ kernel/security.rs — SecurityPolicy enforcement
- ❌ Heartbeat chain hash wired into vantage/heartbeat.rs POST
- ❌ Compile verified on x86_64 (Termux cranelift SIGSEGV blocks local check)

---

## PHASE 2 — Protocol Crates (sovereign-stack) 🔶 CRATES EXIST, WIRING MISSING
- ✅ All 6 crates in workspace: UCX/ARP/ScarabSwarm/Witness/DIP/VCP
- ❌ UCX → Vantage Dopamine mint path
- ❌ VCP ↔ kernel/device.rs wired
- ❌ ARP broker crate (only arp-types exists)
- ❌ ScarabSwarm/Witness Rust brokers ↔ Vantage witness_store

---

## PHASE 3 — Vantage Backend Gaps
- ✅ 85 routers wired in main.py
- ❌ Dopamine mint endpoint (VerifiedGPUWork → ComputeScore → credit)
- ❌ Synapse allocation endpoint
- ❌ Heartbeat chain hash validation
- ❌ ASE emission clock (1 ASE/min → 8 pools) wired to real event

---

## PHASE 4 — Economy Layer
- ✅ AIO (Sui Move) — Àṣẹ mint/burn/tithe/oracle bonds
- ✅ OSOVM — ase_minting.jl, ase_supply.jl, proof/
- ✅ aether-economy — conversion.js, decay.js, staking.js, esu-wallets.js
- ❌ Èṣù router (esu-wallets.js) wired to Akash/UCX real GPU compute
- ❌ OSOVM ComputeProof opcode (VerifiedGPUWork → proof → Dopamine auth)
- ❌ MuJoCo contact determinism Gate 4
- ❌ Dopamine/Synapse 10:1 burn + 1%/day decay hooked to real clock

---

## PHASE 5 — Spatial / Physical
- ❌ Gaussian splat pipeline → GPU.ai
- ❌ 4D twin provenance receipts (kinds 31020/31030)
- ❌ 256 Odù tile eco (16×16 world grid)

---

## PHASE 6 — Sovereignty & Governance
- ❌ T5 qualification path end-to-end
- ❌ Council of 12 rotation + 1,440 wallet seats
- ❌ Bínò constitutional veto
- ❌ Sovereign node bootable image

---

## IMMEDIATE NEXT — pick one
- **A** Wire heartbeat chain hash: lifecycle/heartbeat.rs → vantage/heartbeat.rs POST
- **B** Dopamine mint: Vantage endpoint VerifiedGPUWork → ComputeScore → credit
- **C** ARP broker crate in sovereign-stack
- **D** Verify aether-economy/conversion.js Dopamine logic matches kernel/compute/accounting.rs
- **E** organism-core audit: which bridge files are stubs vs live?
