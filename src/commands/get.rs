use crate::core::constants::CONFIG_FILE;
use crate::core::models::ConfigFile;
use crate::core::persistence;
use crate::core::security;
use anyhow::{anyhow, Result};

pub struct GetCommand {
    pub key: String,
    pub env: String,
}

impl GetCommand {
    pub fn new(key: String, env: String) -> Self {
        GetCommand { key, env }
    }

    pub fn execute(&self) -> Result<()> {
        security::validate_environment_name(&self.env)
            .map_err(|e| anyhow!("Invalid environment name: {}", e))?;
        security::validate_config_key(&self.key)
            .map_err(|e| anyhow!("Invalid config key: {}", e))?;

        let mut config: ConfigFile = persistence::load_json(CONFIG_FILE)
            .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

        // Recursive lookup for inheritance
        let mut current_env = self.env.clone();
        let mut searched_envs = Vec::new();

        while let Some(env_config) = config.environments.get(&current_env).cloned() {
            searched_envs.push(current_env.clone());

            // Check if key exists in this environment
            if env_config.entries.contains_key(&self.key) {
                // Found it. Need to use original config (mutable) to decrypt if needed.
                persistence::decrypt_if_needed(&mut config, &current_env, &self.key)
                    .map_err(|e| anyhow!("Failed to decrypt value: {}", e))?;

                // Get the decrypted entry from the mutable config
                if let Some(env_cfg) = config.environments.get(&current_env) {
                    if let Some(entry) = env_cfg.entries.get(&self.key) {
                        if entry.is_secret {
                            println!("********");
                        } else {
                            println!("{}", entry.value);
                        }
                        return Ok(());
                    }
                }
            }

            // Not found, try parent if it exists
            if let Some(parent) = &env_config.parent {
                if searched_envs.contains(parent) {
                    return Err(anyhow!(
                        "Circular inheritance detected: {} -> {}",
                        current_env,
                        parent
                    ));
                }
                current_env = parent.clone();
            } else {
                break;
            }
        }

        Err(anyhow!(
            "Key '{}' not found in environment '{}' (searched: {})",
            self.key,
            self.env,
            searched_envs.join(" -> ")
        ))
    }
}
