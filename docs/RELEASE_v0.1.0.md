# OrbitStream v0.1.0 (Testnet Alpha)

OrbitStream v0.1.0 introduces non-custodial, per-second token streaming on Stellar Testnet using Soroban.

## Highlights

* **Continuous Stream Engine (`contracts/stream`):** `#![no_std]` Soroban contract managing continuous per-second payments using an allowance-pull architecture. State lookups are $O(1)$ and storage state uses Checks-Effects-Interactions to prevent double-claim and re-entrancy risks.
* **Escrow Vault Architecture (`contracts/escrow`):** Upfront pre-funded vault contract where funds are held by the contract, so receivers do not depend on the sender's wallet balance, with partial unaccrued withdrawals, and multi-party cancellation settlement.
* **Developer CLI (`cli/`):** Command-line tool for automated token allowance approvals, stream initialization, balance querying, and claiming on Testnet.
* **Web Application (`web/`):** React 18 + Vite frontend interfacing with the Freighter browser wallet and Soroban RPC for real-time balance visualization and token claiming.
* **Event Indexer (`indexer/`):** Async Python service for local Soroban RPC event polling and state synchronization.

---

## Deployed Testnet Contracts

| Contract | Network | Contract Address | Explorer Link |
| :--- | :--- | :--- | :--- |
| **Stream Contract** | Testnet | `CAESALD2PIFE636E4C4DDJTOFQBBLOB3GK35WK6SY3IL2WNKUWHGAG6Y` | [Stellar.Expert](https://stellar.expert/explorer/testnet/contract/CAESALD2PIFE636E4C4DDJTOFQBBLOB3GK35WK6SY3IL2WNKUWHGAG6Y) |
| **Native Token (XLM)** | Testnet | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` | [Stellar.Expert](https://stellar.expert/explorer/testnet/contract/CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC) |
| **Escrow Vault** | Testnet | *Not deployed (Local development only)* | — |

---

## Verifying Release Artifacts

To compile and verify the WebAssembly binaries locally:

```bash
# Compile contracts to Wasm
cargo build --target wasm32-unknown-unknown --profile release

# Verify wasm files generated
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

SHA-256 checksums of the release Wasm files can be verified against compiled output.

---

## Documentation

* [Contract Reference](CONTRACTS.md)
* [Gas & Resource Benchmarks](../GAS.md)
* [Security Policy](../SECURITY.md)
* [Contributor Guide](../CONTRIBUTING.md)
