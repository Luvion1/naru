use std::fs::{self, OpenOptions};
use std::io::{BufRead, Write};
use std::path::Path;

use crate::core::audit_chain::{verify_log_integrity, AuditChain};
use crate::core::audit_entry::AuditLogEntry;
use crate::core::locking::FileLock;

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

    log_entry_to_file(&mut entry, log_path)
}

fn log_entry_to_file(
    entry: &mut AuditLogEntry,
    log_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::core::path_sanitizer::sanitize_file_path_internal;

    let sanitized_log_path = sanitize_file_path_internal(log_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let log_path_obj = Path::new(&sanitized_log_path);
    let lock_dir = log_path_obj.parent().unwrap_or(Path::new("."));
    let lock_path = lock_dir.join("audit.lock");

    let max_retries = 5;
    let mut last_error: Option<std::io::Error> = None;
    let mut acquired_lock = None;

    for attempt in 0..max_retries {
        match FileLock::acquire_exclusive(&lock_path) {
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

    let _lock = acquired_lock.unwrap();

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
            None
        }
    } else {
        None
    };

    AuditChain::link_entry(entry, prev_hash);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&sanitized_log_path)?;

    let log_line = serde_json::to_string(entry)?;
    writeln!(file, "{}", log_line)?;

    Ok(())
}

pub fn get_recent_logs(
    log_path: &str,
    count: usize,
) -> Result<Vec<AuditLogEntry>, Box<dyn std::error::Error>> {
    use crate::core::path_sanitizer::sanitize_file_path_internal;
    use std::io::BufRead;

    let sanitized_log_path = sanitize_file_path_internal(log_path)
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

    let start = if entries.len() > count {
        entries.len() - count
    } else {
        0
    };

    Ok(entries.into_iter().skip(start).collect())
}

pub use crate::core::audit_chain::verify_log_integrity as verify_audit_log;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_action() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        log_action(
            "SET",
            "dev",
            Some("key1"),
            None,
            Some("val1"),
            log_path.to_str().unwrap(),
        )
        .unwrap();

        let logs = get_recent_logs(log_path.to_str().unwrap(), 10).unwrap();
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_log_chain() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        log_action(
            "SET",
            "dev",
            Some("key1"),
            None,
            Some("val1"),
            log_path.to_str().unwrap(),
        )
        .unwrap();
        log_action(
            "SET",
            "dev",
            Some("key2"),
            None,
            Some("val2"),
            log_path.to_str().unwrap(),
        )
        .unwrap();

        let logs = get_recent_logs(log_path.to_str().unwrap(), 10).unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[1].previous_hash, logs[0].hash);
    }

    #[test]
    fn test_verify_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        log_action(
            "SET",
            "dev",
            Some("key"),
            None,
            Some("val"),
            log_path.to_str().unwrap(),
        )
        .unwrap();

        let result = verify_log_integrity(log_path.to_str().unwrap()).unwrap();
        assert!(result);
    }
}
