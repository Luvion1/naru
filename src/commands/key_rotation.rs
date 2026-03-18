use crate::core::constants::{CONFIG_FILE, NARU_DIR, SCHEMA_FILE};
use crate::core::key_derivation::generate_salt;
use crate::core::key_rotation::{generate_encryption_key, hex_to_key, key_to_hex, KeyRotation};
use crate::core::persistence;
use anyhow::Result;

pub struct KeyRotateCommand {
    pub old_password: String,
    pub new_password: String,
    pub rotate_schema: bool,
}

impl KeyRotateCommand {
    pub fn new(old_password: String, new_password: String) -> Self {
        Self {
            old_password,
            new_password,
            rotate_schema: true,
        }
    }

    pub fn execute(&self) -> Result<()> {
        let config = persistence::atomic_read_config(|c| c.clone())
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

        let old_salt = config
            .salt
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No salt found in config"))?;

        let old_salt_bytes =
            hex::decode(old_salt).map_err(|e| anyhow::anyhow!("Invalid salt: {}", e))?;

        let new_salt = generate_salt();

        let rotation = KeyRotation::new(
            &self.old_password,
            &self.new_password,
            &old_salt_bytes,
            &new_salt,
        )?;

        let mut config_clone = config.clone();
        rotation.rotate_config(&mut config_clone)?;

        persistence::atomic_update_config(|config| {
            *config = config_clone.clone();
            if let Some(ref mut salt) = config.salt {
                *salt = hex::encode(new_salt);
            }
        })?;

        println!("✓ Key rotation completed successfully");
        println!("  All encrypted values have been re-encrypted with the new key");

        if self.rotate_schema {
            println!("  Schema encryption updated");
        }

        Ok(())
    }
}

pub fn rotate_encryption_key(old_password: &str, new_password: &str) -> Result<()> {
    let cmd = KeyRotateCommand::new(old_password.to_string(), new_password.to_string());
    cmd.execute()
}

pub fn generate_new_key() -> Result<String> {
    let key = generate_encryption_key();
    Ok(key_to_hex(&key))
}

pub fn import_key(hex_key: &str) -> Result<()> {
    let key = hex_to_key(hex_key)?;

    std::env::set_var("NARU_ENCRYPTION_KEY", hex::encode(key));

    println!("✓ Key imported successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_new_key() {
        let key1 = generate_new_key().unwrap();
        let key2 = generate_new_key().unwrap();

        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_hex_key_conversion() {
        let key = generate_encryption_key();
        let hex = key_to_hex(&key);
        let restored = hex_to_key(&hex).unwrap();

        assert_eq!(key, restored);
    }
}
