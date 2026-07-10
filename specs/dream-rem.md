# Dream State: Weekly REM Cycle (v1.0)

The Dream Engine runs two rhythms over Living Odu memory:

Consolidation    every 30 min (default)   stale-entry sweep, light housekeeping
REM cycle        weekly (default)         fractal burst analysis + cluster folding

The REM cycle is the deep dream state: the agent shifts from responding to
pruning and reorganizing its knowledge. Implementation:
`omokoda-core/src/dream.rs` (per-agent, applied locally) and
`omokoda-memory/src/rem_fractal.jl` + `POST /dream/rem` (hive-scale planning,
Elixir-orchestrated).

## Fractal Dimension (Mandelbrot burst analysis)

Conversational memory is bursty: dense information clusters separated by
noise, self-similar across scales. The REM cycle measures this with a
box-counting dimension over the entry-creation timeline:

- Divide the timestamp span into 2, 4, 8, 16, 32, 64 boxes.
- Count occupied boxes N(ε) at each scale.
- Dimension = least-squares slope of ln N(ε) vs ln(1/ε), clamped to [0, 1].

Steady activity → ~1.0. Bursty activity → lower. Degenerate timelines
(fewer than two distinct timestamps) → 1.0 (neutral). The dimension is
reported in `RemReport` — a falling value across weeks means the agent's
life is concentrating into bursts and folding is doing real work.

## Compression Rules (scale invariance)

noise line          importance ≤ 0.35 (default `noise_importance`)
fold                ≥ 3 noise entries sharing a path (default
                    `min_fold_cluster`) collapse into ONE macro node:
                    id `rem:<path>:<timestamp>`, tag `rem-fold`, content
                    `[REM fold] N entries on '<path>': <3 previews>`
macro importance    max(folded importances) + 0.1, floored at the noise
                    line so the macro node survives the residual prune
residual prune      unclustered noise below noise_importance / 2 is deleted

This is the fractal zoom: zoomed out, a week of scattered chatter is one
node per topic; the fold summary keeps the zoom-in preview. Nothing above
the noise line is ever touched by REM — high-importance memory is not the
governor's business.

## Cadence & Concurrency

- Default interval 604,800 s (weekly). First run fires on the first dream
  trigger after birth, then gates on the interval.
- Shares the `ConsolidationLock` with ordinary consolidation — a REM pass
  never overlaps another dream.
- Triggered from the background task scheduler's dream hook
  (`tasks/scheduler.rs::poll`): consolidation and REM ride the same trigger,
  each gating on its own cadence.

## RemReport

fractal_dimension   f64   timeline dimension before compression
nodes_before        usize
clusters_folded     usize
nodes_folded        usize  entries absorbed into macro nodes
nodes_pruned        usize  residual noise deleted
timestamp           u64

## Hive-Scale Endpoint (`POST /dream/rem`, Julia :7778)

Request:

    {
      "nodes": [{"id","path","importance","created_at"}, ...],
      "noise_importance": 0.35,     // optional
      "min_fold_cluster": 3         // optional
    }

Response:

    {
      "fractal_dimension": 0.42,
      "folds": [{"path","ids","preview_count"}, ...],
      "prune_ids": [...],
      "nodes_analyzed": N
    }

Pure planning — the endpoint never mutates state. The orchestrator (Elixir
swarm, which already calls this server for `/predict` and `/garden/feed`)
streams node summaries in, applies the returned plan, and writes back. The
per-agent Rust path needs no network hop: `DreamEngine::try_rem_cycle`
computes and applies the same rules locally.

## Invariants

- REM never touches entries above the noise line.
- A fold always nets fewer nodes (N ≥ 3 → 1).
- Macro nodes are tagged `rem-fold` and never fall below the noise line at
  creation, so a fold cannot be erased by its own residual prune.
- The fractal dimension is measured before compression, never after.
- Plans from `/dream/rem` are advisory; only the memory owner applies them.
