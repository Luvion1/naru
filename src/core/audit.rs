use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, Write};
use std::path::Path;

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
    pub fn new(
        action: String,
        environment: String,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> Self {
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

        let final_old_value = if let Some(ref k) = key {
            mask_value(k, old_value)
        } else {
            old_value
        };

        let final_new_value = if let Some(ref k) = key {
            mask_value(k, new_value)
        } else {
            new_value
        };

        AuditLogEntry {
            timestamp: Utc::now(),
            action,
            environment,
            key,
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
        // 1. Get the previous hash from the last line of the file
        let prev_hash = if Path::new(log_path).exists() {
            let file = fs::File::open(log_path)?;
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

        self.previous_hash = prev_hash.or_else(|| Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()));
        self.hash = Some(self.calculate_hash());

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        let log_line = serde_json::to_string(self)?;
        writeln!(file, "{}", log_line)?;
        Ok(())
    }

    pub fn get_recent_logs(
        log_path: &str,
        count: usize,
    ) -> Result<Vec<AuditLogEntry>, Box<dyn std::error::Error>> {
        if !Path::new(log_path).exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(log_path)?;
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
        if !Path::new(log_path).exists() {
            return Ok(true);
        }

        let file = fs::File::open(log_path)?;
        let reader = std::io::BufReader::new(file);
        
        let mut expected_prev_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

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
        log_action(
            "SET",
            "dev",
            Some("key1"),
            None,
            Some("val1"),
            log_path_str,
        )
        .unwrap();

        // Second log
        log_action(
            "SET",
            "dev",
            Some("key2"),
            None,
            Some("val2"),
            log_path_str,
        )
        .unwrap();

        let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();
        assert_eq!(logs.len(), 2);
        
        // Check genesis hash
        assert_eq!(logs[0].previous_hash.as_deref(), Some("0000000000000000000000000000000000000000000000000000000000000000"));
        
        // Check chain
        assert_eq!(logs[1].previous_hash, logs[0].hash);
        
        // Verify integrity
        assert!(AuditLogEntry::verify_log_integrity(log_path_str).unwrap());
    }

    #[test]
    fn test_audit_masking() {
        let entry = AuditLogEntry::new("SET".to_string(), "dev".to_string(), Some("DB_PASSWORD".to_string()), Some("old_secret".to_string()), Some("new_secret".to_string()));
        assert_eq!(entry.old_value, Some("********".to_string()));
        assert_eq!(entry.new_value, Some("********".to_string()));
        
        let entry_safe = AuditLogEntry::new("SET".to_string(), "dev".to_string(), Some("PUBLIC_PORT".to_string()), Some("8080".to_string()), Some("9090".to_string()));
        assert_eq!(entry_safe.old_value, Some("8080".to_string()));
    }
}