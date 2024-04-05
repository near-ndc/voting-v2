use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_votes(&self, page: u64, limit: u64) -> Vec<EncryptedVoteView> {
        let start = page * limit;
        let end = std::cmp::min(start + limit, self.votes.len());

        (start..end)
            .map(|i| self.votes.get(i).unwrap().into())
            .collect()
    }

    pub fn get_total_votes(&self) -> u64 {
        self.votes.len()
    }

    pub fn get_candidate_weights(&self, page: u64, limit: u64) -> Vec<(AccountId, u64)> {
        let start = std::cmp::min(page * limit, self.candidate_weights.len());
        let end = std::cmp::min(start + limit, self.candidate_weights.len());

        self.candidate_weights
            .iter()
            .skip(start as usize)
            .take((end - start) as usize)
            .collect()
    }

    pub fn get_total_candidate_weights(&self) -> u64 {
        self.candidate_weights.len()
    }

    pub fn get_relayer(&self) -> AccountId {
        self.relayer.clone()
    }

    pub fn get_end_time(&self) -> Timestamp {
        self.end_time_in_ms
    }
}

#[cfg(test)]
mod view_tests {
    use near_sdk::{json_types::Base64VecU8, testing_env, NearToken};

    use crate::{test_utils::*, types::EncryptedVoteView};

    #[test]
    fn pagination_test_on_votes() {
        let (mut context, mut contract) = setup_ctr();

        context.predecessor_account_id = relayer();
        context.attached_deposit = NearToken::from_near(1);
        testing_env!(context.clone());

        let votes_init = (0..107)
            .map(|i| EncryptedVoteView {
                vote: i.to_string(),
                pubkey: Base64VecU8([i; 64].to_vec()),
            })
            .collect::<Vec<_>>();

        contract.send_encrypted_votes(votes_init.clone());

        assert_eq!(contract.get_total_votes(), 107);

        let votes = contract.get_votes(0, 10);
        assert_eq!(votes.len(), 10);
        assert_eq!(votes, &votes_init[0..10]);

        let votes = contract.get_votes(10, 10);
        assert_eq!(votes.len(), 7);
        assert_eq!(votes, &votes_init[100..]);

        let votes = contract.get_votes(55, 10);
        assert_eq!(votes.len(), 0);
    }

    #[test]
    fn pagination_test_on_candidate_weights() {
        let (mut context, mut contract) = setup_ctr();

        context.predecessor_account_id = relayer();
        context.attached_deposit = NearToken::from_near(1);
        context.block_timestamp = (end_time() + 1) * MSECOND;
        testing_env!(context.clone());

        let results_init = (0..107).map(|i| (acc(i), i as u64)).collect::<Vec<_>>();

        contract.sumbit_results(results_init.clone());

        assert_eq!(contract.get_total_candidate_weights(), 107);

        let results = contract.get_candidate_weights(0, 10);
        assert_eq!(results.len(), 10);
        assert_eq!(results, &results_init[0..10]);

        let results = contract.get_candidate_weights(10, 10);
        assert_eq!(results.len(), 7);
        assert_eq!(results, &results_init[100..]);

        let results = contract.get_candidate_weights(55, 10);
        assert_eq!(results.len(), 0);
    }
}
