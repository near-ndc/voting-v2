use near_sdk::env;

use crate::{consts::*, events::emit_phase_change, *};
use common_contracts::finalize_storage_check;

#[near_bindgen]
impl Contract {
    /// *Transaction*: Sets the vote weight configuration
    ///
    /// Requirements:
    /// - Only admin can set this
    /// - Can be set only during initialization phase
    pub fn set_vote_config(&mut self, vote_config: VoteWeightConfig) {
        self.assert_initialization();
        self.assert_admin();

        self.vote_config = vote_config;
    }

    /// *Transaction*: Bulk load voters
    ///
    /// Requirements:
    /// - Only admin can bulk load voters
    /// - Can be done only during initialization phase
    /// - The admin should pay for the extra storage
    #[payable]
    pub fn bulk_load_voters(&mut self, voters: Vec<(AccountId, UserData)>) {
        let current_storage_usage = env::storage_usage();

        self.assert_initialization();
        self.assert_admin();

        self.eligible_voters.extend(voters);

        self.eligible_voters.flush();
        require!(
            finalize_storage_check(current_storage_usage, 0),
            STORAGE_LIMIT_EXCEEDED
        );
    }

    /// *Transaction*: Sets the snapshot configuration
    ///
    /// Requirements:
    /// - Only admin can set this
    /// - Can be set only during initialization phase
    pub fn set_snapshot_config(&mut self, process_config: SnapshotConfig) {
        self.assert_initialization();
        self.assert_admin();

        self.process_config = process_config;
    }

    /// *Transaction*: Starts the snapshot challenge phase once the snapshot is initialized
    ///
    /// Requirements:
    /// - Only admin can start the challenge
    /// - Can be started only during initialization phase
    pub fn start_challenge(&mut self) {
        self.assert_initialization();
        self.assert_admin();
        self.status = Status::SnapshotChallenge(self.status.attempt());
        self.end_time_in_millis =
            env::block_timestamp_ms() + self.process_config.challenge_timeout_in_millis;
        emit_phase_change(self.status);
    }

    /// *Transaction*: Restarts the process to the initialization phase in case of snapshot issues
    ///
    /// Requirements:
    /// - Only admin can restart the process
    /// - Can be restarted only during SnapshotChallenge or SnapshotHalted phase
    pub fn restart_to_initialization(&mut self) {
        // Admin can restart the process before the snapshot is halted
        // If some critical issues are found
        near_sdk::require!(
            matches!(
                self.status,
                Status::SnapshotChallenge(_) | Status::SnapshotHalted(_)
            ),
            RESTART_NOT_ALLOWED
        );

        self.assert_admin();
        self.status = Status::Initialization(self.status.attempt() + 1);
        // We reset the total challenged to 0 so with the new iteration we can start from scratch
        // Though, we preserve the individual challenged amounts,
        // so user can return all the funds in the end
        self.total_challenged = NearToken::from_yoctonear(0);
        emit_phase_change(self.status);

        // Now admin can bulk load data again and start the process
        // once issues are resolved
    }

    fn assert_admin(&self) {
        near_sdk::require!(env::predecessor_account_id() == self.admin, NOT_AUTHORIZED);
    }

