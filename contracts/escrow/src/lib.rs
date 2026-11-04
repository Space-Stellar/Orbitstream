#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    InvalidFlowRate = 2,
    StreamNotFound = 3,
    Unauthorized = 4,
    MathOverflow = 5,
    InsufficientUnaccrued = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    pub sender: Address,
    pub receiver: Address,
    pub token: Address,
    pub total_deposited: i128,
    pub flow_rate: i128,
    pub start_time: u64,
    pub claimed_amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Stream(u64),
    StreamCount,
}

const DAY_IN_LEDGERS: u32 = 17280;
const BUMP_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;
const BUMP_TO: u32 = 120 * DAY_IN_LEDGERS;

fn extend_stream_ttl(env: &Env, stream_id: u64) {
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Stream(stream_id), BUMP_THRESHOLD, BUMP_TO);
}

fn extend_count_ttl(env: &Env) {
    env.storage().instance().extend_ttl(BUMP_THRESHOLD, BUMP_TO);
}

#[contract]
pub struct EscrowVaultContract;

#[contractimpl]
impl EscrowVaultContract {
    pub fn deposit(
        env: Env,
        sender: Address,
        receiver: Address,
        token: Address,
        amount: i128,
        flow_rate: i128,
    ) -> Result<u64, Error> {
        sender.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if flow_rate <= 0 {
            return Err(Error::InvalidFlowRate);
        }

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &amount);

        let stream_count = env
            .storage()
            .instance()
            .get(&DataKey::StreamCount)
            .unwrap_or(0u64);

        let stream_id = stream_count.checked_add(1).ok_or(Error::MathOverflow)?;
        env.storage()
            .instance()
            .set(&DataKey::StreamCount, &stream_id);
        extend_count_ttl(&env);

        let current_time = env.ledger().timestamp();

        let stream = Stream {
            sender,
            receiver,
            token,
            total_deposited: amount,
            flow_rate,
            start_time: current_time,
            claimed_amount: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Stream(stream_id), &stream);
        extend_stream_ttl(&env, stream_id);

        Ok(stream_id)
    }

    pub fn withdraw(env: Env, stream_id: u64, amount: i128) -> Result<(), Error> {
        let mut stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .ok_or(Error::StreamNotFound)?;

        stream.sender.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let current_time = env.ledger().timestamp();
        let elapsed_time = current_time
            .checked_sub(stream.start_time)
            .ok_or(Error::MathOverflow)?;
        let elapsed_i128 = elapsed_time as i128;

        let accrued = elapsed_i128
            .checked_mul(stream.flow_rate)
            .ok_or(Error::MathOverflow)?;

        let total_accrued = if accrued > stream.total_deposited {
            stream.total_deposited
        } else {
            accrued
        };

        let unaccrued = stream
            .total_deposited
            .checked_sub(total_accrued)
            .ok_or(Error::MathOverflow)?;

        if amount > unaccrued {
            return Err(Error::InsufficientUnaccrued);
        }

        stream.total_deposited = stream
            .total_deposited
            .checked_sub(amount)
            .ok_or(Error::MathOverflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::Stream(stream_id), &stream);
        extend_stream_ttl(&env, stream_id);

        let token_client = token::Client::new(&env, &stream.token);
        token_client.transfer(&env.current_contract_address(), &stream.sender, &amount);

        Ok(())
    }

    pub fn claim(env: Env, stream_id: u64) -> Result<(), Error> {
        let mut stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .ok_or(Error::StreamNotFound)?;

        let current_time = env.ledger().timestamp();
        let elapsed_time = current_time
            .checked_sub(stream.start_time)
            .ok_or(Error::MathOverflow)?;
        let elapsed_i128 = elapsed_time as i128;

        let accrued = elapsed_i128
            .checked_mul(stream.flow_rate)
            .ok_or(Error::MathOverflow)?;

        let total_accrued = if accrued > stream.total_deposited {
            stream.total_deposited
        } else {
            accrued
        };

        let claimable = total_accrued
            .checked_sub(stream.claimed_amount)
            .ok_or(Error::MathOverflow)?;

        if claimable > 0 {
            stream.claimed_amount = stream
                .claimed_amount
                .checked_add(claimable)
                .ok_or(Error::MathOverflow)?;
            env.storage()
                .persistent()
                .set(&DataKey::Stream(stream_id), &stream);
            extend_stream_ttl(&env, stream_id);

            let token_client = token::Client::new(&env, &stream.token);
            token_client.transfer(
                &env.current_contract_address(),
                &stream.receiver,
                &claimable,
            );
        } else {
            extend_stream_ttl(&env, stream_id);
        }

        Ok(())
    }

    pub fn cancel_stream(env: Env, caller: Address, stream_id: u64) -> Result<(), Error> {
        let stream: Stream = env
            .storage()
            .persistent()
            .get(&DataKey::Stream(stream_id))
            .ok_or(Error::StreamNotFound)?;

        caller.require_auth();

        if caller != stream.sender && caller != stream.receiver {
            return Err(Error::Unauthorized);
        }

        let current_time = env.ledger().timestamp();
        let elapsed_time = current_time
            .checked_sub(stream.start_time)
            .ok_or(Error::MathOverflow)?;
        let elapsed_i128 = elapsed_time as i128;

        let accrued = elapsed_i128
            .checked_mul(stream.flow_rate)
            .ok_or(Error::MathOverflow)?;

        let total_accrued = if accrued > stream.total_deposited {
            stream.total_deposited
        } else {
            accrued
        };

        let claimable = total_accrued
            .checked_sub(stream.claimed_amount)
            .ok_or(Error::MathOverflow)?;
        let remainder = stream
            .total_deposited
            .checked_sub(total_accrued)
            .ok_or(Error::MathOverflow)?;

        env.storage()
            .persistent()
            .remove(&DataKey::Stream(stream_id));

        let token_client = token::Client::new(&env, &stream.token);

        if claimable > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &stream.receiver,
                &claimable,
            );
        }
        if remainder > 0 {
            token_client.transfer(&env.current_contract_address(), &stream.sender, &remainder);
        }

        Ok(())
    }
}

mod test;
