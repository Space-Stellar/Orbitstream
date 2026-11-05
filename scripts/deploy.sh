#!/usr/bin/env bash
set -euo pipefail

STREAM_WASM="target/wasm32v1-none/release/stream.wasm"
ESCROW_WASM="target/wasm32v1-none/release/orbitstream_escrow.wasm"

echo "Building contracts to WebAssembly with Stellar CLI..."
stellar contract build

if [[ ! -f "$STREAM_WASM" || ! -f "$ESCROW_WASM" ]]; then
  echo "Error: Built WASM files not found at expected paths." >&2
  exit 1
fi

STREAM_HASH=$(stellar contract info hash --wasm "$STREAM_WASM")
ESCROW_HASH=$(stellar contract info hash --wasm "$ESCROW_WASM")

echo "Deploying Stream Contract to Testnet..."
STREAM_ID=$(stellar contract deploy \
  --wasm "$STREAM_WASM" \
  --source deployer \
  --network testnet)
echo "Stream Contract ID: $STREAM_ID"

echo "Deploying Escrow Contract to Testnet..."
ESCROW_ID=$(stellar contract deploy \
  --wasm "$ESCROW_WASM" \
  --source deployer \
  --network testnet)
echo "Escrow Contract ID: $ESCROW_ID"

if [[ -z "$STREAM_ID" || -z "$ESCROW_ID" ]]; then
  echo "Error: One or both contract deployments failed to produce a valid ID." >&2
  exit 1
fi

if [[ "$STREAM_ID" =~ ^C[A]{20,} || "$ESCROW_ID" =~ ^C[A]{20,} || "$STREAM_ID" == *"<"* || "$ESCROW_ID" == *"<"* ]]; then
  echo "Error: Deployed contract ID appears to be a placeholder." >&2
  exit 1
fi

# Safely update root .env only after BOTH deploys succeed without placeholders
ENV_FILE=".env"
touch "$ENV_FILE"
TMP_ENV=$(mktemp)
grep -v -E '^(STREAM_CONTRACT_ID|ESCROW_CONTRACT_ID|SPLIT_CONTRACT_ID)=' "$ENV_FILE" > "$TMP_ENV" || true
{
  echo "STREAM_CONTRACT_ID=$STREAM_ID"
  echo "ESCROW_CONTRACT_ID=$ESCROW_ID"
} >> "$TMP_ENV"
mv "$TMP_ENV" "$ENV_FILE"

echo ""
echo "Deployment complete! Copy-pasteable contract configuration:"
echo "--------------------------------------------------------"
echo "STREAM_CONTRACT_ID=$STREAM_ID"
echo "ESCROW_CONTRACT_ID=$ESCROW_ID"
echo ""
echo "STREAM_WASM_HASH=$STREAM_HASH"
echo "ESCROW_WASM_HASH=$ESCROW_HASH"
echo ""
echo "Stellar.Expert Links:"
echo "Stream Contract: https://stellar.expert/explorer/testnet/contract/$STREAM_ID"
echo "Escrow Contract: https://stellar.expert/explorer/testnet/contract/$ESCROW_ID"
echo "--------------------------------------------------------"
