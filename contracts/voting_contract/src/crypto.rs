use aes_siv::{aead::generic_array::GenericArray, siv::Aes128Siv, KeyInit};
use secp256k1::{ecdh::SharedSecret, PublicKey, SecretKey};

pub fn decrypt_message(bs58message: &str, secret: &SecretKey, pubkey: [u8; 64]) -> Option<Vec<u8>> {
    let pubkey = PublicKey::from_slice(&pubkey).ok()?;

    let common_secret = SharedSecret::new(&pubkey, secret);

    let bytes = near_sdk::bs58::decode(bs58message).into_vec().ok()?;

    let ad_data: &[&[u8]] = &[];
    let mut cipher = Aes128Siv::new(&GenericArray::clone_from_slice(
        &common_secret.secret_bytes(),
    ));

    let decrypted_data = cipher.decrypt(ad_data, &bytes).ok()?;

    Some(decrypted_data)
}
