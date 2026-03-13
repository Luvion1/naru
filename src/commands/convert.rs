use crate::core::formats::{ConfigFormat, JsonFormat, PropertiesFormat, TomlFormat};
use crate::core::models::ConfigFile;
use crate::core::security;
use anyhow::{anyhow, Result};

pub struct ConvertCommand {
    pub from_file: String,
    pub to_file: String,
    pub from_format: String,
    pub to_format: String,
}

impl ConvertCommand {
    pub fn new(from_file: String, to_file: String, from_format: String, to_format: String) -> Self {
        ConvertCommand {
            from_file,
            to_file,
            from_format,
            to_format,
        }
    }

    pub fn execute(&self) -> Result<()> {
        let sanitized_from_file = security::sanitize_file_path(&self.from_file)
            .map_err(|e| anyhow!("Invalid source file path: {}", e))?;
        let sanitized_to_file = security::sanitize_file_path(&self.to_file)
            .map_err(|e| anyhow!("Invalid destination file path: {}", e))?;

        security::check_file_size(&sanitized_from_file, 10 * 1024 * 1024)
            .map_err(|e| anyhow!("Source file too large: {}", e))?;

        let config: ConfigFile = match self.from_format.as_str() {
            "json" => {
                let content = std::fs::read_to_string(
                    sanitized_from_file
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid source file path"))?,
                )?;
                if content.len() > 10 * 1024 * 1024 {
                    return Err(anyhow!("Source file content exceeds 10MB limit"));
                }
                let format = JsonFormat;
                format.deserialize(&content)?
            }
            "toml" => {
                let content = std::fs::read_to_string(
                    sanitized_from_file
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid source file path"))?,
                )?;
                if content.len() > 10 * 1024 * 1024 {
                    return Err(anyhow!("Source file content exceeds 10MB limit"));
                }
                let format = TomlFormat;
                format.deserialize(&content)?
            }
            "properties" => {
                let content = std::fs::read_to_string(
                    sanitized_from_file
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid source file path"))?,
                )?;
                if content.len() > 10 * 1024 * 1024 {
                    return Err(anyhow!("Source file content exceeds 10MB limit"));
                }
                let format = PropertiesFormat;
                format.deserialize(&content)?
            }
            "yaml" | "yml" => {
                let content = std::fs::read_to_string(
                    sanitized_from_file
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid source file path"))?,
                )?;
                if content.len() > 10 * 1024 * 1024 {
                    return Err(anyhow!("Source file content exceeds 10MB limit"));
                }
                serde_yaml::from_str(&content)?
            }
            _ => {
                eprintln!("Unsupported source format: {}", self.from_format);
                return Ok(());
            }
        };

        match self.to_format.as_str() {
            "json" => {
                let format = JsonFormat;
                crate::core::formats::save_config_as_format(
                    sanitized_to_file
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid destination file path"))?,
                    &config,
                    &format,
                )?;
            }
            "toml" => {
                let format = TomlFormat;
                crate::core::formats::save_config_as_format(
                    sanitized_to_file
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid destination file path"))?,
                    &config,
                    &format,
                )?;
            }
            "properties" => {
                let format = PropertiesFormat;
                crate::core::formats::save_config_as_format(
                    sanitized_to_file
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid destination file path"))?,
                    &config,
                    &format,
                )?;
            }
            "yaml" | "yml" => {
                let yaml_content = serde_yaml::to_string(&config)?;
                std::fs::write(
                    sanitized_to_file
                        .to_str()
                        .ok_or_else(|| anyhow!("Invalid destination file path"))?,
                    yaml_content,
                )?;
            }
            _ => {
                eprintln!("Unsupported destination format: {}", self.to_format);
                return Ok(());
            }
        };

        println!(
            "Configuration converted from {} to {} and saved to {}",
            self.from_format, self.to_format, self.to_file
        );
        Ok(())
    }
}
