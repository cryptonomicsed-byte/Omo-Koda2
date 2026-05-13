# Session Specification (v1.0 — FROZEN)

## Purpose
Defines the structure and persistence of conversation sessions, ensuring privacy and continuity.

## Session Object
A session consists of public and private components:

```json
{
  "version": 1,
  "session_id": "string (BLAKE3)",
  "agent_id": "string",
  "public_messages": [
    {
      "role": "User|Assistant|System",
      "blocks": [
        { "type": "Text", "content": "..." },
        { "type": "ToolUse", "id": "...", "name": "...", "input": "..." },
        { "type": "ToolResult", "tool_use_id": "...", "output": "...", "is_error": "boolean" }
      ],
      "usage": { "input_tokens": 0, "output_tokens": 0 }
    }
  ],
  "private_ciphertext": "base64 (encrypted private blocks)",
  "private_nonce": "base64 (12 bytes)",
  "merkle_root": "string (BLAKE3)"
}
```

## Persistence Rules
- **Public Messages**: Stored as versioned JSON. Eligible for receipt anchoring.
- **Private Messages**: Must be encrypted with **ChaCha20Poly1305** using a key derived via **Argon2id** from the agent's memory seed (`K_root`).
- **Asymmetric Persistence**:
  - User messages: Blocking save (required for resume).
  - Assistant messages: Fire-and-forget (optional, regeneratable).

## Encryption Parameters
- **KDF**: Argon2id
- **Memory**: 65536 KB
- **Iterations**: 3
- **Parallelism**: 1
- **Cipher**: ChaCha20Poly1305

## Lifecycle
- **Birth**: Creates the initial session with the agent's Odu seed.
- **Resume**: Decrypts private messages and loads public history.
- **Archive**: Finalizes the Merkle root and seals the session.
- **Destroy**: Zeroizes local keys and archives the encrypted blob.
