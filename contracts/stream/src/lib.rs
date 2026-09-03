#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, symbol_short};

mod storage;
// Link the test module (only compiled when running `cargo test`)
#[cfg(test)]
mod test;

#[contract]
pub struct StreamContract;

#[contractimpl]
impl StreamContract {
    /// Initializes a continuous funding stream and secures its state TTL.
    pub fn init(env: Env, sender: Address, receiver: Address, token: Address, flow_rate: u64) {
        sender.require_auth();
        
        env.storage().instance().set(&symbol_short!("receiver"), &receiver);
        env.storage().instance().set(&symbol_short!("token"), &token);
        env.storage().instance().set(&symbol_short!("rate"), &flow_rate);

        // CRITICAL: Protect the newly initialized state from early archival
        storage::extend_instance_ttl(&env);
    }

    /// Read-only getter to verify the current flow rate, bumping TTL to keep active reads alive.
    pub fn get_rate(env: Env) -> u64 {
        // Active reads indicate the stream is alive; bump the TTL to prevent archival
        storage::extend_instance_ttl(&env);
        env.storage().instance().get(&symbol_short!("rate")).unwrap_or(0)
    }
}
