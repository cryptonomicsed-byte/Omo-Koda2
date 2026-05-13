# Conscious Exclusions: What Ọmọ Kọ́dà is Not

This document lists architectural patterns, features, and components from reference repositories that have been intentionally excluded from Ọmọ Kọ́dà.

## 1. Language & Interface Exclusions
- **Swibe's 35+ Public Keywords**: The public language of Ọmọ Kọ́dà is frozen to exactly three primitives: `birth`, `think`, and `act`. All rich capabilities must stay internal.
- **Claude-2's React + Ink TUI**: Ọmọ Kọ́dà focuses on a backend-first architecture with a Next.js 15 PWA frontend, rather than a terminal-based UI.
- **VSCode/IDE Extensions**: While a bridge is planned, direct IDE extensions are not part of the core sovereign OS mission.

## 2. Tokenomics & Economy Exclusions
- **Àṣẹ Token**: Following the "No Àṣẹ token" rule, human-facing payment is SUI only. Dopamine and Synapse are strictly internal/metabolic resources.
- **Swibe's Three-Token Burn Economy**: Avoided in favor of a simpler SUI-based model with internal resource decay.

## 3. Infrastructure Exclusions
- **Swibe's 44 Compilation Targets**: Ọmọ Kọ́dà is Rust-native. While it uses WASM for sandboxing, it does not aim for universal transpilation.
- **Cloudflare Registry**: Sovereign deployment is preferred over centralized cloud registries.
- **Docker/Lambda Generation**: Out of scope for the core OS.

## 4. Implementation Exclusions
- **Claude-2 Raw Source Code**: Never import code directly from mirrored archives due to legal and sovereign design principles. Extract patterns only.
- **Claw-code Python Port**: Rust remains the mandatory systems language for the core Steward runtime.
- **BIPỌ̀N39 Identity**: Odu Ifá remains the canonical identity system. BIPỌ̀N39 may be used for mnemonic infrastructure but not as the soul model.
