# Contributing to OrbitStream 🪐

First off, thank you for considering contributing to OrbitStream! It's people like you that make the open-source Web3 ecosystem a reality. 

This document outlines the process for contributing to our continuous funding protocol.

## 🏗️ Monorepo Structure

OrbitStream is divided into three main environments. Please ensure you are working in the correct directory for your issue:
* `/contracts`: Soroban smart contracts (Rust)
* `/indexer`: Blockchain event listener (Python)
* `/web`: Frontend dashboard (TypeScript / React / Vite)

## 🛠️ Local Development Setup

### 1. Smart Contracts (Rust)
To compile and test the contracts, you need the `wasm32-unknown-unknown` target.
```bash
cd contracts/stream
rustup target add wasm32-unknown-unknown
cargo test
```

### 2. Frontend Web App (TypeScript)

We use npm and Vite for the dashboard.
```bash
cd web
npm install
npm run dev
```

(Note: You will need Freighter installed in your browser to test wallet interactions).

### 3. Backend Indexer (Python)

It is recommended to use a virtual environment.
```bash
cd indexer
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## 📝 Pull Request Process

*   **Find or Create an Issue:** Always ensure there is an open GitHub issue for the work you are doing. If you want to build a new feature, open an issue first to discuss it with the maintainers.
*   **Fork and Branch:** Fork the repository and create a new branch from main (e.g., `feat/add-historical-charts` or `fix/rpc-polling-timeout`).
*   **Write Clean Code:**
    *   **Rust:** Run `cargo fmt` and ensure `cargo clippy` passes with zero warnings.
    *   **TypeScript:** Ensure there are no unused imports or type errors.
*   **Commit Convention:** We follow standard conventional commits (e.g., `feat:`, `fix:`, `docs:`, `chore:`).
*   **Submit the PR:** Link the issue in your PR description (e.g., Closes #12). A maintainer will review your code, request changes if necessary, and merge it!

## 🛡️ Code of Conduct

By participating in this project, you agree to maintain a respectful and welcoming environment for all developers, regardless of their experience level.

Happy building! 🚀
