use std::str::FromStr;

use near_sdk::{test_utils::VMContextBuilder, testing_env, AccountId, VMContext};
use near_sdk::{NearToken, PublicKey};

/// 1ms in nano seconds
pub const MSECOND: u64 = 1_000_000;

// In milliseconds
pub const START: u64 = 1;

use crate::types::{SnapshotConfig, Status, UserData};
use crate::{types::VoteWeightConfig, Contract};

pub fn acc(idx: u8) -> AccountId {
    AccountId::from_str(&format!("user-{}.near", idx)).unwrap()
}

pub fn pk() -> PublicKey {
    PublicKey::from_str(&"ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap()
}

pub fn admin() -> AccountId {
    AccountId::from_str("admin.near").unwrap()
}

pub fn default_vote_config() -> VoteWeightConfig {
    VoteWeightConfig {
        threshold_in_nears: 100,
        activity_reward_in_votes: 10,
    }
}

pub fn default_snapshot_config() -> SnapshotConfig {
    SnapshotConfig {
        challenge_threshold_in_nears: 100,
        challenge_timeout_in_millis: 60 * 60 * 24 * 7 * 1000,
        registration_timeout_in_millis: 60 * 60 * 24 * 7 * 1000,
    }
}

pub fn move_to_challenge(context: &mut VMContext, contract: &mut Contract) {
    assert!(matches!(contract.get_status(), Status::Initialization(_)));

    context.predecessor_account_id = admin();
    context.block_timestamp = START * MSECOND;
    testing_env!(context.clone());

    contract.start_challenge();
    assert!(matches!(
        contract.get_status(),
        Status::SnapshotChallenge(_)
    ));
    assert_eq!(
        contract.get_end_time(),
        START + contract.get_process_config().challenge_timeout_in_millis
    );
}

pub fn move_to_registration(context: &mut VMContext, contract: &mut Contract) {
    assert!(matches!(
        contract.get_status(),
        Status::SnapshotChallenge(_)
    ));

    context.block_timestamp =
        (START + contract.get_process_config().challenge_timeout_in_millis + 1) * MSECOND;
    testing_env!(context.clone());

    contract.try_move_stage();

    assert!(matches!(contract.get_status(), Status::Registration(_)));
    assert_eq!(
        contract.get_end_time(),
        START
            + contract.get_process_config().challenge_timeout_in_millis
            + contract.get_process_config().registration_timeout_in_millis
    );
}

pub fn move_to_end(context: &mut VMContext, contract: &mut Contract) {
    assert!(matches!(contract.get_status(), Status::Registration(_)));

    context.block_timestamp +=
        contract.get_process_config().registration_timeout_in_millis * MSECOND + 1;
    testing_env!(context.clone());

    contract.try_move_stage();

    assert!(matches!(
        contract.get_status(),
        Status::RegistrationEnded(_)
    ));
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

pub fn setup_ctr() -> (VMContext, Contract) {
    let mut context = VMContextBuilder::new().build();

    let mut contract = Contract::new(admin(), default_vote_config(), default_snapshot_config());
    context.predecessor_account_id = admin();

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
