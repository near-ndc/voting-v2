use near_sdk::borsh::BorshSerialize;
use near_sdk::BorshStorageKey;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    Votes,
    CandidatesWeights,
}
