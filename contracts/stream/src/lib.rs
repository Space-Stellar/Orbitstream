#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, token};

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

#[contractimpl]
impl StreamContract {
    /// Initializes a continuous funding stream. Locks the current ledger timestamp as the start time to prevent timestamp manipulation.
    pub fn init(env: Env, sender: Address, receiver: Address, token: Address, flow_rate: u64) {
        sender.require_auth();
        
        let key = StreamKey { sender: sender.clone(), receiver: receiver.clone() };
        
        if env.storage().persistent().has(&key) {
            panic!("Stream between sender and receiver already exists. Close it first.");
        }

        let start_time = env.ledger().timestamp();
        // Initialize withdrawn to 0
        let config = StreamConfig { token, flow_rate, start_time, withdrawn: 0 };
        
        env.storage().persistent().set(&key, &config);
        storage::extend_persistent_ttl(&env, &key);
        
        env.events().publish((symbol_short!("init"), sender, receiver), flow_rate);
    }

    pub fn get_stream(env: Env, sender: Address, receiver: Address) -> StreamConfig {
        let key = StreamKey { sender, receiver };
        storage::extend_persistent_ttl(&env, &key);
        env.storage().persistent().get(&key).expect("Stream does not exist")
    }

    /// Dynamically calculates accrued tokens. Uses lazy evaluation (elapsed time * flow rate) to avoid state bloat and unnecessary ledger I/O.
    pub fn get_balance(env: Env, sender: Address, receiver: Address) -> u64 {
        let config = Self::get_stream(env.clone(), sender, receiver);
        let current_time = env.ledger().timestamp();
        
        if current_time <= config.start_time {
            return 0;
        }
        
        let elapsed_time = current_time - config.start_time;
        let total_accrued = elapsed_time * config.flow_rate;
        
        // Subtract what they have already claimed
        total_accrued - config.withdrawn
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
        config.withdrawn += claimable;
        let key = StreamKey { sender: sender.clone(), receiver: receiver.clone() };
        env.storage().persistent().set(&key, &config);
        
        // 2. Interact with external contract LAST
        // This requires the sender to have granted an allowance to the Stream Contract
        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&sender, &receiver, &(claimable as i128));
        
        env.events().publish((symbol_short!("claim"), sender, receiver), claimable);
    }
}
