use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    NearSchema,
};

type PubKey = [u8; 64];

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, NearSchema, Debug, PartialEq, Clone,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct EncryptedVote {
    /// bs58 string
    pub vote: String,
    #[serde(with = "serde_bytes")]
    pub pubkey: PubKey,
}
