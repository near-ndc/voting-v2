use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, AccountId};
use types::VoteConfig;

pub mod admin;
pub mod consts;
pub mod types;
pub mod view;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    vote_config: types::VoteConfig,
    admin: AccountId,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(admin: AccountId, vote_config: VoteConfig) -> Self {
        Self { admin, vote_config }
    }
}
