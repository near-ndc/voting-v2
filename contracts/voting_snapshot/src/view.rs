use near_sdk::{
    serde::{Deserialize, Serialize},
    NearSchema,
};

use crate::{types::VoteWeight, *};

#[derive(NearSchema, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct VoterInformation {
    pub vote_weight: VoteWeight,
    pub public_key: PublicKey,
}

#[near_bindgen]
impl Contract {
    /// *View*: Returns the vote weight configuration
    pub fn get_vote_config(&self) -> VoteWeightConfig {
        self.vote_config
    }

    /// *View*: Returns the snapshot configuration (Time for challenge, registration, threshold for challenge)
    pub fn get_process_config(&self) -> SnapshotConfig {
        self.process_config
    }

    /// *View*: Returns the end time of the current phase in milliseconds.
    ///
    /// Only applicable for challenge and registration phase
    pub fn get_end_time(&self) -> u64 {
        self.end_time_in_millis
    }

    /// *View*: Returns the current phase of the snapshot
    pub fn get_status(&self) -> Status {
        self.status
    }

    /// *View*: Return the total amount of NEAR tokens challenged in the current iteration
    pub fn get_total_challenge(&self) -> NearToken {
        self.total_challenged
    }

    /// *View*: Returns the individual challenge amount for a given challenger
    ///
    /// Returns None if the challenger has not challenged or the challenger already withdrew the deposit
    pub fn get_individual_challenge(&self, challenger: &AccountId) -> Option<NearToken> {
        self.challengers.get(challenger).cloned()
    }

    /// *View*: Returns admin account ID
    pub fn get_admin(&self) -> AccountId {
        self.admin.clone()
    }

    /// *View*: Returns the vote power of a individual voter
    pub fn get_vote_power(&self, voter: &AccountId) -> Option<VoteWeight> {
        let voter_info = self.eligible_voters.get(voter)?;
        Some(voter_info.vote_weight(self.vote_config))
    }

    /// *View*: Returns if the given account ID submitted public key and became a voter
    pub fn is_voter(&self, voter: &AccountId) -> bool {
        self.voters.contains_key(voter)
    }

    /// *View*: Returns if the given account ID is a nominee
    pub fn is_nominee(&self, nominee: &AccountId) -> bool {
        self.nominees.contains(nominee)
    }

    /// *View*: Returns if the given account ID is able to become a voter or a nominee
    pub fn is_eligible_voter(&self, voter: &AccountId) -> bool {
        self.eligible_voters.contains_key(voter)
    }

    /// *View*: Returns vote weight and public key of a voter
    pub fn get_voter_information(&self, voter: &AccountId) -> Option<VoterInformation> {
        self.voters.get(voter).and_then(|public_key| {
            self.get_vote_power(voter).map(|weight| VoterInformation {
                vote_weight: weight,
                public_key: public_key.clone(),
            })
        })
    }

    /// *View*: Returns vote weight and public key of a list of voters
    pub fn get_voters_info(&self, voters: Vec<AccountId>) -> Vec<(AccountId, VoterInformation)> {
        voters
            .into_iter()
            .filter_map(|voter| self.get_voter_information(&voter).map(|info| (voter, info)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::{testing_env, NearToken};

    use crate::test_utils::*;

    #[test]
    fn user_can_get_vote_config() {
        let (_context, contract) = setup_ctr();
        assert_eq!(contract.get_vote_config(), default_vote_config());
    }

    #[test]
    fn user_can_get_admin() {
        let (_context, contract) = setup_ctr();
        assert_eq!(contract.get_admin(), admin());
    }

    #[test]
    fn user_can_get_vote_power() {
        let (_context, contract) = setup_ctr();

        let vote_power = contract.get_vote_power(&acc(1)).unwrap();
        assert_eq!(vote_power, 11);
    }

    #[test]
    fn user_can_get_voter_information() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

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

    #[test]
    fn user_can_get_deposit() {
        let (mut context, mut contract) = setup_ctr();
        move_to_challenge(&mut context, &mut contract);

        assert_eq!(contract.get_individual_challenge(&acc(0)), None);

        context.predecessor_account_id = acc(0);
        context.attached_deposit = NearToken::from_near(1);
        testing_env!(context.clone());

        contract.challenge_snapshot();

        assert_eq!(
            contract.get_individual_challenge(&acc(0)),
            Some(NearToken::from_near(1))
        );
        assert_eq!(contract.get_total_challenge(), NearToken::from_near(1));
    }
}
