#!/bin/bash
set -e

echo "Configuring Stellar Testnet..."
soroban config network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

echo "Generating secure deployment identity..."
# Generate a new keypair stored securely in the Soroban keystore (not in plaintext)
soroban config identity generate --global deployer || echo "Deployer identity already exists."

echo "====================================================="
echo "SUCCESS: Network and Identity configured."
echo "ACTION REQUIRED: Retrieve your deployer public key by running:"
echo "soroban config identity address deployer"
echo "Then fund it via Friendbot: https://laboratory.stellar.org/#account-creator?network=testnet"
echo "====================================================="
