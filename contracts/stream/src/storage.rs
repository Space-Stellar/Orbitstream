use soroban_sdk::Env;

// Assuming ~5 seconds per ledger on Stellar Mainnet:
// 30 days ≈ 518,400 ledgers. 
// We will set our threshold to ~7 days (120,960) and extend to ~30 days.
const BUMP_THRESHOLD: u32 = 120_000;
const BUMP_AMOUNT: u32 = 518_400;

/// Extends the TTL of the contract's instance storage if it falls below the threshold.
/// This prevents global configurations (like the default token or global rate) from archiving.
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(BUMP_THRESHOLD, BUMP_AMOUNT);
}

/// Extends the TTL of a specific persistent storage key.
/// Used for individual user stream configurations to ensure active streams stay alive.
pub fn extend_persistent_ttl<K>(env: &Env, key: &K)
where
    K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, BUMP_THRESHOLD, BUMP_AMOUNT);
}
