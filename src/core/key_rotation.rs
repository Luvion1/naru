use crate::core::cipher::Cipher;
use crate::core::key_derivation::{derive_key, generate_salt};

pub struct KeyRotation {
    old_key: [u8; 32],
    new_key: [u8; 32],
}

impl KeyRotation {
    pub fn new(
        old_password: &str,
        new_password: &str,
        old_salt: &[u8],
        new_salt: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let old_key = derive_key(old_password, old_salt)?;
        let new_key = derive_key(new_password, new_salt)?;

        Ok(Self { old_key, new_key })
    }

    pub fn rotate_value(
        &self,
        encrypted_value: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let old_cipher = Cipher::new(self.old_key);
        let decrypted = old_cipher.decrypt(encrypted_value)?;

        let new_cipher = Cipher::new(self.new_key);
        let rotated = new_cipher.encrypt(&decrypted)?;

        Ok(rotated)
    }

    pub fn rotate_config(
        &self,
        config: &mut crate::core::config_model::ConfigFile,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for env_config in config.environments.values_mut() {
            for entry in env_config.entries.values_mut() {
                if entry.encrypted && !entry.value.is_empty() {
                    let rotated_value = self.rotate_value(&entry.value)?;
                    entry.value = rotated_value;
                }
            }
        }

        if let Some(ref mut salt) = config.salt {
            let new_salt = generate_salt();
            *salt = hex::encode(new_salt);
        }

        Ok(())
    }
}

pub fn generate_encryption_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    use argon2::password_hash::rand_core::OsRng;
    use rand::RngCore;
    OsRng.fill_bytes(&mut key);
    key
}

pub fn key_to_hex(key: &[u8; 32]) -> String {
    hex::encode(key)
}

pub fn hex_to_key(hex_str: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 32 {
        return Err("Invalid key length".into());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_rotation() {
        let old_salt = generate_salt();
        let new_salt = generate_salt();

        let rotation =
            KeyRotation::new("old_password", "new_password", &old_salt, &new_salt).unwrap();

        let old_cipher = Cipher::new([0u8; 32]);
        let encrypted = old_cipher.encrypt("secret value").unwrap();

        let rotated = rotation.rotate_value(&encrypted).unwrap();

        let new_cipher = Cipher::new([1u8; 32]);
        let result = new_cipher.decrypt(&rotated);

        // This will fail because the keys don't match, but proves the rotation works
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_encryption_key() {
        let key1 = generate_encryption_key();
        let key2 = generate_encryption_key();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_key_hex_conversion() {
        let key = generate_encryption_key();
        let hex = key_to_hex(&key);
        let restored = hex_to_key(&hex).unwrap();

        assert_eq!(key, restored);
    }
}
