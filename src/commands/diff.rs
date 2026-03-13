use crate::core::constants::CONFIG_FILE;
use crate::core::models::ConfigFile;
use crate::core::persistence;
use crate::core::security;
use anyhow::{anyhow, Result};

pub struct DiffCommand {
    pub env1: String,
    pub env2: String,
}

impl DiffCommand {
    pub fn new(env1: String, env2: String) -> Self {
        DiffCommand { env1, env2 }
    }

    pub fn execute(&self) -> Result<()> {
        security::validate_environment_name(&self.env1)
            .map_err(|e| anyhow!("Invalid environment name: {}", e))?;
        security::validate_environment_name(&self.env2)
            .map_err(|e| anyhow!("Invalid environment name: {}", e))?;

        let config: ConfigFile = persistence::load_json(CONFIG_FILE)
            .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

        if !config.environments.contains_key(&self.env1) {
            return Err(anyhow!("Environment '{}' not found.", self.env1));
        }
        if !config.environments.contains_key(&self.env2) {
            return Err(anyhow!("Environment '{}' not found.", self.env2));
        }

        let env1_config = config
            .environments
            .get(&self.env1)
            .ok_or_else(|| anyhow!("Environment '{}' missing from config", self.env1))?;
        let env2_config = config
            .environments
            .get(&self.env2)
            .ok_or_else(|| anyhow!("Environment '{}' missing from config", self.env2))?;

        println!("\nDiff between '{}' and '{}':", self.env1, self.env2);
        println!("{}", "-".repeat(60));

        let mask_secret = |value: &str, is_secret: bool| -> String {
            if is_secret {
                "********".to_string()
            } else {
                value.to_string()
            }
        };

        let mut different_values = false;
        let mut only_in_env1 = false;
        let mut only_in_env2 = false;
        let mut same_values = false;

        for (key, entry1) in &env1_config.entries {
            if let Some(entry2) = env2_config.entries.get(key) {
                if entry1.value != entry2.value {
                    let display_val1 = mask_secret(&entry1.value, entry1.is_secret);
                    let display_val2 = mask_secret(&entry2.value, entry2.is_secret);

                    if !entry1.is_secret && !entry2.is_secret {
                        println!(
                            "  ~ {}: {} -> {} ({} type)",
                            key, display_val1, display_val2, entry1.r#type
                        );
                    } else {
                        println!("  ~ {}: [modified] ({} type)", key, entry1.r#type);
                    }
                    different_values = true;
                }
            }
        }

        for key in env1_config.entries.keys() {
            if !env2_config.entries.contains_key(key) {
                let is_secret = env1_config
                    .entries
                    .get(key)
                    .map(|e| e.is_secret)
                    .unwrap_or(false);
                if !is_secret {
                    println!("  - {} (only in {})", key, self.env1);
                } else {
                    println!("  - {} (only in {}, secret)", key, self.env1);
                }
                only_in_env1 = true;
            }
        }

        for key in env2_config.entries.keys() {
            if !env1_config.entries.contains_key(key) {
                let is_secret = env2_config
                    .entries
                    .get(key)
                    .map(|e| e.is_secret)
                    .unwrap_or(false);
                if !is_secret {
                    println!("  + {} (only in {})", key, self.env2);
                } else {
                    println!("  + {} (only in {}, secret)", key, self.env2);
                }
                only_in_env2 = true;
            }
        }

        for (key, entry1) in &env1_config.entries {
            if let Some(entry2) = env2_config.entries.get(key) {
                if entry1.value == entry2.value {
                    if !entry1.is_secret {
                        println!("  = {}: {} ({})", key, entry1.value, entry1.r#type);
                    } else {
                        println!("  = {}: [secret] ({})", key, entry1.r#type);
                    }
                    same_values = true;
                }
            }
        }

        if !different_values && !only_in_env1 && !only_in_env2 && !same_values {
            println!("  (none)");
        }

        Ok(())
    }
}
