.PHONY: build-wasm bindings deploy setup

build-wasm:
	@echo "Building Soroban Smart Contracts to WebAssembly..."
	stellar contract build
	@echo "Build complete. Binaries located in target/wasm32v1-none/release/"

bindings: build-wasm
	@echo "Generating TypeScript bindings for CLI..."
	stellar contract bindings typescript \
		--wasm target/wasm32v1-none/release/stream.wasm \
		--id <YOUR_TESTNET_CONTRACT_ID> \
		--network testnet \
		--output-dir cli/src/bindings/stream
	stellar contract bindings typescript \
		--wasm target/wasm32v1-none/release/orbitstream_escrow.wasm \
		--id <YOUR_TESTNET_CONTRACT_ID> \
		--network testnet \
		--output-dir cli/src/bindings/escrow

deploy: build-wasm
	./scripts/deploy.sh

setup:
	@echo "Setting up network and identities..."
	./scripts/setup_network.sh
