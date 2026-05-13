# Tool Manifest Specification (v1.0 — FROZEN)

## Purpose
Defines the structure, requirements, and discovery of tools within the Ọmọ Kọ́dà ecosystem.

## Tool Structure
Every tool must provide a manifest with the following fields:

```json
{
  "id": "string",
  "description": "Natural language description for LLM reasoning",
  "parameters": {
    "type": "object",
    "properties": { ... },
    "required": [ ... ]
  },
  "required_tier": "integer (0-5)",
  "permission_mode": "string (Auto|Ask|Plan|Monitor|Quarantine|Simulate|Refuse)",
  "sandbox_requirement": "boolean",
  "cost_multiplier": "float"
}
```

## Tier Gating
Tools are unlocked based on the agent's **Reputation Tier**:

| Tier | Tool Category | Examples |
| :--- | :--- | :--- |
| 0 | ReadOnly | `read_file`, `grep`, `web_search` |
| 1 | WorkspaceWrite | `write_file`, `note_take` |
| 2 | AdvancedMedia | `image_gen_basic` |
| 3 | Execution | `code_runner`, `bash_sandboxed` |
| 4 | Orchestration | `agent_spawn`, `browser_automation` |
| 5 | Sovereign | `self_modification`, `multi_agent_fabric` |

## Discovery
- The `ToolRegistry` manages available tools.
- Tools are filtered based on the current agent's tier before being presented to the reasoning loop.
- `/sandbox` mode may restrict the tool set to a safe subset.

## Execution Flow
1. **Validation**: Check parameters against JSON Schema.
2. **Permission**: Verify agent's tier and request approval if mode is `Ask`.
3. **Execution**: Run in WASM or Linux namespace sandbox.
4. **Receipt**: Generate signed ActReceipt with results and cost.
