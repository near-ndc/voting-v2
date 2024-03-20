use std::str::FromStr;

use near_sdk::NearToken;
use near_sdk::{test_utils::VMContextBuilder, testing_env, AccountId, VMContext};

/// 1ms in nano seconds
const MSECOND: u64 = 1_000_000;

// In milliseconds
const START: u64 = 60 * 5 * 1000;

use crate::{types::VoteConfig, Contract};

fn acc(idx: u8) -> AccountId {
    AccountId::from_str(&format!("user-{}.near", idx)).unwrap()
}

fn admin() -> AccountId {
    AccountId::from_str("admin.near").unwrap()
}

fn default_vote_config() -> VoteConfig {
    VoteConfig {
        threshold_in_nears: 100,
        activity_reward_in_votes: 10,
    }
}

fn setup_ctr(attach_deposit: u128) -> (VMContext, Contract) {
    let mut context = VMContextBuilder::new().build();

    let contract = Contract::new(admin(), default_vote_config());
    context.block_timestamp = START * MSECOND;
    context.predecessor_account_id = acc(0);
    context.attached_deposit = NearToken::from_millinear(attach_deposit);

    testing_env!(context.clone());
    (context, contract)
}

#[test]
fn create_contract() {
    let (_context, contract) = setup_ctr(0);
    assert_eq!(contract.get_vote_config(), default_vote_config());
    assert_eq!(contract.get_admin(), admin());
}

#[test]
fn admin_can_change_vote_config() {
    let (mut context, mut contract) = setup_ctr(0);
    let new_vote_config = VoteConfig {
        threshold_in_nears: 200,
        activity_reward_in_votes: 20,
    };
    assert_eq!(contract.get_vote_config(), default_vote_config());

    context.predecessor_account_id = admin();
    testing_env!(context.clone());

    contract.set_vote_config(new_vote_config);

    assert_eq!(contract.get_vote_config(), new_vote_config);
}

#[test]
#[should_panic(expected = "Not authorized")]
fn non_admin_cannot_change_vote_config() {
    let (mut context, mut contract) = setup_ctr(0);
    let new_vote_config = VoteConfig {
        threshold_in_nears: 200,
        activity_reward_in_votes: 20,
    };
    assert_eq!(contract.get_vote_config(), default_vote_config());

    context.predecessor_account_id = acc(0);
    testing_env!(context.clone());

    contract.set_vote_config(new_vote_config);
}
