/// Penetration Testing Module for Naru
/// This module contains proof-of-concept exploits for identified vulnerabilities

#[cfg(test)]
mod penetration_tests {
    use std::fs;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Instant;
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

        // Initialize naru
        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_race_condition") };

        // Run init
        let init_output = std::process::Command::new("cargo")
            .args(["run", "--", "init"])
            .current_dir(temp_dir.path())
            .env("NARU_ENCRYPTION_KEY", "test_key_for_race_condition")
            .output()
            .expect("Failed to execute naru init");

        if !init_output.status.success() {
            println!(
                "Init failed: {}",
                String::from_utf8_lossy(&init_output.stderr)
            );
        }

        // Set initial value
        std::process::Command::new("./target/release/naru")
            .args(["set", "SHARED_KEY=initial_value", "--env", "development"])
            .output()
            .unwrap();

        // Simulate concurrent writes (race condition)
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = vec![];

        for i in 0..3 {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                // All threads try to set different values simultaneously
                std::process::Command::new("./target/release/naru")
                    .args([
                        "set",
                        &format!("SHARED_KEY=thread{}_value", i),
                        "--env",
                        "development",
                    ])
                    .output()
                    .unwrap()
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Get the final value - should be one of the thread values
        let get_output = std::process::Command::new("./target/release/naru")
            .args(["get", "SHARED_KEY", "--env", "development"])
            .output()
            .unwrap();

        let final_value = String::from_utf8_lossy(&get_output.stdout)
            .trim()
            .to_string();
        println!("Final value after race: {}", final_value);

        // In a race condition, we might lose some writes
        // The value should be one of: initial_value, thread0_value, thread1_value, thread2_value
        let valid_values = [
            "initial_value",
            "thread0_value",
            "thread1_value",
            "thread2_value",
        ];

        // If race condition exists, we might see unexpected behavior
        if !valid_values.contains(&final_value.as_str()) {
            println!(
                "⚠️  RACE CONDITION DETECTED: Unexpected value '{}'",
                final_value
            );
        } else {
            println!("✓ No obvious race condition detected in this run");
        }

        std::env::set_current_dir(original_dir).unwrap();
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

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_path_traversal") };

        // Initialize naru
        std::process::Command::new("./target/release/naru")
            .arg("init")
            .output()
            .unwrap();

        // Try path traversal attack via import
        let malicious_path = "../../../etc/passwd";
        let import_output = std::process::Command::new("./target/release/naru")
            .args(["import", malicious_path, "--env", "development"])
            .output()
            .unwrap();

        if import_output.status.success() {
            println!("⚠️  PATH TRAVERSAL SUCCESSFUL: Attacker could read /etc/passwd");
        } else {
            let stderr = String::from_utf8_lossy(&import_output.stderr);
            if stderr.contains("traversal") || stderr.contains("Absolute paths") {
                println!("✓ Path traversal blocked: {}", stderr.trim());
            } else {
                println!("? Import failed for other reason: {}", stderr.trim());
            }
        }

        std::env::set_current_dir(original_dir).unwrap();
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
    }

    // ========================================================================
    // EXPLOIT 3: Null Byte Injection (HIGH)
    // CWE-693: Protection Mechanism Failure
    // ========================================================================

    #[test]
    fn exploit_null_byte_injection() {
        println!("\n🟠 EXPLOIT 3: Null Byte Injection Attack");
        println!("==========================================");

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_null_byte") };

        // Initialize naru
        std::process::Command::new("./target/release/naru")
            .arg("init")
            .output()
            .unwrap();

        // Try null byte injection in key name
        let null_byte_key = "MALICIOUS_KEY\0.txt";
        let set_output = std::process::Command::new("./target/release/naru")
            .args([
                "set",
                &format!("{}=malicious_value", null_byte_key),
                "--env",
                "development",
            ])
            .output()
            .unwrap();

        if set_output.status.success() {
            println!("⚠️  NULL BYTE INJECTION SUCCESSFUL");
        } else {
            let stderr = String::from_utf8_lossy(&set_output.stderr);
            println!("✓ Null byte injection blocked: {}", stderr.trim());
        }

        std::env::set_current_dir(original_dir).unwrap();
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
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

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_audit") };

        // Initialize naru
        std::process::Command::new("./target/release/naru")
            .arg("init")
            .output()
            .unwrap();

        // Try to inject malicious content via key name
        let malicious_key = "KEY\n{\"injected\": true}";
        let _set_output = std::process::Command::new("./target/release/naru")
            .args([
                "set",
                &format!("{}=test_value", malicious_key),
                "--env",
                "development",
            ])
            .output()
            .unwrap();

        // Check audit log for injection
        let audit_path = temp_dir.path().join(".naru/audit.log");
        if audit_path.exists() {
            let audit_content = fs::read_to_string(&audit_path).unwrap();
            let line_count = audit_content.lines().count();

            if line_count > 1 {
                println!("⚠️  AUDIT LOG INJECTION: Extra lines detected");
            } else {
                println!("✓ Audit log injection prevented");
            }

            // Check for JSON structure integrity
            for line in audit_content.lines() {
                if let Err(e) = serde_json::from_str::<serde_json::Value>(line) {
                    println!("⚠️  CORRUPTED AUDIT ENTRY: {}", e);
                }
            }
        }

        std::env::set_current_dir(original_dir).unwrap();
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
    }

