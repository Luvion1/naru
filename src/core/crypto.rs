pub use crate::core::cipher::Cipher;
pub use crate::core::key_derivation::{generate_salt, validate_key_strength};

pub fn encrypt_data(data: &str, key: &[u8; 32]) -> Result<String, Box<dyn std::error::Error>> {
    let cipher = Cipher::new(*key);
    cipher.encrypt(data)
}

pub fn decrypt_data(
    encrypted_hex: &str,
    key: &[u8; 32],
) -> Result<String, Box<dyn std::error::Error>> {
    let cipher = Cipher::new(*key);
    cipher.decrypt(encrypted_hex)
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    key: &[u8; 32],
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::core::file_security::check_file_size;
    use crate::core::path_sanitizer::sanitize_file_path;

    let sanitized_input = sanitize_file_path(input_path)?;
    let sanitized_output = sanitize_file_path(output_path)?;

    check_file_size(&sanitized_input, 10 * 1024 * 1024)?;

    let content = std::fs::read_to_string(&sanitized_input)?;
    let encrypted = encrypt_data(&content, key)?;
    std::fs::write(&sanitized_output, encrypted)?;
    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    key: &[u8; 32],
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::core::file_security::check_file_size;
    use crate::core::path_sanitizer::sanitize_file_path;

    let sanitized_input = sanitize_file_path(input_path)?;
    let sanitized_output = sanitize_file_path(output_path)?;

    check_file_size(&sanitized_input, 10 * 1024 * 1024)?;

    let encrypted_content = std::fs::read_to_string(&sanitized_input)?;
    let decrypted = decrypt_data(&encrypted_content, key)?;
    std::fs::write(&sanitized_output, decrypted)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_data() {
        let key = [0u8; 32];
        let original = "Hello World!";

        let encrypted = encrypt_data(original, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_encrypt_file_integration() {
        use file_crypto::{decrypt_file, encrypt_file};
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.txt");
        let encrypted_file = temp_dir.path().join("encrypted.txt");
        let decrypted_file = temp_dir.path().join("decrypted.txt");

        std::fs::write(&input_file, "Test data").unwrap();

        encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            "password",
        )
        .unwrap();

        decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            "password",
        )
        .unwrap();

        let original = std::fs::read_to_string(&input_file).unwrap();
        let decrypted = std::fs::read_to_string(&decrypted_file).unwrap();

        assert_eq!(original, decrypted);
    }
}
