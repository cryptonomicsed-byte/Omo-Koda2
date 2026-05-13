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
think_stmt      ::= "think" QUOTED_STR ("/private" | "/publish")?
act_stmt        ::= "act" QUOTED_STR QUOTED_STR ("/sandbox")?
slash_cmd       ::= "/" WORD (WORD | ADDR)?
text_fallback   ::= NON_EMPTY_TEXT
QUOTED_STR      ::= '"' [^"\n]* '"'
MAX_INPUT       ::= 4096 bytes
```

## Semantics
- `birth` creates an agent and initializes hidden identity, memory, policy, and receipts.
- `think` performs reasoning and memory updates. By default it is private to the agent runtime.
- `act` executes a tool or capability. By default it is public and receipt-bearing.
- `text_fallback` is treated as `think`.

## Rejection rules
- Any surface syntax not reducible to `birth`, `think`, or `act` is rejected.
- The parser MUST reject any input containing stdlib names, module names, or internal identifiers.
- No user-visible `stdlib.*` calls.
- No user-visible internal module names.
- No direct access to memory keys, receipt internals, or hidden policy objects.

## Guaranteed outputs
- `birth` returns agent identity and initial state summary.
- `think` returns reasoning output and may store private/public memory depending on flags.
- `act` returns action result plus receipt metadata when public.
- Inputs longer than `MAX_INPUT` are rejected.
- `birth`, `think`, and `act` remain the only public language primitives.

## Notes
This file is the frozen contract for parser behavior. No parser/runtime implementation should proceed until these rules are satisfied by tests.