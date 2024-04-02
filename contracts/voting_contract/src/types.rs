use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    json_types::Base64VecU8,
    serde::{Deserialize, Serialize},
    NearSchema,
};

type PubKey = [u8; 64];

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Clone)]
#[borsh(crate = "near_sdk::borsh")]
pub struct EncryptedVoteStorage {
    /// bs58 string
    pub vote: String,
    pub pubkey: PubKey,
}

#[derive(Serialize, Deserialize, NearSchema, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct EncryptedVoteView {
    pub vote: String,
    pub pubkey: Base64VecU8,
}

impl From<EncryptedVoteStorage> for EncryptedVoteView {
    fn from(vote: EncryptedVoteStorage) -> Self {
        Self {
            vote: vote.vote,
            pubkey: Base64VecU8(vote.pubkey.to_vec()),
        }
    }
}

impl From<EncryptedVoteView> for Option<EncryptedVoteStorage> {
    fn from(vote: EncryptedVoteView) -> Self {
        Some(EncryptedVoteStorage {
            vote: vote.vote,
            pubkey: vote.pubkey.0.as_slice().try_into().ok()?,
        })
    }
}
