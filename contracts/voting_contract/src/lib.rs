use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::{env, near_bindgen, require, AccountId, Timestamp};

pub mod consts;
pub mod storage;
pub mod types;
pub mod views;

#[cfg(test)]
pub mod test_utils;

use consts::*;
use storage::StorageKey;
use types::EncryptedVote;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    votes: Vector<EncryptedVote>,

    candidate_weights: UnorderedMap<AccountId, u64>,

    relayer: AccountId,
    end_time_in_ms: Timestamp,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(relayer: AccountId, end_time_in_ms: Timestamp) -> Self {
        Contract {
            votes: Vector::new(StorageKey::Votes),
            candidate_weights: UnorderedMap::new(StorageKey::CandidatesWeights),
            relayer,
            end_time_in_ms,
        }
    }

    #[payable]
    pub fn send_encrypted_votes(&mut self, votes: Vec<EncryptedVote>) {
        let storage_start = env::storage_usage();
        require!(
            env::block_timestamp_ms() < self.end_time_in_ms,
            VOTING_PHASE_OVER
        );
        self.assert_relayer();

        self.votes.extend(votes);
        require!(
            common_contracts::finalize_storage_check(storage_start, 0),
            DEPOSIT_NOT_ENOUGH
        );
    }

    #[payable]
    pub fn sumbit_results(&mut self, results: Vec<(AccountId, u64)>) {
        let storage_start = env::storage_usage();
        println!("{}, {}", env::block_timestamp_ms(), self.end_time_in_ms);

        require!(
            env::block_timestamp_ms() > self.end_time_in_ms,
            VOTING_PHASE_IN_PROGRESS
        );
        self.assert_relayer();

        self.candidate_weights.extend(results);

        require!(
            common_contracts::finalize_storage_check(storage_start, 0),
            DEPOSIT_NOT_ENOUGH
        );
    }

    fn assert_relayer(&self) {
        require!(
            env::predecessor_account_id() == self.relayer,
            consts::RELAYER_ONLY
        );
    }
}

#[cfg(test)]
mod relayer_tests {
    use near_sdk::{testing_env, NearToken};

    use crate::{test_utils::*, types::EncryptedVote};

    #[test]
    fn can_init_contract() {
        let (context, contract) = setup_ctr();
        testing_env!(context.clone());
        assert_eq!(contract.get_relayer(), relayer());
        assert_eq!(contract.get_end_time(), end_time());
    }

    #[test]
    fn can_send_encrypted_votes() {
        let (mut context, mut contract) = setup_ctr();
        context.predecessor_account_id = relayer();
        context.attached_deposit = NearToken::from_near(1);
        testing_env!(context.clone());

        let votes: Vec<_> = vec![
            EncryptedVote {
                vote: "vote1".to_string(),
                pubkey: [1; 64],
            },
            EncryptedVote {
                vote: "vote2".to_string(),
                pubkey: [2; 64],
            },
        ];

        contract.send_encrypted_votes(votes.clone());

        assert_eq!(contract.get_votes(0, 10), votes);
    }

    #[test]
    fn can_submit_results() {
        let (mut context, mut contract) = setup_ctr();
        context.predecessor_account_id = relayer();
        context.attached_deposit = NearToken::from_near(1);
        context.block_timestamp = (end_time() + 1) * MSECOND;
        testing_env!(context.clone());

        let results: Vec<_> = vec![(acc(1), 1), (acc(2), 2), (acc(3), 3)];

        contract.sumbit_results(results.clone());

        assert_eq!(contract.get_candidate_weights(0, 10), results);
    }

    #[test]
    #[should_panic(expected = "Only relayer can call this method")]
    fn anybody_cant_add_votes() {
        let (mut context, mut contract) = setup_ctr();
        context.predecessor_account_id = acc(1);
        context.attached_deposit = NearToken::from_near(1);
        testing_env!(context.clone());

        let votes: Vec<_> = vec![
            EncryptedVote {
                vote: "vote1".to_string(),
                pubkey: [1; 64],
            },
            EncryptedVote {
                vote: "vote2".to_string(),
                pubkey: [2; 64],
            },
        ];

        contract.send_encrypted_votes(votes.clone());
    }

    #[test]
    #[should_panic(expected = "Only relayer can call this method")]
    fn anybody_cant_submit_results() {
        let (mut context, mut contract) = setup_ctr();
        context.predecessor_account_id = acc(1);
        context.attached_deposit = NearToken::from_near(1);
        context.block_timestamp = (end_time() + 1) * MSECOND;
        testing_env!(context.clone());

        let results: Vec<_> = vec![(acc(1), 1), (acc(2), 2), (acc(3), 3)];

        contract.sumbit_results(results.clone());
    }

    #[test]
    #[should_panic(expected = "Voting phase is over")]
    fn cant_add_votes_after_voting_phase() {
        let (mut context, mut contract) = setup_ctr();
        context.predecessor_account_id = relayer();
        context.attached_deposit = NearToken::from_near(1);
        context.block_timestamp = (end_time() + 1) * MSECOND;
        testing_env!(context.clone());

        let votes: Vec<_> = vec![
            EncryptedVote {
                vote: "vote1".to_string(),
                pubkey: [1; 64],
            },
            EncryptedVote {
                vote: "vote2".to_string(),
                pubkey: [2; 64],
            },
        ];

        contract.send_encrypted_votes(votes.clone());
    }

    #[test]
    #[should_panic(expected = "Voting phase is in progress")]
    fn cant_submit_results_before_voting_phase() {
        let (mut context, mut contract) = setup_ctr();
        context.predecessor_account_id = relayer();
        context.attached_deposit = NearToken::from_near(1);
        testing_env!(context.clone());

        let results: Vec<_> = vec![(acc(1), 1), (acc(2), 2), (acc(3), 3)];

        contract.sumbit_results(results.clone());
    }

    #[test]
    #[should_panic(expected = "Deposit is not enough to cover the storage cost")]
    fn cant_add_votes_with_insufficient_deposit() {
        let (mut context, mut contract) = setup_ctr();
        context.predecessor_account_id = relayer();
        context.attached_deposit = NearToken::from_near(0);
        testing_env!(context.clone());

        let votes: Vec<_> = vec![
            EncryptedVote {
                vote: "vote1".to_string(),
                pubkey: [1; 64],
            },
            EncryptedVote {
                vote: "vote2".to_string(),
                pubkey: [2; 64],
            },
        ];

        contract.send_encrypted_votes(votes.clone());
    }
}
