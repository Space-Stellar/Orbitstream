# Security Policy

## ⚠️ Unaudited Testnet Software Disclaimer

OrbitStream smart contracts (`contracts/stream`, `contracts/escrow`) and supporting tools are experimental software deployed exclusively to Stellar Testnet. 

**This codebase has NOT undergone a security audit by an independent security firm.** Do not use these smart contracts with real assets, mainnet keys, or production funds.

---

## Reporting a Vulnerability

Report vulnerabilities privately at https://github.com/Space-Stellar/Orbitstream/security/advisories/new. Please do not open public issues for security problems.

---

## Scope

The following components are in scope for security vulnerability reports:
* `contracts/stream/src/` (Allowance-pull stream contract)
* `contracts/escrow/src/` (Escrow vault contract)
* `cli/src/` (CLI transaction builder and bindings)
* `web/src/` (Frontend transaction generation and simulation)
* `indexer/app/` (Event indexing and parsing)

---

## Threat Model & Risk Analysis

The following threat vectors and design constraints are derived strictly from the smart contract implementation:

### 1. Token Allowance & Solvency Dependency (`contracts/stream`)
* **Mechanism:** The Stream contract does not escrow tokens; it executes `token::Client::transfer(&sender, &receiver, ...)` via allowances.
* **Risk:** A sender can revoke or exhaust their allowance, or transfer tokens away from their wallet at any time.
* **Impact:** If the sender's balance or allowance is less than the accrued balance, subsequent `claim()` calls by the receiver will fail. Accrual display on the frontend will show a "phantom balance" that cannot be settled on-chain.

### 2. Trapped Cancellation Hazard (`contracts/stream`)
* **Mechanism:** In `contracts/stream::cancel_stream`, the contract calculates `pending_payout` and immediately attempts to transfer that amount from `sender` to `receiver`.
* **Risk:** If `pending_payout > 0` and the sender's account has insufficient balance or allowance, the transfer fails.
* **Impact:** The transaction reverts, preventing `env.storage().persistent().remove(&key)` from executing. The stream becomes trapped open in persistent storage and cannot be closed by the sender until sufficient balance/allowance is restored.

### 3. Escrow Deposit and Withdrawal Safety (`contracts/escrow`)
* **Mechanism:** `contracts/escrow::deposit` transfers tokens into the contract's own address (`env.current_contract_address()`). `contracts/escrow::withdraw` allows the sender to withdraw unaccrued funds.
* **Protection:** 
  * State ordering: `stream.total_deposited` is decremented in storage before transferring tokens back to `stream.sender`.
  * Math safety: Accrual and unaccrued amounts use checked operations (`checked_sub`, `checked_mul`), returning `Error::InsufficientUnaccrued` or `Error::MathOverflow` on invalid values.
* **Risk:** Escrowed tokens remain in contract custody until claimed or cancelled. Senders must ensure their parameters (`amount` and `flow_rate`) are valid before submitting deposits.

### 4. Claim Authorization Differences
* In `contracts/stream`: `claim()` strictly requires receiver authorization (`receiver.require_auth()`).
* In `contracts/escrow`: `claim()` is permissionless (no `require_auth` check). Anyone can invoke `claim(stream_id)`. However, tokens are transferred strictly to `stream.receiver`, preventing unauthorized redirection of funds.

### 5. Multi-Party Cancellation (`contracts/escrow`)
* `cancel_stream` requires authorization from `caller` (`caller.require_auth()`) and validates that `caller == stream.sender || caller == stream.receiver`. Either party can terminate the stream. Funds are held by the contract, so receivers do not depend on the sender's wallet balance.

### 6. Admin and Privileged Functions
* **No Administrative Keys:** The escrow contract has no admin or upgrade functions in the current code. It is unaudited. Settlement depends on the token contract accepting transfers (issuers of some assets can freeze or claw back balances). Neither `contracts/stream` nor `contracts/escrow` implements contract owner, upgrade, pause, or admin keys. State transitions are governed entirely by autonomous contract logic and caller authorizations.