    // ========================================================================
    // EXPLOIT 6: Secret Masking Bypass (MEDIUM)
    // CWE-532: Insertion of Sensitive Information into Log File
    // ========================================================================

    #[test]
    fn exploit_secret_masking_bypass() {
        println!("\n🟡 EXPLOIT 6: Secret Masking Bypass");
        println!("====================================");

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_masking") };

        // Initialize naru
        std::process::Command::new("./target/release/naru")
            .arg("init")
            .output()
            .unwrap();

        // Try keys that should be masked but might bypass detection
        let bypass_keys = [
            "DB_PWD",         // "pwd" not in mask list
            "API_SECRET_KEY", // Should be masked (contains "secret" and "key")
            "AUTH_TOKEN",     // Should be masked (contains "auth" and "token")
            "MY_PASSPHRASE",  // Should be masked (contains "pass")
        ];

        for key in &bypass_keys {
            std::process::Command::new("./target/release/naru")
                .args([
                    "set",
                    &format!("{}=super_secret_value_123", key),
                    "--env",
                    "development",
                ])
                .output()
                .unwrap();
        }

        // Check audit log
        let audit_path = temp_dir.path().join(".naru/audit.log");
        if audit_path.exists() {
            let audit_content = fs::read_to_string(&audit_path).unwrap();

            for key in &bypass_keys {
                if audit_content.contains("super_secret_value_123") {
                    // Check if it's associated with a non-masked key
                    println!("⚠️  Potential secret exposure for key: {}", key);
                }
            }
        }

        std::env::set_current_dir(original_dir).unwrap();
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
    }

    // ========================================================================
    // EXPLOIT 7: Integer Overflow in Validation (LOW)
    // CWE-190: Integer Overflow or Wraparound
    // ========================================================================

    #[test]
    fn exploit_integer_overflow() {
        println!("\n🟢 EXPLOIT 7: Integer Overflow Test");
        println!("====================================");

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_overflow") };

        // Initialize naru
        std::process::Command::new("./target/release/naru")
            .arg("init")
            .output()
            .unwrap();

        // Try integer overflow values
        let overflow_values = [
            "9223372036854775808",  // i64::MAX + 1
            "-9223372036854775809", // i64::MIN - 1
            "18446744073709551616", // u64::MAX + 1
        ];

        for value in &overflow_values {
            let set_output = std::process::Command::new("./target/release/naru")
                .args([
                    "set",
                    &format!("OVERFLOW_KEY={}", value),
                    "--env",
                    "development",
                ])
                .output()
                .unwrap();

            if set_output.status.success() {
                println!("⚠️  Integer overflow accepted: {}", value);
            } else {
                println!("✓ Integer overflow rejected: {}", value);
            }
        }

        std::env::set_current_dir(original_dir).unwrap();
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
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
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_unicode") };

        std::process::Command::new("./target/release/naru")
            .arg("init")
            .output()
            .unwrap();

        // Set value with composed form
        std::process::Command::new("./target/release/naru")
            .args([
                "set",
                &format!("{}=composed_value", composed),
                "--env",
                "development",
            ])
            .output()
            .unwrap();

        // Try to get with decomposed form
        let get_output = std::process::Command::new("./target/release/naru")
            .args(["get", decomposed, "--env", "development"])
            .output()
            .unwrap();

        if get_output.status.success() {
            println!("✓ Unicode normalization handled correctly");
        } else {
            println!("⚠️  Unicode normalization may cause lookup failures");
        }

        std::env::set_current_dir(original_dir).unwrap();
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
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

        exploit_race_condition_data_loss();
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
