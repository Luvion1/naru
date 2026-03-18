use crate::core::cipher::Cipher;
use crate::core::key_derivation::derive_key;

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::core::file_security::check_file_size;
    use crate::core::path_sanitizer::sanitize_file_path;

    let sanitized_input = sanitize_file_path(input_path)?;
    let sanitized_output = sanitize_file_path(output_path)?;

    check_file_size(&sanitized_input, 10 * 1024 * 1024)?;

    let content = std::fs::read_to_string(&sanitized_input)?;

    let salt = crate::core::key_derivation::generate_salt();
    let key = derive_key(password, &salt)?;

    let cipher = Cipher::new(key);
    let encrypted = cipher.encrypt(&content)?;

    let mut output = hex::encode(salt).to_string();
    output.push_str("\n");
    output.push_str(&encrypted);

    std::fs::write(&sanitized_output, output)?;

    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::core::file_security::check_file_size;
    use crate::core::path_sanitizer::sanitize_file_path;

    let sanitized_input = sanitize_file_path(input_path)?;
    let sanitized_output = sanitize_file_path(output_path)?;

    check_file_size(&sanitized_input, 10 * 1024 * 1024)?;

    let content = std::fs::read_to_string(&sanitized_input)?;

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Err("Invalid encrypted file format".into());
    }

    let salt = hex::decode(lines[0])?;
    let encrypted_data = lines.get(1).unwrap_or(&"");

    let key = derive_key(password, &salt)?;
    let cipher = Cipher::new(key);
    let decrypted = cipher.decrypt(encrypted_data)?;

    std::fs::write(&sanitized_output, decrypted)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_decrypt_file() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.txt");
        let encrypted_file = temp_dir.path().join("encrypted.txt");
        let decrypted_file = temp_dir.path().join("decrypted.txt");

        std::fs::write(&input_file, "Hello, World!").unwrap();

        encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            "password123",
        )
        .unwrap();

        decrypt_file(
            encrypted_file.to_str().unwrap(),
            decrypted_file.to_str().unwrap(),
            "password123",
        )
        .unwrap();

        let original = std::fs::read_to_string(&input_file).unwrap();
        let decrypted = std::fs::read_to_string(&decrypted_file).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_encrypt_file_different_password() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.txt");
        let encrypted_file = temp_dir.path().join("encrypted.txt");

        std::fs::write(&input_file, "Secret data").unwrap();

        encrypt_file(
            input_file.to_str().unwrap(),
            encrypted_file.to_str().unwrap(),
            "correct_password",
        )
        .unwrap();

        let temp_dir2 = TempDir::new().unwrap();
        let wrong_output = temp_dir2.path().join("wrong.txt");

        let result = decrypt_file(
            encrypted_file.to_str().unwrap(),
            wrong_output.to_str().unwrap(),
            "wrong_password",
        );

        assert!(result.is_err());
    }
}
