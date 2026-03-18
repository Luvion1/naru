use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub environment: String,
    pub key: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub user: Option<String>,
    pub previous_hash: Option<String>,
    pub hash: Option<String>,
}

impl AuditLogEntry {
    fn sanitize_for_log(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                continue;
            }
            match c {
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                _ => result.push(c),
            }
        }
        result
    }

    fn mask_sensitive_value(key: &str, value: Option<String>) -> Option<String> {
        if let Some(val) = value {
            let k_lower = key.to_lowercase();
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
    }

    pub fn new(
        action: String,
        environment: String,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> Self {
        let sanitized_action = Self::sanitize_for_log(&action);
        let sanitized_environment = Self::sanitize_for_log(&environment);
        let sanitized_key = key.map(|k| Self::sanitize_for_log(&k));
        let sanitized_old_value = old_value.map(|v| Self::sanitize_for_log(&v));
        let sanitized_new_value = new_value.map(|v| Self::sanitize_for_log(&v));

        let final_old_value = if let Some(ref k) = sanitized_key {
            Self::mask_sensitive_value(k, sanitized_old_value)
        } else {
            sanitized_old_value
        };

        let final_new_value = if let Some(ref k) = sanitized_key {
            Self::mask_sensitive_value(k, sanitized_new_value)
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
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
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

    pub fn get_recent_logs(
        log_path: &str,
        count: usize,
    ) -> Result<Vec<AuditLogEntry>, Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let sanitized = crate::core::path_sanitizer::sanitize_file_path_internal(log_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        if !std::path::Path::new(&sanitized).exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&sanitized)?;
        let reader = BufReader::new(file);
        let mut entries: Vec<AuditLogEntry> = Vec::with_capacity(count);

        for line_str in reader.lines().map_while(Result::ok) {
            if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(&line_str) {
                entries.push(entry);
            }
        }

        let start = if entries.len() > count {
            entries.len() - count
        } else {
            0
        };

        Ok(entries.into_iter().skip(start).collect())
    }

    pub fn verify_log_integrity(log_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let sanitized = crate::core::path_sanitizer::sanitize_file_path_internal(log_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        if !std::path::Path::new(&sanitized).exists() {
            return Ok(true);
        }

        let file = File::open(&sanitized)?;
        let reader = BufReader::new(file);

        let mut expected_prev_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        for line in reader.lines().map_while(Result::ok) {
            let entry: AuditLogEntry = serde_json::from_str(&line)?;

            if let Some(ph) = &entry.previous_hash {
                if ph != &expected_prev_hash {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_new() {
        let entry = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("KEY1".to_string()),
            None,
            Some("value1".to_string()),
        );

        assert_eq!(entry.action, "SET");
        assert_eq!(entry.environment, "dev");
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
    }

    #[test]
    fn test_audit_no_mask_for_non_secret() {
        let entry = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("PORT".to_string()),
            None,
            Some("8080".to_string()),
        );

        assert_eq!(entry.new_value, Some("8080".to_string()));
    }

    #[test]
    fn test_audit_hash_consistency() {
        let mut entry = AuditLogEntry::new(
            "TEST".to_string(),
            "env".to_string(),
            Some("key".to_string()),
            None,
            Some("value".to_string()),
        );

        let hash1 = entry.calculate_hash();
        let hash2 = entry.calculate_hash();
        assert_eq!(hash1, hash2);
    }
}
