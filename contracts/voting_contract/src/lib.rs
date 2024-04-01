use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::env::panic_str;
use near_sdk::json_types::Base64VecU8;
use near_sdk::{env, near_bindgen, require, AccountId, Timestamp};
use secp256k1::SecretKey;

pub mod consts;
pub mod crypto;
pub mod ext;
pub mod storage;
pub mod types;
pub mod views;

use consts::*;
use storage::StorageKey;
use types::Vote;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    votes: Vector<Vote>,

    candidate_weights: UnorderedMap<AccountId, u64>,

    relayer: AccountId,
    snapshot: AccountId,

    private_key: Option<[u8; 32]>,
    end_time: Timestamp,

    latest_decrypted: usize,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(relayer: AccountId, snapshot: AccountId, end_time: Timestamp) -> Self {
        Contract {
            votes: Vector::new(StorageKey::Votes),
            candidate_weights: UnorderedMap::new(StorageKey::CandidatesWeights),
            relayer,
            snapshot,
            private_key: None,
            end_time,
            latest_decrypted: 0,
        }
    }

    pub fn send_encrypted_votes(&mut self) {
        require!(env::block_timestamp_ms() < self.end_time, VOTING_PHASE_OVER);
        self.assert_relayer();

        todo!()
    }

    pub fn reveal_seed(&mut self, secret_seed: Base64VecU8) {
        require!(
            env::block_timestamp_ms() > self.end_time,
            VOTING_PHASE_IN_PROGRESS
        );
        require!(self.private_key.is_none(), SEED_PHRASE_ALREADY_ADDED);
        self.assert_relayer();

        let key = SecretKey::from_slice(&secret_seed.0);
        if let Ok(key) = key {
            self.private_key = Some(key.secret_bytes())
        } else {
            panic_str(INVALID_SECRET_KEY)
        }
    }

    pub fn decrypt(&mut self) {
        require!(
            env::block_timestamp_ms() > self.end_time,
            VOTING_PHASE_IN_PROGRESS
        );
        require!(self.private_key.is_some(), SEED_PHRASE_IS_NOT_RELAYER);

        todo!()
    }

    fn assert_relayer(&self) {
        require!(
            env::predecessor_account_id() == self.relayer,
            consts::RELAYER_ONLY
        );
    }
}
