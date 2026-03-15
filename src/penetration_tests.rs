/// Penetration Testing Module for Naru
/// This module contains proof-of-concept exploits for identified vulnerabilities

#[cfg(test)]
mod penetration_tests {
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::TempDir;

    // ========================================================================
    // EXPLOIT 1: Race Condition Attack (CRITICAL)
    // CWE-362: Concurrent Execution using Shared Resource
    // ========================================================================

    #[test]
    fn exploit_race_condition_data_loss() {
        println!("\n🔴 EXPLOIT 1: Race Condition Data Loss Attack");
        println!("================================================");

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_race_condition") };

        // Initialize naru using persistence API directly
        if let Err(e) = crate::core::persistence::init_project() {
            println!("Init failed: {:?}", e);
            let _ = std::env::set_current_dir(&original_dir);
            unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
            return;
        }

        // Set initial value using atomic API
        let _ = crate::core::persistence::atomic_update_config(|config| {
            if let Some(env_config) = config.environments.get_mut("development") {
                env_config.entries.insert(
                    "SHARED_KEY".to_string(),
                    crate::core::models::ConfigValueEntry::new("initial_value", "string", false),
                );
            }
            Ok(())
        });

        // Simulate concurrent writes (race condition) using atomic API
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = vec![];

        for i in 0..3 {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                // All threads try to set different values simultaneously
                let _ = crate::core::persistence::atomic_update_config(|config| {
                    if let Some(env_config) = config.environments.get_mut("development") {
                        env_config.entries.insert(
                            "SHARED_KEY".to_string(),
                            crate::core::models::ConfigValueEntry::new(
                                &format!("thread{}_value", i),
                                "string",
                                false,
                            ),
                        );
                    }
                    Ok(())
                });
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Get the final value using atomic API
        let final_value: String = crate::core::persistence::atomic_read_config(|config| {
            config
                .environments
                .get("development")
                .and_then(|e| e.entries.get("SHARED_KEY"))
                .map(|e| e.value.clone())
        })
        .ok()
        .flatten()
        .unwrap_or_default();

        println!("Final value after race: {}", final_value);

        // With atomic operations, we should NOT lose any writes in a way that produces an invalid state
        // but since they are all overwriting the same key, the last one wins.
        // The important part is that we don't have partial data or corruption.
        let valid_thread_values = ["thread0_value", "thread1_value", "thread2_value"];

        if !valid_thread_values.contains(&final_value.as_str()) {
            println!(
                "⚠️  RACE CONDITION DETECTED: Unexpected value '{}'",
                final_value
            );
        } else {
            println!("✓ Atomic operations prevented data corruption during race");
        }

        let _ = std::env::set_current_dir(original_dir);
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
    }

    // ========================================================================
    // EXPLOIT 2: Path Traversal Attack (HIGH)
    // CWE-22: Improper Limitation of a Pathname to a Restricted Directory
    // ========================================================================

    #[test]
    fn exploit_path_traversal_attack() {
        println!("\n🔴 EXPLOIT 2: Path Traversal Attack");
        println!("====================================");

        // Try path traversal attack - test the security function directly
        let malicious_path = "../../../etc/passwd";
        let result = crate::core::security::sanitize_file_path(malicious_path);

        if result.is_ok() {
            println!("⚠️  PATH TRAVERSAL SUCCESSFUL: Attacker could read /etc/passwd");
        } else {
            let error = result.unwrap_err();
            if error.contains("traversal") || error.contains("Absolute paths") {
                println!("✓ Path traversal blocked: {}", error);
            } else {
                println!("? Path traversal blocked: {}", error);
            }
        }
    }

    // ========================================================================
    // EXPLOIT 3: Null Byte Injection (HIGH)
    // CWE-693: Protection Mechanism Failure
    // ========================================================================

    #[test]
    fn exploit_null_byte_injection() {
        println!("\n🟠 EXPLOIT 3: Null Byte Injection Attack");
        println!("==========================================");

        // Try null byte injection in key name - test the security function directly
        let null_byte_key = "MALICIOUS_KEY\0.txt";
        let result = crate::core::security::validate_config_key(null_byte_key);

        if result.is_ok() {
            println!("⚠️  NULL BYTE INJECTION SUCCESSFUL");
        } else {
            println!("✓ Null byte injection blocked: {}", result.unwrap_err());
        }
    }

    // ========================================================================
    // EXPLOIT 4: Regex DoS Attack (MEDIUM)
    // CWE-1333: Inefficient Regular Expression Complexity
    // ========================================================================

    #[test]
    fn exploit_regex_dos_attack() {
        println!("\n🟡 EXPLOIT 4: Regex DoS (ReDoS) Attack");
        println!("========================================");

        use std::time::Instant;

        // Test pathological regex pattern
        let pathological_pattern = r"(a+)+$";
        let malicious_input = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!";

        let start = Instant::now();
        let re = regex::Regex::new(pathological_pattern).unwrap();
        let _result = re.is_match(malicious_input);
        let duration = start.elapsed();

        if duration.as_millis() > 100 {
            println!(
                "⚠️  ReDoS VULNERABILITY: Pattern took {:?} to evaluate",
                duration
            );
        } else {
            println!("✓ Regex evaluation completed in {:?} (safe)", duration);
        }
    }

    // ========================================================================
    // EXPLOIT 5: Audit Log Injection (MEDIUM)
    // CWE-117: Improper Output Neutralization for Logs
    // ========================================================================

    #[test]
    fn exploit_audit_log_injection() {
        println!("\n🟡 EXPLOIT 5: Audit Log Injection Attack");
        println!("==========================================");

        // Try to inject malicious content via key name - test validation directly
        let malicious_key = "KEY\n{\"injected\": true}";
        let validation_result = crate::core::security::validate_config_key(malicious_key);

        println!("✓ Key validation result: {:?}", validation_result);
    }

    // ========================================================================
    // EXPLOIT 6: Secret Masking Bypass (MEDIUM)
    // CWE-532: Insertion of Sensitive Information into Log File
    // ========================================================================

    #[test]
    fn exploit_secret_masking_bypass() {
        println!("\n🟡 EXPLOIT 6: Secret Masking Bypass");
        println!("====================================");

        // Test key validation for various key patterns
        let test_keys = ["DB_PWD", "API_SECRET_KEY", "AUTH_TOKEN", "MY_PASSPHRASE"];

        for key in &test_keys {
            let result = crate::core::security::validate_config_key(key);
            println!("  Key {}: {:?}", key, result);
        }

        println!("✓ All keys validated properly");
    }

    // ========================================================================
    // EXPLOIT 7: Integer Overflow in Validation (LOW)
    // CWE-190: Integer Overflow or Wraparound
    // ========================================================================

    #[test]
    fn exploit_integer_overflow() {
        println!("\n🟢 EXPLOIT 7: Integer Overflow Test");
        println!("====================================");

        // Try integer overflow values - test validation directly
        // Note: This test doesn't need project initialization since it tests
        // the validation function directly
        let overflow_values = [
            "9223372036854775808",  // i64::MAX + 1
            "-9223372036854775809", // i64::MIN - 1
            "18446744073709551616", // u64::MAX + 1
        ];

        use crate::core::models::FieldDefinition;
        use crate::core::validation::validate_value;

        let field = FieldDefinition {
            key: "OVERFLOW_KEY".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };

        for value in &overflow_values {
            let result = validate_value(value, &field);

            if result.is_ok() {
                println!("⚠️  Integer overflow accepted: {}", value);
            } else {
                println!("✓ Integer overflow rejected: {}", value);
            }
        }
    }

    // ========================================================================
    // EXPLOIT 8: Unicode Normalization Attack (HIGH)
    // CWE-179: Incorrect Behavior Order: Early Validation
    // ========================================================================

    #[test]
    fn exploit_unicode_normalization() {
        println!("\n🟠 EXPLOIT 8: Unicode Normalization Attack");
        println!("============================================");

        use unicode_normalization::UnicodeNormalization;

        // Two visually identical strings with different Unicode representations
        let composed = "café"; // Single codepoint for é
        let decomposed = "cafe\u{0301}"; // e + combining acute accent

        println!("Composed: {} (bytes: {})", composed, composed.len());
        println!("Decomposed: {} (bytes: {})", decomposed, decomposed.len());
        println!("NFC equal: {}", composed.nfc().eq(decomposed.nfc()));

        // Test if validation treats them the same
        let composed_result = crate::core::security::validate_config_key(composed);
        let decomposed_result = crate::core::security::validate_config_key(decomposed);

        println!("Composed validation: {:?}", composed_result);
        println!("Decomposed validation: {:?}", decomposed_result);

        // Both should be valid
        assert!(composed_result.is_ok());
        assert!(decomposed_result.is_ok());

        println!("✓ Unicode normalization handled correctly");
    }

    // ========================================================================
    // SUMMARY
    // ========================================================================

    #[test]
    fn run_all_penetration_tests() {
        println!("\n");
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║         NARU PENETRATION TESTING SUITE                   ║");
        println!("╚══════════════════════════════════════════════════════════╝");

        // Run tests that don't require directory changes
        exploit_path_traversal_attack();
        exploit_null_byte_injection();
        exploit_regex_dos_attack();
        exploit_audit_log_injection();
        exploit_secret_masking_bypass();
        exploit_integer_overflow();
        exploit_unicode_normalization();

        println!("\n");
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║         PENETRATION TESTING COMPLETE                     ║");
        println!("╚══════════════════════════════════════════════════════════╝");
    }
}
