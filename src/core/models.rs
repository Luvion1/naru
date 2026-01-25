use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigFile {
    pub project_name: String,
    pub version: String,
    pub environments: HashMap<String, EnvironmentConfig>,
}

impl ConfigFile {
    pub fn new(project_name: &str, version: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            version: version.to_string(),
            environments: HashMap::new(),
        }
    }

    pub fn add_environment(&mut self, name: &str) {
        self.environments.insert(
            name.to_string(),
            EnvironmentConfig { entries: HashMap::new() },
        );
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvironmentConfig {
    pub entries: HashMap<String, ConfigValueEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigValueEntry {
    pub value: String,
    pub r#type: String,
    pub is_secret: bool,
    #[serde(default)]
    pub encrypted: bool,
}

impl ConfigValueEntry {
    pub fn new(value: &str, r#type: &str, is_secret: bool) -> Self {
        Self {
            value: value.to_string(),
            r#type: r#type.to_string(),
            is_secret,
            encrypted: false,
        }
    }

    pub fn get_display_value(&self) -> String {
        if self.is_secret {
            "********".to_string()
        } else {
            self.value.clone()
        }
    }

    pub fn validate(&self, field: &FieldDefinition) -> Result<(), String> {
        crate::core::validation::validate_value(&self.value, field)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaFile {
    pub version: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidationRules {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldDefinition {
    pub key: String,
    pub r#type: String,
    pub description: Option<String>,
    pub validation: Option<ValidationRules>,
    #[serde(default)]
    pub is_secret: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupData {
    pub config: ConfigFile,
    pub schema: SchemaFile,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_config_file_rich_behavior() {
        let mut config = ConfigFile::new("Rich Project", "2.0.0");
        config.add_environment("production");

        assert_eq!(config.project_name, "Rich Project");
        assert!(config.environments.contains_key("production"));
    }

    #[test]
    fn test_config_value_entry_new() {
        let entry = ConfigValueEntry::new("secret_pass", "string", true);
        assert_eq!(entry.value, "secret_pass");
        assert!(entry.is_secret);
        assert!(!entry.encrypted);
    }

    #[test]
    fn test_config_value_entry_masking() {
        let secret_entry = ConfigValueEntry::new("super_secret", "string", true);
        let plain_entry = ConfigValueEntry::new("normal_value", "string", false);

        assert_eq!(secret_entry.get_display_value(), "********");
        assert_eq!(plain_entry.get_display_value(), "normal_value");
    }

    #[test]
    fn test_config_value_entry() {
        let entry = ConfigValueEntry {
            value: "test_value".to_string(),
            r#type: "string".to_string(),
            is_secret: true,
            encrypted: false,
        };

        assert_eq!(entry.value, "test_value");
        assert_eq!(entry.r#type, "string");
        assert_eq!(entry.is_secret, true);
        assert_eq!(entry.encrypted, false);
    }

    #[test]
    fn test_config_value_entry_validation() {
        let field = FieldDefinition {
            key: "port".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_value: Some(1024),
                max_value: Some(65535),
                min_length: None,
                max_length: None,
            }),
            is_secret: false,
        };

        let valid_entry = ConfigValueEntry::new("8080", "integer", false);
        let invalid_entry = ConfigValueEntry::new("80", "integer", false);
        let wrong_type_entry = ConfigValueEntry::new("abc", "integer", false);

        assert!(valid_entry.validate(&field).is_ok());
        assert!(invalid_entry.validate(&field).is_err());
        assert!(wrong_type_entry.validate(&field).is_err());
    }
}
