#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Events, Address, Env};

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

    assert_eq!(client.get_rate(), 1_000_u64);
    assert_eq!(env.events().all().len(), 1);
}

#[test]
#[should_panic(expected = "soroban_sdk::require_auth")]
fn test_initialization_fails_without_auth() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, StreamContract);
    let client = StreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let token = Address::generate(&env);
    let flow_rate = 1_000_u64;

    client.init(&sender, &receiver, &token, &flow_rate);
}
