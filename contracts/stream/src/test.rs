#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, token};

#[test]
fn test_claim_execution() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth(); // Bypasses auth checks for testing

    let contract_id = env.register_contract(None, StreamContract);
    let client = StreamContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    
    // Register a mock Stellar token and clients
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    let token_client = token::Client::new(&env, &token);

    // Mint 100,000 tokens to the sender's wallet
    token_admin_client.mint(&sender, &100_000);

    // Start stream at ledger timestamp 100,000
    env.ledger().set_timestamp(100_000);
    let flow_rate = 50_u64;
    client.init(&sender, &receiver, &token, &flow_rate);

    // Fast forward 10 seconds (Accrued = 500)
    env.ledger().set_timestamp(100_010);

    // Receiver claims
    client.claim(&sender, &receiver);

    // Verify tokens were physically transferred
    let receiver_balance = token_client.balance(&receiver);
    assert_eq!(receiver_balance, 500);

    // Verify the stream state tracked the withdrawal
    let stream = client.get_stream(&sender, &receiver);
    assert_eq!(stream.withdrawn, 500);
    
    // Remaining claimable balance should immediately reflect as 0
    assert_eq!(client.get_balance(&sender, &receiver), 0);
}
