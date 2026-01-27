use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hex;
use rand::RngCore;
use rand::rngs::OsRng;

pub fn encrypt_data(data: &str, key: &[u8; 32]) -> Result<String, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        )) as Box<dyn std::error::Error>
    })?;

    // Generate a random nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data.as_ref()).map_err(|e| {
        Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
    })?;

    // Combine nonce and ciphertext, then encode as hex
    let mut encrypted_data = nonce_bytes.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);
    Ok(hex::encode(&encrypted_data))
}

pub fn decrypt_data(
    encrypted_hex: &str,
    key: &[u8; 32],
) -> Result<String, Box<dyn std::error::Error>> {
    let encrypted_data = hex::decode(encrypted_hex)?;

    if encrypted_data.len() < 12 {
        return Err("Invalid encrypted data length".into());
    }

    let nonce_bytes = &encrypted_data[..12];
    let ciphertext = &encrypted_data[12..];

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        )) as Box<dyn std::error::Error>
    })?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| {
        Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
    })?;

    Ok(String::from_utf8(plaintext)?)
}

/// Fungsi untuk mengenkripsi seluruh file
pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    key: &[u8; 32],
) -> Result<(), Box<dyn std::error::Error>> {
    use super::security;

    // Sanitize file paths to prevent directory traversal
    let sanitized_input_path = security::sanitize_file_path(input_path)?;
    let sanitized_output_path = security::sanitize_file_path(output_path)?;

    // Check input file size before reading (max 10MB)
    security::check_file_size(&sanitized_input_path, 10 * 1024 * 1024) // 10MB limit
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let content = std::fs::read_to_string(sanitized_input_path)?;
    let encrypted = encrypt_data(&content, key)?;
    std::fs::write(sanitized_output_path, encrypted)?;
    Ok(())
}

/// Fungsi untuk mendekripsi seluruh file
pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    key: &[u8; 32],
) -> Result<(), Box<dyn std::error::Error>> {
    use super::security;

    // Sanitize file paths to prevent directory traversal
    let sanitized_input_path = security::sanitize_file_path(input_path)?;
    let sanitized_output_path = security::sanitize_file_path(output_path)?;

    // Check input file size before reading (max 10MB)
    security::check_file_size(&sanitized_input_path, 1024 * 1024 * 10) // 10MB limit
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let encrypted_content = std::fs::read_to_string(sanitized_input_path)?;
    let decrypted = decrypt_data(&encrypted_content, key)?;
    std::fs::write(sanitized_output_path, decrypted)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32];
        let original = "Hello World!";
        let result = encrypt_data(original, &key);
        assert!(result.is_ok());

        if let Ok(encrypted) = result {
            let decrypted = decrypt_data(&encrypted, &key).unwrap();
            assert_eq!(original, decrypted);
            assert_ne!(original, encrypted);
        }
    }

    #[test]
    fn test_encryption_randomness() {
        let key = [0u8; 32];
        let data = "consistent data";
        let encrypted1 = encrypt_data(data, &key).unwrap();
        let encrypted2 = encrypt_data(data, &key).unwrap();
        // Each encryption should have a different nonce, thus different output
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_encrypt_empty_string() {
        let key = [0u8; 32];
        let data = "";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_decrypt_invalid_hex() {
        let key = [0u8; 32];
        let result = decrypt_data("not-hex-at-all", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        let key = [0u8; 32];
        let result = decrypt_data("aabbccddeeff", &key); // Less than 12 bytes nonce
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_ciphertext() {
        let key = [0u8; 32];
        let data = "original secret";
        let encrypted = encrypt_data(data, &key).unwrap();

        // Corrupt one character in the ciphertext part
        let mut corrupted = encrypted.clone();
        if let Some(last_char) = corrupted.pop() {
            let new_char = if last_char == '0' { '1' } else { '0' };
            corrupted.push(new_char);
        }

        let result = decrypt_data(&corrupted, &key);
        assert!(
            result.is_err(),
            "Decryption should fail if ciphertext is tampered"
        );
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let data = "super secret";

        let encrypted = encrypt_data(data, &key1).unwrap();
        let result = decrypt_data(&encrypted, &key2);

        assert!(result.is_err(), "Decryption must fail with wrong key");
    }

    #[test]
    fn test_encrypt_decrypt_unicode_data() {
        let key = [0u8; 32];
        let data = "秘事: 🚀🦀";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }
}
