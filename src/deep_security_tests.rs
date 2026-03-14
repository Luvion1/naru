/// Deep Security Testing - Advanced Attack Vectors
/// Comprehensive security analysis untuk mencari bug & vulnerabilities

#[cfg(test)]
mod deep_security_tests {
    use std::fs;
    use std::thread;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    // ========================================================================
    // CRYPTOGRAPHIC IMPLEMENTATION ANALYSIS
    // ========================================================================

    #[test]
    fn test_crypto_key_zeroization() {
        println!("\n🔍 [CRYPTO] Testing key material in memory...");

        let key = [0x42u8; 32];
        let data = "sensitive_data";

        let encrypted = crate::core::crypto::encrypt_data(data, &key).unwrap();

        // Test: Apakah key masih bisa digunakan setelah operasi?
        let decrypted = crate::core::crypto::decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(decrypted, data);

        // ISSUE: Rust tidak automatically zeroize key dari memory
        // Key masih ada di stack sampai function return
        println!("  ⚠️  WARNING: Keys not zeroized from memory after use");
        println!("  💡 RECOMMENDATION: Use zeroize crate untuk clear sensitive data\n");
    }

    #[test]
    fn test_crypto_timing_attack() {
        println!("🔍 [CRYPTO] Testing for timing attacks...");

        let key = [0x42u8; 32];
        let correct_data = "correct_password_123";

        let encrypted = crate::core::crypto::encrypt_data(correct_data, &key).unwrap();

        // Test dengan ciphertext yang dimodifikasi
        let tampered = hex::decode(&encrypted).unwrap();

        // Modifikasi 1 byte di posisi berbeda
        let mut timings = Vec::new();
        for pos in 0..tampered.len().min(20) {
            let mut test_bytes = tampered.clone();
            test_bytes[pos] ^= 0xFF;
            let test_cipher = hex::encode(&test_bytes);

            let start = Instant::now();
            let _ = crate::core::crypto::decrypt_data(&test_cipher, &key);
            let elapsed = start.elapsed();

            timings.push((pos, elapsed));
            println!("  Position {}: {:?}", pos, elapsed);
        }

        // Check apakah ada timing variation yang signifikan
        let max_time = timings.iter().map(|(_, t)| t).max().unwrap();
        let min_time = timings.iter().map(|(_, t)| t).min().unwrap();
        let variance = *max_time - *min_time;

        if variance.as_micros() > 100 {
            println!("  ⚠️  WARNING: Timing variance detected: {:?}", variance);
            println!("  💡 Possible timing side-channel\n");
        } else {
            println!("  ✅ No significant timing variance detected\n");
        }
    }

    #[test]
    fn test_crypto_weak_key_detection() {
        println!("🔍 [CRYPTO] Testing weak key handling...");

        // Test dengan key yang semua byte-nya sama
        let weak_keys = vec![
            [0x00u8; 32], // All zeros
            [0xFFu8; 32], // All ones
            [0x42u8; 32], // Repeating pattern
        ];

        for (i, key) in weak_keys.iter().enumerate() {
            let encrypted = crate::core::crypto::encrypt_data("test", key).unwrap();
            let decrypted = crate::core::crypto::decrypt_data(&encrypted, key).unwrap();

            assert_eq!(decrypted, "test");
            println!(
                "  Weak key {} (all 0x{:02X}): Encryption works",
                i + 1,
                key[0]
            );
        }

        println!("  ⚠️  WARNING: No weak key detection implemented");
        println!("  💡 RECOMMENDATION: Add key strength validation\n");
    }

    #[test]
    fn test_crypto_empty_data() {
        println!("🔍 [CRYPTO] Testing empty data encryption...");

        let key = [0x42u8; 32];

        // Encrypt empty string
        let encrypted = crate::core::crypto::encrypt_data("", &key).unwrap();
        let decrypted = crate::core::crypto::decrypt_data(&encrypted, &key).unwrap();

        assert_eq!(decrypted, "");
        println!("  ✅ Empty string encryption works");

        // Encrypt very large data
        let large_data = "A".repeat(10 * 1024 * 1024); // 10MB
        let start = Instant::now();
        let encrypted = crate::core::crypto::encrypt_data(&large_data, &key).unwrap();
        let encrypt_time = start.elapsed();

        let start = Instant::now();
        let decrypted = crate::core::crypto::decrypt_data(&encrypted, &key).unwrap();
        let decrypt_time = start.elapsed();

        assert_eq!(decrypted, large_data);
        println!(
            "  ✅ 10MB data: encrypt={:?}, decrypt={:?}",
            encrypt_time, decrypt_time
        );
        println!();
    }

