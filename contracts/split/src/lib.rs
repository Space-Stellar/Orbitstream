#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, symbol_short};

#[contract]
pub struct SplitContract;

#[contractimpl]
impl SplitContract {
    /// Initializes a split configuration. 
    /// `shares` is a Map where Key = Maintainer Address, Value = basis points (10000 = 100%).
    pub fn init(env: Env, admin: Address, token: Address, shares: Map<Address, u32>) {
        admin.require_auth();
        
        let mut total_shares: u32 = 0;
        for (_, share) in shares.iter() {
            total_shares += share;
        }
        
        if total_shares > 10_000 {
            panic!("Shares exceed 100 percent (10,000 basis points)");
        }

        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("token"), &token);
        env.storage().instance().set(&symbol_short!("shares"), &shares);
    }

    /// Read-only getter to verify the current split configuration.
    pub fn get_shares(env: Env) -> Map<Address, u32> {
        env.storage().instance().get(&symbol_short!("shares")).unwrap_or(Map::new(&env))
    }
}

#[cfg(test)]
mod test;
