# ADR 1: Three Primitives Only

## Status
Accepted

## Context
Ọmọ Kọ́dà aims to be a sovereign Agent OS with a minimal, unchangeable public interface to ensure long-term stability and ease of integration.

## Decision
The public language of Ọmọ Kọ́dà is forever limited to exactly three primitives:
- `birth "name"`: Agent instantiation.
- `think "intent"`: Cognitive processing.
- `act "tool" "params"`: World interaction.

All rich capabilities and internal modules must be hidden behind these three primitives.

## Consequences
- Simplifies user interaction and API surface.
- Prevents feature creep in the public interface.
- Requires all internal complexity to be managed by the Steward module.
