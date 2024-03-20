use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_vote_config(&self) -> VoteConfig {
        self.vote_config
    }

    pub fn get_admin(&self) -> AccountId {
        self.admin.clone()
    }
}
