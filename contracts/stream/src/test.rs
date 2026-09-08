#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

#[test]
fn test_streaming_math_over_time() {
    let env = Env::default();
    env.mock_all_auths(); 

    // Start the blockchain at a specific UNIX timestamp (e.g., 100,000)
    env.ledger().set_timestamp(100_000);

    let contract_id = env.register_contract(None, StreamContract);
    let client = StreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let token = Address::generate(&env);
    // 50 tokens per second
    let flow_rate = 50_u64;

    client.init(&sender, &receiver, &token, &flow_rate);

    // Fast forward the blockchain by exactly 10 seconds
    env.ledger().set_timestamp(100_010);

    let accrued_balance = client.get_balance(&sender, &receiver);
    
    // 10 seconds * 50 tokens/sec = 500 tokens
    assert_eq!(accrued_balance, 500);
}

#[test]
#[should_panic(expected = "Stream between sender and receiver already exists")]
fn test_prevents_silent_overwrite() {
    let env = Env::default();
    env.mock_all_auths(); 

    let contract_id = env.register_contract(None, StreamContract);
    let client = StreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let token = Address::generate(&env);
    let flow_rate = 50_u64;

    client.init(&sender, &receiver, &token, &flow_rate);
    client.init(&sender, &receiver, &token, &flow_rate);
}
