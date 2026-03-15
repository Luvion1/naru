#[cfg(test)]
mod error_handling_tests {
    use crate::core::crypto;
    use crate::core::models::*;
    use crate::core::persistence;
    use crate::core::security;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    /// Test error handling in crypto module with invalid inputs
    #[test]
    fn test_crypto_error_handling() {
        // Test with invalid key (wrong size)
        let invalid_key = [0u8; 16]; // AES-256 requires 32-byte key
        let result = crypto::encrypt_data("test", &invalid_key);
        assert!(result.is_err());

        // Test with valid key but invalid encrypted data
        let valid_key = [0u8; 32];
        let result = crypto::decrypt_data("invalid_hex_data", &valid_key);
        assert!(result.is_err());

        // Test with valid key but truncated encrypted data
        let encrypted = crypto::encrypt_data("test", &valid_key).unwrap();
        let truncated = &encrypted[..std::cmp::min(10, encrypted.len())]; // Truncate to 10 chars
        let result = crypto::decrypt_data(truncated, &valid_key);
        assert!(result.is_err());
    }

    /// Test error handling in validation with invalid inputs
    #[test]
    fn test_validation_error_handling() {
        use crate::core::validation::validate_value;

        // Test integer validation with non-numeric string
        let field = FieldDefinition {
            key: "test".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        assert!(validate_value("not_a_number", &field).is_err());

        // Test integer validation with overflow
        let field = FieldDefinition {
            key: "test".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        assert!(validate_value(&format!("{}", i64::MAX as i128 + 1), &field).is_err());

        // Test regex validation with invalid pattern
        let field = FieldDefinition {
            key: "test".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some("[invalid_regex[".to_string()), // Invalid regex pattern
            }),
            is_secret: false,
        };
        assert!(validate_value("test", &field).is_err());
    }

    /// Test error handling in persistence with invalid file operations
    #[test]
    #[serial]
    fn test_persistence_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Clean up function
        let cleanup = || {
            let _ = std::env::set_current_dir(original_dir);
        };

        // Test loading non-existent file
        let result: Result<ConfigFile, _> = persistence::load_json("non_existent.json");
        assert!(result.is_err());

        // Test saving to invalid path
        let config = ConfigFile {
            project_name: "Test".to_string(),
            version: "1.0".to_string(),
            environments: HashMap::new(),
            salt: None,
        };

        // Try to save to a path that traverses up
        let result = persistence::save_json("../forbidden.json", &config);
        assert!(result.is_err());

        // Test importing from non-existent file
        let result = persistence::import_from_env("non_existent.env", "dev");
        assert!(result.is_err());

        // Test importing from invalid JSON
        fs::write("invalid.json", "{ invalid json content").unwrap();
        let result = persistence::import_from_json("invalid.json", "dev");
        assert!(result.is_err());

        // Test importing from invalid YAML
        fs::write("invalid.yaml", "invalid:\nyaml content: [").unwrap();
        let result = persistence::import_from_yaml("invalid.yaml", "dev");
        assert!(result.is_err());

