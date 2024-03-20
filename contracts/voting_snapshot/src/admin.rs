use near_sdk::env;

use crate::{consts::NOT_AUTHORIZED, *};

#[near_bindgen]
impl Contract {
    pub fn set_vote_config(&mut self, vote_config: VoteConfig) {
        near_sdk::require!(env::predecessor_account_id() == self.admin, NOT_AUTHORIZED);
        self.vote_config = vote_config;
    }

    pub fn bulk_load_voters(&mut self, voters: Vec<(AccountId, UserData)>) {
        near_sdk::require!(env::predecessor_account_id() == self.admin, NOT_AUTHORIZED);
        self.eligible_voters.extend(voters);
    }
}
