.PHONY: build-wasm bindings deploy setup

build-wasm:
	@echo "Building Soroban Smart Contracts to WebAssembly..."
	cargo build --target wasm32-unknown-unknown --profile release
	@echo "Build complete. Binaries located in target/wasm32-unknown-unknown/release/"

bindings: build-wasm
	@echo "Generating TypeScript bindings for CLI..."
	soroban contract bindings typescript \
		--wasm target/wasm32-unknown-unknown/release/stream.wasm \
		--id <YOUR_TESTNET_CONTRACT_ID> \
		--network testnet \
		--output-dir cli/src/bindings/stream
	soroban contract bindings typescript \
		--wasm target/wasm32-unknown-unknown/release/orbitstream_escrow.wasm \
		--id <YOUR_TESTNET_CONTRACT_ID> \
		--network testnet \
		--output-dir cli/src/bindings/escrow

deploy: build-wasm
	./scripts/deploy.sh

setup:
	@echo "Setting up network and identities..."
	./scripts/setup_network.sh
