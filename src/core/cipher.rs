use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::password_hash::rand_core::OsRng;
use hex;
use rand::RngCore;
use zeroize::{Zeroize, Zeroizing};

pub struct Cipher {
    key: [u8; 32],
}

impl Cipher {
    pub fn new(key: [u8; 32]) -> Self {
        Cipher { key }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, Box<dyn std::error::Error>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )) as Box<dyn std::error::Error>
        })?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut plaintext_data = Zeroizing::new(plaintext.as_bytes().to_vec());
        let ciphertext = cipher
            .encrypt(nonce, plaintext_data.as_ref())
            .map_err(|e| {
                Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
            })?;

        plaintext_data.zeroize();

        let mut encrypted_data = nonce_bytes.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);
        Ok(hex::encode(&encrypted_data))
    }

    pub fn decrypt(&self, encrypted_hex: &str) -> Result<String, Box<dyn std::error::Error>> {
        let encrypted_data = hex::decode(encrypted_hex)?;

        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data length".into());
        }

        let nonce_bytes = &encrypted_data[..12];
        let ciphertext = &encrypted_data[12..];

        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )) as Box<dyn std::error::Error>
        })?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext_bytes = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| {
            Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
        })?;

        let mut plaintext = Zeroizing::new(plaintext_bytes);
        let result = String::from_utf8(plaintext.to_vec());
        plaintext.zeroize();

        Ok(result?)
    }
}

impl Drop for Cipher {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cipher_encrypt_decrypt() {
        let key = [0u8; 32];
        let cipher = Cipher::new(key);
        let original = "Hello World!";

        let encrypted = cipher.encrypt(original).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted);
    }

    #[test]
    fn test_cipher_randomness() {
        let key = [0u8; 32];
        let cipher = Cipher::new(key);
        let data = "consistent data";

        let encrypted1 = cipher.encrypt(data).unwrap();
        let encrypted2 = cipher.encrypt(data).unwrap();

        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_cipher_wrong_key() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let cipher1 = Cipher::new(key1);
        let cipher2 = Cipher::new(key2);

        let encrypted = cipher1.encrypt("secret").unwrap();
        assert!(cipher2.decrypt(&encrypted).is_err());
    }
}
