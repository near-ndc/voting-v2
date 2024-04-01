use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    AccountId, NearSchema,
};
use secp256k1::{ecdh::SharedSecret, PublicKey, SecretKey};

use crate::crypto::decrypt_message;

type PubKey = [u8; 64];

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, NearSchema, Debug, PartialEq, Clone,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct EncryptedVote {
    /// bs58 string
    vote: String,
    #[serde(with = "serde_bytes")]
    pubkey: PubKey,
}

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, NearSchema, Debug, PartialEq, Clone,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum Vote {
    Encrypted(EncryptedVote),
    Decrypted {
        account_id: AccountId,
        vote_for: AccountId,
        #[serde(with = "serde_bytes")]
        signature: [u8; 64],
    },
    Accepted {
        account_id: AccountId,
        vote_for: AccountId,
    },
    Incorrect {},
}

impl Vote {
    pub fn is_encrypted(&self) -> bool {
        matches!(self, Vote::Encrypted(_))
    }

    pub fn decrypt(&self, secret: &SecretKey) -> Result<Self, Error> {
        if let Vote::Encrypted(encrypted) = self {
            let decrypted_msg = decrypt_message(&encrypted.vote, secret, encrypted.pubkey)
                .ok_or(Error::DecryptionError)?;

            todo!()
        } else {
            Err(Error::WrongType)
        }
    }
}

pub enum Error {
    DecryptionError,
    FormatError,
    WrongType,
}
