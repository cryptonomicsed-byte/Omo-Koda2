# Provider Routing Specification (v1.0 — FROZEN)

## Purpose
Defines the fallback and routing logic for LLM providers, enforcing privacy boundaries and managing costs.

## Routing Logic
The Steward uses a prioritized fallback chain to select a provider based on the current visibility mode.

### 1. Visibility: /private
- **Allowed Providers**: Local only (Ollama, WebLLM, User-Registered Local).
- **Fallback Chain**: `local_default` → `local_secondary` → `fail`.
- **Failure Mode**: HARD FAIL on any local failure or timeout. NEVER fallback to external.

### 2. Visibility: /public (or /publish)
- **Allowed Providers**: Any (Local, External).
- **Fallback Chain**: `local` → `external_free` → `external_paid` → `mock` (testing only).
- **Incentive**: Prefer local providers to minimize Synapse/Dopamine burn.

## Provider Metadata
Every provider must expose:
- `id`: Unique identifier (e.g., `ollama:llama3`).
- `tier`: 0 (local) to 5 (enterprise).
- `privacy_class`: `Local` or `External`.
- `cost_per_token`: synapse burn rate.

## Budget Checks
- Before routing, the Steward must verify sufficient **Synapse** balance in the agent's account.
- For hive-wide operations, the **Dopamine** pool must be checked.

## Audit Trails
- Every routing decision is recorded in the session metadata.
- Fallback events from local to external (in public mode) must be signed and receipted.