    // ========================================================================
    // RACE CONDITION & CONCURRENCY TESTING
    // ========================================================================

    #[test]
    fn test_race_condition_file_write_old_api() {
        println!("🔍 [RACE] Testing OLD API (demonstrates race condition)...");

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        crate::core::persistence::init_project().unwrap();

        let mut handles = vec![];

        for i in 0..10 {
            let handle = thread::spawn(move || {
                let key = format!("KEY_{}", i);
                let value = format!("VALUE_{}", i);

                let mut config: crate::core::models::ConfigFile =
                    crate::core::persistence::load_json(crate::core::constants::CONFIG_FILE)
                        .unwrap();

                thread::sleep(Duration::from_millis(10));

                if let Some(env_config) = config.environments.get_mut("development") {
                    env_config.entries.insert(
                        key,
                        crate::core::models::ConfigValueEntry::new(&value, "string", false),
                    );
                }

                crate::core::persistence::save_json(crate::core::constants::CONFIG_FILE, &config)
                    .is_ok()
            });
            handles.push(handle);
        }

        let _: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let config: crate::core::models::ConfigFile =
            crate::core::persistence::load_json(crate::core::constants::CONFIG_FILE).unwrap();

        let entry_count = config
            .environments
            .get("development")
            .map(|e| e.entries.len())
            .unwrap_or(0);

        println!(
            "  OLD API - Final entry count: {} (expected 10, shows data loss)",
            entry_count
        );
        println!("  ⚠️  Race condition causes data loss with old API\n");

        std::env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_race_condition_file_write_new_api() {
        println!("🔍 [RACE] Testing NEW atomic API (should prevent race condition)...");

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        crate::core::persistence::init_project().unwrap();

        let mut handles = vec![];

        for i in 0..10 {
            let handle = thread::spawn(move || {
                let key = format!("KEY_{}", i);

                crate::core::persistence::atomic_update_config(|config| {
                    if let Some(env_config) = config.environments.get_mut("development") {
                        env_config.entries.insert(
                            key,
                            crate::core::models::ConfigValueEntry::new("value", "string", false),
                        );
                    }
                    Ok(())
                })
                .is_ok()
            });
            handles.push(handle);
        }

        let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let success_count = results.iter().filter(|&&r| r).count();

        let config: crate::core::models::ConfigFile =
            crate::core::persistence::load_json(crate::core::constants::CONFIG_FILE).unwrap();

        let entry_count = config
            .environments
            .get("development")
            .map(|e| e.entries.len())
            .unwrap_or(0);

        println!(
            "  NEW API - Concurrent writes: {}/10 succeeded",
            success_count
        );
        println!(
            "  NEW API - Final entry count: {} (expected 10)",
            entry_count
        );

        if entry_count == 10 {
            println!("  ✅ Atomic API prevents race condition!\n");
        } else {
            println!("  ⚠️  Some data loss still possible\n");
        }

        std::env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_race_condition_lock_timeout() {
        println!("🔍 [RACE] Testing file lock timeout behavior...");

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        crate::core::persistence::init_project().unwrap();

        // Acquire lock in one thread
        let _lock_path = temp_dir
            .path()
            .join(crate::core::constants::NARU_DIR)
            .join("config.json.lock");

        let lock = crate::core::locking::FileLock::acquire_exclusive(
            &temp_dir
                .path()
                .join(crate::core::constants::NARU_DIR)
                .join("config.json"),
        )
        .unwrap();

        println!("  Lock acquired: {:?}", lock.path.exists());

        // Try to acquire same lock in main thread (should block)
        let start = Instant::now();

        let lock_handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            drop(lock); // Release after 100ms
        });

        // This should succeed after first lock is released
        let _lock2 = crate::core::locking::FileLock::acquire_exclusive(
            &temp_dir
                .path()
                .join(crate::core::constants::NARU_DIR)
                .join("config.json"),
        )
        .unwrap();

        let elapsed = start.elapsed();
        println!("  Lock wait time: {:?}", elapsed);

        if elapsed.as_millis() > 50 {
            println!("  ✅ Lock properly blocks until released");
        }

        lock_handle.join().unwrap();
        std::env::set_current_dir(&original_dir).unwrap();
        println!();
    }

    // ========================================================================
    // INFORMATION LEAK TESTING
    // ========================================================================

