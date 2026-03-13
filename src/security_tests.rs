/// Security Testing Suite untuk Naru

#[cfg(test)]
mod security_tests {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_path_traversal_basic() {
        println!("\n🔴 [PATH TRAVERSAL] Testing basic traversal attacks...");
        let attacks = vec![
            "../etc/passwd",
            "../../etc/shadow",
            "../../../root/.ssh/id_rsa",
        ];
        for attack in attacks {
            let result = crate::core::security::sanitize_file_path(attack);
            println!("  Testing: {:40} → {:?}", attack, result.is_err());
            assert!(result.is_err());
        }
        println!("  ✅ Path traversal attacks blocked\n");
    }

    #[test]
    fn test_path_traversal_null_byte() {
        println!("🔴 [PATH TRAVERSAL] Testing null byte injection...");
        let result = crate::core::security::sanitize_file_path("config.json\0.txt");
        assert!(result.is_err());
        println!("  ✅ Null byte injections blocked\n");
    }

    #[test]
    fn test_env_name_injection() {
        println!("🔴 [INJECTION] Testing environment name injection...");
        let attacks = vec![
            "dev; rm -rf /",
            "prod && cat /etc/passwd",
            "staging$(whoami)",
        ];
        for attack in attacks {
            let result = crate::core::security::validate_environment_name(attack);
            assert!(result.is_err());
        }
        println!("  ✅ Environment injection attacks blocked\n");
    }

    #[test]
    fn test_config_key_injection() {
        println!("🔴 [INJECTION] Testing config key injection...");
        let attacks = vec!["KEY; DROP TABLE users;--", "SECRET|cat /etc/passwd"];
        for attack in attacks {
            let result = crate::core::security::validate_config_key(attack);
            assert!(result.is_err());
        }
        println!("  ✅ Config key injection attacks blocked\n");
    }

    #[test]
    fn test_encryption_nonce_uniqueness() {
        println!("🔴 [ENCRYPTION] Testing nonce uniqueness...");
        let key = [0u8; 32];
        let mut ciphertexts = std::collections::HashSet::new();
        for _ in 0..100 {
            let encrypted = crate::core::crypto::encrypt_data("data", &key).unwrap();
            assert!(ciphertexts.insert(encrypted));
        }
        println!("  ✅ 100 encryptions produced unique ciphertexts\n");
    }

    #[test]
    fn test_encryption_tampering() {
        println!("🔴 [ENCRYPTION] Testing ciphertext tampering detection...");
        let key = [0x42u8; 32];
        let encrypted = crate::core::crypto::encrypt_data("secret", &key).unwrap();
        let mut bytes = hex::decode(&encrypted).unwrap();
        if bytes.len() > 13 {
            bytes[13] ^= 0xFF;
        }
        let result = crate::core::crypto::decrypt_data(&hex::encode(&bytes), &key);
        assert!(result.is_err());
        println!("  ✅ Tampered ciphertext rejected\n");
    }

    #[test]
    fn test_encryption_wrong_key() {
        println!("🔴 [ENCRYPTION] Testing wrong key detection...");
        let key1 = [0x01u8; 32];
        let key2 = [0x02u8; 32];
        let encrypted = crate::core::crypto::encrypt_data("data", &key1).unwrap();
        assert!(crate::core::crypto::decrypt_data(&encrypted, &key2).is_err());
        println!("  ✅ Wrong key detection working\n");
    }

    #[test]
    fn test_file_size_limit() {
        println!("🔴 [DOS] Testing file size limit...");
        let temp_dir = TempDir::new().unwrap();
        let small = temp_dir.path().join("small.txt");
        fs::write(&small, "small").unwrap();
        assert!(crate::core::security::check_file_size(&small, 1024 * 1024).is_ok());

        let large = temp_dir.path().join("large.txt");
        fs::write(&large, &vec![0u8; 2 * 1024 * 1024]).unwrap();
        assert!(crate::core::security::check_file_size(&large, 1024 * 1024).is_err());
        println!("  ✅ File size limits working\n");
    }

    #[test]
    fn run_all_security_tests() {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║         NARU SECURITY TESTING SUITE                      ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");

        test_path_traversal_basic();
        test_path_traversal_null_byte();
        test_env_name_injection();
        test_config_key_injection();
        test_encryption_nonce_uniqueness();
        test_encryption_tampering();
        test_encryption_wrong_key();
        test_file_size_limit();

        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║         SECURITY TESTING COMPLETE                        ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
    }
}
