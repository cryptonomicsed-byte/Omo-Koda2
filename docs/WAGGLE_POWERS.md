# The Powers on the Field — Ọya is the Substrate

Connection Map v2 §6, formalized. Every Omo-Koda2 power touches the Waggle
stigmergic field through its own temperament; this page is the canonical map
of who does what, and names the identity that was always implicit:

## Ọya *is* `waggled`

Ọya — wind, sudden change, the sweeper of dead things — is not a client of
the substrate. She **is** the substrate: the decay kernels that fade stale
knowledge, the evaporation sweep that clears the dead, the diffusion bleed
that carries scent to neighbors, the event stream that is the wind itself.
`waggled` (the Go daemon in the Agentic repo) is her implementation.

Ọya-as-rhythm is literal now: **territories** (`POST /v1/territories`)
expose the decay tempo per region of the URI tree. Fast-moving LOOM trading
territory runs `tempo < 1` (scent clears quickly); Ọbàtálá's ethics
territory runs `tempo > 1` (judgments linger). Claim-velocity evaporation
stacks on top — contested ground decays up to twice as fast. Different
regions, different rhythms, one daemon.

## The other powers

| Power | Code | Field role |
|---|---|---|
| Èṣù | `omokoda-core/src/waggle` | Gatekeeper of the verbs: capability tokens on claim/mark/release/dance (`CapabilityGate`), per-agent mark throttling (`MarkThrottle`) so no looping agent out-writes evaporation. |
| Ọbàtálá | `omokoda-core/src/waggle::taboo_from_halt` | A 7-gate HALT becomes a slow-decay `taboo` deposit whose meta carries the failing Hermetic gate and reason — `sniff_explain` answers *why* a territory is excluded. The taboo channel cross-inhibits gold (floor 0.1): ethical gating as field physics. |
| Ògún | `omokoda-core/src/waggle::tool_outcome` | Tool executions auto-signal through a watch: success → gold, failure → dead-end, at watch-derived trust. The whole tool surface is stigmergic without per-tool instrumentation. |
| Yemọja | `omokoda-elixir/lib/territory_supervisor.ex` | Supervision tree mirrors scent-territory topology: one DynamicSupervisor per territory, crash blast radius follows field boundaries. Spawn budgets follow the trust-weighted gradient, damped by the territory's `bounded` stability (§8.3: never add agents to a destabilizing region). |
| Ọ̀ṣun | `omokoda-julia/src/resonance_consolidation.jl` | Reads the journal (recall), scores resources by *persistence* of gold/bounded scent across time, and consolidates resonant patterns into durable memory — the field forgets on purpose, Ọ̀ṣun remembers on purpose. |
| Ṣàngó | Move contracts + relay | On-chain finalization deposits at the top evidence tier (`on-chain-anchored`); significant bounded verdicts are anchorable too; reputation decay reuses Waggle's own kernels — one decay physics for trust and scent alike. |

## The trust ladder

All powers read one ladder instead of inventing their own trust math:

```
self-report (0.2) < corroborated (0.4) < watch-derived (0.6)
                  < zangbeto-verified (0.8) < on-chain-anchored (1.0)
```

Promotion is always a new deposit at a higher tier — history is append-only,
Zangbeto receipts and Sui anchors climb the ladder, nothing rewrites the past.
