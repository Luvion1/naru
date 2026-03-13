use crate::core::constants::CONFIG_FILE;
use crate::core::models::ConfigFile;
use crate::core::persistence;
use crate::core::security;
use anyhow::Result;

pub struct ExportCommand {
    pub file_path: String,
    pub env: String,
    pub format: String,
}

impl ExportCommand {
    pub fn new(file_path: String, env: String, format: String) -> Self {
        ExportCommand {
            file_path,
            env,
            format,
        }
    }

    pub fn execute(&self) -> Result<()> {
        security::validate_environment_name(&self.env)
            .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;

        let config: ConfigFile = persistence::load_json(CONFIG_FILE)
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

        match self.format.as_str() {
            "env" => {
                persistence::export_to_env(&config, &self.env, &self.file_path)?;
                println!(
                    "Successfully exported environment '{}' to {} in .env format",
                    self.env, self.file_path
                );
            }
            "yaml" | "yml" => {
                persistence::export_to_yaml(&config, &self.env, &self.file_path)?;
                println!(
                    "Successfully exported environment '{}' to {} in YAML format",
                    self.env, self.file_path
                );
            }
            _ => {
                eprintln!("Unsupported format: {}. Use 'env' or 'yaml'.", self.format);
            }
        }
        Ok(())
    }
}
