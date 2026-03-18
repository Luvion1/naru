use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    pub project_name: String,
    pub version: String,
    pub environments: HashMap<String, EnvironmentConfig>,
    pub salt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EnvironmentConfig {
    pub parent: Option<String>,
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

    pub fn validate(
        &self,
        field: &crate::core::schema_model::FieldDefinition,
    ) -> Result<(), String> {
        crate::core::validator::validate_value(&self.value, field)
    }
}

impl ConfigFile {
    pub fn new(project_name: &str) -> Self {
        ConfigFile {
            project_name: project_name.to_string(),
            version: "1.0.0".to_string(),
            environments: HashMap::new(),
            salt: None,
        }
    }

    pub fn add_environment(&mut self, name: &str) -> &mut EnvironmentConfig {
        self.environments
            .entry(name.to_string())
            .or_insert_with(|| EnvironmentConfig::default())
    }

    pub fn get_environment(&self, name: &str) -> Option<&EnvironmentConfig> {
        self.environments.get(name)
    }

    pub fn get_environment_mut(&mut self, name: &str) -> Option<&mut EnvironmentConfig> {
        self.environments.get_mut(name)
    }
}

impl EnvironmentConfig {
    pub fn new() -> Self {
        EnvironmentConfig {
            parent: None,
            entries: HashMap::new(),
        }
    }

    pub fn with_parent(parent: &str) -> Self {
        EnvironmentConfig {
            parent: Some(parent.to_string()),
            entries: HashMap::new(),
        }
    }

    pub fn set_value(&mut self, key: &str, value: ConfigValueEntry) {
        self.entries.insert(key.to_string(), value);
    }

    pub fn get_value(&self, key: &str) -> Option<&ConfigValueEntry> {
        self.entries.get(key)
    }

    pub fn remove_value(&mut self, key: &str) -> Option<ConfigValueEntry> {
        self.entries.remove(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &ConfigValueEntry> {
        self.entries.values()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
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
    fn test_config_file_new() {
        let config = ConfigFile::new("TestProject");
        assert_eq!(config.project_name, "TestProject");
        assert_eq!(config.version, "1.0.0");
        assert!(config.environments.is_empty());
    }

    #[test]
    fn test_environment_config_new() {
        let env = EnvironmentConfig::new();
        assert!(env.parent.is_none());
        assert!(env.entries.is_empty());
    }

    #[test]
    fn test_environment_config_with_parent() {
        let env = EnvironmentConfig::with_parent("development");
        assert_eq!(env.parent, Some("development".to_string()));
    }

    #[test]
    fn test_config_file_add_environment() {
        let mut config = ConfigFile::new("Test");
        config.add_environment("production");
        assert!(config.environments.contains_key("production"));
    }

    #[test]
    fn test_config_value_entry_validate_mismatch() {
        let entry = ConfigValueEntry::new("not_an_int", "integer", false);
        let field = crate::core::schema_model::FieldDefinition {
            key: "test".into(),
            r#type: "integer".into(),
            description: None,
            validation: None,
            is_secret: false,
        };
        assert!(entry.validate(&field).is_err());
    }

    #[test]
    fn test_config_file_clone() {
        let mut environments = HashMap::new();
        let mut entries = HashMap::new();
        entries.insert("K1".into(), ConfigValueEntry::new("V1", "string", false));
        environments.insert(
            "dev".into(),
            EnvironmentConfig {
                parent: None,
                entries,
            },
        );

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
    fn test_environment_set_and_get_value() {
        let mut env = EnvironmentConfig::new();
        env.set_value("KEY1", ConfigValueEntry::new("value1", "string", false));

        assert!(env.get_value("KEY1").is_some());
        assert_eq!(env.get_value("KEY1").unwrap().value, "value1");
    }

    #[test]
    fn test_environment_remove_value() {
        let mut env = EnvironmentConfig::new();
        env.set_value("KEY1", ConfigValueEntry::new("value1", "string", false));

        let removed = env.remove_value("KEY1");
        assert!(removed.is_some());
        assert!(env.get_value("KEY1").is_none());
    }

    #[test]
    fn test_environment_len_and_is_empty() {
        let mut env = EnvironmentConfig::new();
        assert!(env.is_empty());
        assert_eq!(env.len(), 0);

        env.set_value("KEY1", ConfigValueEntry::new("value1", "string", false));
        assert!(!env.is_empty());
        assert_eq!(env.len(), 1);
    }
}
