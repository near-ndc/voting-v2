use serde::{Deserialize, Serialize};

use crate::{types::VoteWeight, *};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct VoterInformation {
    pub vote_weight: VoteWeight,
    pub public_key: PublicKey,
}

#[near_bindgen]
impl Contract {
    pub fn get_vote_config(&self) -> VoteConfig {
        self.vote_config
    }

    pub fn get_admin(&self) -> AccountId {
        self.admin.clone()
    }

    pub fn get_vote_power(&self, voter: &AccountId) -> Option<VoteWeight> {
        let voter_info = self.eligible_voters.get(voter)?;
        Some(voter_info.vote_weight(self.vote_config))
    }

    pub fn is_voter(&self, voter: &AccountId) -> bool {
        self.voters.contains_key(voter)
    }

    pub fn is_nominee(&self, nominee: &AccountId) -> bool {
        self.nominees.contains(nominee)
    }

    pub fn is_eligible_voter(&self, voter: &AccountId) -> bool {
        self.eligible_voters.contains_key(voter)
    }

    pub fn get_voter_information(&self, voter: &AccountId) -> Option<VoterInformation> {
        self.voters.get(voter).and_then(|public_key| {
            self.get_vote_power(voter).map(|weight| VoterInformation {
                vote_weight: weight,
                public_key: public_key.clone(),
            })
        })
    }

    pub fn get_voters_info(&self, voters: Vec<AccountId>) -> Vec<(AccountId, VoterInformation)> {
        voters
            .into_iter()
            .filter_map(|voter| self.get_voter_information(&voter).map(|info| (voter, info)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::testing_env;

    use crate::test_utils::*;

    #[test]
    fn user_can_get_vote_config() {
        let (_context, contract) = setup_ctr(0);
        assert_eq!(contract.get_vote_config(), default_vote_config());
    }

    #[test]
    fn user_can_get_admin() {
        let (_context, contract) = setup_ctr(0);
        assert_eq!(contract.get_admin(), admin());
    }

    #[test]
    fn user_can_get_vote_power() {
        let (_context, contract) = setup_ctr(0);

        let vote_power = contract.get_vote_power(&acc(1)).unwrap();
        assert_eq!(vote_power, 11);
    }

    #[test]
    fn user_can_get_voter_information() {
        let (mut context, mut contract) = setup_ctr(0);

        context.signer_account_id = acc(1);
        context.predecessor_account_id = acc(1);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        assert!(contract.get_voter_information(&acc(1)).is_none());

        contract.register_as_voter();

        let voter_info = contract.get_voter_information(&acc(1)).unwrap();
        assert_eq!(voter_info.vote_weight, 11);
        assert_eq!(voter_info.public_key, pk());

        assert_eq!(
            (acc(1), voter_info),
            contract.get_voters_info(vec![acc(1)])[0]
        );
    }
}