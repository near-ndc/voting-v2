use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    NearSchema,
};

type PubKey = [u8; 65];

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
    pub pubkey: String,
}

impl From<EncryptedVoteStorage> for EncryptedVoteView {
    fn from(vote: EncryptedVoteStorage) -> Self {
        Self {
            vote: vote.vote,
            pubkey: bs58::encode(vote.pubkey).into_string(),
        }
    }
}

impl From<EncryptedVoteView> for Option<EncryptedVoteStorage> {
    fn from(vote: EncryptedVoteView) -> Self {
        Some(EncryptedVoteStorage {
            vote: vote.vote,
            pubkey: bs58::decode(vote.pubkey).into_vec().ok()?.try_into().ok()?,
        })
    }
}
