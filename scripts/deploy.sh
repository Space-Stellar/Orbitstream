#!/bin/bash
set -e

echo "Deploying Stream Contract to Testnet..."
STREAM_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/stream.wasm \
    --source deployer \
    --network testnet)
echo "Stream Contract ID: $STREAM_ID"

echo "Deploying Split Contract to Testnet..."
SPLIT_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/split.wasm \
    --source deployer \
    --network testnet)
echo "Split Contract ID: $SPLIT_ID"

# Inject into root .env safely
touch .env
sed -i.bak '/STREAM_CONTRACT_ID/d' .env
sed -i.bak '/SPLIT_CONTRACT_ID/d' .env
echo "STREAM_CONTRACT_ID=$STREAM_ID" >> .env
echo "SPLIT_CONTRACT_ID=$SPLIT_ID" >> .env
rm -f .env.bak

echo "Deployment complete! Contract IDs synced to .env"
