use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    NearToken,
};

pub type VoteWeight = u32;

#[derive(
    Clone,
    Copy,
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
pub struct VoteConfig {
    pub threshold_in_nears: u32,
    pub activity_reward_in_votes: u32,
}

#[derive(
    Clone, BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize, Debug, PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
pub struct UserData {
    pub active_months: u32,
    pub stake: NearToken,
}

impl UserData {
    pub fn new(active_months: u32, stake: NearToken) -> Self {
        Self {
            active_months,
            stake,
        }
    }

    pub fn vote_weight(
        &self,
        VoteConfig {
            threshold_in_nears,
            activity_reward_in_votes,
        }: VoteConfig,
    ) -> VoteWeight {
        let stake = self.stake.as_near() as u32;
        let stake_votes = if stake <= threshold_in_nears {
            stake
        } else {
            f64::from(stake - threshold_in_nears).sqrt() as u32 + threshold_in_nears
        };
        let activity_votes = self.active_months * activity_reward_in_votes;
        stake_votes + activity_votes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vote_weight() {
        let user = UserData::new(1, NearToken::from_near(1));
        let vote_config = VoteConfig {
            threshold_in_nears: 1,
            activity_reward_in_votes: 1,
        };
        assert_eq!(user.vote_weight(vote_config), 2);
    }

    #[test]
    fn test_threshhold() {
        let user = UserData::new(5, NearToken::from_near(10500));
        let vote_config = VoteConfig {
            threshold_in_nears: 500,
            activity_reward_in_votes: 1,
        };
        assert_eq!(user.vote_weight(vote_config), 100 + 500 + 5); // sqrt(10000) + 500 + 5 activity
    }

    #[test]
    fn test_threshhold_rounding() {
        let user: UserData = UserData::new(5, NearToken::from_near(10600));
        let vote_config = VoteConfig {
            threshold_in_nears: 500,
            activity_reward_in_votes: 3,
        };
        assert_eq!(user.vote_weight(vote_config), 100 + 500 + 5 * 3); // sqrt(10100) + 500 + 5 activity
    }
}