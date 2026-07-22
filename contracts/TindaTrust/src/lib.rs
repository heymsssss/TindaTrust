#![no_std]
//! TindaTrust — an on-chain "utang" (store credit) ledger for sari-sari stores.
//!
//! Real-world flow: Aling Rosa runs a sari-sari store in Marilao, Bulacan.
//! Regulars buy on credit ("utang, saglit lang") and pay it off later, but
//! paper notebooks get lost, disputed, or "forgotten" by customers who move
//! away. TindaTrust puts each debt and repayment on-chain so both sides can
//! trust the running balance, while settlement still happens in USDC/cash.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

/// Storage keys.
/// - `Owner` holds the address of the store owner (the only one allowed to
///   record new utang).
/// - `Debt(customer)` holds that customer's current outstanding balance.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Owner,
    Debt(Address),
}

#[contract]
pub struct TindaTrustContract;

#[contractimpl]
impl TindaTrustContract {
    /// Set up the store owner. Must be called once, right after deployment.
    /// Only the owner address itself can authorize this call.
    pub fn initialize(env: Env, owner: Address) {
        owner.require_auth();

        if env.storage().instance().has(&DataKey::Owner) {
            panic!("TindaTrust: already initialized");
        }

        env.storage().instance().set(&DataKey::Owner, &owner);
    }

    /// Store owner records a new utang (credit purchase) for a customer.
    /// This is the "on-chain action" half of the MVP flow: customer buys on
    /// credit at the counter -> owner taps "record utang" in the app -> this
    /// function runs and the customer's balance goes up.
    pub fn record_utang(env: Env, customer: Address, amount: i128) -> i128 {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("TindaTrust: not initialized");
        owner.require_auth();

        if amount <= 0 {
            panic!("TindaTrust: amount must be positive");
        }

        let key = DataKey::Debt(customer.clone());
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_balance = current + amount;

        env.storage().persistent().set(&key, &new_balance);
        new_balance
    }

    /// Customer repays some or all of their utang. Requires the customer's
    /// own authorization (they're the one paying). This is the "result" half
    /// of the MVP flow: customer pays -> balance goes down -> both sides can
    /// see the ledger is clean.
    pub fn repay(env: Env, customer: Address, amount: i128) -> i128 {
        customer.require_auth();

        if amount <= 0 {
            panic!("TindaTrust: amount must be positive");
        }

        let key = DataKey::Debt(customer.clone());
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);

        if amount > current {
            panic!("TindaTrust: repay amount exceeds outstanding debt");
        }

        let new_balance = current - amount;
        env.storage().persistent().set(&key, &new_balance);
        new_balance
    }

    /// Read-only: check a customer's current outstanding balance.
    pub fn get_balance(env: Env, customer: Address) -> i128 {
        let key = DataKey::Debt(customer);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Read-only: check who the store owner is.
    pub fn get_owner(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("TindaTrust: not initialized")
    }
}

#[cfg(test)]
mod test;
