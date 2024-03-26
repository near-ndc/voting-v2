use near_sdk::borsh::BorshSerialize;
use near_sdk::BorshStorageKey;

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    EligibleVoters,
    Voters,
    Nominees,
    Challengers,
}
