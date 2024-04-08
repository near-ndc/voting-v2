use events::emit_phase_change;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::env::{predecessor_account_id, signer_account_id};
use near_sdk::store::{LookupMap, LookupSet};
use near_sdk::{
    env, near_bindgen, require, AccountId, NearToken, PanicOnDefault, Promise, PromiseResult,
    PublicKey,
};

pub mod admin;
pub mod consts;
pub mod events;
pub mod ext;
pub mod storage;
pub mod types;
pub mod view;

use common_contracts::finalize_storage_check;
use consts::*;
use storage::StorageKey;
use types::{SnapshotConfig, Status, UserData, VoteWeightConfig};

#[cfg(all(test, not(target_arch = "wasm32")))]
pub mod test_utils;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    vote_config: types::VoteWeightConfig,
    process_config: types::SnapshotConfig,
    admin: AccountId,
    status: Status,

    end_time_in_millis: u64,

    // This is basically a snapshot but without unnecessary data
    // for full snapshot, please refer to the IPFS storage
    // Will be cleaned on halt.
    eligible_voters: LookupMap<AccountId, UserData>,
    total_eligible_users: u32,

    // We need to collect the ones who want to participate in the vote process
    // We collect the public key of the voter to verify the signature
    // in the encoded message.
    // Also, this user indicates that he/she accepts conduct of fair voting
    voters: LookupMap<AccountId, PublicKey>,
    total_voters: u32,

    nominees: LookupSet<AccountId>,

    // People can deposit NEAR to challenge snapshot
    // If the challenge is successful, the voting snapshot will be halted
    // Also, this provides amount of how much user spent to challenge the snapshot
    // The total number of challengers can be > than the config parameters because of the
    // several attempts.
    challengers: LookupMap<AccountId, NearToken>,
    total_challenged: NearToken,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given admin and configs
    #[init]
    pub fn new(
        admin: AccountId,
        vote_config: VoteWeightConfig,
        process_config: SnapshotConfig,
    ) -> Self {
        let status = Status::Initialization(0);
        emit_phase_change(status);
        Self {
            admin,
            status,
            process_config,
            vote_config,
            end_time_in_millis: 0,
            total_voters: 0,
            total_eligible_users: 0,
            eligible_voters: LookupMap::new(StorageKey::EligibleVoters),
            voters: LookupMap::new(StorageKey::Voters),
            nominees: LookupSet::new(StorageKey::Nominees),
            challengers: LookupMap::new(StorageKey::Challengers),
            total_challenged: NearToken::from_millinear(0),
        }
    }

    /// *Transaction*: Registers the user as a voter
    ///
    /// Requirements:
    /// - The contract should be in the registration phase
    /// - User should be eligible
    /// - User should not be registered before
    /// - User should pay for storage including the snapshot record cost
    /// - User should call directly as we parse signer public key from the input
    #[payable]
    pub fn register_as_voter(&mut self) {
        let storage = env::storage_usage();

        let signer = signer_account_id();
        require!(signer == predecessor_account_id(), DIRECT_CALL);
        require!(!self.voters.contains_key(&signer), ALREADY_REGISTERED);

        self.try_move_stage();

        self.assert_eligible_voter(&signer);

        self.voters.insert(signer, env::signer_account_pk());
        self.total_voters += 1;

        self.voters.flush();
        require!(
            finalize_storage_check(storage, SNAPSHOT_RECORD_COST),
            STORAGE_LIMIT_EXCEEDED
        );
    }

    /// *Transaction*: Registers the user as a voter with the given public key
    ///
    /// Requirements:
    /// - The contract should be in the registration phase
    /// - User should be eligible
    /// - User should not be registered before
    /// - User should pay for storage including the snapshot record cost
    #[payable]
    pub fn register_as_voter_with_pubkey(&mut self, public_key: PublicKey) {
        let storage = env::storage_usage();

        self.try_move_stage();

        let user = env::predecessor_account_id();
        self.assert_eligible_voter(&user);
        require!(!self.voters.contains_key(&user), ALREADY_REGISTERED);

        self.voters.insert(user, public_key);
        self.total_voters += 1;

        self.voters.flush();

        require!(
            finalize_storage_check(storage, SNAPSHOT_RECORD_COST),
            STORAGE_LIMIT_EXCEEDED
        );
    }

    /// *Transaction*: Changes the public key of the user
    ///
    /// Requirements:
    /// - User should be registered
    pub fn change_public_key(&mut self, public_key: PublicKey) {
        let user = env::predecessor_account_id();
        require!(self.voters.contains_key(&user), NOT_REGISTERED);

        self.voters.set(user, Some(public_key));
    }

    /// *Transaction*: Registers the user as a nominee
    ///
    /// Requirements:
    /// - The contract should be in the registration phase
    /// - User should be eligible
    /// - User should not be registered before
    /// - User should pay for storage
    #[payable]
    pub fn register_as_nominee(&mut self) {
        let storage = env::storage_usage();

        self.try_move_stage();

        let user = env::predecessor_account_id();
        self.assert_eligible_voter(&user);
        require!(!self.nominees.contains(&user), ALREADY_REGISTERED);

        self.nominees.insert(user);

        require!(finalize_storage_check(storage, 0), STORAGE_LIMIT_EXCEEDED);
    }

    /// *Transaction*: Any user can challenge the snapshot. User can deposit NEAR several times
    ///
    /// Requirements:
    /// - The contract should be in the snapshot challenge phase
    /// - User should deposit more than 1 milli NEAR
    #[payable]
    pub fn challenge_snapshot(&mut self) {
        self.try_move_stage();
        require!(
            matches!(self.status, Status::SnapshotChallenge(_)),
            ON_SNAPSHOT_CHALLENGE_ONLY
        );

        let user = env::predecessor_account_id();
        let deposit = env::attached_deposit();

        require!(deposit.as_millinear() > 0, EXPECTED_DEPOSIT);

        self.challengers
            .entry(user)
            .and_modify(|user_deposit| {
                if let Some(new_total) = user_deposit.checked_add(deposit) {
                    *user_deposit = new_total;
                } else {
                    env::panic_str(CHALLENGE_OVERFLOW);
                }
            })
            .or_insert(deposit);
        if let Some(total) = self.total_challenged.checked_add(deposit) {
            self.total_challenged = total;
        } else {
            env::panic_str(CHALLENGE_OVERFLOW);
        }

        self.try_halt();
    }

    /// *Transaction*: Refunds the challenge deposit to the user
    ///
    /// Requirements:
    /// - The contract should not be in the snapshot challenge phase
    /// - User should have a deposit
    pub fn refund_bond(&mut self) -> Promise {
        self.try_move_stage();

        require!(
            !matches!(self.status, Status::SnapshotChallenge(_)),
            NOT_ON_SNAPSHOT_CHALLENGE
        );

        let user = env::predecessor_account_id();
        let deposit = self.challengers.get(&user);

        if let Some(deposit) = deposit {
            Promise::new(user.clone()).transfer(*deposit).then(
                ext::ext_self::ext(env::current_account_id())
                    .with_static_gas(ON_REFUND_SUCCESS_GAS)
                    .on_refund_success(user),
            )
        } else {
            env::panic_str(NO_DEPOSIT)
        }
    }

    /// *Callback*: Callback function to handle the refund success
    ///
    /// Private function
    #[private]
    pub fn on_refund_success(&mut self, account_id: AccountId) {
        require!(env::promise_results_count() == 1, EXPECTED_PROMISE_RESULT);

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                // We are not interested in total challenged as we are passed the challenge phase
                self.challengers.remove(&account_id);
            }
            PromiseResult::Failed => {}
        }
    }

    /// *Transaction*: Tries to move the status to the next phase
    pub fn try_move_stage(&mut self) {
        let should_move = env::block_timestamp_ms() >= self.end_time_in_millis;

        match self.status {
            Status::SnapshotChallenge(attempt) if should_move => {
                // Last try to halt
                if !self.try_halt() {
                    self.status = Status::Registration(attempt);

                    // We don't use block_timestamp_ms() here to have strict timings
                    self.end_time_in_millis += self.process_config.registration_timeout_in_millis;
                    emit_phase_change(self.status);
                }
            }
            Status::Registration(attempt) if should_move => {
                self.status = Status::RegistrationEnded(attempt);
                emit_phase_change(self.status);
            }
            // Explicitly write all cases to fail on new status
            Status::Initialization(_)
            | Status::SnapshotChallenge(_)
            | Status::SnapshotHalted(_)
            | Status::Registration(_)
            | Status::RegistrationEnded(_) => {}
        }
    }

    fn assert_eligible_voter(&self, user: &AccountId) {
        require!(
            matches!(self.status, Status::Registration(_),),
            ON_REGISTRATION_ONLY
        );
        require!(self.eligible_voters.contains_key(user), NOT_ELIGIBLE_VOTER);
    }

    fn try_halt(&mut self) -> bool {
        if self.total_challenged.as_near()
            >= self.process_config.challenge_threshold_in_nears as u128
        {
            self.status = Status::SnapshotHalted(self.status.attempt());
            emit_phase_change(self.status);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use near_sdk::{testing_env, NearToken, PublicKey};

    use crate::{test_utils::*, types::Status};

    #[test]
    fn eligible_user_can_register_as_voter() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.signer_account_id = acc(1);
        context.predecessor_account_id = acc(1);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        contract.register_as_voter();

        assert!(contract.is_voter(&acc(1)));
    }

    #[test]
    #[should_panic(expected = "Deposit is not enough to cover storage usage")]
    fn user_should_pay_for_storage_to_become_voter() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.signer_account_id = acc(1);
        context.predecessor_account_id = acc(1);
        context.signer_account_pk = pk();
        context.attached_deposit = NearToken::from_yoctonear(0);
        testing_env!(context.clone());

        contract.register_as_voter();
    }

    #[test]
    #[should_panic(expected = "Deposit is not enough to cover storage usage")]
    fn user_should_pay_for_storage_to_become_voter2() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.predecessor_account_id = acc(1);
        context.attached_deposit = NearToken::from_yoctonear(0);
        testing_env!(context.clone());

        contract.register_as_voter_with_pubkey(pk());
    }

    #[test]
    #[should_panic(expected = "Deposit is not enough to cover storage usage")]
    fn user_should_pay_for_storage_to_become_nominee() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.predecessor_account_id = acc(1);
        context.attached_deposit = NearToken::from_yoctonear(0);
        testing_env!(context.clone());

        contract.register_as_nominee();
    }

    #[test]
    #[should_panic(expected = "Not eligible voter")]
    fn non_eligible_user_cannot_register_as_voter() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.signer_account_id = acc(0);
        context.predecessor_account_id = acc(0);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        contract.register_as_voter();
    }

    #[test]
    #[should_panic(expected = "Should be called directly")]
    fn register_as_voter_should_be_called_directly() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.signer_account_id = acc(1);
        context.predecessor_account_id = acc(0);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        contract.register_as_voter();
    }

    #[test]
    fn eligible_user_can_register_as_voter_with_pubkey() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.predecessor_account_id = acc(1);
        testing_env!(context.clone());

        contract.register_as_voter_with_pubkey(pk());

        assert!(contract.is_voter(&acc(1)));
    }

    #[test]
    #[should_panic(expected = "Not eligible voter")]
    fn non_eligible_user_cannot_register_as_voter_with_pubkey() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.predecessor_account_id = acc(0);
        testing_env!(context.clone());

        contract.register_as_voter_with_pubkey(pk());
    }

    #[test]
    fn user_can_change_pubkey() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

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

        contract.change_public_key(another_pk.clone());

        assert_eq!(
            contract.get_voter_information(&acc(1)).unwrap().public_key,
            another_pk
        );
    }

    #[test]
    fn user_can_register_as_nominee() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.predecessor_account_id = acc(1);
        testing_env!(context.clone());

        contract.register_as_nominee();

        assert!(contract.is_nominee(&acc(1)));
    }

    #[test]
    #[should_panic(expected = "Not eligible voter")]
    fn non_eligible_user_cannot_register_as_nominee() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.predecessor_account_id = acc(0);
        testing_env!(context.clone());

        contract.register_as_nominee();
    }

    #[test]
    #[should_panic(expected = "Allowed only during registration phase")]
    fn user_cannot_register_as_nominee_after_registration() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);
        move_to_end(&mut context, &mut contract);

        context.predecessor_account_id = acc(1);
        testing_env!(context.clone());

        contract.register_as_nominee();
    }

    #[test]
    #[should_panic(expected = "Allowed only during registration phase")]
    fn user_cannot_register_as_voter_after_registration() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);
        move_to_end(&mut context, &mut contract);

        context.predecessor_account_id = acc(1);
        testing_env!(context.clone());

        contract.register_as_voter_with_pubkey(pk());
    }

    #[test]
    #[should_panic(expected = "Allowed only during registration phase")]
    fn user_cannot_register_as_voter_before_registration() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);

        context.signer_account_id = acc(1);
        context.predecessor_account_id = acc(1);
        context.signer_account_pk = pk();
        testing_env!(context.clone());

        contract.register_as_voter();
    }

    #[test]
    #[should_panic(expected = "Allowed only during registration phase")]
    fn user_cannot_register_as_nominee_before_registration() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);

        context.predecessor_account_id = acc(1);
        testing_env!(context.clone());

        contract.register_as_nominee();
    }

    #[test]
    #[should_panic(expected = "Not allowed on snapshot challenge phase")]
    fn user_can_challenge_snapshot_but_cannot_retrieve_money_before_challenge_ends() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);

        context.predecessor_account_id = acc(0);
        context.attached_deposit = NearToken::from_near(1);
        testing_env!(context.clone());

        contract.challenge_snapshot();

        assert_eq!(
            contract.get_individual_challenge(&acc(0)),
            Some(NearToken::from_near(1))
        );

        contract.refund_bond();
    }

    #[test]
    #[should_panic(expected = "No deposit found for the user")]
    fn wrong_user_cannot_refund() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.predecessor_account_id = acc(0);
        testing_env!(context.clone());
        contract.refund_bond();
    }

    #[test]
    #[should_panic(expected = "Expected deposit greater than 1 milli NEAR")]
    fn user_cannot_challenge_snapshot_without_deposit() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);

        context.predecessor_account_id = acc(0);
        context.attached_deposit = NearToken::from_millinear(0);
        testing_env!(context.clone());

        contract.challenge_snapshot();
    }

    #[test]
    fn user_can_halt_snapshot_and_retrieve_funds() {
        let (mut context, mut contract) = setup_ctr();

        move_to_challenge(&mut context, &mut contract);

        context.predecessor_account_id = acc(0);
        context.attached_deposit = NearToken::from_near(
            contract.get_process_config().challenge_threshold_in_nears as u128,
        );
        testing_env!(context.clone());

        contract.challenge_snapshot();

        assert!(matches!(contract.get_status(), Status::SnapshotHalted(_)));

        contract.refund_bond();
        testing_env!(context.clone());

        // Admin can restart
        context.predecessor_account_id = admin();
        testing_env!(context.clone());

        contract.restart_to_initialization();

        assert!(matches!(contract.get_status(), Status::Initialization(1)));
        assert_eq!(contract.get_total_challenge(), NearToken::from_millinear(0));
    }
}
