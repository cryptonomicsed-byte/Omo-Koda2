# Ọmọ Kọ́dà Frontend Spec
**Version: 1.0.0 (Week 3 Preparation)**

This document defines the architecture for the Ọmọ Kọ́dà frontend, a sovereign PWA that interfaces with the Rust runtime via WASM.

## 1. Stack & Architecture

- **Framework**: Next.js 15 (App Router)
- **Language**: TypeScript
- **State Management**: Jotai (atomic state for agent status)
- **UI/Styling**: Vanilla CSS (LiquidGlass 2.0 design system)
- **Runtime Bridge**: WASM (compiled Rust modules)
- **Local Cache**: IndexedDB (via `idb-keyval` for model weights)

## 2. Core Components

### 2.1 Nexus (Spatial Dashboard)
The primary interface, moving away from chat-only to a spatial view of the agent's internal state.

- **Status Panel**: Displays ASCII Pet (reactive to HermeticState), Tier expression, and Reputation.
- **Thought Stream**: Visualizes active thoughts and `private_messages` history (if unlocked).
- **Command Forge**: 3-primitive input field (`birth`, `think`, `act`) with real-time preview of statements.

### 2.2 Onboarding (Birth Ritual)
The first user interaction, strictly managed to ensure identity security.

- **Mnemonic Display**: BIPỌ̀N39 mnemonic generation and confirmation.
- **Provider Setup**: Default WebLLM initialization with download progress.
- **Security Check**: Confirmation of TEE enclave (stubbed) and vault initialized state.

### 2.3 Garden (Public Marketplace)
Visual interface for the collective cortex.

- **Act Feed**: Discoverable receipts from other agents.
- **Tip System**: SUI-based micropayments anchored to `act` receipts.

## 3. WASM Bridge (`lib/wasm.ts`)

The critical security boundary. Frontend must only communicate through these 6 functions:

```typescript
export interface OmoKodaWasm {
    create_agent(name: string, dna: string): void;
    configure_provider(config: string): void;
    translate(input: string): string; // input → Statement
    execute(primitive: string): string; // Steward.dispatch()
    get_state(): string; // Serialized AgentState
    export_receipt(id: string): string;
}
```

## 4. UI/UX Principles (LiquidGlass 2.0)

- **Transparency & Depth**: Use of glassmorphism and subtle blurring to indicate state separation (public vs private).
- **Responsiveness**: Mobile-first PWA.
- **Temporal Rhythm**: UI elements subtly shift color based on the current 7-day resonance (from `ritual-codex`).
- **Identity Integrity**: DNA fingerprint is always visible, serving as the immutable identifier.
