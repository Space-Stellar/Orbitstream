#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_successful_initialization() {
    let env = Env::default();
    env.mock_all_auths(); 

    let contract_id = env.register_contract(None, StreamContract);
    let client = StreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let token = Address::generate(&env);
    let flow_rate = 1_000_u64;

    client.init(&sender, &receiver, &token, &flow_rate);

    let config = client.get_stream(&sender, &receiver);
    assert_eq!(config.flow_rate, 1_000_u64);
    assert_eq!(config.token, token);
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
    let flow_rate = 1_000_u64;

    client.init(&sender, &receiver, &token, &flow_rate);
    // This second call MUST panic
    client.init(&sender, &receiver, &token, &flow_rate);
}
