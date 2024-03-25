use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    NearSchema, NearToken,
};

#[derive(
    BorshDeserialize,
    BorshSerialize,
    Deserialize,
    Serialize,
    NearSchema,
    Debug,
    PartialEq,
    Clone,
    Copy,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
// Status is combination of status and attempt count
pub enum Status {
    // Loading snapshot data
    Initialization(u32),
    // Data loaded and can't be changed
    SnapshotChallenge(u32),
    // Snapshot challenged and should be re-initialized with attempt count increased
    SnapshotHalted(u32),
    // Snapshot approved after time. The data can't be challenged anymore.
    // Re-initialization is not possible. The data is final. User can nominate and registered.
    Registration(u32),
    // Registration phase ended. No more registration is allowed.
    RegistrationEnded(u32),
}

impl Status {
    pub fn attempt(&self) -> u32 {
        match self {
            Status::Initialization(attempt) => *attempt,
            Status::SnapshotChallenge(attempt) => *attempt,
            Status::SnapshotHalted(attempt) => *attempt,
            Status::Registration(attempt) => *attempt,
            Status::RegistrationEnded(attempt) => *attempt,
        }
    }
}

pub type VoteWeight = u32;

#[derive(
    Clone,
    Copy,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    NearSchema,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct VoteWeightConfig {
    pub threshold_in_nears: u32,
    pub activity_reward_in_votes: u32,
}

#[derive(
    Clone,
    Copy,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    NearSchema,
    Debug,
    PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct SnapshotConfig {
    pub challenge_threshold_in_nears: u32,
    pub challenge_timeout_in_millis: u64,
    pub registration_timeout_in_millis: u64,
}

#[derive(
    Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, NearSchema, Debug, PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
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

    pub fn vote_weight(&self, config: VoteWeightConfig) -> VoteWeight {
        let stake = self.stake.as_near() as u32;
        let stake_votes = if stake <= config.threshold_in_nears {
            stake
        } else {
            f64::from(stake - config.threshold_in_nears).sqrt() as u32 + config.threshold_in_nears
        };
        let activity_votes = self.active_months * config.activity_reward_in_votes;
        stake_votes + activity_votes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vote_weight() {
        let user = UserData::new(1, NearToken::from_near(1));
        let vote_config = VoteWeightConfig {
            threshold_in_nears: 1,
            activity_reward_in_votes: 1,
        };
        assert_eq!(user.vote_weight(vote_config), 2);
    }

    #[test]
    fn test_threshhold() {
        let user = UserData::new(5, NearToken::from_near(10500));
        let vote_config = VoteWeightConfig {
            threshold_in_nears: 500,
            activity_reward_in_votes: 1,
        };
        assert_eq!(user.vote_weight(vote_config), 100 + 500 + 5); // sqrt(10000) + 500 + 5 activity
    }

    #[test]
    fn test_threshhold_rounding() {
        let user: UserData = UserData::new(5, NearToken::from_near(10600));
        let vote_config = VoteWeightConfig {
            threshold_in_nears: 500,
            activity_reward_in_votes: 3,
        };
        assert_eq!(user.vote_weight(vote_config), 100 + 500 + 5 * 3); // sqrt(10100) + 500 + 5 activity
    }
}
