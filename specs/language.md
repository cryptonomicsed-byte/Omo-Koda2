# Language Spec

## Goal
Keep the public surface to exactly three primitives:
- `birth`
- `think`
- `act`

Everything else is hidden behind stdlibs, policy, or internal runtime calls.

## Grammar (frozen)
```ebnf
program         ::= statement*
statement       ::= birth_stmt | think_stmt | act_stmt | slash_cmd | text_fallback
birth_stmt      ::= "birth" QUOTED_STR (metadata_pair)*
metadata_pair   ::= WORD ":" (WORD | NUMBER)
think_stmt      ::= "think" QUOTED_STR think_modifier*
think_modifier  ::= "/private" | "/publish" | "/sandbox"
                  | "loop:" BOOLEAN
                  | "max_iterations:" NUMBER
                  | "priority:" WORD
act_stmt        ::= "act" QUOTED_STR QUOTED_STR ("/sandbox")?
slash_cmd       ::= "/" WORD (WORD | ADDR)?
text_fallback   ::= NON_EMPTY_TEXT
QUOTED_STR      ::= '"' [^"\n]* '"'
MAX_INPUT       ::= 4096 bytes
```

## Semantics
- `birth` creates an agent and initializes hidden identity, memory, policy, and receipts.
- `think` performs reasoning, Natural Think intent compilation, memory updates, and receipt generation. By default it is private to the agent runtime.
- Natural Think accepts rich single-sentence or multi-paragraph goals in the quoted prompt. The public primitive remains only `think`; rich behavior is compiled internally by the Steward.
- Optional `think` modifiers are hints to the internal compiler, not new public primitives:
  - `/private` / `/publish`: privacy mode selection. Private mode stays local-provider-only and sealed-memory-aware.
  - `/sandbox`: request sandboxed execution for any safe direct `act` calls produced by the compiler.
  - `loop:true`: allow iterative planning inside the tier budget.
  - `max_iterations:N`: request a bounded iteration budget. The Steward clamps it by tier and the global per-turn cap.
  - `priority:high|normal|low`: scheduling/cost hint validated by reputation and Justice.
- `act` executes a tool or capability. By default it is public and receipt-bearing.
- `text_fallback` is treated as `think`.

## Natural Think internal compilation
For every `think`, Rust/Èṣù (the Steward) is the mandatory gatekeeper and runs `compile_intent` before provider or tool execution. The compiler derives an 86-parameter neural router from the Odu soul seed and uses Hermetic state, reputation tier, privacy mode, and Justice hooks to produce an internal IR:

- intent class: `simple_query`, `complex_task`, `creative`, or `monitoring`
- step-by-step plan
- optional tool sequence
- optional safe direct `act` calls
- optional sub-agent birth suggestion
- validation report: allowed, warnings, confirmation requirements, tier/cost/privacy gates

High-risk intents, especially fund movement or long-running monitoring, compile to plans that require explicit confirmation rather than executing automatically. Sub-agent suggestions remain internal recommendations behind `think`; users still only type `birth`, `think`, or `act`.

## Rejection rules
- Any surface syntax not reducible to `birth`, `think`, or `act` is rejected.
- The parser MUST reject any input containing stdlib names, module names, or internal identifiers.
- No user-visible `stdlib.*` calls.
- No user-visible internal module names.
- No direct access to memory keys, receipt internals, or hidden policy objects.

## Guaranteed outputs
- `birth` returns agent identity and initial state summary.
- `think` returns reasoning output, compiled-plan output, tool-chain output, or a confirmation request depending on the internal plan. Every successful `think` emits a receipt whose payload is an audit hash/summary and never raw private prompt text.
- `act` returns action result plus receipt metadata when public.
- Inputs longer than `MAX_INPUT` are rejected.
- `birth`, `think`, and `act` remain the only public language primitives.

## Notes
This file is the frozen contract for parser behavior. No parser/runtime implementation should proceed until these rules are satisfied by tests.
