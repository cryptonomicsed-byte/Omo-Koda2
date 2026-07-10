# Fractal SOMA Memory — Tier Map (v1.0)

The memory system of a sovereign agent must hold volatile reasoning, private
sealed thought, queryable long-term structure, on-chain truth, and a
compressed archive — and let the agent zoom between scales without loss.
This spec maps those tiers onto the components that exist in this repo and
its sibling services. It is a topology document: each tier names its code.

## Tier 0 — Volatile Working Memory

Where     omokoda-core interpreter session (Rust) + WASM sandbox for tools
Bound     Busy Beaver governor: `justice/busy_beaver.rs` — steps per
          session from synapses × tier × reputation × DNA entropy
          (`specs/busy-beaver.md`)
Life      one `think`/`act` dispatch; reflective pause saves state before
          the ceiling

## Tier 1 — Session / Private Memory

Where     `interpreter.rs` MemoryEntry (encrypted via Living Odu key,
          rotated per act), `memory_vault/`, private thoughts gated to
          local providers only (webllm, ollama, larql)
Guarantee private content never reaches an External-class provider; the
          gate is name-based in the think arm and class+localhost-based in
          `providers.rs::is_allowed_in_private`

## Tier 2 — Graph-Native Long-Term

Where     `memory/memdir.rs` (Living Odu Directory: importance decay,
          paths, swarm shares), `memory/dag.rs` (causal DAG), SOMA
          (`memory/soma.rs`, `omokoda-julia/src/soma/`)
Query     LARQL (`larql` skill — LQL select, describe, relations over a
          model-as-database) and Zerolang (`zero` tool — zero.graph
          query/patch for program-shaped memory)
Retrieval unfolds fractal folds on demand (see Tier 4)

## Tier 3 — On-Chain Anchors

Where     receipt chain (`receipt/` — Blake3-chained, Ed25519-signed,
          Merkle roots), Sui contracts (`omokoda-on-chain/sources/`:
          soul.move, agent.move, synapse.move, hive.move)
What      Merkle roots, reputation snapshots, high-value act receipts.
          Big blobs (dense sub-graphs, media) belong in Walrus with the
          blob hash anchored on Sui — anchor the hash, not the data.

## Tier 4 — Fractal Compressed Archive (the Dream)

Where     `dream.rs` REM cycle (per-agent, Sabbath-gated — see
          `specs/dream-rem.md`) and `omokoda-memory/src/rem_fractal.jl`
          + `POST /dream/rem` (hive-scale planning, Elixir-orchestrated)
Math      box-counting fractal dimension over the activity timeline
          separates bursty signal from noise (Mandelbrot); noise clusters
          fold per-path into macro nodes
Zoom      folds are lossless. `OduDirectory::archived_folds` keeps every
          folded micro entry keyed by its macro node;
          `OduDirectory::unfold(macro_id)` restores the sub-graph and
          removes the macro node. Restored entries keep their importance,
          so unresolved noise re-folds next Sabbath. Only the residual
          prune (below noise_importance / 2, unclustered) deletes.

## The Zoom Invariant

At every scale the graph answers the same questions with less detail:

    macro node  "[REM fold] 14 entries on 'topics/trading' (unfold ... )"
        │  unfold
        ▼
    14 micro entries, original content, original importance

- fold(unfold(x)) converges: unfold then re-fold yields an equivalent
  macro node (same path, same members).
- Deletion happens in exactly one place (residual prune) and is the only
  lossy operation in the memory system.
- A macro node without an archived fold is invalid; `unfold` returning
  `None` on a second call is the correct no-op signal.

## Orchestration Split

Rust      safety + settlement: fold/unfold/prune applied only by the
          memory owner; Merkle proofs; TEE sealing hooks
Julia     analysis only: fractal dimension, cluster planning — pure
          functions, never mutates state
Elixir    flow only: triggers hive REM on the Sabbath cron, streams node
          summaries to Julia, applies returned plans, writes back
Move/Sui  truth: anchors for what must outlive the process

## Non-Goals

- No embedding store here: semantic similarity lives in LARQL/SOMA rack
  memory, not in the fold archive.
- The fold archive is not a backup system; it holds noise-tier entries
  only. High-importance memory is never folded.
- No cross-agent folds: swarm-shared entries are excluded from REM.
