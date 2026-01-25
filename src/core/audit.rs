use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub environment: String,
    pub key: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub user: Option<String>,
}

impl AuditLogEntry {
    pub fn new(
        action: String,
        environment: String,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> Self {
        AuditLogEntry {
            timestamp: Utc::now(),
            action,
            environment,
            key,
            old_value,
            new_value,
            user: std::env::var("USER").or_else(|_| std::env::var("USERNAME")).ok(), // Get username from environment
        }
    }

    pub fn log_to_file(&self, log_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        let log_line = serde_json::to_string(self)?;
        writeln!(file, "{}", log_line)?;
        Ok(())
    }

    pub fn get_recent_logs(log_path: &str, count: usize) -> Result<Vec<AuditLogEntry>, Box<dyn std::error::Error>> {
        if !Path::new(log_path).exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(log_path)?;
        let reader = std::io::BufReader::new(file);
        let mut entries: Vec<AuditLogEntry> = Vec::with_capacity(count);

        use std::io::BufRead;
        for line in reader.lines() {
            if let Ok(line_str) = line {
                if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(&line_str) {
                    entries.push(entry);
                }
            }
        }

        // Return the most recent entries
        let start = if entries.len() > count { entries.len() - count } else { 0 };
        Ok(entries.into_iter().skip(start).collect())
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
    let entry = AuditLogEntry::new(
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

        fn test_log_action() {

            let temp_dir = TempDir::new().unwrap();

            let log_path = temp_dir.path().join("audit.log");

            let log_path_str = log_path.to_str().unwrap();

    

            log_action("SET", "dev", Some("key1"), Some("old"), Some("new"), log_path_str).unwrap();

    

            let logs = AuditLogEntry::get_recent_logs(log_path_str, 10).unwrap();

            assert_eq!(logs.len(), 1);

            assert_eq!(logs[0].action, "SET");

                        assert_eq!(logs[0].key, Some("key1".to_string()));

                    }

            

                            #[test]

            

                            fn test_audit_user_identification() {

            

                                unsafe { std::env::set_var("USER", "test_user") };

            

                                let entry = AuditLogEntry::new(

            

                                    "TEST".to_string(),

            

                                    "dev".to_string(),

            

                                    None,

            

                                    None,

            

                                    None,

            

                                );

            

                                assert_eq!(entry.user, Some("test_user".to_string()));

            

                                unsafe { std::env::remove_var("USER") };

            

                            }

            

                    

                }

            

    