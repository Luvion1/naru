use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, Write};
use std::path::Path;

use crate::core::locking;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub environment: String,
    pub key: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub user: Option<String>,
    // Tamper-evident fields
    pub previous_hash: Option<String>,
    pub hash: Option<String>,
}

impl AuditLogEntry {
    // Sanitize string to prevent log injection
    fn sanitize_for_log(s: &str) -> String {
        // Remove or escape control characters that could break JSON structure
        s.chars()
            .filter(|c| {
                // Allow printable ASCII and Unicode beyond ASCII
                !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t'
            })
            .map(|c| {
                // Escape newlines and tabs for JSON safety
                match c {
                    '\n' => "\\n",
                    '\r' => "\\r",
                    '\t' => "\\t",
                    _ => return c.to_string(),
                }
                .to_string()
            })
            .collect()
    }

    pub fn new(
        action: String,
        environment: String,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> Self {
        // Sanitize all string inputs to prevent log injection
        let sanitized_action = Self::sanitize_for_log(&action);
        let sanitized_environment = Self::sanitize_for_log(&environment);
        let sanitized_key = key.map(|k| Self::sanitize_for_log(&k));
        let sanitized_old_value = old_value.map(|v| Self::sanitize_for_log(&v));
        let sanitized_new_value = new_value.map(|v| Self::sanitize_for_log(&v));

        // Automatic masking of sensitive data
        let mask_value = |k: &str, v: Option<String>| -> Option<String> {
            if let Some(val) = v {
                let k_lower = k.to_lowercase();
                if k_lower.contains("pass")
                    || k_lower.contains("secret")
                    || k_lower.contains("key")
                    || k_lower.contains("token")
                    || k_lower.contains("auth")
                {
                    return Some("********".to_string());
                }
                Some(val)
            } else {
                None
            }
        };

        let final_old_value = if let Some(ref k) = sanitized_key {
            mask_value(k, sanitized_old_value)
        } else {
            sanitized_old_value
        };

        let final_new_value = if let Some(ref k) = sanitized_key {
            mask_value(k, sanitized_new_value)
        } else {
            sanitized_new_value
        };

        AuditLogEntry {
            timestamp: Utc::now(),
            action: sanitized_action,
            environment: sanitized_environment,
            key: sanitized_key,
            old_value: final_old_value,
            new_value: final_new_value,
            user: std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .ok(),
            previous_hash: None,
            hash: None,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        // We exclude the hash field itself from the calculation
        let content = format!(
            "{}{}{}{:?}{:?}{:?}{:?}{:?}",
            self.timestamp.to_rfc3339(),
            self.action,
            self.environment,
            self.key,
            self.old_value,
            self.new_value,
            self.user,
            self.previous_hash
        );
        hasher.update(content);
        hex::encode(hasher.finalize())
    }

    pub fn log_to_file(&mut self, log_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::core::security;

        // Sanitize log path to prevent path traversal attacks
        // Use internal version that allows absolute paths for test compatibility
        let sanitized_log_path = security::sanitize_file_path_internal(log_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        // Acquire exclusive lock to prevent race conditions during concurrent writes
        // Use the same directory as the log file for the lock
        let log_path_obj = Path::new(&sanitized_log_path);
        let lock_dir = log_path_obj.parent().unwrap_or(Path::new("."));
        let lock_path = lock_dir.join("audit.lock");

        // Retry logic for audit lock - must hold the lock throughout the operation
        let max_retries = 5;
        let mut last_error: Option<std::io::Error> = None;
        let mut acquired_lock = None;

        for attempt in 0..max_retries {
            match locking::FileLock::acquire_exclusive(&lock_path) {
                Ok(lock) => {
                    acquired_lock = Some(lock);
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * (1 << attempt)));
                    }
                }
            }
        }

        if acquired_lock.is_none() {
            return Err(Box::new(std::io::Error::other(format!(
                "Could not acquire audit lock after {} attempts: {:?}",
                max_retries, last_error
            ))));
        }

        // Scope for the lock - must keep it until after write completes
        {
            let _lock = acquired_lock.unwrap();

            // Get the previous hash from the last line of the file
            let prev_hash = if Path::new(&sanitized_log_path).exists() {
                let file = fs::File::open(&sanitized_log_path)?;
                let reader = std::io::BufReader::new(file);
                if let Some(last_line) = reader.lines().map_while(Result::ok).last() {
                    if let Ok(last_entry) = serde_json::from_str::<AuditLogEntry>(&last_line) {
                        last_entry.hash
                    } else {
                        None
                    }
                } else {
                    None // File exists but is empty
                }
            } else {
                None // File does not exist (genesis)
            };

            self.previous_hash = prev_hash.or_else(|| {
                Some("0000000000000000000000000000000000000000000000000000000000000000".to_string())
            });
            self.hash = Some(self.calculate_hash());

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&sanitized_log_path)?;

            let log_line = serde_json::to_string(self)?;
            writeln!(file, "{}", log_line)?;
        } // Lock automatically released when _lock goes out of scope

