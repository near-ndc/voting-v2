use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::env::{predecessor_account_id, signer_account_id};
use near_sdk::store::{LookupMap, LookupSet};
use near_sdk::{env, near_bindgen, require, AccountId, PanicOnDefault, PublicKey};

pub mod admin;
pub mod consts;
pub mod storage;
pub mod types;
pub mod view;

use consts::*;
use storage::StorageKey;
use types::{UserData, VoteConfig};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test_utils;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    vote_config: types::VoteConfig,
    admin: AccountId,

    // This is basically a snapshot of the voters at the time of the vote
    eligible_voters: LookupMap<AccountId, UserData>,

    // We need to collect the ones who want to participate in the vote process
    // We collect the public key of the voter to verify the signature
    // in the encoded message.
    // Also, this user indicates that he/she accepts conduct of fair voting
    voters: LookupMap<AccountId, PublicKey>,
    nominees: LookupSet<AccountId>,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(admin: AccountId, vote_config: VoteConfig) -> Self {
        Self {
            admin,
            vote_config,
            eligible_voters: LookupMap::new(StorageKey::EligibleVoters),
            voters: LookupMap::new(StorageKey::Voters),
            nominees: LookupSet::new(StorageKey::Nominees),
        }
    }

    // Should be called directly as we parse public key from the signer
    // As a result, we need to make sure that the signer is the predecessor
    pub fn register_as_voter(&mut self) {
        let signer = signer_account_id();
        require!(signer == predecessor_account_id(), DIRECT_CALL);
        require!(
            self.eligible_voters.contains_key(&signer),
            NOT_ELIGIBLE_VOTER
        );

        self.voters.insert(signer, env::signer_account_pk());
    }

    // Can be called indirectly as we parse public key from the input
    pub fn register_as_voter_with_pubkey(&mut self, public_key: PublicKey) {
        let user = env::predecessor_account_id();
        require!(self.eligible_voters.contains_key(&user), NOT_ELIGIBLE_VOTER);

        self.voters.insert(user, public_key);
    }

    pub fn register_as_nominee(&mut self) {
        let user = env::predecessor_account_id();
        require!(self.eligible_voters.contains_key(&user), NOT_ELIGIBLE_VOTER);
        self.nominees.insert(user);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use near_sdk::{testing_env, PublicKey};

    use crate::test_utils::*;

    #[test]
    fn eligible_user_can_register_as_voter() {
        let (mut context, mut contract) = setup_ctr(0);

        context.signer_account_id = acc(1);
        context.predecessor_account_id = acc(1);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        contract.register_as_voter();

        assert!(contract.is_voter(&acc(1)));
    }

    #[test]
    #[should_panic(expected = "Not eligible voter")]
    fn non_eligible_user_cannot_register_as_voter() {
        let (mut context, mut contract) = setup_ctr(0);

        context.signer_account_id = acc(0);
        context.predecessor_account_id = acc(0);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        contract.register_as_voter();
    }

    #[test]
    #[should_panic(expected = "Should be called directly")]
    fn register_as_voter_should_be_called_directly() {
        let (mut context, mut contract) = setup_ctr(0);

        context.signer_account_id = acc(1);
        context.predecessor_account_id = acc(0);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        contract.register_as_voter();
    }

    #[test]
    fn eligible_user_can_register_as_voter_with_pubkey() {
        let (mut context, mut contract) = setup_ctr(0);

        context.predecessor_account_id = acc(1);
        testing_env!(context.clone());

        contract.register_as_voter_with_pubkey(pk());

        assert!(contract.is_voter(&acc(1)));
    }

    #[test]
    #[should_panic(expected = "Not eligible voter")]
    fn non_eligible_user_cannot_register_as_voter_with_pubkey() {
        let (mut context, mut contract) = setup_ctr(0);

        context.predecessor_account_id = acc(0);
        testing_env!(context.clone());

        contract.register_as_voter_with_pubkey(pk());
    }

    #[test]
    fn user_can_change_pubkey() {
        let (mut context, mut contract) = setup_ctr(0);

        context.predecessor_account_id = acc(1);
        context.signer_account_id = acc(1);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        contract.register_as_voter();

        assert_eq!(
            contract.get_voter_information(&acc(1)).unwrap().public_key,
            pk()
        );

        let another_pk =
            PublicKey::from_str("ed25519:XSCka9nSaKt1xhtXumnpSPvJmLAEjSHgiTC5kQGo5Xv").unwrap();

        contract.register_as_voter_with_pubkey(another_pk.clone());

        assert_eq!(
            contract.get_voter_information(&acc(1)).unwrap().public_key,
            another_pk
        );
    }

    #[test]
    fn user_can_register_as_nominee() {
        let (mut context, mut contract) = setup_ctr(0);

        context.predecessor_account_id = acc(1);
        testing_env!(context.clone());

        contract.register_as_nominee();

        assert!(contract.is_nominee(&acc(1)));
    }

    #[test]
    #[should_panic(expected = "Not eligible voter")]
    fn non_eligible_user_cannot_register_as_nominee() {
        let (mut context, mut contract) = setup_ctr(0);

        context.predecessor_account_id = acc(0);
        testing_env!(context.clone());

        contract.register_as_nominee();
    }
}
