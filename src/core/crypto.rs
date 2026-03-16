use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{password_hash::rand_core::OsRng, Argon2};
use hex;
use rand::RngCore;
use zeroize::{Zeroize, Zeroizing};

#[allow(dead_code)]
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| {
            Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
        })?;
    let result = key;
    key.zeroize();
    Ok(result)
}

pub fn is_key_too_weak(key: &[u8; 32]) -> bool {
    use std::collections::HashSet;

    // Check 1: Too few unique bytes (entropy check)
    let unique_bytes = key.iter().collect::<HashSet<_>>().len();
    if unique_bytes < 8 {
        return true;
    }

    // Check 2: All bytes the same
    if key.iter().all(|&b| b == key[0]) {
        return true;
    }

    // Check 3: Sequential pattern (0,1,2,3... or 255,254,253...)
    let is_sequential_asc = key
        .iter()
        .enumerate()
        .all(|(i, &b)| b as i16 == (key[0] as i16 + i as i16) % 256);
    let is_sequential_desc = key
        .iter()
        .enumerate()
        .all(|(i, &b)| b as i16 == (key[0] as i16 - i as i16).wrapping_rem(256));
    if is_sequential_asc || is_sequential_desc {
        return true;
    }

    // Check 4: Alternating pattern (like 0xAA, 0x55, 0xAA, 0x55...)
    if key.len() >= 4 {
        let alternating_two = key[0] == key[2] && key[1] == key[3] && key[0] != key[1];
        let all_alternating = key
            .windows(2)
            .all(|w| w[0] == key[0] && w[1] == key[1] || w[0] == key[1] && w[1] == key[0]);
        if alternating_two && all_alternating {
            return true;
        }
    }

    // Check 5: Half zeros or half 0xFF
    let zeros = key.iter().filter(|&&b| b == 0).count();
    let ones = key.iter().filter(|&&b| b == 0xFF).count();
    if zeros > 24 || ones > 24 {
        return true;
    }

    false
}

pub fn validate_key_strength(key: &[u8; 32]) -> Result<(), &'static str> {
    if is_key_too_weak(key) {
        return Err("Encryption key is too weak (insufficient entropy). Use a strong, random key.");
    }
    Ok(())
}

pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

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

    // Use Zeroizing for plaintext data in memory
    let mut plaintext = Zeroizing::new(data.as_bytes().to_vec());
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).map_err(|e| {
        Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
    })?;

    // Zeroize plaintext immediately after encryption
    plaintext.zeroize();

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

    let plaintext_bytes = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| {
        Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
    })?;

    // Use Zeroizing for decrypted plaintext
    let mut plaintext = Zeroizing::new(plaintext_bytes);
    let result = String::from_utf8(plaintext.to_vec());
    plaintext.zeroize();

    Ok(result?)
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

    #[test]
    fn test_encrypt_with_different_keys() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let data = "test data";

        let encrypted1 = encrypt_data(data, &key1).unwrap();
        let encrypted2 = encrypt_data(data, &key2).unwrap();

        assert_ne!(encrypted1, encrypted2); // Different keys should produce different ciphertexts
    }

    #[test]
    fn test_decrypt_with_invalid_nonce() {
        let key = [0u8; 32];
        let data = "test";
        let encrypted = encrypt_data(data, &key).unwrap();

        // Modify the nonce part of the encrypted data
        let mut bytes = hex::decode(&encrypted).unwrap();
        if bytes.len() >= 12 {
            bytes[0] ^= 0xFF; // Flip bits in nonce
            let modified_hex = hex::encode(&bytes);
            assert!(decrypt_data(&modified_hex, &key).is_err());
        }
    }

    #[test]
    fn test_encrypt_large_data() {
        let key = [0u8; 32];
        let large_data = "x".repeat(10000); // 10KB of data
        let encrypted = encrypt_data(&large_data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(large_data, decrypted);
    }

    #[test]
    fn test_decrypt_with_truncated_ciphertext() {
        let key = [0u8; 32];
        let data = "test data for truncation";
        let encrypted = encrypt_data(data, &key).unwrap();

        // Truncate the ciphertext part (after the nonce)
        let mut bytes = hex::decode(&encrypted).unwrap();
        if bytes.len() > 12 {
            bytes.truncate(13); // Just nonce + 1 byte
            let truncated_hex = hex::encode(&bytes);
            assert!(decrypt_data(&truncated_hex, &key).is_err());
        }
    }

    #[test]
    fn test_encrypt_decrypt_with_zero_key() {
        let key = [0u8; 32]; // All zeros
        let data = "test with zero key";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_with_max_byte_key() {
        let key = [0xFFu8; 32]; // All max bytes
        let data = "test with max byte key";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_decrypt_with_invalid_hex_characters() {
        let key = [0u8; 32];
        // Test various invalid hex strings
        assert!(decrypt_data("XYZ123", &key).is_err());
        assert!(decrypt_data("gggg", &key).is_err());
        assert!(decrypt_data("!@#$%", &key).is_err());
    }

    #[test]
    fn test_encrypt_with_special_unicode_characters() {
        let key = [0u8; 32];
        let data = "Null byte:\0, Tab:\t, Newline:\n, Carriage return:\r";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_empty_key() {
        // This test verifies behavior when key is invalid
        // Since the key is fixed-size [u8; 32], we can't really have an empty key
        // But we can test with a key that has minimal entropy
        let mut key = [0u8; 32];
        key[0] = 1; // Only one bit set
        let data = "minimal entropy test";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_all_possible_bytes() {
        let key = [0u8; 32];
        // Create a string with all possible byte values (0-255) represented as readable characters
        let data = (0..255)
            .map(|i| (i % 95 + 32) as u8 as char)
            .collect::<String>();
        let encrypted = encrypt_data(&data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_extremely_long_key() {
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = i as u8;
        }
        let data = "test with patterned key";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_alternating_bit_pattern_key() {
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = if i % 2 == 0 { 0xAA } else { 0x55 }; // Alternating bit patterns
        }
        let data = "test with alternating key";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_maximal_entropy_key() {
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = (i * 7) as u8; // Spread values across range
        }
        let data = "test with maximal entropy key";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_with_different_sizes() {
        let key = [0u8; 32];

        // Test various sizes
        let sizes = [1, 10, 100, 1000, 5000, 10000];
        for size in sizes {
            let data = "x".repeat(size);
            let encrypted = encrypt_data(&data, &key).unwrap();
            let decrypted = decrypt_data(&encrypted, &key).unwrap();
            assert_eq!(data, decrypted);
        }
    }

    #[test]
    fn test_encrypt_with_special_control_characters() {
        let key = [0u8; 32];
        let data = "\0\n\r\t\x0B\x0C"; // null, newline, carriage return, tab, vertical tab, form feed
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_binary_data() {
        let key = [0u8; 32];
        // Create binary data that looks like it could be hex or other encodings
        let binary_data = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
        let data = String::from_utf8_lossy(&binary_data);
        let encrypted = encrypt_data(&data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_decrypt_with_padding_variations() {
        let key = [0u8; 32];
        let data = "padding test";
        let encrypted = encrypt_data(data, &key).unwrap();

        // Test that decryption fails with extra padding
        let padded_encrypted = format!("{}00", encrypted);
        assert!(decrypt_data(&padded_encrypted, &key).is_err());

        // Test that decryption fails with missing padding
        if encrypted.len() > 2 {
            let truncated_encrypted = &encrypted[..encrypted.len() - 2];
            assert!(decrypt_data(truncated_encrypted, &key).is_err());
        }
    }

    #[test]
    fn test_encrypt_decrypt_with_extreme_unicode() {
        let key = [0u8; 32];
        // Test with various Unicode planes
        let data = "ASCII: hello\nBMP: café\nSMP: 𐐷\nEmoji: 😀🎉🚀\nCJK: 你好世界\nArabic: مرحبا";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_empty_key_bytes() {
        let key = [0u8; 32]; // All zero key
        let data = "test data";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_max_key_bytes() {
        let key = [0xFFu8; 32]; // All max bytes
        let data = "test data";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_alternating_key_pattern() {
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = if i % 2 == 0 { 0 } else { 0xFF };
        }
        let data = "alternating key test";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_primes_in_key() {
        let mut key = [0u8; 32];
        let primes = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97, 101, 103, 107, 109, 113, 127, 131,
        ];
        for i in 0..32 {
            key[i] = primes[i % primes.len()];
        }
        let data = "prime key test";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_fibonacci_key() {
        let mut key = [0u8; 32];
        let mut a = 0u8;
        let mut b = 1u8;
        for i in 0..32 {
            key[i] = a;
            let next = a.wrapping_add(b);
            a = b;
            b = next;
        }
        let data = "fibonacci key test";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_very_long_strings() {
        let key = [0u8; 32];
        let data = "A".repeat(50000); // 50KB string
        let encrypted = encrypt_data(&data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_all_ascii_chars() {
        let key = [0u8; 32];
        let data: String = (0..128).map(|c| c as u8 as char).collect(); // All ASCII chars
        let encrypted = encrypt_data(&data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_null_bytes_in_data() {
        let key = [0u8; 32];
        let data = "string\0with\0null\0bytes";
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_encrypt_with_special_control_chars() {
        let key = [0u8; 32];
        let data = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"; // First 16 control chars
        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }
}
