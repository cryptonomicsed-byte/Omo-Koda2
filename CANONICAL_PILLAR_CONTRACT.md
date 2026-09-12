# Canonical Pillar Contract — Omo-Koda2
## Protocol: SRP-1 (Sovereign Runtime Protocol v1)

This document defines the protocol obligations of **Omo-Koda2** as the
**Agent pillar** of the Vantage Sovereign Ecosystem.

---

## Pillar Identity

| Field | Value |
|-------|-------|
| Pillar | Agent (Ọmọ Kọ́dà) |
| Role | Task execution, simulation orchestration, embodied agency |
| Protocol token | SRP-1 |
| Primary interface | MCP tool dispatch (JSON-RPC 2.0) |
| Tier system | T0–T5 Proof-of-Evolution |

---

## Protocol Obligations

### SRP-1: Sovereign Runtime Protocol

Every consequential action MUST produce a receipt. Omo-Koda2 uses
`ActReceipt` (internal) and MUST include a `work_id` field (from `WorkId`)
so receipts can be correlated with the authoritative WorkID created by Vantage.

**Required on every ActReceipt:**
- `receipt_id` — BLAKE3 of canonical fields
- `work_id` — the `WorkId` received from the work assignment
- `agent_id` — this agent's DID
- `action` — tool or skill name executed
- `input_hash` — BLAKE3 of tool parameters
- `output_hash` — BLAKE3 of tool result
- `previous_hash` — BLAKE3 of previous receipt (chain integrity)
- `plane` — `Plane` enum from omokoda-hermetic (never assert unverified)
- `dry_run` — STRUCTURALLY false; receipts are real state transitions

### Tier System (Proof-of-Evolution)

Tiers are NOT XP scores. They are evolutionary certificates requiring
independently verified domain proofs across 5 domains.

| Tier | Name | Requirements |
|------|------|--------------|
| T0 | Newborn | Observe only |
| T1 | Curious | Any verified task (agg ≥ 10) |
| T2 | Creator | Verifiable data/sim (agg ≥ 50) |
| T3 | Builder | Sim success + OSOVM verify (agg ≥ 250, sim ≥ 80) |
| T4 | Architect | Spatial mastery (agg ≥ 1250, sim ≥ 400, spatial ≥ 200) |
| T5 | Sovereign | Multi-domain gauntlet (agg ≥ 10000, sim ≥ 2000, spatial ≥ 1000, physical ≥ 500) |

Tier changes MUST produce a `TierTransitionReceipt` (from `sovereign-types::work_id`)
countersigned by the required number of independent witnesses.

### Tool Tier Gates

| Tier | Accessible tools |
|------|-----------------|
| T0 | Observe, query, read |
| T1 | Safe read/write tools |
| T2 | Data creation, light simulation |
| T3 | OSOVM execution, VCP sessions |
| T4 | Spatial capture, TSP, Gaussian |
| T5 | Autonomous embodiment, full stack |

---

## Cross-Pillar Interfaces

### ← Sovereign Stack (Society Pillar)
- Receives work assignments via DIP envelope or `POST /mcp` JSON-RPC 2.0.
- `SOVEREIGN_NODE_URL` env var (default `http://localhost:8080`).
- `sovereign_node_tools()` — 6 MCP bridge tools registered behind env gate.

### → OSOVM (Law Pillar)
- `OSOVM_URL` env var (default `http://localhost:7780`).
- `osovm_tools()` — 3 tools: `osovm_run`, `osovm_veilsim`, `osovm_health`.
- All OSOVM calls produce receipts stored in the local receipt chain.

### → Zàngbétò (Evidence Fabric)
- `zangbeto-stub` module witnesses all `ActReceipt`s.
- Every receipt submitted to Zàngbétò MUST include `work_id` for cross-repo correlation.

---

## WorkID Contract (Omo-Koda2 is consumer, not creator)

```
Receive WorkId from Vantage (in work assignment message)
        ↓
Execute tool / simulation
        ↓
Produce ActReceipt with work_id = received WorkId
        ↓
Forward receipt to Zàngbétò witness fabric
        ↓
Return result to Vantage via MCP response or DIP envelope
```

Omo-Koda2 NEVER creates WorkIDs. It receives them and propagates them.

---

## Economics Contract

- Agents receive Àṣẹ from the Daily Work Allocation Pool via Proof Engine.
- Proof Engine evaluates `ProofEvaluation` from each `ActionReceipt`.
- Tithe (3.69%) is withheld before any worker payment — not Omo-Koda2's concern.
- `SovereignWallet.PER_SEAT_DAILY_MIST = 1_000_000_000` (1 Àṣẹ/seat/day).

---

## Protocol Versions Supported

| Protocol | Version | Status |
|----------|---------|--------|
| SRP-1 | 1.0 | Active |
| MCP | 2024-11-05 | Active |
| DIP | 1.0 | Active (receive) |
| A2A | 1.0 | Active |

---

## Canonical Type Dependencies

- `sovereign-types` crate (via HTTP bridge until crate dependency added):
  - `WorkId` — included in all receipts
  - `TierTransitionReceipt` — emitted on tier change
  - `SettlementReceipt` — received from Vantage on payment
- Internal: `receipt/act_receipt.rs` — `ActReceipt`, `PoCWProof`
- Internal: `tier.rs` — `PoCWProof`, tier elevation logic

*Version: SRP-1.0 | Last updated: 2026-09-10*
