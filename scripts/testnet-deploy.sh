#!/usr/bin/env bash
set -euo pipefail

# testnet-deploy.sh — Deploy Omo-koda contracts to Sui testnet
# Requires: sui CLI, active testnet wallet with gas

NETWORK="${NETWORK:-testnet}"
echo "[deploy] Network: $NETWORK"
echo "[deploy] Deploying Omo-koda Move contracts..."

cd "$(dirname "$0")/../omokoda-on-chain"

if ! command -v sui &>/dev/null; then
  echo "[deploy] ERROR: sui CLI not installed. Install from: https://docs.sui.io/guides/developer/getting-started/sui-install"
  exit 1
fi

echo "[deploy] Publishing contracts (order matters):"
echo "  1. soul.move"
echo "  2. agent.move"
echo "  3. synapse.move"
echo "  4. zbt_errors.move + zbt_guard.move + zbt_core.move"
echo "  5. consensus_ledger.move + epistemic_nft.move"
echo "  6. garden.move"
echo "  7. hive.move (LAST — after Nautilus API confirmed stable)"

sui client publish \
  --gas-budget 100000000 \
  --json 2>&1 | tee deploy-output.json

echo "[deploy] Done. Package ID saved to deploy-output.json"
echo "[deploy] Next: update omokoda-frontend/.env with NEXT_PUBLIC_PACKAGE_ID"
