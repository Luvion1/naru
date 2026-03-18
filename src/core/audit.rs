pub use crate::core::audit_chain::verify_log_integrity;
pub use crate::core::audit_entry::AuditLogEntry;
pub use crate::core::audit_log::{get_recent_logs, log_action};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_full_audit_flow() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        log_action(
            "SET",
            "production",
            Some("DB_PASSWORD"),
            None,
            Some("secret123"),
            log_path.to_str().unwrap(),
        )
        .unwrap();

        let logs = get_recent_logs(log_path.to_str().unwrap(), 1).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].new_value, Some("********".to_string())); // Masked

        let verified = verify_log_integrity(log_path.to_str().unwrap()).unwrap();
        assert!(verified);
    }
}