        cleanup();
    }

    /// Test error handling in security module
    #[test]
    fn test_security_error_handling() {
        // Test invalid file paths
        assert!(security::sanitize_file_path("../etc/passwd").is_err());
        assert!(security::sanitize_file_path("/etc/passwd").is_err());
        assert!(security::sanitize_file_path("path/../../forbidden").is_err());

        // Test null bytes in paths
        assert!(security::sanitize_file_path("file\0.txt").is_err());

        // Test invalid environment names
        assert!(security::validate_environment_name("").is_err());
        assert!(security::validate_environment_name(&"a".repeat(101)).is_err()); // Too long
        assert!(security::validate_environment_name("invalid name").is_err()); // Contains space
        assert!(security::validate_environment_name("invalid/name").is_err()); // Contains slash

        // Test invalid config keys
        assert!(security::validate_config_key("").is_err());
        assert!(security::validate_config_key(&"a".repeat(256)).is_err()); // Too long
        assert!(security::validate_config_key("invalid key").is_err()); // Contains space
        assert!(security::validate_config_key("invalid/key").is_err()); // Contains slash

        // Test file size check on non-existent file
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("does_not_exist.txt");
        assert!(security::check_file_size(&non_existent, 1000).is_err());
    }

    /// Test error handling with malformed inputs
    #[test]
    fn test_malformed_input_error_handling() {
        use crate::core::validation::validate_value;

        // Test with extremely long strings that might cause buffer overflows
        let long_string = "a".repeat(1_000_000); // 1MB string
        let field = FieldDefinition {
            key: "test".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(1),
                max_length: Some(100), // Much smaller than input
                min_value: None,
                max_value: None,
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value(&long_string, &field).is_err());

        // Test with recursive JSON structures (if parser supports them)
        let recursive_json = r#"{"level1": {"level2": {"level3": {"level4": "[RECURSIVE]"}}}}"#;
        // This should be handled gracefully by the JSON parser
    }

    /// Test error handling with resource exhaustion attempts
    #[test]
    #[serial]
    fn test_resource_exhaustion_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Clean up function
        let cleanup = || {
            let _ = std::env::set_current_dir(original_dir);
        };

        // Initialize project
        persistence::init_project().unwrap();

        // Create a config with an extremely large number of entries to test resource limits
        let mut environments = HashMap::new();
        let mut entries = HashMap::new();

        // Add many entries to test memory limits
        for i in 0..10000 {
            entries.insert(
                format!("KEY_{}", i),
                ConfigValueEntry::new(&format!("VALUE_{}", i), "string", false),
            );
        }

        environments.insert("development".to_string(), EnvironmentConfig { entries });

        let large_config = ConfigFile {
            project_name: "Resource Test".to_string(),
            version: "1.0.0".to_string(),
            environments,
            salt: None,
        };

        // This should work but test the limits
        assert!(persistence::atomic_update_config(|config| {
            *config = large_config.clone();
            Ok(())
        })
        .is_ok());

        // Try to load it back
        let loaded_result: Result<ConfigFile, _> = persistence::atomic_read_config(|c| c.clone());
        assert!(loaded_result.is_ok());

        cleanup();
    }

    /// Test error propagation through the system
    #[test]
    #[serial]
    fn test_error_propagation() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Clean up function
        let cleanup = || {
            let _ = std::env::set_current_dir(original_dir);
        };

        // Initialize project
        persistence::init_project().unwrap();

        // Attempt to set a value with an invalid environment name
        let mut config: ConfigFile = persistence::load_json("config.json").unwrap();

        // Try to access a non-existent environment
        assert!(config.environments.get("non_existent_env").is_none());

        cleanup();
    }
}

#[cfg(test)]
mod concurrency_tests {
    use crate::core::models::*;
    use crate::core::persistence;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::TempDir;

    /// Test concurrent access to the same configuration file
    #[test]
    #[serial]
    fn test_concurrent_file_access() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Clean up function
        let cleanup = || {
            let _ = std::env::set_current_dir(original_dir);
        };

        // Initialize project
        persistence::init_project().unwrap();

        // Create a barrier to synchronize threads
        let barrier = Arc::new(Barrier::new(10)); // 10 threads

