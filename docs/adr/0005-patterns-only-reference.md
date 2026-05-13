# ADR 5: Patterns-Only Reference Material

## Status
Accepted

## Context
The project draws inspiration from several high-quality repositories (Claude Code, Swibe, Claw-code).

## Decision
All imported repository material must be treated as concepts and patterns only. No raw source code from Claude Code or other proprietary/semi-proprietary sources shall be copied directly into Ọmọ Kọ́dà. First-party crates (like `bipon39`) may be added as dependencies if they are clean and intentionally integrated.

## Consequences
- Maintains legal integrity and sovereignty of the codebase.
- Ensures that the Rust core is built with fresh, purpose-driven designs.
- Prevents technical debt from external, potentially incompatible architectures.
