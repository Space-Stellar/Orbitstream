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
}

#[contract]
pub struct StreamContract;

#[contractimpl]
impl StreamContract {
    /// Initializes a continuous funding stream uniquely identified by the sender and receiver.
    pub fn init(env: Env, sender: Address, receiver: Address, token: Address, flow_rate: u64) {
        sender.require_auth();
        
        let key = StreamKey { sender: sender.clone(), receiver: receiver.clone() };
        
        // SECURITY: Prevent silent overwrites of active streams
        if env.storage().persistent().has(&key) {
            panic!("Stream between sender and receiver already exists. Close it first.");
        }

        let config = StreamConfig { token, flow_rate };
        
        // Write to isolated persistent storage
        env.storage().persistent().set(&key, &config);
        
        // Protect the individual stream from archival
        storage::extend_persistent_ttl(&env, &key);
        
        // Emit the indexing event
        env.events().publish((symbol_short!("init"), sender, receiver), flow_rate);
    }

    /// Read-only getter to verify a specific stream configuration.
    pub fn get_stream(env: Env, sender: Address, receiver: Address) -> StreamConfig {
        let key = StreamKey { sender, receiver };
        
        // Active reads keep the specific stream alive
        storage::extend_persistent_ttl(&env, &key);
        
        env.storage().persistent().get(&key).expect("Stream does not exist")
    }
}