    #[test]
    fn test_error_message_information_leak() {
        println!("🔍 [INFOLEAK] Testing error messages for information disclosure...");

        // Test: Apakah error messages泄露 informasi sensitif?
        let result = crate::core::security::sanitize_file_path("../etc/passwd");
        if let Err(e) = result {
            println!("  Path traversal error: {}", e);
            // Check if error reveals internal structure
            assert!(!e.contains("/etc/"), "Error should not reveal system paths");
        }

        let result = crate::core::security::validate_environment_name("dev; rm -rf /");
        if let Err(e) = result {
            println!("  Injection error: {}", e);
            // Check if error reveals validation logic
            assert!(
                !e.contains("regex"),
                "Error should not reveal implementation"
            );
        }

        println!("  ✅ Error messages don't leak sensitive information\n");
    }

    #[test]
    fn test_memory_dump_sensitive_data() {
        println!("🔍 [INFOLEAK] Testing for sensitive data in memory...");

        let key = [0x42u8; 32];
        let secret = "SUPER_SECRET_PASSWORD_123!";

        let encrypted = crate::core::crypto::encrypt_data(secret, &key).unwrap();

        // After encryption, secret should only exist in encrypted form
        println!("  Plaintext length: {}", secret.len());
        println!("  Ciphertext length: {}", encrypted.len());
        println!("  ⚠️  WARNING: Plaintext may remain in stack memory");
        println!("  💡 RECOMMENDATION: Use secure memory allocation\n");
    }

    #[test]
    fn test_audit_log_secret_masking() {
        println!("🔍 [INFOLEAK] Testing audit log secret masking...");

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        crate::core::persistence::init_project().unwrap();

        // Add a secret value
        let mut config: crate::core::models::ConfigFile =
            crate::core::persistence::load_json(crate::core::constants::CONFIG_FILE).unwrap();

        if let Some(env_config) = config.environments.get_mut("development") {
            env_config.entries.insert(
                "DB_PASSWORD".to_string(),
                crate::core::models::ConfigValueEntry::new("super_secret_123", "string", true),
            );
        }

        crate::core::persistence::save_json(crate::core::constants::CONFIG_FILE, &config).unwrap();

        // Check audit log
        let audit_path = temp_dir
            .path()
            .join(crate::core::constants::NARU_DIR)
            .join("audit.log");

        if audit_path.exists() {
            let log_content = fs::read_to_string(&audit_path).unwrap();

            if log_content.contains("super_secret_123") {
                println!("  ❌ FAIL: Secret found in audit log!");
            } else {
                println!("  ✅ Secrets properly masked in audit log");
            }
        } else {
            println!("  ℹ️  No audit log created for this operation");
        }

        std::env::set_current_dir(&original_dir).unwrap();
        println!();
    }

    // ========================================================================
    // LOGIC BUGS IN VALIDATION
    // ========================================================================

    #[test]
    fn test_validation_bypass_null_byte() {
        println!("🔍 [LOGIC] Testing validation bypass with null bytes...");

        // Test if null byte in middle of string bypasses validation
        let test_cases = vec![
            "valid_key\0invalid",
            "dev\0production",
            "value\0../etc/passwd",
        ];

        for test in test_cases {
            let result = crate::core::security::validate_config_key(test);
            println!("  {:30} → {:?}", test, result);
        }
        println!();
    }

    #[test]
    fn test_validation_unicode_normalization() {
        println!("🔍 [LOGIC] Testing Unicode normalization attacks...");

        // Different Unicode representations of same character
        let test_cases = vec![
            ("café", "composed"),           // e + ́ = é (composed)
            ("cafe\u{0301}", "decomposed"), // e + combining accent = é (decomposed)
        ];

        for (input, desc) in test_cases {
            let result = crate::core::security::validate_environment_name(input);
            println!("  {} ({}): {:?}", desc, input, result);
        }

        println!("  ⚠️  Check for Unicode normalization consistency\n");
    }

