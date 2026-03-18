use crate::core::config_model::ConfigFile;
use crate::core::format_trait::ConfigFormat;
use anyhow::Result;

pub struct JsonFormat;

impl JsonFormat {
    pub fn new() -> Self {
        JsonFormat
    }
}

impl Default for JsonFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigFormat for JsonFormat {
    fn name(&self) -> &str {
        "JSON"
    }

    fn extension(&self) -> &str {
        "json"
    }

    fn serialize(&self, config: &ConfigFile) -> Result<String> {
        serde_json::to_string_pretty(config)
            .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
    }

    fn deserialize(&self, data: &str) -> Result<ConfigFile> {
        serde_json::from_str(data).map_err(|e| anyhow::anyhow!("JSON deserialization error: {}", e))
    }
}

pub fn serialize_json(config: &ConfigFile) -> Result<String> {
    JsonFormat::new().serialize(config)
}

pub fn deserialize_json(data: &str) -> Result<ConfigFile> {
    JsonFormat::new().deserialize(data)
}

pub fn serialize_json_pretty(config: &ConfigFile) -> Result<String> {
    serde_json::to_string_pretty(config)
        .map_err(|e| anyhow::anyhow!("JSON pretty print error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_json_format_new() {
        let format = JsonFormat::new();
        assert_eq!(format.name(), "JSON");
        assert_eq!(format.extension(), "json");
    }

    #[test]
    fn test_serialize_json() {
        let config = ConfigFile::new("TestProject");
        let json = serialize_json(&config).unwrap();
        assert!(json.contains("TestProject"));
    }

    #[test]
    fn test_deserialize_json() {
        let data = r#"{
            "project_name": "TestProject",
            "version": "1.0.0",
            "environments": {}
        }"#;

        let config = deserialize_json(data).unwrap();
        assert_eq!(config.project_name, "TestProject");
    }

    #[test]
    fn test_roundtrip_json() {
        let mut config = ConfigFile::new("RoundTripTest");
        let env = config.add_environment("production");
        env.set_value(
            "HOST",
            crate::core::config_model::ConfigValueEntry::new("localhost", "string", false),
        );

        let json = serialize_json(&config).unwrap();
        let restored = deserialize_json(&json).unwrap();

        assert_eq!(restored.project_name, config.project_name);
        assert!(restored.environments.contains_key("production"));
    }
}
