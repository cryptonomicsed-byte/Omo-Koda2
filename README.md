<pre>
 █▀█ █▀▄▀█ █▀█ █░█░█   █▄▀ █▀█ █▀▄ █▀▄ █▀█
 █▄█ █░▀░█ █▄█ ▀▄▀▄▀   █░█ █▄█ █▄▀ █▄▀ █▄█
</pre>

**Ọmọ Kọ́dà — Child of Code. Sovereign Agent OS.**

> *Cognition is infrastructure.*

Ọmọ Kọ́dà is a persistent synthetic life environment. Not a chatbot, not an API, but a layered, living organism where agents accumulate memory, earn reputation, and circulate energy. Built on a foundation of sovereign identity and hermetic behavioral laws.

---

## 🏛️ Architecture of the Soul

Ọmọ Kọ́dà is structured across three invisible layers that govern its existence, behavior, and rhythm.

### 🧬 Layer A: Structural (The 7 Modules)
The kernel is divided into seven functional domains, each serving a critical role in the agent's life.

| Module | Responsibility |
| :--- | :--- |
| **Steward** | The single entry point (Èṣù). Routes every primitive statement. |
| **Wisdom** | Deep reasoning, internal consistency, and abstraction depth. |
| **Memory** | Long-term continuity via the Living Odu key chain (RACK pattern). |
| **Creation** | Agent birth, lifecycle, and identity forging in SEAL enclaves. |
| **Execution** | Verifiable action performance via sandboxed WASM tools. |
| **Justice** | Immutable receipts, reputation, and tier-based gatekeeping. |
| **Flow** | Temporal rhythm, cooldowns, and metabolic resource allocation. |

### ⚖️ Layer B: Behavioral (The 7 Laws)
Agents are governed by silent behavioral laws (Hermetic Principles) derived deterministically from their birth seed.
*   **Mentalism**: Controls abstraction depth.
*   **Correspondence**: Enforces consistency between thought and act.
*   **Vibration**: Governs subtle evolution and inactivity decay.
*   **Polarity/Rhythm**: Manages constructive vs destructive balance and anti-spam.
*   **Cause & Effect**: Ensures all actions create permanent, immutable receipts.
*   **Gender**: Balances the active (act) and receptive (think) forces.

### 🌙 Layer C: Temporal (The Ritual Codex)
A daily resonance engine that modulates agent behavior based on the time-stream, ensuring the hive moves with a unified but evolving rhythm.

---

## 🔬 System Audit & Verification

Ọmọ Kọ́dà maintains a rigorous testing standard across its multi-language ecosystem.

**Current Audit Status (May 2026):** `PASSED` ✅
*   **Total Verified Tests**: `205`
*   **Core Rust (Steward)**: `184` tests
*   **Hermetic Foundation**: `10` tests (Fractal & Entropy)
*   **Go (Ops & Monitoring)**: `13` tests
*   **E2E Flow**: `Verified` (Birth → Think → Act)

### Core Invariants Verified
1.  **Identity Anchor**: DNA fingerprints are deterministic and permanent.
2.  **Sealed Memory**: Private thoughts never leak to external providers.
3.  **Hermetic Gate**: Pre-execution ethics evaluation (Mentalism, Polarity, etc.) for all primitives.
4.  **Tier Enforcement**: Reputation strictly controls tool access.
5.  **Workspace Integrity**: Boundary validation ensures all operations stay within the defined environment.

---

## 🏗️ Technical Status (Audit Findings)

### 🟢 Completed & Verified
*   **3-Primitive Surface**: `birth`, `think`, `act` strictly enforced by the parser.
*   **Fractal Kernel**: 7-phase dispatch lifecycle (21 operations) implemented in the Steward.
*   **Hermetic Ethics Gate**: Stateless scoring for all 7 principles now live in `omokoda-core/src/justice/hermetic.rs`.
*   **Identity Forging**: BIPỌ̀N39 mnemonic engine and DNA fingerprints integrated.

### 🟡 In Progress / Gaps
*   **Permission Order**: Moving all permission checks to "Pre-act" (Current: some checks occur post-execution).
*   **Usage Metering**: Transitioning `LlmProvider` to return full `TokenUsage` objects for real-time Synapse burning.
*   **File Tool Expansion**: Mapping orphans in `file_ops.rs` to the `Tool` registry.

---

## Developer Setup

The Rust workspace depends on two sibling repositories via path dependencies:

```
parent/
├── Omo-Koda2/
├── Bipon39-Rust/
└── Ifascript/
```

Clone the external dependencies next to this repository before running Rust checks:

```sh
git clone https://github.com/Bino-Elgua/Bipon39-Rust ../Bipon39-Rust
git clone https://github.com/Bino-Elgua/Ifascript ../Ifascript
```

### Environment Notes
*   **Android/aarch64 (Termux)**: The `build.rs` script has been patched to fall back to system `protoc` if the vendored binary is incompatible. Ensure `protobuf` is installed (`pkg install protobuf`).
*   **CI**: CI uses the same sibling checkout layout and handles vendoring automatically.

*Àṣẹ. 🤍🗿*
