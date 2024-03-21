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

#[cfg(test)]
mod tests {
    use near_sdk::{testing_env, NearToken};

    use crate::{
        test_utils::*,
        types::{UserData, VoteConfig},
    };

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

    #[test]
    #[should_panic(expected = "Not authorized")]
    fn non_admin_cannot_bulk_load_voters() {
        let (mut context, mut contract) = setup_ctr(0);
        let voters = vec![(
            acc(0),
            UserData {
                active_months: 2,
                stake: NearToken::from_near(1),
            },
        )];
        assert_eq!(contract.get_vote_config(), default_vote_config());

        context.predecessor_account_id = acc(0);
        testing_env!(context.clone());

        contract.bulk_load_voters(voters);
    }

    #[test]
    fn admin_can_bulk_load_voters() {
        let (mut context, mut contract) = setup_ctr(0);
        let voters = load_voters();

        context.predecessor_account_id = admin();
        testing_env!(context.clone());
        assert!(!contract.is_eligible_voter(&voters[0].0));

        contract.bulk_load_voters(voters.clone());

        assert!(contract.is_eligible_voter(&voters[0].0));
    }
}