    fn assert_initialization(&self) {
        near_sdk::require!(
            matches!(self.status, Status::Initialization(_)),
            ON_INITIALIZATION_ONLY
        );
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::{testing_env, NearToken};

    use crate::{
        test_utils::*,
        types::{SnapshotConfig, Status, UserData, VoteWeightConfig},
    };

    #[test]
    fn create_contract() {
        let (_context, contract) = setup_ctr();
        assert_eq!(contract.get_vote_config(), default_vote_config());
        assert_eq!(contract.get_admin(), admin());
    }

    #[test]
    fn admin_can_change_configs() {
        let (mut context, mut contract) = setup_ctr();
        let new_vote_config = VoteWeightConfig {
            threshold_in_nears: 200,
            activity_reward_in_votes: 20,
        };
        let new_snapshot_config = SnapshotConfig {
            challenge_threshold_in_nears: 200,
            challenge_timeout_in_millis: 200,
            registration_timeout_in_millis: 200,
        };

        assert_eq!(contract.get_vote_config(), default_vote_config());
        assert_eq!(contract.get_process_config(), default_snapshot_config());

        context.predecessor_account_id = admin();
        testing_env!(context.clone());

        contract.set_vote_config(new_vote_config);
        contract.set_snapshot_config(new_snapshot_config);

        assert_eq!(contract.get_vote_config(), new_vote_config);
        assert_eq!(contract.get_process_config(), new_snapshot_config);
    }

    #[test]
    #[should_panic(expected = "Not authorized")]
    fn non_admin_cannot_change_vote_config() {
        let (mut context, mut contract) = setup_ctr();
        let new_vote_config = VoteWeightConfig {
            threshold_in_nears: 200,
            activity_reward_in_votes: 20,
        };

        context.predecessor_account_id = acc(0);
        testing_env!(context.clone());

        contract.set_vote_config(new_vote_config);
    }

    #[test]
    #[should_panic(expected = "Not authorized")]
    fn non_admin_cannot_change_snapshot_config() {
        let (mut context, mut contract) = setup_ctr();
        let new_snapshot_config = SnapshotConfig {
            challenge_threshold_in_nears: 200,
            challenge_timeout_in_millis: 200,
            registration_timeout_in_millis: 200,
        };

        context.predecessor_account_id = acc(0);
        testing_env!(context.clone());

        contract.set_snapshot_config(new_snapshot_config);
    }

    #[test]
    #[should_panic(expected = "Not authorized")]
    fn non_admin_cannot_bulk_load_voters() {
        let (mut context, mut contract) = setup_ctr();
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
        let (mut context, mut contract) = setup_ctr();
        let voters = load_voters();

        context.predecessor_account_id = admin();
        context.attached_deposit = NearToken::from_millinear(10);
        testing_env!(context.clone());
        assert!(!contract.is_eligible_voter(&voters[0].0));

        contract.bulk_load_voters(voters.clone());

        assert!(contract.is_eligible_voter(&voters[0].0));
    }

    #[test]
    #[should_panic(expected = "Allowed only during initialization phase")]
    fn non_admin_cannot_bulk_load_voters_after_initialization() {
        let (mut context, mut contract) = setup_ctr();
        let voters = load_voters();
        move_to_challenge(&mut context, &mut contract);

        contract.bulk_load_voters(voters.clone());
    }

    #[test]
    #[should_panic(expected = "Allowed only during initialization phase")]
    fn non_admin_cannot_change_snapshot_config_after_initialization() {
        let (mut context, mut contract) = setup_ctr();
        let new_snapshot_config = SnapshotConfig {
            challenge_threshold_in_nears: 200,
            challenge_timeout_in_millis: 200,
            registration_timeout_in_millis: 200,
        };
        move_to_challenge(&mut context, &mut contract);

        contract.set_snapshot_config(new_snapshot_config);
    }

    #[test]
    fn admin_can_restart_to_initialization() {
        let (mut context, mut contract) = setup_ctr();
        move_to_challenge(&mut context, &mut contract);

        context.predecessor_account_id = admin();
        testing_env!(context.clone());

        contract.restart_to_initialization();

        assert!(matches!(contract.get_status(), Status::Initialization(1)));
    }

    #[test]
    #[should_panic(expected = "Restart is not allowed")]
    fn admin_cannot_restart_approved_snapshot() {
        let (mut context, mut contract) = setup_ctr();
        move_to_challenge(&mut context, &mut contract);
        move_to_registration(&mut context, &mut contract);

        context.predecessor_account_id = admin();
        testing_env!(context.clone());

        contract.restart_to_initialization();
    }
}
