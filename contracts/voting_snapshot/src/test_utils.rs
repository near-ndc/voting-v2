use std::str::FromStr;

use near_sdk::{test_utils::VMContextBuilder, testing_env, AccountId, VMContext};
use near_sdk::{NearToken, PublicKey};

/// 1ms in nano seconds
pub const MSECOND: u64 = 1_000_000;

// In milliseconds
pub const START: u64 = 60 * 5 * 1000;

use crate::types::UserData;
use crate::{types::VoteConfig, Contract};

pub fn acc(idx: u8) -> AccountId {
    AccountId::from_str(&format!("user-{}.near", idx)).unwrap()
}

pub fn pk() -> PublicKey {
    PublicKey::from_str(&"ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap()
}

pub fn admin() -> AccountId {
    AccountId::from_str("admin.near").unwrap()
}

pub fn default_vote_config() -> VoteConfig {
    VoteConfig {
        threshold_in_nears: 100,
        activity_reward_in_votes: 10,
    }
}

pub fn load_voters() -> Vec<(AccountId, UserData)> {
    vec![
        (
            acc(2),
            UserData {
                stake: NearToken::from_near(2),
                active_months: 2,
            },
        ),
        (
            acc(3),
            UserData {
                stake: NearToken::from_near(3),
                active_months: 3,
            },
        ),
        (
            acc(4),
            UserData {
                stake: NearToken::from_near(4),
                active_months: 4,
            },
        ),
    ]
}

pub fn setup_ctr(attach_deposit: u128) -> (VMContext, Contract) {
    let mut context = VMContextBuilder::new().build();

    let mut contract = Contract::new(admin(), default_vote_config());
    context.block_timestamp = START * MSECOND;
    context.predecessor_account_id = admin();
    context.attached_deposit = NearToken::from_millinear(attach_deposit);

    testing_env!(context.clone());

    contract.bulk_load_voters(vec![(
        acc(1),
        UserData {
            stake: NearToken::from_near(1),
            active_months: 1,
        },
    )]);

    context.predecessor_account_id = acc(1);
    testing_env!(context.clone());

    (context, contract)
}
