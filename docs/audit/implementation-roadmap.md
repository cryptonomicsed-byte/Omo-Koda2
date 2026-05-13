# Implementation Roadmap: Ọmọ Kọ́dà

This roadmap outlines the phased development of the Ọmọ Kọ́dà sovereign Agent OS, leveraging patterns from reference repositories while maintaining the core vision.

## Phase 0: Foundation (Week 2)
**Goal**: Make Ọmọ Kọ́dà functional and secure.
- **Steward Engine**: Implement the core conversation loop (TurnStream) using async generators.
- **Encrypted Session Persistence**: Versioned JSON storage with Argon2id key derivation and ChaCha20Poly1305 encryption for private content.
- **CLI/REPL**: Production-grade CLI with birth/think/act commands and an interactive shell.
- **Identity Hardening**: Replace deterministic soul generation with CSPRNG-based 256 Odu Ifá entropy.

## Phase 1: Wisdom & Reasoning (Weeks 2-3)
- **Neural Router**: 86 cortical parameters derived from Odu seed for model selection and ethics gating.
- **Hermetic Ethics Engine**: 7 principles enforcement via AST visitor and karma tracking.
- **Provider Abstraction**: Fallback chain (Local → Free/Public → Paid → Mock).

## Phase 2: Memory & Persistence (Week 3)
- **Three-Tier Memory**: Working memory (volatile), short-term (persisted/pruned), long-term (encrypted/permanent).
- **RACK Evictor**: Random Approximate Cache Kicking based on relevance, recency, and importance.
- **Context Compression**: 5-level pipeline for long-lived sessions.

## Phase 3: Execution & Sandbox (Weeks 3-4)
- **WASM Security Sandbox**: strict-vm isolation for tool execution.
- **Tool Registry**: Tier-gated tools with JSON Schema validation and receipt emission.
- **MCP Hub**: Auto-discovery of tools from external servers.
- **Parallel Execution**: Execute multiple tools in parallel with various merge strategies.

## Phase 4: Justice & Reputation (Week 4)
- **Merkle Receipt Chain**: Append-only hash chain with Ed25519 signatures and Sui anchoring.
- **Reputation Mining**: Dynamic difficulty formula with diminishing returns and quality gates.
- **Permission Matrix**: 7-mode matrix modulated by reputation tier.

## Phase 5: Flow & Resonance (Week 4)
- **Rhythm Engine**: Sabbath awareness (UTC day 6) and action cooldowns.
- **Cost Tracking**: Synapse/Dopamine budget management.
- **Event Bus**: Lifecycle hooks for plugins and monitoring.

## Phase 6: Creation & Lifecycle (Week 4)
- **Agent Evolution**: Soul state advancement based on persistence and quality.
- **Sub-agent Spawning**: Child agents with inherited Odu seed and reputation-tier inheritance.

## Phase 7: Bridges & Interfaces (Weeks 4-5)
- **IDE Bridge**: JSON-RPC 2.0 bidirectional bridge for editor integration.
- **HTTP/SSE Server**: Axum-based server for remote access and real-time events.

## Phase 8: Sui & Garden (Future)
- **Move Contracts**: On-chain identity (dNFT), reputation scaling, and public receipt anchoring.
- **Garden**: Public receipt publication and tipping.
