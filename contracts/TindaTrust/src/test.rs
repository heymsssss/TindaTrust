#![cfg(test)]

use super::{TindaTrustContract, TindaTrustContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Test 1 — Happy path (initialization).
/// Verifies the contract can be deployed and initialized with a store owner,
/// and that the owner is correctly retrievable afterward.
#[test]
fn test_initialize_sets_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TindaTrustContract);
    let client = TindaTrustContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.initialize(&owner);

    assert_eq!(client.get_owner(), owner);
}

/// Test 2 — Happy path (full MVP transaction flow).
/// Aling Rosa records a utang for a customer, then the customer repays it
/// in full. End-to-end: record -> repay -> balance zero.
#[test]
fn test_record_and_full_repay_happy_path() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TindaTrustContract);
    let client = TindaTrustContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let customer = Address::generate(&env);
    client.initialize(&owner);

    client.record_utang(&customer, &150);
    assert_eq!(client.get_balance(&customer), 150);

    client.repay(&customer, &150);
    assert_eq!(client.get_balance(&customer), 0);
}

/// Test 3 — State verification.
/// After recording utang, the persistent storage balance for that specific
/// customer must exactly reflect the recorded amount (no cross-contamination
/// between customers).
#[test]
fn test_multiple_customers_have_independent_balances() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TindaTrustContract);
    let client = TindaTrustContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let customer_a = Address::generate(&env);
    let customer_b = Address::generate(&env);
    client.initialize(&owner);

    client.record_utang(&customer_a, &75);
    client.record_utang(&customer_b, &40);

    assert_eq!(client.get_balance(&customer_a), 75);
    assert_eq!(client.get_balance(&customer_b), 40);
}

/// Test 4 — State verification (partial repayment).
/// A customer with an existing debt repays only part of it; the remaining
/// balance in storage must equal debt minus the partial payment.
#[test]
fn test_partial_repay_leaves_correct_remaining_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TindaTrustContract);
    let client = TindaTrustContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let customer = Address::generate(&env);
    client.initialize(&owner);

    client.record_utang(&customer, &200);
    client.repay(&customer, &80);

    assert_eq!(client.get_balance(&customer), 120);
}

/// Test 5 — Edge case / failure scenario.
/// A customer tries to repay more than they actually owe. The contract must
/// reject this rather than silently allowing a negative balance.
#[test]
#[should_panic(expected = "TindaTrust: repay amount exceeds outstanding debt")]
fn test_repay_more_than_owed_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TindaTrustContract);
    let client = TindaTrustContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let customer = Address::generate(&env);
    client.initialize(&owner);

    client.record_utang(&customer, &50);
    client.repay(&customer, &51);
}
