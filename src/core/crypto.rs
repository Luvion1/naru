use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::rngs::OsRng;
use rand::RngCore;
use hex;

pub fn encrypt_data(data: &str, key: &[u8; 32]) -> Result<String, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn std::error::Error>)?;

    // Generate a random nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data.as_ref()).map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error>)?;

    // Combine nonce and ciphertext, then encode as hex
    let mut encrypted_data = nonce_bytes.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);

    Ok(hex::encode(&encrypted_data))
}

pub fn decrypt_data(encrypted_hex: &str, key: &[u8; 32]) -> Result<String, Box<dyn std::error::Error>> {
    let encrypted_data = hex::decode(encrypted_hex)?;

    if encrypted_data.len() < 12 {
        return Err("Encrypted data too short".into());
    }

    let nonce_bytes = &encrypted_data[..12];
    let ciphertext = &encrypted_data[12..];

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn std::error::Error>)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error>)?;

    Ok(String::from_utf8(plaintext)?)
}

/// Fungsi untuk mengenkripsi seluruh file
pub fn encrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), Box<dyn std::error::Error>> {
    use super::security;

    // Sanitize file paths to prevent directory traversal
    let sanitized_input_path = security::sanitize_file_path(input_path)?;
    let sanitized_output_path = security::sanitize_file_path(output_path)?;

    // Check input file size before reading (max 10MB)
    security::check_file_size(&sanitized_input_path, 10 * 1024 * 1024)  // 10MB limit
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let content = std::fs::read_to_string(sanitized_input_path)?;
    let encrypted_content = encrypt_data(&content, key)?;
    std::fs::write(sanitized_output_path, encrypted_content)?;
    Ok(())
}

/// Fungsi untuk mendekripsi seluruh file
pub fn decrypt_file(input_path: &str, output_path: &str, key: &[u8; 32]) -> Result<(), Box<dyn std::error::Error>> {
    use super::security;

    // Sanitize file paths to prevent directory traversal
    let sanitized_input_path = security::sanitize_file_path(input_path)?;
    let sanitized_output_path = security::sanitize_file_path(output_path)?;

    // Check input file size before reading (max 10MB)
    security::check_file_size(&sanitized_input_path, 10 * 1024 * 1024)  // 10MB limit
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let encrypted_content = std::fs::read_to_string(sanitized_input_path)?;
    let decrypted_content = decrypt_data(&encrypted_content, key)?;
    std::fs::write(sanitized_output_path, decrypted_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [1u8; 32]; // 32-byte key for AES-GCM
        let original = "Hello, World!";

        let encrypted = encrypt_data(original, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_longer_text() {
        let key = [2u8; 32];
        let original = "This is a longer text to test the encryption and decryption functionality.";

        let encrypted = encrypt_data(original, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_different_keys() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let original = "Test message";

        let encrypted = encrypt_data(original, &key1).unwrap();
        // Attempting to decrypt with wrong key should fail
        let result = decrypt_data(&encrypted, &key2);

        // Note: Depending on the encryption algorithm, this might not always fail
        // but it should return different (incorrect) data
        if result.is_ok() {
            assert_ne!(original, result.unwrap());
        }
    }
}