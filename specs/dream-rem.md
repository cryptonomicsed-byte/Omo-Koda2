# Dream State: Weekly REM Cycle (v1.0)

The Dream Engine runs two rhythms over Living Odu memory:

Consolidation    every 30 min (default)     stale-entry sweep, light housekeeping
REM cycle        on the Sabbath (UTC Sat)   fractal burst analysis + cluster folding

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

Folds are **lossless**: the micro entries move into the directory's fold
archive (`OduDirectory::archived_folds`, keyed by macro id) and
`OduDirectory::unfold(macro_id)` restores the full sub-graph on demand,
removing the macro node. Restored entries keep their importance — still
noise, they re-fold at the next REM. Only the *residual prune* deletes.

This is the fractal zoom: zoomed out, a week of scattered chatter is one
node per topic; zoomed in, the original entries. Nothing above the noise
line is ever touched by REM — high-importance memory is not the governor's
business.

## Cadence & Concurrency — the Sabbath

- The REM cycle falls on the **Sabbath**: UTC Saturday, the same day
  `RhythmGate::is_sabbath()` observes. The rhythm gate queues irreversible
  *outward* action for the Sabbath; the dream engine turns *inward* and
  dreams. At most one REM pass per Sabbath (gated on UTC day, not wall
  interval).
- **Overdue catch-up**: if more than `overdue_after_secs` (default
  1,209,600 s — two missed Sabbaths) passes since the last REM, the cycle
  runs at the next dream trigger regardless of weekday, so a slept-through
  Sabbath never becomes unbounded drift.
- A newborn's first REM waits for its first Sabbath.
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
