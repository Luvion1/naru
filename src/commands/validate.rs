use crate::core::constants::SCHEMA_FILE;
use crate::core::models::{ConfigFile, SchemaFile};
use crate::core::persistence;
use anyhow::{anyhow, Result};

pub struct ValidateCommand;

impl ValidateCommand {
    pub fn new() -> Self {
        ValidateCommand
    }

    pub fn execute(&self) -> Result<()> {
        let config: ConfigFile = persistence::atomic_read_config(|c| c.clone())
            .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

        let schema: SchemaFile =
            persistence::atomic_read_json(SCHEMA_FILE, |s: &SchemaFile| s.clone())
                .map_err(|e| anyhow!("Failed to load schema: {}", e))?;

        println!("\nValidating configuration against schema...");
        let mut errors = 0;

        for (env_name, env_config) in &config.environments {
            println!("  Environment: {}", env_name);
            for (key, entry) in &env_config.entries {
                if let Some(field) = schema.fields.iter().find(|f| f.key == *key) {
                    if let Err(e) = entry.validate(field) {
                        println!("  [ERROR] {}: {}", key, e);
                        errors += 1;
                    }
                }
            }
        }

        if errors == 0 {
            println!("✅ All configurations are valid!");
        } else {
            println!("\n❌ Found {} validation errors.", errors);
            return Err(anyhow!("Validation failed."));
        }
        Ok(())
    }
}

impl Default for ValidateCommand {
    fn default() -> Self {
        Self::new()
    }
}
