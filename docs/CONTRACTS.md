# OrbitStream Smart Contract Reference

This document provides a technical specification for the smart contracts in OrbitStream: `contracts/stream` (the allowance-pull stream contract) and `contracts/escrow` (the escrow vault contract). All specifications are derived strictly from the `#![no_std]` Rust implementation.

---

## 1. Stream Contract (`contracts/stream`)

The Stream contract implements a non-custodial, allowance-pull token streaming mechanism. The sender approves the Stream contract to spend tokens on their behalf up to a specified expiration ledger. Accrued tokens remain in the sender's account until claimed by the receiver.

### 1.1 Data Structures

#### `StreamKey`
Persistent storage lookup key identifying a unique stream channel.
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamKey {
    pub sender: Address,
    pub receiver: Address,
}
```

#### `StreamConfig`
Configuration and state stored under `StreamKey` in persistent storage.
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamConfig {
    pub token: Address,
    pub flow_rate: u64,
    pub start_time: u64,
    pub withdrawn: u64,
}
```

### 1.2 Storage & TTL Policy

* **Storage Tier**: Persistent storage (`env.storage().persistent()`).
* **Key**: `StreamKey { sender, receiver }`.
* **Value**: `StreamConfig`.
* **TTL Policy**: `extend_persistent_ttl`:
  * `BUMP_THRESHOLD`: 120,000 ledgers (~7 days).
  * `BUMP_AMOUNT`: 518,400 ledgers (~30 days).

### 1.3 Public Functions

#### `init`
```rust
pub fn init(env: Env, sender: Address, receiver: Address, token: Address, flow_rate: u64)
```
* **Description**: Initializes an active token stream between `sender` and `receiver`. Locks the current ledger timestamp (`env.ledger().timestamp()`) as `start_time`.
* **Authorization**: Requires authorization from `sender` (`sender.require_auth()`).
* **Parameters**:
  * `sender: Address` – The funding account creating the stream.
  * `receiver: Address` – The destination account entitled to accrued tokens.
  * `token: Address` – The Soroban token contract address (e.g. SEP-41 token).
  * `flow_rate: u64` – Payout rate in stroops (atomic units) per second.
* **Return Value**: `()`.
* **Panics / Failure Modes**:
  * Panics with `"Stream between sender and receiver already exists. Close it first."` if `StreamKey { sender, receiver }` already exists.
* **Events Emitted**:
  * **Topics**: `(symbol_short!("init"), sender: Address, receiver: Address)`
  * **Data**: `flow_rate: u64`

#### `get_stream`
```rust
pub fn get_stream(env: Env, sender: Address, receiver: Address) -> StreamConfig
```
* **Description**: Reads the stored configuration and extends the persistent storage TTL.
* **Authorization**: None (public read-only query).
* **Parameters**:
  * `sender: Address` – Stream creator address.
  * `receiver: Address` – Stream receiver address.
* **Return Value**: `StreamConfig`.
* **Panics / Failure Modes**:
  * Panics with `"Stream does not exist"` if key is not found.
* **Events Emitted**: None.

#### `get_balance`
```rust
pub fn get_balance(env: Env, sender: Address, receiver: Address) -> u64
```
* **Description**: Dynamically calculates accrued, unclaimed tokens based on elapsed ledger time.
* **Authorization**: None (public read-only calculation).
* **Parameters**:
  * `sender: Address` – Stream creator address.
  * `receiver: Address` – Stream receiver address.
* **Return Value**: `u64` – Claimable balance in stroops.
* **Calculation**:
  $$\text{elapsed} = \max(0, \text{current\_time} - \text{start\_time})$$
  $$\text{accrued} = \text{elapsed} \times \text{flow\_rate}$$
  $$\text{balance} = \max(0, \text{accrued} - \text{withdrawn})$$
* **Panics / Failure Modes**:
  * Panics if stream does not exist.
  * Panics with `"Current time before start time"` if ledger time is earlier than start time.
  * Panics with `"Accrual calculation overflow"` if multiplication overflows `u64`.
* **Events Emitted**: None.

#### `claim`
```rust
pub fn claim(env: Env, sender: Address, receiver: Address)
```
* **Description**: Withdraws accrued tokens and transfers them from `sender` to `receiver`.
* **Authorization**: Requires authorization from `receiver` (`receiver.require_auth()`).
* **Parameters**:
  * `sender: Address` – Stream creator address.
  * `receiver: Address` – Stream receiver address.
