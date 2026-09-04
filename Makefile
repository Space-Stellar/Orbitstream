.PHONY: build-wasm bindings

build-wasm:
	@echo "Building Soroban Smart Contracts to WebAssembly..."
	cargo build --target wasm32-unknown-unknown --profile release
	@echo "Build complete. Binaries located in target/wasm32-unknown-unknown/release/"

bindings: build-wasm
	@echo "Generating TypeScript bindings for CLI..."
	soroban contract bindings typescript \
		--wasm target/wasm32-unknown-unknown/release/stream.wasm \
		--id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM \
		--network testnet \
		--output-dir cli/src/bindings/stream
	soroban contract bindings typescript \
		--wasm target/wasm32-unknown-unknown/release/split.wasm \
		--id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM \
		--network testnet \
		--output-dir cli/src/bindings/split

deploy: build-wasm
	@echo "Executing Testnet Deployment..."
	./scripts/deploy.sh

setup:
	@echo "Setting up network and identities..."
	./scripts/setup_network.sh
