use crate::core::config_model::{ConfigFile, ConfigValueEntry, EnvironmentConfig};
use crate::core::format_trait::ConfigFormat;
use anyhow::Result;
use std::collections::HashMap;

pub struct PropertiesFormat;

impl PropertiesFormat {
    pub fn new() -> Self {
        PropertiesFormat
    }
}

impl Default for PropertiesFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigFormat for PropertiesFormat {
    fn name(&self) -> &str {
        "Properties"
    }

    fn extension(&self) -> &str {
        "properties"
    }

    fn serialize(&self, config: &ConfigFile) -> Result<String> {
        let mut result = String::new();

        for (env_name, env_config) in &config.environments {
            result.push_str(&format!("# Environment: {}\n", env_name));
            for (key, entry) in &env_config.entries {
                result.push_str(&format!("{}.{}={}\n", env_name, key, entry.value));
            }
            result.push('\n');
        }

        Ok(result)
    }

    fn deserialize(&self, data: &str) -> Result<ConfigFile> {
        let mut environments: HashMap<String, EnvironmentConfig> = HashMap::new();

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key_part = line[..pos].trim();
                let value = line[pos + 1..].trim().to_string();

                if let Some((env_name, key)) = key_part.split_once('.') {
                    let env_config = environments
                        .entry(env_name.to_string())
                        .or_insert(EnvironmentConfig::default());

                    env_config.entries.insert(
                        key.to_string(),
                        ConfigValueEntry::new(&value, "string", false),
                    );
                }
            }
        }

        Ok(ConfigFile {
            project_name: "Imported Project".to_string(),
            version: "1.0.0".to_string(),
            environments,
            salt: None,
        })
    }
}

pub fn serialize_properties(config: &ConfigFile) -> Result<String> {
    PropertiesFormat::new().serialize(config)
}

pub fn deserialize_properties(data: &str) -> Result<ConfigFile> {
    PropertiesFormat::new().deserialize(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_properties_format_new() {
        let format = PropertiesFormat::new();
        assert_eq!(format.name(), "Properties");
        assert_eq!(format.extension(), "properties");
    }

    #[test]
    fn test_serialize_properties() {
        let mut config = ConfigFile::new("TestProject");
        let env = config.add_environment("production");
        env.set_value("HOST", ConfigValueEntry::new("localhost", "string", false));

        let properties = serialize_properties(&config).unwrap();
        assert!(properties.contains("production.HOST=localhost"));
    }

    #[test]
    fn test_deserialize_properties() {
        let data = r#"
# Comment
production.HOST=localhost
production.PORT=8080
development.HOST=dev.local
"#;

        let config = deserialize_properties(data).unwrap();
        assert!(config.environments.contains_key("production"));
        assert!(config.environments.contains_key("development"));
    }

    #[test]
    fn test_roundtrip_properties() {
        let mut config = ConfigFile::new("TestProject");
        let env = config.add_environment("prod");
        env.set_value("KEY", ConfigValueEntry::new("value", "string", false));

        let serialized = serialize_properties(&config).unwrap();
        let restored = deserialize_properties(&serialized).unwrap();

        assert!(restored.environments.contains_key("prod"));
    }
}
