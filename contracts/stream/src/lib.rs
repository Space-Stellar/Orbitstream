#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

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
}

#[contract]
pub struct StreamContract;

#[contractimpl]
impl StreamContract {
    /// Initializes a continuous funding stream uniquely identified by the sender and receiver.
    pub fn init(env: Env, sender: Address, receiver: Address, token: Address, flow_rate: u64) {
        sender.require_auth();
        
        let key = StreamKey { sender: sender.clone(), receiver: receiver.clone() };
        
        if env.storage().persistent().has(&key) {
            panic!("Stream between sender and receiver already exists. Close it first.");
        }

        // Capture the exact ledger UNIX timestamp when the stream is created
        let start_time = env.ledger().timestamp();
        let config = StreamConfig { token, flow_rate, start_time };
        
        env.storage().persistent().set(&key, &config);
        storage::extend_persistent_ttl(&env, &key);
        
        env.events().publish((symbol_short!("init"), sender, receiver), flow_rate);
    }

    /// Read-only getter to verify a specific stream configuration.
    pub fn get_stream(env: Env, sender: Address, receiver: Address) -> StreamConfig {
        let key = StreamKey { sender, receiver };
        storage::extend_persistent_ttl(&env, &key);
        env.storage().persistent().get(&key).expect("Stream does not exist")
    }

    /// Dynamically calculates the amount of tokens the receiver has accrued.
    pub fn get_balance(env: Env, sender: Address, receiver: Address) -> u64 {
        let config = Self::get_stream(env.clone(), sender, receiver);
        let current_time = env.ledger().timestamp();
        
        // Prevent underflow if called in the exact same ledger it was created
        if current_time <= config.start_time {
            return 0;
        }
        
        let elapsed_time = current_time - config.start_time;
        
        // elapsed_time (seconds) * flow_rate (tokens per second)
        elapsed_time * config.flow_rate
    }
}
