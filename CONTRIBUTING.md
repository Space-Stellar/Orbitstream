# Contributing to OrbitStream 🪐

Thank you for contributing to OrbitStream. This guide explains the repository structure, local environment setup, branching rules, commit conventions, and pull request checklist.

---

## Repository Structure

OrbitStream is organized as a monorepo:
* `contracts/`: Soroban smart contracts (`stream` and `escrow`) written in `#![no_std]` Rust.
* `cli/`: TypeScript command-line interface using `@stellar/stellar-sdk`.
* `web/`: Web application built with React, Vite, Tailwind CSS, and `@stellar/freighter-api`.
* `indexer/`: Python event poller using `aiohttp`, `SQLAlchemy`, and `aiosqlite`.
* `docs/`: Technical references, contract specifications, and release notes.

---

## Local Development Setup

Every command listed below has been verified to run against the codebase.

### 1. Smart Contracts (Rust / Soroban)

**Prerequisites:** Rust stable toolchain and the WebAssembly compilation target.

```bash
# Add the wasm32 compilation target
rustup target add wasm32-unknown-unknown

# Run all contract unit tests across the entire workspace
cargo test --workspace

# Validate code formatting
cargo fmt --all -- --check

# Run linter with strict warning checks
cargo clippy --workspace --all-targets -- -D warnings

# Build release WebAssembly artifacts
cargo build --target wasm32-unknown-unknown --profile release
```

Wasm artifacts are compiled to `target/wasm32-unknown-unknown/release/`.

---

### 2. Frontend Web App (TypeScript / Vite)

**Prerequisites:** Node.js 20+ and npm.

```bash
cd web

# Install dependencies cleanly
npm ci

# Verify typechecking and production build
npm run build

# Start local development server
npm run dev
```

The web dashboard is served at `http://localhost:5173`. Install the [Freighter browser extension](https://www.freighter.app/) to interact with Stellar Testnet.

---

### 3. Developer CLI (TypeScript)

**Prerequisites:** Node.js 20+ and npm.

```bash
cd cli

# Install dependencies
npm ci

# Compile TypeScript
npm run build

# Verify command execution
node dist/index.js --help
```

---

### 4. Event Indexer (Python)

**Prerequisites:** Python 3.10+.

```bash
cd indexer

# Create and activate virtual environment
python3 -m venv venv
source venv/bin/activate

# Install requirements
pip install -r requirements.txt

# Start the poller against Testnet
python main.py
```

The indexer connects to `https://soroban-testnet.stellar.org:443` by default and writes to `orbitstream.db`.

---

## Branch Naming Conventions

Create feature branches off the latest `main` branch using the format:
`<type>/<short-description>`

* `feat/<feature-name>` – New features or contract capabilities (e.g. `feat/escrow-indexer-events`)
* `fix/<bug-name>` – Bug fixes (e.g. `fix/cancel-stream-settlement`)
* `docs/<topic>` – Documentation updates (e.g. `docs/contracts-spec`)
* `chore/<task>` – Maintenance or dependency updates (e.g. `chore/upgrade-soroban-sdk`)
* `test/<scope>` – Adding or improving test suites (e.g. `test/escrow-fuzzing`)

---

## Commit Message Format

We strictly adhere to [Conventional Commits](https://www.conventionalcommits.org/):

```text
<type>(<scope>): <imperative description>

[optional body]

[optional footer(s)]
```

### Types
* `feat`: A new feature
* `fix`: A bug fix
* `docs`: Documentation changes
* `test`: Adding or correcting tests
* `refactor`: Code change that neither fixes a bug nor adds a feature
* `chore`: Build process, tooling, or dependency changes
* `ci`: Changes to CI workflow configuration files

### Common Scopes
* `contracts`: Workspace contracts
* `stream`: Allowance-pull stream contract
* `escrow`: Escrow vault contract
* `cli`: TypeScript CLI
* `web`: React dashboard
* `indexer`: Python event listener
* `ci`: GitHub Actions workflows
* `docs`: Documentation files

**Examples:**
* `feat(escrow): implement withdraw method for unaccrued deposit`
* `fix(stream): prevent re-entrancy in claim execution`
* `docs(readme): update quickstart with verified cli commands`

---

## Pull Request Checklist

Before submitting a Pull Request, verify each item:

- [ ] Branch is branched from up-to-date `main`.
- [ ] Contract tests pass: `cargo test --workspace`.
- [ ] Contract formatting passes: `cargo fmt --all -- --check`.
- [ ] Contract linter passes: `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Web application builds cleanly: `(cd web && npm ci && npm run build)`.
- [ ] CLI compiles without error: `(cd cli && npm run build)`.
- [ ] Commit history follows Conventional Commits.
- [ ] No secrets, private keys, or `.env` files are included.
- [ ] The PR description references the corresponding issue (e.g. `Closes #15`).
