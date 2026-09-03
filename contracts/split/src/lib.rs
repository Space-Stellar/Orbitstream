#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, symbol_short};

#[contract]
pub struct SplitContract;

#[contractimpl]
impl SplitContract {
    /// Initializes a split configuration. 
    /// `shares` is a Map where the Key is the Maintainer Address, and Value is their percentage (in basis points, where 10000 = 100%).
    pub fn init(env: Env, admin: Address, token: Address, shares: Map<Address, u32>) {
        // Enforce strict access control
        admin.require_auth();
        
        // Validate that basis points do not exceed 100% (10,000)
        let mut total_shares: u32 = 0;
        for (_, share) in shares.iter() {
            total_shares += share;
        }
        
        if total_shares > 10_000 {
            panic!("Shares exceed 100 percent (10,000 basis points)");
        }

        // Store configuration securely in instance storage
        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("token"), &token);
        env.storage().instance().set(&symbol_short!("shares"), &shares);
        
        // Note: TTL bump logic goes here
    }
}