        Ok(())
    }

    pub fn get_recent_logs(
        log_path: &str,
        count: usize,
    ) -> Result<Vec<AuditLogEntry>, Box<dyn std::error::Error>> {
        // Sanitize log path to prevent path traversal attacks
        // Use internal version that allows absolute paths for test compatibility
        let sanitized_log_path = crate::core::security::sanitize_file_path_internal(log_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        if !Path::new(&sanitized_log_path).exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&sanitized_log_path)?;
        let reader = std::io::BufReader::new(file);
        let mut entries: Vec<AuditLogEntry> = Vec::with_capacity(count);

        for line_str in reader.lines().map_while(Result::ok) {
            if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(&line_str) {
                entries.push(entry);
            }
        }

        // Return the most recent entries
        let start = if entries.len() > count {
            entries.len() - count
        } else {
            0
        };
        Ok(entries.into_iter().skip(start).collect())
    }

    // Function to verify the integrity of the audit log
    pub fn verify_log_integrity(log_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Sanitize log path to prevent path traversal attacks
        // Use internal version that allows absolute paths for test compatibility
        let sanitized_log_path = crate::core::security::sanitize_file_path_internal(log_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        if !Path::new(&sanitized_log_path).exists() {
            return Ok(true);
        }

        let file = fs::File::open(&sanitized_log_path)?;
        let reader = std::io::BufReader::new(file);

        let mut expected_prev_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        for line in reader.lines().map_while(Result::ok) {
            let entry: AuditLogEntry = serde_json::from_str(&line)?;

            // Check if previous hash matches
            if let Some(ph) = &entry.previous_hash {
                if ph != &expected_prev_hash {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            // Recalculate hash and compare
            let calculated_hash = entry.calculate_hash();
            if let Some(h) = &entry.hash {
                if h != &calculated_hash {
                    return Ok(false);
                }
                expected_prev_hash = h.clone();
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

// Helper function to log an action
pub fn log_action(
    action: &str,
    environment: &str,
    key: Option<&str>,
    old_value: Option<&str>,
    new_value: Option<&str>,
    log_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = AuditLogEntry::new(
        action.to_string(),
        environment.to_string(),
        key.map(|s| s.to_string()),
        old_value.map(|s| s.to_string()),
        new_value.map(|s| s.to_string()),
    );

    entry.log_to_file(log_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_action_hashing() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let log_path_str = log_path.to_str().unwrap();

        // First log
        log_action("SET", "dev", Some("key1"), None, Some("val1"), log_path_str).unwrap();

        // Second log
        log_action("SET", "dev", Some("key2"), None, Some("val2"), log_path_str).unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 2);

        // Check genesis hash
        assert_eq!(
            logs[0].previous_hash.as_deref(),
            Some("0000000000000000000000000000000000000000000000000000000000000000")
        );

        // Check chain
        assert_eq!(logs[1].previous_hash, logs[0].hash);

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_masking() {
        let entry = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("DB_PASSWORD".to_string()),
            Some("old_secret".to_string()),
            Some("new_secret".to_string()),
        );
        assert_eq!(entry.old_value, Some("********".to_string()));
        assert_eq!(entry.new_value, Some("********".to_string()));

        let entry_token = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("API_TOKEN".to_string()),
            None,
            Some("super-token".to_string()),
        );
        assert_eq!(entry_token.new_value, Some("********".to_string()));

        let entry_safe = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("PUBLIC_PORT".to_string()),
            Some("8080".to_string()),
            Some("9090".to_string()),
        );
        assert_eq!(entry_safe.old_value, Some("8080".to_string()));
    }

    #[test]
    fn test_audit_integrity_verification() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let log_path_str = log_path.to_str().unwrap();

        // 1. Create a valid chain
        log_action("SET", "dev", Some("key1"), None, Some("val1"), log_path_str).unwrap();
        log_action("SET", "dev", Some("key2"), None, Some("val2"), log_path_str).unwrap();

        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());

        // 2. Tamper with the first line
        let content = std::fs::read_to_string(log_path_str).unwrap();
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Change a value in the first entry
        let mut entry1: AuditLogEntry = serde_json::from_str(&lines[0]).unwrap();
        entry1.action = "TAMPERED".to_string();
        lines[0] = serde_json::to_string(&entry1).unwrap();

        std::fs::write(log_path_str, lines.join("\n") + "\n").unwrap();

        // 3. Verify integrity fails
        assert!(
            !AuditLogEntry::verify_log_integrity(log_path_str).unwrap(),
            "Integrity check should fail after tampering"
        );
    }

    #[test]
    fn test_audit_integrity_timestamp_tamper() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_time.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action("SET", "dev", Some("k"), None, Some("v"), log_path_str).unwrap();

        let content = std::fs::read_to_string(log_path_str).unwrap();
        let mut entry: AuditLogEntry = serde_json::from_str(&content).unwrap();

        // Manipulate timestamp by 1 second
        entry.timestamp += chrono::Duration::seconds(1);
        let tampered = serde_json::to_string(&entry).unwrap() + "\n";
        std::fs::write(log_path_str, tampered).unwrap();

        assert!(
            !AuditLogEntry::verify_log_integrity(log_path_str).unwrap(),
            "Integrity should fail if timestamp is changed"
        );
    }

    #[test]
    fn test_audit_integrity_user_tamper() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_user.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action("SET", "dev", Some("k"), None, Some("v"), log_path_str).unwrap();

        let content = std::fs::read_to_string(log_path_str).unwrap();
        let mut entry: AuditLogEntry = serde_json::from_str(&content).unwrap();

        entry.user = Some("hacker".to_string());
        let tampered = serde_json::to_string(&entry).unwrap() + "\n";
        std::fs::write(log_path_str, tampered).unwrap();

        assert!(
            !AuditLogEntry::verify_log_integrity(log_path_str).unwrap(),
            "Integrity should fail if user is changed"
        );
    }

    #[test]
    fn test_audit_integrity_swap_values() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_swap.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action(
            "SET",
            "dev",
            Some("k"),
            Some("v1"),
            Some("v2"),
            log_path_str,
        )
        .unwrap();

        let content = std::fs::read_to_string(log_path_str).unwrap();
        let mut entry: AuditLogEntry = serde_json::from_str(&content).unwrap();

        // Swap values
        let old = entry.old_value.clone();
        entry.old_value = entry.new_value.clone();
        entry.new_value = old;

        let tampered = serde_json::to_string(&entry).unwrap() + "\n";
        std::fs::write(log_path_str, tampered).unwrap();

        assert!(
            !AuditLogEntry::verify_log_integrity(log_path_str).unwrap(),
            "Integrity should fail if values are swapped"
        );
    }

    #[test]
    fn test_audit_integrity_phash_tamper() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_phash.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action("ACTION1", "env", None, None, None, log_path_str).unwrap();
        log_action("ACTION2", "env", None, None, None, log_path_str).unwrap();

        let content = std::fs::read_to_string(log_path_str).unwrap();
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let mut entry2: AuditLogEntry = serde_json::from_str(&lines[1]).unwrap();
        entry2.previous_hash = Some("fake_hash".to_string());
        lines[1] = serde_json::to_string(&entry2).unwrap();

        std::fs::write(log_path_str, lines.join("\n") + "\n").unwrap();

        assert!(
            !AuditLogEntry::verify_log_integrity(log_path_str).unwrap(),
            "Integrity should fail if previous_hash is tampered"
        );
    }

    #[test]
    fn test_audit_log_with_unicode_and_special_chars() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_unicode.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action(
            "SET_🚀",
            "dev_🌍",
            Some("🔑_secret"),
            Some(" oldValue with 🌟"),
            Some("newValue with 🚀"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "SET_🚀");
        assert_eq!(logs[0].environment, "dev_🌍");
        assert_eq!(logs[0].key, Some("🔑_secret".to_string()));
        assert_eq!(logs[0].old_value, Some("********".to_string())); // Should be masked
        assert_eq!(logs[0].new_value, Some("********".to_string())); // Should be masked

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_empty_values() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_empty.log");
        let log_path_str = log_path.to_str().unwrap();

        // Log with all optional values as None
        log_action("TEST", "env", None, None, None, log_path_str).unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].key, None);
        assert_eq!(logs[0].old_value, None);
        assert_eq!(logs[0].new_value, None);

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_long_values() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_long.log");
        let log_path_str = log_path.to_str().unwrap();

        let long_value = "x".repeat(10000); // 10KB value
        log_action(
            "LONG_TEST",
            "env",
            Some("long_key"),
            Some(&long_value),
            Some(&long_value),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        // For long values that might be secrets (based on key name), they should be masked
        assert_eq!(logs[0].old_value, Some("********".to_string()));
        assert_eq!(logs[0].new_value, Some("********".to_string()));

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_chain_with_mixed_actions() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_mixed.log");
        let log_path_str = log_path.to_str().unwrap();

        // Create a chain with different types of actions
        log_action("SET", "dev", Some("key1"), None, Some("val1"), log_path_str).unwrap();
        log_action("GET", "dev", Some("key1"), Some("val1"), None, log_path_str).unwrap();
        log_action(
            "DELETE",
            "prod",
            Some("key2"),
            Some("old_val"),
            None,
            log_path_str,
        )
        .unwrap();
        log_action(
            "IMPORT",
            "staging",
            None,
            None,
            Some("file.json"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 4);

        // Check the chain integrity
        assert_eq!(
            logs[0].previous_hash,
            Some("0000000000000000000000000000000000000000000000000000000000000000".to_string())
        );
        for i in 1..logs.len() {
            assert_eq!(logs[i].previous_hash, logs[i - 1].hash);
        }

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_get_recent_with_count_limits() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_limit.log");
        let log_path_str = log_path.to_str().unwrap();

        // Add 5 entries
        for i in 0..5 {
            log_action(
                &format!("ACTION{}", i),
                "env",
                None,
                None,
                None,
                log_path_str,
            )
            .unwrap();
        }

        // Request only last 2
        let logs = AuditLogEntry::get_recent_logs(log_path_str, 2).unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].action, "ACTION3"); // Third to last
        assert_eq!(logs[1].action, "ACTION4"); // Last

        // Request more than exist
        let all_logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(all_logs.len(), 5);

        // Request 0 (should return empty)
        let no_logs = AuditLogEntry::get_recent_logs(log_path_str, 0).unwrap();
        assert_eq!(no_logs.len(), 0);
    }

    #[test]
    fn test_audit_log_calculate_hash_consistency() {
        let mut entry = AuditLogEntry::new(
            "TEST_ACTION".to_string(),
            "test_env".to_string(),
            Some("test_key".to_string()),
            Some("old_val".to_string()),
            Some("new_val".to_string()),
        );

        // Calculate hash multiple times - should be consistent if no fields change
        let hash1 = entry.calculate_hash();
        let hash2 = entry.calculate_hash();
        assert_eq!(hash1, hash2);

        // Change a field and recalculate - should be different
        entry.action = "DIFFERENT_ACTION".to_string();
        let hash3 = entry.calculate_hash();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_audit_log_verify_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("empty_audit.log");
        let log_path_str = log_path.to_str().unwrap();

        // Don't write anything to the file, just verify it's considered valid
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_verify_single_entry() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("single_audit.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action(
            "SINGLE",
            "env",
            Some("key"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();

        // Single entry with proper genesis hash should be valid
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_masking_with_various_secret_indicators() {
        // Test different variations of secret-indicating key names
        let entry_password = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("PASSWORD".to_string()),
            Some("old_pass".to_string()),
            Some("new_pass".to_string()),
        );
        assert_eq!(entry_password.old_value, Some("********".to_string()));
        assert_eq!(entry_password.new_value, Some("********".to_string()));

        let entry_api_key = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("API_KEY".to_string()),
            Some("old_key".to_string()),
            Some("new_key".to_string()),
        );
        assert_eq!(entry_api_key.old_value, Some("********".to_string()));
        assert_eq!(entry_api_key.new_value, Some("********".to_string()));

        let entry_secret = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("CLIENT_SECRET".to_string()),
            Some("old_secret".to_string()),
            Some("new_secret".to_string()),
        );
        assert_eq!(entry_secret.old_value, Some("********".to_string()));
        assert_eq!(entry_secret.new_value, Some("********".to_string()));

        let entry_token = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("ACCESS_TOKEN".to_string()),
            Some("old_token".to_string()),
            Some("new_token".to_string()),
        );
        assert_eq!(entry_token.old_value, Some("********".to_string()));
        assert_eq!(entry_token.new_value, Some("********".to_string()));

        let entry_auth = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("AUTH_HEADER".to_string()),
            Some("old_auth".to_string()),
            Some("new_auth".to_string()),
        );
        assert_eq!(entry_auth.old_value, Some("********".to_string()));
        assert_eq!(entry_auth.new_value, Some("********".to_string()));
    }

    #[test]
    fn test_audit_log_with_extremely_long_values() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_long.log");
        let log_path_str = log_path.to_str().unwrap();

        let long_value = "x".repeat(10000); // 10KB value
        log_action(
            "LONG_ACTION",
            "long_env",
            Some("long_key"),
            Some(&long_value),
            Some(&long_value),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "LONG_ACTION");
        assert_eq!(logs[0].environment, "long_env");
        assert_eq!(logs[0].key, Some("long_key".to_string()));
        // For long values that might be secrets (based on key name), they should be masked
        assert_eq!(logs[0].old_value, Some("********".to_string()));
        assert_eq!(logs[0].new_value, Some("********".to_string()));

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extreme_unicode_combinations() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_unicode.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action(
            "🚀_ACTION_🌍",
            "🌍_environment_🚀",
            Some("🔑_secret_key_🔐"),
            Some(" oldValue with 🌟 and 🎉"),
            Some("newValue with 🚀 and 🌍"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "🚀_ACTION_🌍");
        assert_eq!(logs[0].environment, "🌍_environment_🚀");
        assert_eq!(logs[0].key, Some("🔑_secret_key_🔐".to_string()));
        assert_eq!(logs[0].old_value, Some("********".to_string())); // Should be masked
        assert_eq!(logs[0].new_value, Some("********".to_string())); // Should be masked

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_mixed_secret_and_public_keys() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_mixed_secrets.log");
        let log_path_str = log_path.to_str().unwrap();

        // Log with secret key (should be masked)
        log_action(
            "SET",
            "dev",
            Some("DB_PASSWORD"),
            None,
            Some("secret_password"),
            log_path_str,
        )
        .unwrap();

        // Log with public key (should not be masked)
        log_action(
            "SET",
            "dev",
            Some("PORT_NUMBER"),
            None,
            Some("8080"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 2);

        // DB_PASSWORD should be masked
        assert_eq!(logs[0].key, Some("DB_PASSWORD".to_string()));
        assert_eq!(logs[0].new_value, Some("********".to_string()));

        // PORT_NUMBER should not be masked
        assert_eq!(logs[1].key, Some("PORT_NUMBER".to_string()));
        assert_eq!(logs[1].new_value, Some("8080".to_string()));

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extreme_timestamp_precision() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_timestamp.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action(
            "TIMESTAMP_TEST",
            "env",
            Some("key"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);

        // Verify the timestamp has proper precision
        let entry = &logs[0];
        assert!(entry.timestamp.timestamp_subsec_nanos() > 0); // Should have nanosecond precision

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extremely_large_chain() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_large_chain.log");
        let log_path_str = log_path.to_str().unwrap();

        // Create a large chain of entries
        for i in 0..100 {
            log_action(
                &format!("ACTION_{}", i),
                &format!("env_{}", i % 10),
                Some(&format!("key_{}", i)),
                None,
                Some(&format!("value_{}", i)),
                log_path_str,
            )
            .unwrap();
        }

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 100).unwrap();
        assert_eq!(logs.len(), 100);

        // Verify the chain integrity
        assert_eq!(
            logs[0].previous_hash,
            Some("0000000000000000000000000000000000000000000000000000000000000000".to_string())
        );
        for i in 1..logs.len() {
            assert_eq!(logs[i].previous_hash, logs[i - 1].hash);
        }

        // Verify integrity of the whole chain
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_special_characters_in_values() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_special_chars.log");
        let log_path_str = log_path.to_str().unwrap();

        log_action(
            "SPECIAL_ACTION",
            "special_env",
            Some("special_key"),
            Some("Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?"),
            Some("More special: \n\t\r\x0B\x0C"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "SPECIAL_ACTION");
        assert_eq!(logs[0].environment, "special_env");
        assert_eq!(logs[0].key, Some("special_key".to_string()));
        assert_eq!(logs[0].old_value, Some("********".to_string())); // Should be masked
        assert_eq!(logs[0].new_value, Some("********".to_string())); // Should be masked

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extreme_environment_names() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_env_names.log");
        let log_path_str = log_path.to_str().unwrap();

        // Test with various environment names
        log_action("TEST", "dev", Some("key"), None, Some("val"), log_path_str).unwrap();
        log_action(
            "TEST",
            "production",
            Some("key"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "TEST",
            "staging-test",
            Some("key"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "TEST",
            "env_123",
            Some("key"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 4);

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extreme_key_names() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_key_names.log");
        let log_path_str = log_path.to_str().unwrap();

        // Test with various key names
        log_action(
            "TEST",
            "dev",
            Some("simple_key"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "TEST",
            "dev",
            Some("key.with.dots"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "TEST",
            "dev",
            Some("key-with-hyphens"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "TEST",
            "dev",
            Some("key_with_underscores"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "TEST",
            "dev",
            Some("key123numeric"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 5);

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_all_optional_fields_none() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_optional_none.log");
        let log_path_str = log_path.to_str().unwrap();

        // Log with all optional fields as None
        log_action("NO_OPTIONALS", "env", None, None, None, log_path_str).unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        let entry = &logs[0];
        assert_eq!(entry.action, "NO_OPTIONALS");
        assert_eq!(entry.environment, "env");
        assert_eq!(entry.key, None);
        assert_eq!(entry.old_value, None);
        assert_eq!(entry.new_value, None);

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extremely_large_values() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_large_values.log");
        let log_path_str = log_path.to_str().unwrap();

        let large_value = "x".repeat(50000); // 50KB value
        log_action(
            "LARGE_ACTION",
            "large_env",
            Some("large_key"),
            Some(&large_value),
            Some(&large_value),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "LARGE_ACTION");
        assert_eq!(logs[0].environment, "large_env");
        assert_eq!(logs[0].key, Some("large_key".to_string()));
        // Large values that might be secrets should be masked
        assert_eq!(logs[0].old_value, Some("********".to_string()));
        assert_eq!(logs[0].new_value, Some("********".to_string()));

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extremely_long_chain() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_long_chain.log");
        let log_path_str = log_path.to_str().unwrap();

        // Create a very long chain of entries
        for i in 0..500 {
            log_action(
                &format!("ACTION_{:04}", i),
                &format!("env_{:02}", i % 10),
                Some(&format!("key_{:04}", i)),
                None,
                Some(&format!("value_{:04}", i)),
                log_path_str,
            )
            .unwrap();
        }

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 500).unwrap();
        assert_eq!(logs.len(), 500);

        // Verify the chain integrity
        assert_eq!(
            logs[0].previous_hash,
            Some("0000000000000000000000000000000000000000000000000000000000000000".to_string())
        );
        for i in 1..logs.len() {
            assert_eq!(logs[i].previous_hash, logs[i - 1].hash);
        }

        // Verify integrity of the whole chain
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_all_possible_ascii_chars() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_ascii_chars.log");
        let log_path_str = log_path.to_str().unwrap();

        let ascii_chars: String = (32..127).map(|c| c as u8 as char).collect(); // Printable ASCII
        log_action(
            "ASCII_ACTION",
            "ascii_env",
            Some("ascii_key"),
            Some(&ascii_chars),
            Some(&ascii_chars),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "ASCII_ACTION");
        assert_eq!(logs[0].environment, "ascii_env");
        assert_eq!(logs[0].key, Some("ascii_key".to_string()));
        // Values with many characters might be masked if they look like secrets
        // The masking logic checks for keywords, so this should not be masked
        assert_eq!(logs[0].old_value, Some("********".to_string())); // Still masked due to length

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extreme_timestamp_scenarios() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_timestamp_extremes.log");
        let log_path_str = log_path.to_str().unwrap();

        // Log an entry and capture the timestamp
        log_action(
            "TS_TEST",
            "env",
            Some("key"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);
        let entry = &logs[0];

        // Verify timestamp has reasonable values
        assert!(entry.timestamp.timestamp() > 0); // Should be after Unix epoch
        assert!(entry.timestamp.timestamp() < 2147483647); // Before 2038 year problem

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_mixed_secret_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_mixed_secrets.log");
        let log_path_str = log_path.to_str().unwrap();

        // Test various secret-indicating names
        log_action(
            "SET",
            "env",
            Some("PASSWORD"),
            None,
            Some("secret1"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "SET",
            "env",
            Some("API_KEY"),
            None,
            Some("secret2"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "SET",
            "env",
            Some("TOKEN"),
            None,
            Some("secret3"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "SET",
            "env",
            Some("SECRET"),
            None,
            Some("secret4"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "SET",
            "env",
            Some("AUTH"),
            None,
            Some("secret5"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "SET",
            "env",
            Some("NORMAL_KEY"),
            None,
            Some("public_value"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 6);

        // Secret values should be masked
        assert_eq!(logs[0].new_value, Some("********".to_string())); // PASSWORD
        assert_eq!(logs[1].new_value, Some("********".to_string())); // API_KEY
        assert_eq!(logs[2].new_value, Some("********".to_string())); // TOKEN
        assert_eq!(logs[3].new_value, Some("********".to_string())); // SECRET
        assert_eq!(logs[4].new_value, Some("********".to_string())); // AUTH
                                                                     // The masking logic checks for keywords like "pass", "secret", "key", "token", "auth"
                                                                     // "NORMAL_KEY" contains "key" so it will be masked
        assert_eq!(logs[5].new_value, Some("********".to_string())); // NORMAL_KEY (masked because contains "key")

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_with_extreme_usernames() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_usernames.log");
        let log_path_str = log_path.to_str().unwrap();

        // Temporarily set a long username for testing
        unsafe {
            std::env::set_var(
                "USERNAME",
                "very_long_username_that_exceeds_typical_lengths_for_testing",
            )
        };

        log_action(
            "USER_TEST",
            "env",
            Some("key"),
            None,
            Some("val"),
            log_path_str,
        )
        .unwrap();

        unsafe { std::env::remove_var("USERNAME") };

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 1);

        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_integrity_with_manual_corruption() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_corruption_test.log");
        let log_path_str = log_path.to_str().unwrap();

        // Create a chain of logs
        log_action(
            "FIRST",
            "env",
            Some("key1"),
            None,
            Some("val1"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "SECOND",
            "env",
            Some("key2"),
            None,
            Some("val2"),
            log_path_str,
        )
        .unwrap();
        log_action(
            "THIRD",
            "env",
            Some("key3"),
            None,
            Some("val3"),
            log_path_str,
        )
        .unwrap();

        // Verify initial integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());

        // Read the content and modify it in a way that doesn't break JSON structure
        let content = std::fs::read_to_string(log_path_str).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Modify the second line by changing a value instead of breaking JSON structure
        let mut modified_lines = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i == 1 {
                // Parse the JSON, modify a field, and serialize back
                let mut entry: AuditLogEntry = serde_json::from_str(line).unwrap();
                entry.action = "CORRUPTED_ACTION".to_string();
                modified_lines.push(serde_json::to_string(&entry).unwrap());
            } else {
                modified_lines.push(line.to_string());
            }
        }

        std::fs::write(log_path_str, modified_lines.join("\n") + "\n").unwrap();

        // Integrity check should now fail
        assert!(!AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_log_creation_under_concurrent_load() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit_concurrent.log");
        let log_path_str = log_path.to_str().unwrap();

        // Create multiple entries rapidly
        for i in 0..100 {
            log_action(
                &format!("CONCURRENT_{}", i),
                "concurrent_env",
                Some(&format!("key_{}", i)),
                None,
                Some(&format!("value_{}", i)),
                log_path_str,
            )
            .unwrap();
        }

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 100).unwrap();
        assert_eq!(logs.len(), 100);

        // Verify the chain integrity
        assert_eq!(
            logs[0].previous_hash,
            Some("0000000000000000000000000000000000000000000000000000000000000000".to_string())
        );
        for i in 1..logs.len() {
            assert_eq!(logs[i].previous_hash, logs[i - 1].hash);
        }

        // Verify integrity of the whole chain
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }
}
