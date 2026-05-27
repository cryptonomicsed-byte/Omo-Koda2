#!/usr/bin/env bash
set -euo pipefail

# Build the WASM bridge from omokoda-core
# Required before any frontend development with real WASM.
# The 6-function bridge only: create_agent, configure_provider, translate, execute, get_state, export_receipt

echo "[wasm-build] Building omokoda-core → WASM..."

cd "$(dirname "$0")/.."

if ! command -v wasm-pack &>/dev/null; then
  echo "[wasm-build] Installing wasm-pack..."
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

wasm-pack build omokoda-core \
  --target web \
  --out-dir ../omokoda-frontend/public/wasm \
  --release

echo "[wasm-build] Done. Output: omokoda-frontend/public/wasm/"
echo "[wasm-build] Verify: exactly 6 exports (create_agent, configure_provider, translate, execute, get_state, export_receipt)"
