#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, symbol_short};

mod storage;
#[cfg(test)]
mod test;

#[contract]
pub struct StreamContract;

#[contractimpl]
impl StreamContract {
    pub fn init(env: Env, sender: Address, receiver: Address, token: Address, flow_rate: u64) {
        sender.require_auth();
        
        env.storage().instance().set(&symbol_short!("receiver"), &receiver);
        env.storage().instance().set(&symbol_short!("token"), &token);
        env.storage().instance().set(&symbol_short!("rate"), &flow_rate);

        storage::extend_instance_ttl(&env);
        
        env.events().publish((symbol_short!("init"), sender.clone()), (receiver.clone(), flow_rate));
    }

    pub fn get_rate(env: Env) -> u64 {
        storage::extend_instance_ttl(&env);
        env.storage().instance().get(&symbol_short!("rate")).unwrap_or(0)
    }
}
