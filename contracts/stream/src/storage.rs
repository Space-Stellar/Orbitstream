use soroban_sdk::Env;

const BUMP_THRESHOLD: u32 = 120_000;
const BUMP_AMOUNT: u32 = 518_400;


pub fn extend_persistent_ttl<K>(env: &Env, key: &K)
where
    K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, BUMP_THRESHOLD, BUMP_AMOUNT);
}
