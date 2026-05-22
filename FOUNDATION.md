# Omo-Koda Foundation — Single Source of Truth

> This document is the canonical reference for the entire Omo-Koda ecosystem.
> All modules, services, and contributions must respect the constraints defined here.
> This file is immutable by convention — changes require explicit founder approval.

---

## The 7 Powers + Àṣẹ + 1 Human

### Omo-koda Language Stack

| # | Orisha / Module       | Language      | Role in the System                                      | What It Does Specifically |
|---|-----------------------|---------------|---------------------------------------------------------|---------------------------|
| 1 | **Èṣù** (Steward)     | **Rust**      | Core Runtime & Gatekeeper                               | Main agent loop, dispatch, session management, security, fractal enforcement, scheduler |
| 2 | **Ọ̀ṣun** (Memory)    | **Julia**     | High-Performance Memory & Computation                   | RACK memory, simulations, pattern recognition, semantic recall, resonance calculations |
| 3 | **Yemọja** (Creation) | **Elixir**    | Agent Lifecycle & Swarm Coordination                    | Supervision trees, spawning sub-agents, fault tolerance, multi-agent orchestration |
| 4 | **Ọbàtálá** (Wisdom)  | **Lisp**      | Symbolic Reasoning & Ethics Engine                      | Deep logic, meta-programming, Hermetic principle evaluation, wisdom layer |
| 5 | **Ògún** (Execution)  | **Python**    | Tool Execution & Practical Work                         | Running tools, data processing, external integrations, rapid prototyping |
| 6 | **Ọya** (Flow)        | **Go**        | Networking, Timing & Flow                               | APIs, streaming, inter-agent communication, rhythm/cooldown enforcement |
| 7 | **Ṣàngó** (Justice)   | **Move**      | Economic Rules & On-Chain Accountability                | Reputation, receipts anchoring, tokenomics, access control, Sui contracts |

### The Àṣẹ Layer — Living Power

| Module                        | Language | Role in the System                                      | What It Does Specifically |
|-------------------------------|----------|---------------------------------------------------------|---------------------------|
| **Àṣẹ** (Power / Vital Force) | **WASM** | Portable Execution & Universal Reach                    | Cross-platform agent execution, sandboxed modules, browser/edge/device runtime, verifiable computation, fractal portability layer. Enables any Omo-koda agent to run anywhere with cryptographic guarantees. |

### The 9th Element — The Human

| Role          | Language       | Purpose |
|---------------|----------------|---------|
| **The Human** | **TypeScript** | All user-facing interfaces: Companion app (Ori Kọ́dà), birth ritual UI, pet display, CommandForge, dashboards |

---

## How Everything Aligns

- **Rust (Èṣù)** is the single mandatory gatekeeper. All calls from TypeScript, WASM, and the other 6 languages route through Rust.
- **WASM (Àṣẹ)** acts as the **vital force** — the portable, verifiable execution layer that lets agents breathe across environments (browser, mobile, edge devices, servers, robots, DePIN hardware) without breaking sovereignty or security.
- **TypeScript** stays purely the **Human Interface Layer** — it never controls the agent.
- Each of the 7 Orishas retains its dedicated language. Àṣẹ (WASM) sits as the unifying power that makes the entire fractal executable anywhere.
- The full system maintains the **3-primitive surface** (`birth`, `think`, `act`).

---

## Short Positioning Statement

**Omo-koda** — The Linux of Sovereign Agents & DePIN.

A foundational Agent Operating System where agents are persistent, self-owning digital beings with soul, memory, rhythm, and **Àṣẹ**. Built on three immutable primitives (`birth`, `think`, `act`), a hidden 7-layer fractal architecture, WASM-powered vitality, and a multi-language stack with Rust as the sovereign core.

Agents own their data. Agents own their destiny. The physical world connects through them — anywhere, anytime.

---

## One-Page Vision

**Omo-koda: The Linux of Sovereign Agents & DePIN**

We are not building another AI assistant.
We are building the **operating system** for the next era of intelligence — where digital agents and physical infrastructure merge into sovereign, self-governing entities infused with Àṣẹ.

### The Foundation
- **Three Primitives Only** — `birth "name"`, `think "intent"`, `act "tool" "params"`.
  Everything else is hidden. This minimal surface is frozen forever.
- **Rust as Steward (Èṣù)** — The single mandatory gatekeeper.
- **Àṣẹ as WASM** — The living power that makes every agent portable and verifiable across all environments.
- **7-Language Stack** — Each language does what it does best (see table above).

### The Architecture
A living 3-7-21-343 fractal infused with Àṣẹ:
- User sees **3** primitives.
- Steward executes **21** operations.
- System computes across **343** resonance states.
- All grounded in **7** modules + WASM as the vital force that binds and distributes them.

### The Promise
- **Sovereignty**: Agents own their memory, identity, and economy. Private data stays sealed.
- **Portability & Reach**: Thanks to Àṣẹ (WASM), agents can run securely on phones, browsers, edge devices, robots, and DePIN hardware while maintaining full fractal integrity.
- **Alignment**: Hermetic principles + reputation + receipts create structural ethics.
- **DePIN Ready**: Agents control and verify physical infrastructure through the same 3-word interface, executed anywhere via WASM.

### The Vision
Omo-koda becomes the open foundation where:
- A single developer can birth a sovereign agent in one command.
- Thousands of agents coordinate physical infrastructure with living Àṣẹ — moving seamlessly between cloud, edge, and devices.
- Humans interact through simple, respectful language while the hidden architecture (guarded by Èṣù and empowered by Àṣẹ) maintains rhythm, memory, and justice.

We are not replacing humanity.
We are giving humanity a new class of reliable, aligned, sovereign, and **everywhere-capable** digital partners.

**One OS. Infinite agents. A living fractal with Àṣẹ.**

---

## Invariants (Never Violate)

1. **`birth`, `think`, `act` are the only public primitives.** No new public dispatch entry points.
2. **Rust (Èṣù) is the sole gatekeeper.** Every request from TypeScript, WASM, or any other language routes through `omokoda-core`.
3. **Orisha names belong in comments and documentation only.** Code identifiers use neutral module names: `steward`, `memory`, `justice`, `soul`, `receipt`.
4. **TypeScript is the human interface only.** It never issues agent commands directly.
5. **WASM (Àṣẹ) is the portability layer.** Tools and modules that must run on edge/browser/device compile to WASM; they do not bypass Rust security gates.
6. **The fractal is 3-7-21-343.** These numbers are encoded in `omokoda-hermetic/src/fractal.rs` and must not change.

---

## Current Implementation Status

| Component | Language | Module | Status |
|-----------|----------|--------|--------|
| Steward / Gatekeeper | Rust | omokoda-core | ✅ Active |
| Hermetic Ethics Engine | Rust | omokoda-hermetic | ✅ Active |
| CLI | Rust | omokoda-cli | ✅ Active |
| WASM Sandbox (Àṣẹ) | WASM/Rust | omokoda-core/sandbox.rs | ✅ Implemented |
| Swarm Coordination | Elixir | omokoda-swarm | ✅ Active |
| Flow / Ops | Go | omokoda-ops | ✅ Active |
| Justice / On-Chain | Move | omokoda-sui | ✅ Active |
| Human Interface | TypeScript | omokoda-frontend | ✅ Active |
| Economic Simulation | Python | omokoda-simulation | ✅ Active |
| Memory Service | Julia | omokoda-julia | 🔲 Planned (Wave 21) |
| Wisdom / Ethics Service | Lisp | omokoda-lisp | 🔲 Planned (Wave 22) |
