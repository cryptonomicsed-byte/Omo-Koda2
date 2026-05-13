# ADR 3: Private-Provider Hard Fail

## Status
Accepted

## Context
Privacy is a core pillar of Ọmọ Kọ́dà. The `/private` flag ensures that data never leaves the local environment.

## Decision
When the `/private` flag is used (or implied by default in `think`), the runtime must enforce the use of local providers ONLY (e.g., Ollama, WebLLM). If a local provider is unavailable or times out, the system must return a HARD FAIL. There must be no silent fallback to external providers.

## Consequences
- Guarantees data sovereignty for private operations.
- Prevents accidental data leakage to external APIs.
- Forces a clear distinction between public and private execution paths.
