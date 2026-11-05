#!/bin/bash
set -e

echo "Building contracts to WebAssembly..."
cargo build --target wasm32-unknown-unknown --profile release

echo "Deploying Stream Contract to Testnet..."
STREAM_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/stream.wasm \
    --source deployer \
    --network testnet)
echo "Stream Contract ID: $STREAM_ID"

echo "Deploying Escrow Contract to Testnet..."
ESCROW_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/orbitstream_escrow.wasm \
    --source deployer \
    --network testnet)
echo "Escrow Contract ID: $ESCROW_ID"

# Inject into root .env safely
touch .env
sed -i.bak '/STREAM_CONTRACT_ID/d' .env
sed -i.bak '/SPLIT_CONTRACT_ID/d' .env
sed -i.bak '/ESCROW_CONTRACT_ID/d' .env
echo "STREAM_CONTRACT_ID=$STREAM_ID" >> .env
echo "ESCROW_CONTRACT_ID=$ESCROW_ID" >> .env
rm -f .env.bak

echo ""
echo "Deployment complete! Copy-pasteable contract configuration:"
echo "--------------------------------------------------------"
echo "STREAM_CONTRACT_ID=$STREAM_ID"
echo "ESCROW_CONTRACT_ID=$ESCROW_ID"
echo "--------------------------------------------------------"
