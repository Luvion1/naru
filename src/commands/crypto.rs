use crate::core::crypto;
use crate::core::security;
use anyhow::{anyhow, Result};
use std::env;

pub struct CryptoEncryptCommand {
    pub input_file: String,
    pub output_file: String,
}

pub struct CryptoDecryptCommand {
    pub input_file: String,
    pub output_file: String,
}

pub fn execute_encrypt(cmd: CryptoEncryptCommand) -> Result<()> {
    let key_str = env::var("NARU_ENCRYPTION_KEY")
        .map_err(|_| anyhow!("NARU_ENCRYPTION_KEY environment variable not set"))?;

    if key_str.len() < 32 {
        return Err(anyhow!(
            "Encryption key must be at least 32 characters long"
        ));
    }

    let unique_chars: std::collections::HashSet<char> = key_str.chars().collect();
    let unique_ratio = unique_chars.len() as f64 / key_str.len() as f64;
    if unique_ratio < 0.5 {
        return Err(anyhow!(
            "Encryption key has low entropy (too many repeated characters). Use a stronger key with more variety."
        ));
    }

    let mut key = [0u8; 32];
    let bytes = key_str.as_bytes();
    let len = std::cmp::min(bytes.len(), 32);
    key[..len].copy_from_slice(&bytes[..len]);

    let sanitized_input_file = security::sanitize_file_path(&cmd.input_file)
        .map_err(|e| anyhow!("Invalid input file path: {}", e))?;

    let sanitized_output_file = security::sanitize_file_path(&cmd.output_file)
        .map_err(|e| anyhow!("Invalid output file path: {}", e))?;

    crypto::encrypt_file(
        sanitized_input_file
            .to_str()
            .ok_or_else(|| anyhow!("Invalid input file path"))?,
        sanitized_output_file
            .to_str()
            .ok_or_else(|| anyhow!("Invalid output file path"))?,
        &key,
    )
    .map_err(|e| anyhow!("Failed to encrypt file: {}", e))?;
    println!(
        "File encrypted successfully from {} to {}",
        cmd.input_file, cmd.output_file
    );
    Ok(())
}

pub fn execute_decrypt(cmd: CryptoDecryptCommand) -> Result<()> {
    let key_str = env::var("NARU_ENCRYPTION_KEY")
        .map_err(|_| anyhow!("NARU_ENCRYPTION_KEY environment variable not set"))?;

    if key_str.len() < 32 {
        return Err(anyhow!(
            "Encryption key must be at least 32 characters long"
        ));
    }

    let unique_chars: std::collections::HashSet<char> = key_str.chars().collect();
    let unique_ratio = unique_chars.len() as f64 / key_str.len() as f64;
    if unique_ratio < 0.5 {
        return Err(anyhow!(
            "Encryption key has low entropy (too many repeated characters). Use a stronger key with more variety."
        ));
    }

    let mut key = [0u8; 32];
    let bytes = key_str.as_bytes();
    let len = std::cmp::min(bytes.len(), 32);
    key[..len].copy_from_slice(&bytes[..len]);

    let sanitized_input_file = security::sanitize_file_path(&cmd.input_file)
        .map_err(|e| anyhow!("Invalid input file path: {}", e))?;

    let sanitized_output_file = security::sanitize_file_path(&cmd.output_file)
        .map_err(|e| anyhow!("Invalid output file path: {}", e))?;

    crypto::decrypt_file(
        sanitized_input_file
            .to_str()
            .ok_or_else(|| anyhow!("Invalid input file path"))?,
        sanitized_output_file
            .to_str()
            .ok_or_else(|| anyhow!("Invalid output file path"))?,
        &key,
    )
    .map_err(|e| anyhow!("Failed to decrypt file: {}", e))?;
    println!(
        "File decrypted successfully from {} to {}",
        cmd.input_file, cmd.output_file
    );
    Ok(())
}
