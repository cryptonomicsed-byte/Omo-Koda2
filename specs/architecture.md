# Agent OS Architecture Spec

## Goal
Define a spec-first Agent OS with a tiny public surface and a rich hidden runtime.

## Layers
1. **Surface language**: `birth`, `think`, `act`
2. **Interpreter / Steward (Rust/Èṣù)**: parses and routes every surface command; nothing bypasses it
3. **Natural Think compiler**: internal `compile_intent` stage for rich natural-language goals behind `think`
4. **Policy + Hermetic engine**: privacy, permissions, hidden rules, ethical validation
5. **Memory + receipts**: persistence and auditability
6. **Stdlib / hidden modules**: identity, tools, swarm, reputation, economy
7. **Frontends**: terminal, web, mobile, dashboard

## Principles
- Surface simplicity first.
- Hidden complexity allowed only behind internal modules.
- Every new capability should be expressed as a runtime module, not a new primitive.
- The public language must remain stable even as internals evolve.
- Rust/Èṣù Steward is the mandatory gatekeeper for `birth`, `think`, and `act`.
- Private data stays sealed: `/private` `think` uses local providers only, private direct tools cannot route through external-capable tools, and receipts never expose raw private prompt text.
- Every successful output path emits a receipt: direct `act`, compiled `think`, tool-chain `think`, and confirmation/refusal `think` all leave an auditable hash-chain entry.

## Natural Think flow
Natural Think extends only the internal runtime. The user still enters `think "natural language goal"` plus optional modifiers. Steward compiles the prompt before any provider/tool call:

```diagram
╭────────────╮     ╭────────────────╮     ╭────────────────────╮
│ User think │────▶│ Steward / Èṣù   │────▶│ compile_intent IR   │
╰────────────╯     ╰───────┬────────╯     ╰─────────┬──────────╯
                           │                        │
                           ▼                        ▼
                   ╭──────────────╮        ╭────────────────────╮
                   │ Justice +    │        │ 86-param Odu router │
                   │ Hermetic +   │        │ + tier/cost/privacy │
                   │ privacy gate │        ╰─────────┬──────────╯
                   ╰──────┬───────╯                  │
                          ▼                          ▼
       ╭────────────╮ ╭────────────╮ ╭────────────╮ ╭────────────╮
       │ LLM answer │ │ Tool chain │ │ Sub-agent  │ │ Confirm /  │
       │            │ │ direct act │ │ suggestion │ │ refuse     │
       ╰─────┬──────╯ ╰─────┬──────╯ ╰─────┬──────╯ ╰─────┬──────╯
             ╰──────────────┴──────────────┴──────────────╯
                                    ▼
                            ╭────────────╮
                            │ Receipt    │
                            │ generated  │
                            ╰────────────╯
```

The compiler classifies each intent as `simple_query`, `complex_task`, `creative`, or `monitoring`, then emits a structured plan containing steps, tool sequence, safe direct `act` calls, optional sub-agent birth suggestion, and validation outcomes. High-risk actions such as fund movement require explicit confirmation and never execute automatically.

## Turn events
The runtime stream may emit these Natural Think events between `started` and `finished`:

- `intent_compiled`: full internal compilation summary
- `plan_generated`: plan steps, iteration budget, priority, sandbox mode
- `sub_agent_suggested`: suggested isolated agent purpose and required tier

Existing receipt, warning, error, and tool-detection events continue to apply.

## Build order
1. Freeze language, privacy, memory, and receipt specs.
2. Align runtime data structures to the spec.
3. Split hidden internals into clean modules.
4. Add tests for parsing, Natural Think compilation, privacy, memory routing, Justice validation, tier gating, and receipts.
5. Add frontends only after the runtime contract is stable.