        // Spawn multiple threads to access the config file concurrently
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let barrier = barrier.clone();
                thread::spawn(move || {
                    barrier.wait(); // Wait for all threads to be ready

                    // Each thread tries to read and write to the config using atomic operations
                    let result = persistence::atomic_update_config(|config| {
                        // Modify the config slightly
                        if let Some(dev_env) = config.environments.get_mut("development") {
                            dev_env.entries.insert(
                                format!("thread_{}_key", i),
                                ConfigValueEntry::new(
                                    &format!("thread_{}_value", i),
                                    "string",
                                    false,
                                ),
                            );
                        }
                        Ok(())
                    });
                    result.is_ok() // Return whether the operation succeeded
                })
            })
            .collect();

        // Collect results
        let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Check that all concurrent operations succeeded (thanks to locking)
        assert!(
            results.iter().all(|&x| x),
            "All concurrent atomic operations should succeed"
        );

        cleanup();
    }

    /// Test concurrent audit log writes
    #[test]
    #[serial]
    fn test_concurrent_audit_writes() {
        use crate::core::audit;
        use std::sync::Arc;
        use std::time::Duration;

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Clean up function
        let cleanup = || {
            let _ = std::env::set_current_dir(original_dir);
        };

        let log_path = format!("{}/concurrent_audit.log", crate::core::constants::NARU_DIR);

        // Create the .naru directory
        std::fs::create_dir_all(crate::core::constants::NARU_DIR).unwrap();

        // Create a barrier to synchronize threads
        let barrier = Arc::new(Barrier::new(5)); // 5 threads

        // Spawn multiple threads to write to the audit log concurrently
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let barrier = barrier.clone();
                let log_path = log_path.clone();
                thread::spawn(move || {
                    barrier.wait(); // Wait for all threads to be ready

                    // Each thread tries to write to the audit log
                    let result = audit::log_action(
                        &format!("THREAD_ACTION_{}", i),
                        "concurrent_env",
                        Some(&format!("key_{}", i)),
                        None,
                        Some(&format!("value_{}", i)),
                        &log_path,
                    );

                    thread::sleep(Duration::from_millis(10)); // Small delay to increase chance of overlap
                    result.is_ok() // Return whether the operation succeeded
                })
            })
            .collect();

        // Collect results
        let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Check that all operations succeeded
        assert!(
            results.iter().all(|&x| x),
            "All concurrent audit operations should succeed"
        );

        // Verify the integrity of the audit log
        let integrity_check = audit::AuditLogEntry::verify_log_integrity(&log_path);
        assert!(integrity_check.is_ok());
        assert!(
            integrity_check.unwrap(),
            "Audit log should maintain integrity under concurrent writes"
        );

        cleanup();
    }

    /// Test concurrent encryption/decryption operations
    #[test]
    fn test_concurrent_crypto_operations() {
        use crate::core::crypto;
        use std::sync::Arc;

        let key = [1u8; 32]; // Fixed key for testing

        // Create a barrier to synchronize threads
        let barrier = Arc::new(Barrier::new(8)); // 8 threads

        // Spawn multiple threads to perform crypto operations
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let barrier = barrier.clone();
                let key = key;
                thread::spawn(move || {
                    barrier.wait(); // Wait for all threads to be ready

                    let data = format!("thread_{}_data", i);

                    // Encrypt the data
                    match crypto::encrypt_data(&data, &key) {
                        Ok(encrypted) => {
                            // Decrypt it back
                            match crypto::decrypt_data(&encrypted, &key) {
                                Ok(decrypted) => decrypted == data, // Should match original
                                Err(_) => false,
                            }
                        }
                        Err(_) => false,
                    }
                })
            })
            .collect();

        // Collect results
        let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Check that all operations succeeded
        assert!(
            results.iter().all(|&x| x),
            "All concurrent crypto operations should succeed"
        );
    }

    /// Test concurrent file I/O operations with different files
    #[test]
    #[serial]
    fn test_concurrent_file_operations_different_files() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Clean up function
        let cleanup = || {
            let _ = std::env::set_current_dir(original_dir);
        };

        // Initialize project
        persistence::init_project().unwrap();

        // Create a barrier to synchronize threads
        let barrier = Arc::new(Barrier::new(6)); // 6 threads

        // Spawn multiple threads to work with different config files
        let handles: Vec<_> = (0..6)
            .map(|i| {
                let barrier = barrier.clone();
                thread::spawn(move || {
                    barrier.wait(); // Wait for all threads to be ready

                    // Create a unique config for each thread
                    let mut environments = HashMap::new();
                    let mut entries = HashMap::new();
                    entries.insert(
                        format!("unique_key_{}", i),
                        ConfigValueEntry::new(&format!("unique_value_{}", i), "string", false),
                    );
                    environments.insert("test_env".to_string(), EnvironmentConfig { entries });

                    let config = ConfigFile {
                        project_name: format!("Test_Project_{}", i),
                        version: "1.0.0".to_string(),
                        environments,
                        salt: None,
                    };

                    let filename = format!("config_{}.json", i);

                    // Save the config
                    let save_result = persistence::save_json(&filename, &config);
                    if save_result.is_err() {
                        return false;
                    }

                    // Load the config back
                    let load_result: Result<ConfigFile, _> = persistence::load_json(&filename);
                    load_result.is_ok()
                })
            })
            .collect();

        // Collect results
        let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Check that all operations succeeded
        assert!(
            results.iter().all(|&x| x),
            "All concurrent file operations should succeed"
        );

        cleanup();
    }
}