* **Return Value**: `()`.
* **Ordering (Checks-Effects-Interactions)**:
  1. Calculates `claimable` balance via `get_balance`.
  2. Verifies `claimable > 0` (panics with `"No tokens available to claim"` if zero).
  3. Increments `config.withdrawn += claimable` and writes `config` to persistent storage.
  4. Invokes external token contract: `token::Client::transfer_from(&env.current_contract_address(), &sender, &receiver, &(claimable as i128))` using the sender's token allowance to the Stream contract.
* **Panics / Failure Modes**:
  * Panics with `"No tokens available to claim"` if `claimable == 0`.
  * Panics with `"Withdrawn overflow"` if `withdrawn + claimable` overflows `u64`.
  * Panics if `sender` has insufficient balance, insufficient token allowance, or the allowance has passed its expiration ledger.
* **Events Emitted**:
  * **Topics**: `(symbol_short!("claim"), sender: Address, receiver: Address)`
  * **Data**: `claimable: u64`

#### `cancel_stream`
```rust
pub fn cancel_stream(env: Env, sender: Address, receiver: Address)
```
* **Description**: Terminates the active stream, settles any remaining accrued tokens, and deletes the stream from persistent storage.
* **Authorization**: Requires authorization from `sender` (`sender.require_auth()`).
* **Parameters**:
  * `sender: Address` – Stream creator address.
  * `receiver: Address` – Stream receiver address.
* **Return Value**: `()`.
* **Ordering (Checks-Effects-Interactions)**:
  1. Computes `pending_payout`.
  2. Deletes `StreamKey` from persistent storage via `env.storage().persistent().remove(&key)`.
  3. If `pending_payout > 0`, invokes `token::Client::transfer_from(&env.current_contract_address(), &sender, &receiver, &(pending_payout as i128))`.
* **Panics / Failure Modes**:
  * Panics with `"Stream does not exist"` if not found.
  * Reverts if the token transfer fails (e.g. sender revoked allowance, allowance expired past its expiration ledger, or sender has insufficient balance).
* **Events Emitted**:
  * **Topics**: `(symbol_short!("cancel"), sender: Address, receiver: Address)`
  * **Data**: `pending_payout: u64`

---

## 2. Escrow Vault Contract (`contracts/escrow`)

The Escrow Vault contract implements a vault-custodial model where senders pre-deposit tokens directly into the contract upon initialization. The escrow contract has no admin or upgrade functions in the current code. It is unaudited. Settlement depends on the token contract accepting transfers (issuers of some assets can freeze or claw back balances). Funds are held by the contract, so receivers do not depend on the sender's wallet balance.

### 2.1 Data Structures

#### `Stream`
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    pub sender: Address,
    pub receiver: Address,
    pub token: Address,
    pub total_deposited: i128,
    pub flow_rate: i128,
    pub start_time: u64,
    pub claimed_amount: i128,
}
```

#### `DataKey`
```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Stream(u64),
    StreamCount,
}
```

### 2.2 Error Codes

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    InvalidFlowRate = 2,
    StreamNotFound = 3,
    Unauthorized = 4,
    MathOverflow = 5,
    InsufficientUnaccrued = 6,
}
```

### 2.3 Storage & TTL Policy

* **Instance Storage**: `DataKey::StreamCount` stores the auto-incrementing `u64` stream identifier counter.
* **Persistent Storage**: `DataKey::Stream(u64)` stores the `Stream` state record.
* **Constants**:
  * `DAY_IN_LEDGERS`: 17,280 ledgers (approx. 24 hours at 5s ledger close time).
  * `BUMP_THRESHOLD`: 30 days (`518,400` ledgers).
  * `BUMP_TO`: 120 days (`2,073,600` ledgers).

### 2.4 Public Functions

#### `deposit`
```rust
pub fn deposit(
    env: Env,
    sender: Address,
    receiver: Address,
    token: Address,
    amount: i128,
    flow_rate: i128,
) -> Result<u64, Error>
```
* **Description**: Transfers `amount` tokens from `sender` into the escrow vault contract and creates a new stream record.
* **Authorization**: Requires authorization from `sender` (`sender.require_auth()`).
* **Parameters**:
  * `sender: Address` – Account funding the stream and depositing tokens.
  * `receiver: Address` – Destination beneficiary account.
  * `token: Address` – Token contract address.
  * `amount: i128` – Total upfront deposit in atomic units (must be $> 0$).
  * `flow_rate: i128` – Stream speed in atomic units per second (must be $> 0$).
