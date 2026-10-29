//! What Relay keeps secret (signing keys, dedicated database addresses) is sealed with
//! AES-256-GCM under `RELAY_ENCRYPTION_KEY`; the random nonce goes in front of the ciphertext.

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};

const NONCE_LEN: usize = 12;

#[derive(Debug, PartialEq, Eq)]
pub struct DecryptError;

impl std::fmt::Display for DecryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("couldn't decrypt a stored secret (wrong RELAY_ENCRYPTION_KEY?)")
    }
}

impl std::error::Error for DecryptError {}

pub fn encrypt(key: &[u8; 32], plaintext: &str) -> Vec<u8> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let mut sealed = nonce.to_vec();
    sealed.extend(cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes()).expect("AES-GCM encryption can't fail for valid input"));
    sealed
}

pub fn decrypt(key: &[u8; 32], sealed: &[u8]) -> Result<String, DecryptError> {
    if sealed.len() < NONCE_LEN {
        return Err(DecryptError);
    }
    let (nonce, ciphertext) = sealed.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let plain = cipher.decrypt(Nonce::from_slice(nonce), ciphertext).map_err(|_| DecryptError)?;
    String::from_utf8(plain).map_err(|_| DecryptError)
}
#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [7; 32];

    #[test]
    fn what_is_encrypted_comes_back_and_nothing_else_does() {
        let sealed = encrypt(&KEY, "postgres://secret");
        assert_eq!(decrypt(&KEY, &sealed).unwrap(), "postgres://secret");
        assert!(decrypt(&[8; 32], &sealed).is_err());
        let mut tampered = sealed.clone();
        *tampered.last_mut().unwrap() ^= 1;
        assert!(decrypt(&KEY, &tampered).is_err());
        assert!(decrypt(&KEY, &[1, 2, 3]).is_err());
    }
}
