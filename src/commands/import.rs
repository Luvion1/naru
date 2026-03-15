use crate::core::audit;
use crate::core::constants::NARU_DIR;
use crate::core::models::ConfigFile;
use crate::core::persistence;
use anyhow::{anyhow, Result};

pub struct ImportCommand {
    pub file_path: String,
    pub env: String,
}

impl ImportCommand {
    pub fn new(file_path: String, env: String) -> Self {
        ImportCommand { file_path, env }
    }

    pub fn execute(&self) -> Result<()> {
        let file_path_lower = self.file_path.to_lowercase();
        let _config: ConfigFile = if file_path_lower.ends_with(".env") {
            persistence::import_from_env(&self.file_path, &self.env)?
        } else if file_path_lower.ends_with(".yaml") || file_path_lower.ends_with(".yml") {
            persistence::import_from_yaml(&self.file_path, &self.env)?
        } else if file_path_lower.ends_with(".json") {
            persistence::import_from_json(&self.file_path, &self.env)?
        } else {
            return Err(anyhow!(
                "Unsupported file format. Supported formats: .env, .yaml, .yml, .json"
            ));
        };

        let log_path = format!("{}/audit.log", NARU_DIR);
        if let Err(e) = audit::log_action(
            "IMPORT",
            &self.env,
            None,
            None,
            Some(&self.file_path),
            &log_path,
        ) {
            eprintln!("Warning: Failed to log audit entry: {}", e);
        }

        println!(
            "Successfully imported from {} to environment '{}'",
            self.file_path, self.env
        );
        Ok(())
    }
}
