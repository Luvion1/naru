use crate::core::models::{ConfigFile, EnvironmentConfig};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Trait untuk format konfigurasi
pub trait ConfigFormat {
    fn serialize(&self, config: &ConfigFile) -> Result<String>;
    fn deserialize(&self, data: &str) -> Result<ConfigFile>;
}

/// Format JSON standar
pub struct JsonFormat;

impl ConfigFormat for JsonFormat {
    fn serialize(&self, config: &ConfigFile) -> Result<String> {
        Ok(serde_json::to_string_pretty(config)?)
    }

    fn deserialize(&self, data: &str) -> Result<ConfigFile> {
        Ok(serde_json::from_str(data)?)
    }
}

/// Format TOML
pub struct TomlFormat;

impl ConfigFormat for TomlFormat {
    fn serialize(&self, config: &ConfigFile) -> Result<String> {
        Ok(toml::to_string_pretty(config)?)
    }

    fn deserialize(&self, data: &str) -> Result<ConfigFile> {
        Ok(toml::from_str(data)?)
    }
}

/// Format Properties (seperti Java properties)
pub struct PropertiesFormat;

impl ConfigFormat for PropertiesFormat {
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

                // Extract environment and key using the first dot as separator
                // For properties format, we expect: <env>.<key>=<value>
                if let Some((env_name, key)) = key_part.split_once('.') {
                    let env_config =
                        environments
                            .entry(env_name.to_string())
                            .or_insert(EnvironmentConfig {
                                entries: HashMap::new(),
                            });

                    env_config.entries.insert(
                        key.to_string(),
                        crate::core::models::ConfigValueEntry {
                            value,
                            r#type: "string".to_string(),
                            is_secret: false,
                            encrypted: false,
                        },
                    );
                }
            }
        }

        Ok(ConfigFile {
            project_name: "Imported Project".to_string(),
            version: "1.0.0".to_string(),
            environments,
            salt: None, // Will be set when saving to actual config
        })
    }
}

/// Fungsi utilitas untuk menyimpan konfigurasi dalam format tertentu
pub fn save_config_as_format<P: AsRef<Path>>(
    path: P,
    config: &ConfigFile,
    format: &dyn ConfigFormat,
) -> Result<()> {
    let serialized = format.serialize(config)?;
    fs::write(path, serialized)?;
    Ok(())
}
