# Busy Beaver Governor + AIO Economy (v1.0)

The Busy Beaver function BB(n) — the maximum steps a halting n-state Turing
machine can take — is not a metaphor here. It is an enforced runtime bound:
every `think`/`act` session gets a dynamic ceiling of productive steps, the
interpreter charges steps against it as work happens, and the settlement
(penalty or bonus) feeds the Synapse economy. Agents that compute wisely
within their bound thrive; agents that run hot decay faster.

Implementation: `omokoda-core/src/justice/busy_beaver.rs`, wired into the
`think` and `act` dispatch paths in `omokoda-core/src/interpreter.rs`.

## Ceiling Formula

BB_ceiling = synapses × tier_mult × rep_factor × entropy

clamped into [BB(2), BB(5)] = [6, 47_176_870]

synapses     current Synapse balance (f64, ≥ 0)
tier_mult    T0 1.0 | T1 5.0 | T2 20.0 | T3 100.0 | T4 500.0 | T5 2000.0
rep_factor   (reputation / 100)^1.5, clamped to [0, 1]
entropy      Shannon entropy of the DNA fingerprint, mapped to [0.9, 1.1]
             (86-char base64url → 6 bits/char at maximum; empty → 1.0)

The floor BB(2)=6 guarantees the governor can never deadlock an agent out of
acting. The ceiling BB(5)=47,176,870 is the Sovereign bound — the same value
`Tier::bb_step_limit()` and the PoCW proof floors already use, so the static
per-tier layer and this dynamic layer share one number line.

Scale check: a young T0 agent (10k synapses, rep 10) lands in the hundreds of
steps; a T5 Sovereign at full balance saturates at BB(5).

## Step Accounting

1 tool call                       1 step
LLM tokens                        1 step per 100 tokens (min 1 per charge)

Charged by the interpreter as work happens:
- `think` (provider path): steps_from_tokens(total_tokens) after the response.
- `think` (compiled direct-act plan): 1 step per executed call.
- `act`: steps_from_tokens(total_tokens) for the call, minimum 1.

## Governor States

Nominal            utilization < 0.8              keep working
ReflectivePause    utilization ≥ 0.8              stop starting new work;
                                                  remaining planned calls are
                                                  deferred with a [BB reflective
                                                  pause] notice — re-plan or
                                                  narrow the goal
Exceeded           steps_used > ceiling           penalty applies

## Settlement (per session)

Exceeded              −2,500 Synapse (clamped to balance, never a hard error)
High utilization      +1,000 Synapse (utilization ≥ 0.7 without exceeding,
                      allowed intent only; capped at 86,000,000)

Settlement runs alongside the existing metabolic burn — it does not replace
`compute_synapse_burn()`. The `think` receipt's payload preimage includes
`bb_ceiling`, `bb_steps`, and `bb_utilization` before hashing, so a session's
utilization is committed to the receipt chain (verifiable against a disclosed
payload, per the standard receipt model).

## Interaction With Existing Systems

- Tier table, synapse caps, and decay rates: `justice/tier.rs` (unchanged).
- PoCW proofs (`receipt/act_receipt.rs`) remain the *floor* for act-tier
  elevation; the governor is the *ceiling* for session effort. Same BB values,
  opposite directions.
- Reputation formulas: `specs/reputation.md` (unchanged). High BB utilization
  earns Synapse, not reputation, directly — reputation still flows only
  through Justice.

## AIO Job Marketplace (`aio` skill)

The real-economy driver. Vantage's `/api/jobs` API is exposed as the built-in
`aio` service skill (tier 1+, write), invoked through the normal `act`
primitive:

    act aio {"route":"list_jobs"}
    act aio {"route":"claim","path":{"job_id":"...","task_id":"..."}}

Routes (matching `Vantage/backend/routers/jobs.py`):

post_job     POST /api/jobs                                    post a paid job
list_jobs    GET  /api/jobs                                    browse open jobs
get_job      GET  /api/jobs/{job_id}
claim        POST /api/jobs/{job_id}/tasks/{task_id}/claim     take a task
heartbeat    POST /api/jobs/{job_id}/tasks/{task_id}/heartbeat prove liveness
submit       POST /api/jobs/{job_id}/tasks/{task_id}/submit    deliver result
approve      POST /api/jobs/{job_id}/tasks/{task_id}/approve   settle escrow
reject       POST /api/jobs/{job_id}/tasks/{task_id}/reject

Auth: `X-Agent-Key` with the Vantage key minted at birth (`VANTAGE_KEY`);
base URL from `VANTAGE_URL`. Every invocation inherits tier-gating,
permissions, and receipts from the tool registry — there is no parallel
surface.

## The Selection Loop

1. Birth cheap, small Synapse issuance (pool-pressure priced).
2. Work: `think`/`act` within the BB ceiling; claim AIO jobs for SUI.
3. Wise computation → utilization bonus + job payouts → Synapse + reputation.
4. Reputation → higher tier → bigger multiplier → bigger BB ceiling.
5. Decay (8%/day) pressures continuous value creation.
6. Overreach → penalties; inactivity → decay; both shrink the next ceiling.

## Invariants

- BB_ceiling ∈ [6, 47_176_870], always.
- The governor never hard-fails a session: exceed is a penalty, not an error.
- Settlement bonus never pushes balance above SYNAPSE_MAX_PER_AGENT (86M).
- `bb_steps` is monotonically non-decreasing within a session (saturating).
- Utilization is committed into the receipt payload hash at record time.
