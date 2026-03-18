use serde::{Deserialize, Serialize};

use crate::core::config_model::ConfigFile;
use crate::core::schema_model::SchemaFile;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupData {
    pub config: ConfigFile,
    pub schema: SchemaFile,
    pub backup_timestamp: String,
    pub version: String,
}

impl BackupData {
    pub fn new(config: ConfigFile, schema: SchemaFile) -> Self {
        use chrono::Utc;
        BackupData {
            config,
            schema,
            backup_timestamp: Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn from_config_and_schema(config: ConfigFile, schema: SchemaFile) -> Self {
        Self::new(config, schema)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version.is_empty() {
            return Err("Backup version cannot be empty".to_string());
        }
        if self.backup_timestamp.is_empty() {
            return Err("Backup timestamp cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn config(&self) -> &ConfigFile {
        &self.config
    }

    pub fn schema(&self) -> &SchemaFile {
        &self.schema
    }

    pub fn timestamp(&self) -> &str {
        &self.backup_timestamp
    }

    pub fn backup_version(&self) -> &str {
        &self.version
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupMetadata {
    pub created_at: String,
    pub version: String,
    pub project_name: String,
    pub environment_count: usize,
    pub total_keys: usize,
}

impl BackupMetadata {
    pub fn from_backup(backup: &BackupData) -> Self {
        let total_keys = backup
            .config
            .environments
            .values()
            .map(|e| e.entries.len())
            .sum();

        BackupMetadata {
            created_at: backup.backup_timestamp.clone(),
            version: backup.version.clone(),
            project_name: backup.config.project_name.clone(),
            environment_count: backup.config.environments.len(),
            total_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_backup_data_new() {
        let config = ConfigFile::new("TestProject");
        let schema = SchemaFile::new("1.0.0");
        let backup = BackupData::new(config.clone(), schema.clone());

        assert_eq!(backup.config.project_name, "TestProject");
        assert!(!backup.backup_timestamp.is_empty());
        assert!(!backup.version.is_empty());
    }

    #[test]
    fn test_backup_data_from_config_and_schema() {
        let config = ConfigFile::new("MyProject");
        let schema = SchemaFile::new("1.0.0");
        let backup = BackupData::from_config_and_schema(config, schema);

        assert_eq!(backup.config.project_name, "MyProject");
    }

    #[test]
    fn test_backup_data_validate() {
        let config = ConfigFile::new("Test");
        let schema = SchemaFile::new("1.0.0");
        let backup = BackupData::new(config, schema);

        assert!(backup.validate().is_ok());
    }

    #[test]
    fn test_backup_metadata_from_backup() {
        let mut config = ConfigFile::new("TestProject");
        let mut env = config.add_environment("production");
        env.set_value(
            "KEY1",
            crate::core::config_model::ConfigValueEntry::new("value1", "string", false),
        );
        env.set_value(
            "KEY2",
            crate::core::config_model::ConfigValueEntry::new("value2", "string", false),
        );

        let schema = SchemaFile::new("1.0.0");
        let backup = BackupData::new(config, schema);
        let metadata = BackupMetadata::from_backup(&backup);

        assert_eq!(metadata.project_name, "TestProject");
        assert_eq!(metadata.environment_count, 1);
        assert_eq!(metadata.total_keys, 2);
    }

    #[test]
    fn test_backup_with_multiple_environments() {
        let mut config = ConfigFile::new("MultiEnv");
        config.add_environment("development");
        config.add_environment("staging");
        config.add_environment("production");

        let mut env = config.add_environment("production");
        env.set_value(
            "DB_HOST",
            crate::core::config_model::ConfigValueEntry::new("localhost", "string", false),
        );

        let schema = SchemaFile::new("1.0.0");
        let backup = BackupData::new(config, schema);
        let metadata = BackupMetadata::from_backup(&backup);

        assert_eq!(metadata.environment_count, 4);
    }
}
