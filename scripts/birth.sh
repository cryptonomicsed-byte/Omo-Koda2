#!/usr/bin/env bash
set -euo pipefail

# birth.sh — CLI agent birth ritual
# Usage: ./scripts/birth.sh "agent-name" [--provider ollama|webllm]

AGENT_NAME="${1:-}"
PROVIDER="${2:---provider}"
PROVIDER_VAL="${3:-webllm}"

if [[ -z "$AGENT_NAME" ]]; then
  echo "Usage: $0 \"agent-name\" [--provider webllm|ollama]"
  exit 1
fi

echo "[birth] Forging soul for: $AGENT_NAME"
echo "[birth] Provider: $PROVIDER_VAL (sovereign only — no external providers at birth)"

# Invoke CLI birth command
if command -v omokoda-cli &>/dev/null; then
  omokoda-cli birth "$AGENT_NAME" --provider "$PROVIDER_VAL"
else
  echo "[birth] omokoda-cli not installed. Build with: cargo build -p omokoda-cli --release"
  echo "[birth] Then: ./target/release/omokoda-cli birth \"$AGENT_NAME\""
  exit 1
fi