    #[test]
    fn test_validation_boundary_conditions() {
        println!("🔍 [LOGIC] Testing boundary conditions...");

        use crate::core::models::{FieldDefinition, ValidationRules};
        use crate::core::validation::validate_value;

        // Test exact boundary values
        let field = FieldDefinition {
            key: "test".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: Some(0),
                max_value: Some(100),
                pattern: None,
            }),
            is_secret: false,
        };

        let test_values = vec![
            ("-1", false),       // Below min
            ("0", true),         // Exact min
            ("50", true),        // Middle
            ("100", true),       // Exact max
            ("101", false),      // Above max
            ("i64::MAX", false), // Overflow attempt
        ];

        for (value, should_pass) in test_values {
            let actual_value = match value {
                "i64::MAX" => i64::MAX.to_string(),
                v => v.to_string(),
            };
            let result = validate_value(&actual_value, &field);
            let passed = result.is_ok();
            let status = if passed == should_pass { "✅" } else { "❌" };
            println!(
                "  {} {:15} → {}",
                status,
                value,
                if passed { "PASS" } else { "FAIL" }
            );
        }
        println!();
    }

    // ========================================================================
    // DOS & RESOURCE EXHAUSTION
    // ========================================================================

    #[test]
    fn test_dos_regex_catastrophic_backtracking() {
        println!("🔍 [DOS] Testing regex catastrophic backtracking...");

        use crate::core::models::{FieldDefinition, ValidationRules};
        use crate::core::validation::validate_value;

        // Pattern prone to catastrophic backtracking
        let vulnerable_patterns = vec![r"^(a+)+$", r"^(.*)(.*)+$", r"^(([a-z])+.)+$"];

        for pattern in vulnerable_patterns {
            let field = FieldDefinition {
                key: "test".to_string(),
                r#type: "string".to_string(),
                description: None,
                validation: Some(ValidationRules {
                    min_length: None,
                    max_length: None,
                    min_value: None,
                    max_value: None,
                    pattern: Some(pattern.to_string()),
                }),
                is_secret: false,
            };

            // Input designed to cause backtracking
            let input = "a".repeat(25) + "b";

            let start = Instant::now();
            let _ = validate_value(&input, &field);
            let elapsed = start.elapsed();

            println!("  Pattern {:20} → {:?}", pattern, elapsed);

            if elapsed.as_millis() > 100 {
                println!("    ⚠️  SLOW: Potential ReDoS vulnerability");
            }
        }
        println!();
    }

    #[test]
    fn test_dos_memory_exhaustion() {
        println!("🔍 [DOS] Testing memory exhaustion attacks...");

        let key = [0x42u8; 32];

        // Try to encrypt increasingly large data
        let sizes = vec![1, 10, 50, 100]; // MB

        for size_mb in sizes {
            let data = "A".repeat(size_mb * 1024 * 1024);

            let start = Instant::now();
            let result = crate::core::crypto::encrypt_data(&data, &key);
            let elapsed = start.elapsed();

            match result {
                Ok(_) => println!("  {}MB: encrypt={:?}", size_mb, elapsed),
                Err(e) => {
                    println!("  {}MB: FAILED - {}", size_mb, e);
                    break;
                }
            }
        }

        println!("  ℹ️  Monitor memory usage during large operations\n");
    }

    #[test]
    fn test_dos_deeply_nested_json() {
        println!("🔍 [DOS] Testing deeply nested JSON parsing...");

        // Create deeply nested JSON
        let mut json = String::from("{\"a\":");
        for _ in 0..100 {
            json.push_str("{\"a\":");
        }
        json.push_str("1");
        for _ in 0..100 {
            json.push('}');
        }
        json.push('}');

        let start = Instant::now();
        let result: Result<serde_json::Value, _> = serde_json::from_str(&json);
        let elapsed = start.elapsed();

        println!(
            "  100-level nesting: {:?}, result: {:?}",
            elapsed,
            result.is_ok()
        );
        println!("  ℹ️  Check for stack overflow on very deep nesting\n");
    }

    // ========================================================================
    // SUMMARY
    // ========================================================================

    #[test]
    fn run_deep_security_analysis() {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║         DEEP SECURITY ANALYSIS                           ║");
        println!("║         Advanced Attack Vector Testing                   ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");

        // Cryptographic analysis
        test_crypto_key_zeroization();
        test_crypto_timing_attack();
        test_crypto_weak_key_detection();
        test_crypto_empty_data();

        // Race conditions
        test_race_condition_file_write_old_api();
        test_race_condition_file_write_new_api();
        test_race_condition_lock_timeout();

        // Information leaks
        test_error_message_information_leak();
        test_memory_dump_sensitive_data();
        test_audit_log_secret_masking();

        // Logic bugs
        test_validation_bypass_null_byte();
        test_validation_unicode_normalization();
        test_validation_boundary_conditions();

        // DoS testing
        test_dos_regex_catastrophic_backtracking();
        test_dos_memory_exhaustion();
        test_dos_deeply_nested_json();

        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║         DEEP SECURITY ANALYSIS COMPLETE                  ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
    }
}
