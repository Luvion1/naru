use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    pub project_name: String,
    pub version: String,
    pub environments: HashMap<String, EnvironmentConfig>,
    pub salt: Option<String>, // Base64 encoded salt for key derivation
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentConfig {
    pub entries: HashMap<String, ConfigValueEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigValueEntry {
    pub value: String,
    pub r#type: String,
    pub is_secret: bool,
    pub encrypted: bool,
}

impl ConfigValueEntry {
    pub fn new(value: &str, r#type: &str, is_secret: bool) -> Self {
        ConfigValueEntry {
            value: value.to_string(),
            r#type: r#type.to_string(),
            is_secret,
            encrypted: false,
        }
    }

    pub fn validate(&self, field: &FieldDefinition) -> Result<(), String> {
        crate::core::validation::validate_value(&self.value, field)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaFile {
    pub version: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldDefinition {
    pub key: String,
    pub r#type: String, // string, integer, boolean
    pub description: Option<String>,
    pub validation: Option<ValidationRules>,
    pub is_secret: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationRules {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub config: ConfigFile,
    pub schema: SchemaFile,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_entry_new() {
        let entry = ConfigValueEntry::new("test_value", "string", true);
        assert_eq!(entry.value, "test_value");
        assert_eq!(entry.r#type, "string");
        assert!(entry.is_secret);
        assert!(!entry.encrypted);
    }

    #[test]
    fn test_schema_field_definition() {
        let field = FieldDefinition {
            key: "db_host".to_string(),
            r#type: "string".to_string(),
            description: Some("Database host address".to_string()),
            validation: None,
            is_secret: false,
        };
        assert_eq!(field.key, "db_host");
        assert!(field.description.is_some());
    }

    #[test]
    fn test_config_file_clone() {
        let mut environments = HashMap::new();
        let mut entries = HashMap::new();
        entries.insert("K1".into(), ConfigValueEntry::new("V1", "string", false));
        environments.insert("dev".into(), EnvironmentConfig { entries });

        let config = ConfigFile {
            project_name: "Test".into(),
            version: "1.0".into(),
            environments,
            salt: None,
        };

        let cloned = config.clone();
        assert_eq!(cloned.project_name, config.project_name);
        assert_eq!(cloned.environments.len(), config.environments.len());
    }

    #[test]
    fn test_empty_config_value_entry() {
        let entry = ConfigValueEntry::new("", "string", false);
        assert_eq!(entry.value, "");
    }

    #[test]
    fn test_config_value_entry_validation_mismatch() {
        let entry = ConfigValueEntry::new("not_an_int", "integer", false);
        let field = FieldDefinition {
            key: "test".into(),
            r#type: "integer".into(),
            description: None,
            validation: None,
            is_secret: false,
        };
        assert!(entry.validate(&field).is_err());
    }
}
