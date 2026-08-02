use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("decryption failed")]
    DecryptionFailed,
}

pub fn encrypt(key: &[u8; 32], plaintext: &str) -> Vec<u8> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("aes-gcm encryption does not fail for valid inputs");
    let mut out = nonce_bytes.to_vec();
    out.extend_from_slice(&ciphertext);
    out
}

pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<String, CryptoError> {
    if data.len() < 12 {
        return Err(CryptoError::DecryptionFailed);
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| CryptoError::DecryptionFailed)?;
    String::from_utf8(plaintext).map_err(|_| CryptoError::DecryptionFailed)
}

pub fn parse_key(hex_str: &str) -> Option<[u8; 32]> {
    let bytes = hex::decode(hex_str).ok()?;
    bytes.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [7u8; 32];

    #[test]
    fn encrypt_decrypt_roundtrip_recovers_the_plaintext() {
        let ciphertext = encrypt(&KEY, "sk-super-secret-key");
        let plaintext = decrypt(&KEY, &ciphertext).unwrap();
        assert_eq!(plaintext, "sk-super-secret-key");
    }

    #[test]
    fn decrypt_rejects_tampered_ciphertext() {
        let mut ciphertext = encrypt(&KEY, "sk-super-secret-key");
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;
        assert!(matches!(decrypt(&KEY, &ciphertext), Err(CryptoError::DecryptionFailed)));
    }

    #[test]
    fn decrypt_rejects_the_wrong_key() {
        let ciphertext = encrypt(&KEY, "sk-super-secret-key");
        let wrong_key = [9u8; 32];
        assert!(matches!(decrypt(&wrong_key, &ciphertext), Err(CryptoError::DecryptionFailed)));
    }

    #[test]
    fn parse_key_accepts_64_hex_chars() {
        let hex_str = "07".repeat(32);
        assert_eq!(parse_key(&hex_str), Some(KEY));
    }

    #[test]
    fn parse_key_rejects_wrong_length() {
        assert_eq!(parse_key("07"), None);
    }

    #[test]
    fn parse_key_rejects_non_hex() {
        assert_eq!(parse_key(&"zz".repeat(32)), None);
    }
}
