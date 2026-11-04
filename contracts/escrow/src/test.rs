#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

fn setup_test() -> (
    Env,
    EscrowVaultContractClient<'static>,
    Address,
    Address,
    token::Client<'static>,
    token::StellarAssetClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Set a known ledger time
    env.ledger().set_timestamp(12345);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);

    // Deploy test token
    let admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = token::Client::new(&env, &token_contract.address());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_contract.address());

    // Mint tokens to sender
    token_admin_client.mint(&sender, &1_000_000);

    // Deploy escrow contract
    let contract_id = env.register_contract(None, EscrowVaultContract);
    let client = EscrowVaultContractClient::new(&env, &contract_id);

    (
        env,
        client,
        sender,
        receiver,
        token_client,
        token_admin_client,
    )
}

#[test]
fn test_successful_deposit() {
    let (env, client, sender, receiver, token_client, _) = setup_test();

    // Deposit 10,000 tokens with a flow rate of 100 tokens per second
    let amount = 10_000;
    let flow_rate = 100;

    client.deposit(
        &sender,
        &receiver,
        &token_client.address,
        &amount,
        &flow_rate,
    );

    // Contract should hold the tokens
    assert_eq!(token_client.balance(&client.address), 10_000);
    // Sender should have 1_000_000 - 10_000 = 990_000
    assert_eq!(token_client.balance(&sender), 990_000);

    env.budget().print();
}

#[test]
fn test_successful_claim() {
    let (env, client, sender, receiver, token_client, _) = setup_test();

    let amount = 10_000;
    let flow_rate = 100;

    let stream_id = client.deposit(
        &sender,
        &receiver,
        &token_client.address,
        &amount,
        &flow_rate,
    );

    // Fast forward 50 seconds
    env.ledger().set_timestamp(12345 + 50);

    client.claim(&stream_id);

    // 50 seconds * 100 tokens/sec = 5000 tokens should be claimed
    assert_eq!(token_client.balance(&receiver), 5_000);

    // Contract balance should be 5_000
    assert_eq!(token_client.balance(&client.address), 5_000);

    env.budget().print();
}

#[test]
fn test_cancel_stream_by_receiver() {
    let (env, client, sender, receiver, token_client, _) = setup_test();

    let amount = 10_000;
    let flow_rate = 100;

    let stream_id = client.deposit(
        &sender,
        &receiver,
        &token_client.address,
        &amount,
        &flow_rate,
    );

    // Fast forward 50 seconds
    env.ledger().set_timestamp(12345 + 50);

    // Receiver cancels the stream
    client.cancel_stream(&receiver, &stream_id);

    // Accrued 5,000 tokens should go to receiver
    assert_eq!(token_client.balance(&receiver), 5_000);

    // Remainder 5,000 tokens should return to sender
    // Sender originally had 990_000 after deposit, now gets 5_000 back = 995_000
    assert_eq!(token_client.balance(&sender), 995_000);

    // Contract balance should be 0
    assert_eq!(token_client.balance(&client.address), 0);

    env.budget().print();
}
