use crate::core::audit_entry::AuditLogEntry;

pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub struct AuditChain;

impl AuditChain {
    pub fn link_entry(entry: &mut AuditLogEntry, previous_hash: Option<String>) {
        entry.previous_hash = previous_hash.or_else(|| Some(GENESIS_HASH.to_string()));
        entry.hash = Some(entry.calculate_hash());
    }

    pub fn verify_chain(entries: &[AuditLogEntry]) -> bool {
        if entries.is_empty() {
            return true;
        }

        let mut expected_prev_hash = GENESIS_HASH.to_string();

        for entry in entries {
            if entry.previous_hash.as_ref() != Some(&expected_prev_hash) {
                return false;
            }

            let calculated = entry.calculate_hash();
            if entry.hash.as_ref() != Some(&calculated) {
                return false;
            }

            expected_prev_hash = calculated;
        }

        true
    }

    pub fn verify_integrity(entries: &[AuditLogEntry]) -> Result<bool, String> {
        Ok(Self::verify_chain(entries))
    }
}

pub fn verify_log_integrity(log_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::core::path_sanitizer::sanitize_file_path_internal;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let sanitized_path = sanitize_file_path_internal(log_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    if !std::path::Path::new(&sanitized_path).exists() {
        return Ok(true);
    }

    let file = File::open(&sanitized_path)?;
    let reader = BufReader::new(file);

    let entries: Vec<AuditLogEntry> = reader
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect();

    Ok(AuditChain::verify_chain(&entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_genesis_hash() {
        assert_eq!(GENESIS_HASH.len(), 64);
    }

    #[test]
    fn test_link_entry() {
        let mut entry = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("key".to_string()),
            None,
            Some("value".to_string()),
        );

        AuditChain::link_entry(&mut entry, None);

        assert!(entry.previous_hash.is_some());
        assert!(entry.hash.is_some());
    }

    #[test]
    fn test_verify_chain_empty() {
        let entries: Vec<AuditLogEntry> = vec![];
        assert!(AuditChain::verify_chain(&entries));
    }

    #[test]
    fn test_verify_chain_valid() {
        let mut entry1 = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("key1".to_string()),
            None,
            Some("value1".to_string()),
        );

        let mut entry2 = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("key2".to_string()),
            None,
            Some("value2".to_string()),
        );

        AuditChain::link_entry(&mut entry1, None);
        AuditChain::link_entry(&mut entry2, entry1.hash.clone());

        let entries = vec![entry1, entry2];
        assert!(AuditChain::verify_chain(&entries));
    }

    #[test]
    fn test_verify_chain_tampered() {
        let mut entry1 = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("key1".to_string()),
            None,
            Some("value1".to_string()),
        );

        let mut entry2 = AuditLogEntry::new(
            "SET".to_string(),
            "dev".to_string(),
            Some("key2".to_string()),
            None,
            Some("value2".to_string()),
        );

        AuditChain::link_entry(&mut entry1, None);

        // Tamper with entry1
        entry1.action = "TAMPERED".to_string();

        AuditChain::link_entry(&mut entry2, entry1.hash.clone());

        let entries = vec![entry1, entry2];
        assert!(!AuditChain::verify_chain(&entries));
    }
}
