use crate::core::config_model::ConfigFile;
use crate::core::format_trait::ConfigFormat;
use anyhow::Result;

pub struct TomlFormat;

impl TomlFormat {
    pub fn new() -> Self {
        TomlFormat
    }
}

impl Default for TomlFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigFormat for TomlFormat {
    fn name(&self) -> &str {
        "TOML"
    }

    fn extension(&self) -> &str {
        "toml"
    }

    fn serialize(&self, config: &ConfigFile) -> Result<String> {
        toml::to_string_pretty(config)
            .map_err(|e| anyhow::anyhow!("TOML serialization error: {}", e))
    }

    fn deserialize(&self, data: &str) -> Result<ConfigFile> {
        toml::from_str(data).map_err(|e| anyhow::anyhow!("TOML deserialization error: {}", e))
    }
}

pub fn serialize_toml(config: &ConfigFile) -> Result<String> {
    TomlFormat::new().serialize(config)
}

pub fn deserialize_toml(data: &str) -> Result<ConfigFile> {
    TomlFormat::new().deserialize(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_format_new() {
        let format = TomlFormat::new();
        assert_eq!(format.name(), "TOML");
        assert_eq!(format.extension(), "toml");
    }

    #[test]
    fn test_serialize_toml() {
        let config = ConfigFile::new("TestProject");
        let toml = serialize_toml(&config).unwrap();
        assert!(toml.contains("TestProject"));
    }

    #[test]
    fn test_deserialize_toml() {
        let data = r#"
project_name = "TestProject"
version = "1.0.0"

[environments]
"#;

        let config = deserialize_toml(data).unwrap();
        assert_eq!(config.project_name, "TestProject");
    }
}
