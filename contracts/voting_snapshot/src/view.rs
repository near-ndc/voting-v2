use crate::{types::VoteWeight, *};

#[near_bindgen]
impl Contract {
    pub fn get_vote_config(&self) -> VoteConfig {
        self.vote_config
    }

    pub fn get_admin(&self) -> AccountId {
        self.admin.clone()
    }

    pub fn get_voters_info(&self, voters: Vec<AccountId>) -> Vec<(AccountId, UserData)> {
        voters
            .into_iter()
            .filter_map(|voter| {
                let voter_info = self.eligible_voters.get(&voter)?.clone();
                Some((voter, voter_info))
            })
            .collect()
    }

    pub fn get_vote_power(&self, voter: AccountId) -> Option<VoteWeight> {
        let voter_info = self.eligible_voters.get(&voter)?;
        Some(voter_info.vote_weight(self.vote_config))
    }

    pub fn get_voters_power(&self, voters: Vec<AccountId>) -> Vec<(AccountId, VoteWeight)> {
        voters
            .into_iter()
            .filter_map(|voter| {
                let voter_info = self.eligible_voters.get(&voter)?;
                Some((voter, voter_info.vote_weight(self.vote_config)))
            })
            .collect()
    }
}
