#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

mod storage;

#[cfg(test)]
mod test;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamKey {
    pub sender: Address,
    pub receiver: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamConfig {
    pub token: Address,
    pub flow_rate: u64,
    pub start_time: u64,
    pub withdrawn: u64,
}

#[contract]
pub struct StreamContract;


#[allow(clippy::manual_saturating_arithmetic)]
fn compute_pending_payout(env: &Env, config: &StreamConfig) -> u64 {
    let current_time = env.ledger().timestamp();
    if current_time <= config.start_time {
        return 0;
    }
    let elapsed_time = current_time
        .checked_sub(config.start_time)
        .expect("Current time before start time");

    let total_accrued = elapsed_time
        .checked_mul(config.flow_rate)
        .expect("Accrual calculation overflow");

    total_accrued.checked_sub(config.withdrawn).unwrap_or(0)
}

#[contractimpl]
impl StreamContract {
    /// Initializes a continuous funding stream. Locks the current ledger timestamp as the start time to prevent timestamp manipulation.
    pub fn init(env: Env, sender: Address, receiver: Address, token: Address, flow_rate: u64) {
        sender.require_auth();

        let key = StreamKey {
            sender: sender.clone(),
            receiver: receiver.clone(),
        };

        if env.storage().persistent().has(&key) {
            panic!("Stream between sender and receiver already exists. Close it first.");
        }

        let start_time = env.ledger().timestamp();
        // Initialize withdrawn to 0
        let config = StreamConfig {
            token,
            flow_rate,
            start_time,
            withdrawn: 0,
        };

        env.storage().persistent().set(&key, &config);
        storage::extend_persistent_ttl(&env, &key);

        env.events()
            .publish((symbol_short!("init"), sender, receiver), flow_rate);
    }

    pub fn get_stream(env: Env, sender: Address, receiver: Address) -> StreamConfig {
        let key = StreamKey { sender, receiver };
        storage::extend_persistent_ttl(&env, &key);
        env.storage()
            .persistent()
            .get(&key)
            .expect("Stream does not exist")
    }

    /// Dynamically calculates accrued tokens. Uses lazy evaluation (elapsed time * flow rate) to avoid state bloat and unnecessary ledger I/O.
    pub fn get_balance(env: Env, sender: Address, receiver: Address) -> u64 {
        let config = Self::get_stream(env.clone(), sender, receiver);
        compute_pending_payout(&env, &config)
    }

    /// Executes a secure withdrawal. Implements the Checks-Effects-Interactions pattern to prevent re-entrancy attacks and double-spends.
    pub fn claim(env: Env, sender: Address, receiver: Address) {
        // Only the receiver can initiate a claim
        receiver.require_auth();

        let mut config = Self::get_stream(env.clone(), sender.clone(), receiver.clone());
        let claimable = Self::get_balance(env.clone(), sender.clone(), receiver.clone());

        if claimable == 0 {
            panic!("No tokens available to claim");
        }

        // SECURITY: Checks-Effects-Interactions Pattern
        // 1. Update state FIRST to prevent re-entrancy
        config.withdrawn = config
            .withdrawn
            .checked_add(claimable)
            .expect("Withdrawn overflow");
        let key = StreamKey {
            sender: sender.clone(),
            receiver: receiver.clone(),
        };
        env.storage().persistent().set(&key, &config);

        // 2. Interact with external contract LAST
        // This requires the sender to have granted an allowance to the Stream Contract
        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&sender, &receiver, &(claimable as i128));

        env.events()
            .publish((symbol_short!("claim"), sender, receiver), claimable);
    }

    /// Cancels an active stream. Settles any remaining accrued balance to the receiver,
    /// requires authorization from the sender, and removes the stream configuration from persistent storage.
    pub fn cancel_stream(env: Env, sender: Address, receiver: Address) {
        sender.require_auth();

        let key = StreamKey {
            sender: sender.clone(),
            receiver: receiver.clone(),
        };
        let config: StreamConfig = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Stream does not exist");

        let pending_payout = compute_pending_payout(&env, &config);

        // Checks-Effects-Interactions: Remove stream first to prevent re-entrancy / double-cancel
        env.storage().persistent().remove(&key);

        // Settle remaining accrued tokens if available
        if pending_payout > 0 {
            let token_client = token::Client::new(&env, &config.token);
            token_client.transfer(&sender, &receiver, &(pending_payout as i128));
        }

        // Emit cancellation event
        env.events()
            .publish((symbol_short!("cancel"), sender, receiver), pending_payout);
    }
}
