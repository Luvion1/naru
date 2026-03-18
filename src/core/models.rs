pub use crate::core::backup_model::BackupData;
pub use crate::core::config_model::ConfigFile;
pub use crate::core::config_model::ConfigValueEntry;
pub use crate::core::config_model::EnvironmentConfig;
pub use crate::core::schema_model::FieldDefinition;
pub use crate::core::schema_model::SchemaFile;
pub use crate::core::schema_model::ValidationRules;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_exports() {
        let config = ConfigFile::new("Test");
        assert_eq!(config.project_name, "Test");

        let schema = SchemaFile::new("1.0");
        assert_eq!(schema.version, "1.0");
    }
}
