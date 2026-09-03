#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Map};

#[test]
fn test_valid_split_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, SplitContract);
    let client = SplitContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let maintainer_one = Address::generate(&env);
    let maintainer_two = Address::generate(&env);

    let mut shares: Map<Address, u32> = Map::new(&env);
    shares.set(maintainer_one.clone(), 6_000); // 60%
    shares.set(maintainer_two.clone(), 4_000); // 40%

    // Total is exactly 10,000 (100%), so this should succeed
    client.init(&admin, &token, &shares);

    let stored_shares = client.get_shares();
    assert_eq!(stored_shares.get(maintainer_one).unwrap(), 6_000);
    assert_eq!(stored_shares.get(maintainer_two).unwrap(), 4_000);
}

#[test]
#[should_panic(expected = "Shares exceed 100 percent (10,000 basis points)")]
fn test_invalid_split_exceeds_100_percent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, SplitContract);
    let client = SplitContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let maintainer_one = Address::generate(&env);
    let maintainer_two = Address::generate(&env);

    let mut shares: Map<Address, u32> = Map::new(&env);
    shares.set(maintainer_one, 6_000); // 60%
    shares.set(maintainer_two, 5_000); // 50%

    // Total is 11,000 (110%), so this MUST panic
    client.init(&admin, &token, &shares);
}
