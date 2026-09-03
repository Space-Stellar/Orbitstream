#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_successful_initialization() {
    // 1. Setup the test environment
    let env = Env::default();
    env.mock_all_auths(); // Simulate valid cryptographic signatures for all addresses

    // 2. Register the contract and instantiate a client
    let contract_id = env.register_contract(None, StreamContract);
    let client = StreamContractClient::new(&env, &contract_id);

    // 3. Generate mock addresses for the test
    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let token = Address::generate(&env);
    let flow_rate = 1_000_u64;

    // 4. Execute the state-mutating function
    client.init(&sender, &receiver, &token, &flow_rate);

    // 5. Assert the state changed correctly
    assert_eq!(client.get_rate(), 1_000_u64);
}

#[test]
#[should_panic(expected = "soroban_sdk::require_auth")]
fn test_initialization_fails_without_auth() {
    let env = Env::default();
    // CRITICAL: Intentionally omitting `env.mock_all_auths()` to test security
    
    let contract_id = env.register_contract(None, StreamContract);
    let client = StreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let token = Address::generate(&env);
    let flow_rate = 1_000_u64;

    // This MUST panic because the sender's signature is missing, proving require_auth() works.
    client.init(&sender, &receiver, &token, &flow_rate);
}