* **Return Value**: `Result<u64, Error>` – Returns the newly generated `stream_id`.
* **Errors**:
  * `Error::InvalidAmount (1)`: `amount <= 0`.
  * `Error::InvalidFlowRate (2)`: `flow_rate <= 0`.
  * `Error::MathOverflow (5)`: Counter increment overflow.
* **Events Emitted**: None.

#### `withdraw`
```rust
pub fn withdraw(env: Env, stream_id: u64, amount: i128) -> Result<(), Error>
```
* **Description**: Allows the stream sender to recover unaccrued deposit balance before it streams.
* **Authorization**: Requires authorization from the stream sender (`stream.sender.require_auth()`).
* **Parameters**:
  * `stream_id: u64` – Target stream identifier.
  * `amount: i128` – Amount of unaccrued deposit to withdraw (must be $> 0$).
* **Return Value**: `Result<(), Error>`.
* **Errors**:
  * `Error::StreamNotFound (3)`: `stream_id` does not exist.
  * `Error::InvalidAmount (1)`: `amount <= 0`.
  * `Error::InsufficientUnaccrued (6)`: Requested `amount` exceeds unaccrued balance:
    $$\text{unaccrued} = \text{total\_deposited} - \min(\text{elapsed} \times \text{flow\_rate}, \text{total\_deposited})$$
  * `Error::MathOverflow (5)`: Arithmetic overflow or underflow.
* **Events Emitted**: None.

#### `claim`
```rust
pub fn claim(env: Env, stream_id: u64) -> Result<(), Error>
```
* **Description**: Transfers accrued, unclaimed tokens from the contract vault to `stream.receiver`.
* **Authorization**: None required. Any caller can trigger the claim, but payouts always transfer strictly to `stream.receiver`.
* **Parameters**:
  * `stream_id: u64` – Target stream identifier.
* **Return Value**: `Result<(), Error>`.
* **Errors**:
  * `Error::StreamNotFound (3)`: `stream_id` does not exist.
  * `Error::MathOverflow (5)`: Arithmetic overflow or underflow.
* **Events Emitted**: None.

#### `cancel_stream`
```rust
pub fn cancel_stream(env: Env, caller: Address, stream_id: u64) -> Result<(), Error>
```
* **Description**: Closes the stream, transfers accrued tokens to `stream.receiver`, returns unaccrued remainder to `stream.sender`, and removes the stream record from storage.
* **Authorization**: Requires authorization from `caller` (`caller.require_auth()`).
* **Parameters**:
  * `caller: Address` – Must be either `stream.sender` or `stream.receiver`.
  * `stream_id: u64` – Target stream identifier.
* **Return Value**: `Result<(), Error>`.
* **Errors**:
  * `Error::StreamNotFound (3)`: `stream_id` does not exist.
  * `Error::Unauthorized (4)`: `caller` is neither `stream.sender` nor `stream.receiver`.
  * `Error::MathOverflow (5)`: Arithmetic overflow or underflow.
* **Events Emitted**: None.

---

## 3. Architecture Comparison: Stream vs. Escrow

| Property | Stream Contract (`contracts/stream`) | Escrow Vault Contract (`contracts/escrow`) |
| :--- | :--- | :--- |
| **Custody Model** | Allowance-pull (Non-custodial, funds in sender wallet) | Vault-custodial (Funds locked in contract) |
| **Capital Efficiency** | High (sender retains funds until claimed) | Low (sender pre-funds full locked amount) |
| **Solvency** | Dependent on sender's wallet balance, allowance, and expiration ledger | Funds are held by the contract, so receivers do not depend on the sender's wallet balance |
| **Stream Identification** | Tuple `(sender, receiver)` (1 active stream per pair) | Auto-incrementing numeric `stream_id: u64` |
| **Cancellation Auth** | Sender only (`sender.require_auth()`) | Either party (`sender` or `receiver`) |
| **Cancellation Failure Risk** | Reverts if sender balance or allowance is insufficient or expired | Funds are held by the contract, so receivers do not depend on the sender's wallet balance |
| **On-Chain Events** | `init`, `claim`, `cancel` | None |
