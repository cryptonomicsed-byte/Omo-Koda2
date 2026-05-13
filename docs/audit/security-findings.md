# Security Findings: Ọmọ Kọ́dà Audit

This document summarizes the security and privacy findings from the initial audit of the Ọmọ Kọ́dà codebase.

## 1. Critical Vulnerabilities (P0)

### 1.1 Privacy Failure: Plaintext Private Memory
**Severity**: CRITICAL
**Finding**: While the spec promises sealed, encrypted memory, the current implementation stores private thoughts as plaintext strings with a `"private"` scope tag. 
**Impact**: Any local compromise or accidental serialization of the `AgentState` leaks all private thoughts.
**Remediation**: Implement Argon2id key derivation and ChaCha20Poly1305 encryption for all memory entries tagged as private.

### 1.2 Reputation Gaming
**Severity**: HIGH
**Finding**: The current reputation system is a simple accumulator with no cost to act, no rate limiting, and no quality verification.
**Impact**: An attacker can spam `act` calls to grind reputation and bypass tier gates.
**Remediation**: Introduce Synapse/Dopamine costs for actions, implement cooldowns, and add a quality oracle/witness consensus mechanism.

### 1.3 Deterministic Soul
**Severity**: MEDIUM
**Finding**: The "soul" is derived from `BLAKE3(name || birth_timestamp)`.
**Impact**: Predictable and non-entropic. An attacker who knows the birth timestamp can reproduce the soul and associated Hermetic traits.
**Remediation**: Use CSPRNG entropy for birth and map it to 256 Odu Ifá configurations.

## 2. Structural Security Issues

### 2.1 Lack of Sandboxing
**Severity**: HIGH
**Finding**: `sandbox_mode` is a boolean flag with no actual isolation. Tools execute with full process privileges.
**Impact**: Path traversal, network exfiltration, and malicious tool execution are possible.
**Remediation**: Implement Linux `unshare` namespace isolation and WASM sandboxing (WASI) for tool execution.

### 2.2 Brittle Identifier Filtering
**Severity**: MEDIUM
**Finding**: The `BLOCKED_IDENTIFIERS` list uses a blacklist approach, which is easily bypassed by Unicode homoglyphs or casing variations.
**Remediation**: Move to a whitelist approach for the public surface (`birth`, `think`, `act`).

### 2.3 Weak Agent Identity
**Severity**: MEDIUM
**Finding**: Agent IDs are truncated SHA-256 hashes (8 bytes/16 hex chars), providing only 64 bits of entropy.
**Impact**: Vulnerable to birthday collisions.
**Remediation**: Use full 32-byte BLAKE3 hashes for Agent IDs.

## 3. Cryptographic Recommendations
- **Receipts**: Replace the in-memory HashMap with an append-only Merkle tree. Add Ed25519 signatures to prevent forgery.
- **Key Material**: Use the `zeroize` crate for sensitive key material in memory.
- **Anchoring**: Implement Sui blockchain anchoring for public receipts to provide immutable proof of action.
