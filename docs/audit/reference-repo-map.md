# Reference Repository Map: Portability Analysis

This document maps architectural patterns and components from the omo-koda ecosystem and other reference repositories that can be ported or adapted for Ọmọ Kọ́dà.

## 1. Claw-code (Rust)
Claw-code is a mature, production-grade agent runtime. Since it is also written in Rust, many components can be directly ported or used as high-fidelity references.

### High-Fidelity Ports
- **Session Persistence System**: Replace `MemoryEntry` with structured `ConversationMessage` and `ContentBlock`. Add versioned JSON persistence.
- **Permission System**: Map `PermissionMode` to Ọmọ Kọ́dà reputation tiers (0-5). Implement tiered authorization.
- **Config System**: Agent-centric configuration loading (e.g., `~/.omokoda/agents/{id}/settings.json`).
- **File Operations**: Tier-gated `read_file`, `write_file`, `edit_file`, `glob`, and `grep`.
- **Bash Execution with Sandbox**: Linux `unshare` namespace isolation for sandboxed tool execution.
- **Hook System**: Pre/post tool use hooks for the Justice module (reputation scoring, tier assignment).

### Moderate Adaptation
- **ConversationRuntime**: The core loop for the Steward, generic over provider and tool executor.
- **API Provider Abstraction**: Implement `LocalProvider` (Ollama, WebLLM) for `/private` and `ExternalProvider` (Claude, OpenAI) for public use.
- **Usage Tracking**: Map token usage to Synapse/Dopamine cost.

## 2. Claude-2 (TypeScript/Claude Code)
Claude-2 represents a sophisticated production agent harness. Ports are conceptual translations from TypeScript to Rust.

### Key Architectural Patterns
- **Async Generator Agent Loop**: Unifies streaming, termination, and errors into a single flow.
- **5-Level Context Compression**: Content replacement → Snip → Microcompact → Collapse → Autocompact.
- **7-Layer Safety Stack**: Tool pre-filtering → Deny-first rules → Permission modes → Auto-mode classifier → Shell sandboxing → No permission restore on resume → Hook interception.
- **Process-Based Sub-agents**: Independent subprocesses with restricted tool sets for isolation.

## 3. Swibe (Agent-native Scripting)
Swibe provides the conceptual reference for the `birth`/`think`/`act` surface language.
- **Neural Router**: 86 cortical parameters derived from seed.
- **Three-Tier Memory**: Working, short-term, and long-term (encrypted) hierarchy.
- **Hermetic Ethics Engine**: 7 principles with AST visitor enforcement.

## 4. Conflict Resolution & Alignment
| Conflict | Reference Repos | Ọmọ Kọ́dà Alignment |
| :--- | :--- | :--- |
| Identity | BIPỌ̀N39 (16x16) | **Keep 256 Odu Ifá**. |
| Tokenomics | Àṣẹ/Dopamine/Synapse | **SUI-only** human-facing; Dopamine/Synapse internal. |
| Language | 35+ keywords | **3 words forever** (`birth`, `think`, `act`). |
| Code Source | Claude-2 mirrored source | **Patterns only**. Write fresh Rust designs. |
