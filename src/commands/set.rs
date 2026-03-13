use crate::core::audit;
use crate::core::constants::{NARU_DIR, SCHEMA_FILE};
use crate::core::models::{ConfigValueEntry, SchemaFile};
use crate::core::persistence;
use crate::core::security;
use anyhow::{anyhow, Result};

pub struct SetCommand {
    pub key: String,
    pub value: String,
    pub env: String,
    pub secret: bool,
}

impl SetCommand {
    pub fn new(key: String, value: String, env: String, secret: bool) -> Self {
        SetCommand {
            key,
            value,
            env,
            secret,
        }
    }

    pub fn execute(&self) -> Result<()> {
        const MAX_VALUE_LENGTH: usize = 1_000_000;
        if self.value.len() > MAX_VALUE_LENGTH {
            return Err(anyhow!("Value too long (max {} bytes)", MAX_VALUE_LENGTH));
        }

        security::validate_environment_name(&self.env)
            .map_err(|e| anyhow!("Invalid environment name: {}", e))?;
        security::validate_config_key(&self.key)
            .map_err(|e| anyhow!("Invalid config key: {}", e))?;

        let schema: SchemaFile = persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| {
            eprintln!("Warning: Could not load schema file, using default schema");
            SchemaFile {
                version: "1.0".to_string(),
                fields: vec![],
            }
        });

        let mut target_type = "string".to_string();
        let mut is_secret = self.secret;

        if let Some(field) = schema.fields.iter().find(|f| f.key == self.key) {
            target_type = field.r#type.clone();
            is_secret = self.secret || field.is_secret;

            let entry_to_validate = ConfigValueEntry::new(&self.value, &target_type, is_secret);
            entry_to_validate
                .validate(field)
                .map_err(|e| anyhow!("Validation error: {}", e))?;
        }

        let value_clone = self.value.clone();
        let key_clone = self.key.clone();
        let env_clone = self.env.clone();
        let target_type_clone = target_type.clone();
        let is_secret_clone = is_secret;

        let mut old_value: Option<String> = None;
        let mut is_previously_secret = false;

        persistence::atomic_update_config(|config| {
            let env_config = config.environments.get_mut(&env_clone).ok_or_else(|| {
                persistence::PersistenceError::ValidationError(format!(
                    "Environment '{}' not found.",
                    env_clone
                ))
            })?;

            let old_entry = env_config.entries.get(&key_clone);
            old_value = old_entry.map(|e| e.value.clone());
            is_previously_secret = old_entry.is_some_and(|e| e.is_secret);

            env_config.entries.insert(
                key_clone.clone(),
                ConfigValueEntry {
                    value: value_clone.clone(),
                    r#type: target_type_clone.clone(),
                    is_secret: is_secret_clone,
                    encrypted: false,
                },
            );

            if is_secret_clone {
                persistence::encrypt_if_needed(config, &env_clone, &key_clone).map_err(|e| {
                    persistence::PersistenceError::IoError {
                        source: std::io::Error::other(e.to_string()),
                    }
                })?;
            }

            Ok(())
        })
        .map_err(|e| anyhow!("Failed to update config: {}. Run 'naru init' first.", e))?;

        let log_path = format!("{}/audit.log", NARU_DIR);
        let log_value = if is_secret { "********" } else { &self.value };
        let log_old = if is_secret || is_previously_secret {
            if old_value.is_some() {
                Some("********")
            } else {
                None
            }
        } else {
            old_value.as_deref()
        };

        if let Err(e) = audit::log_action(
            "SET",
            &self.env,
            Some(&self.key),
            log_old,
            Some(log_value),
            &log_path,
        ) {
            eprintln!("Warning: Failed to log audit entry: {}", e);
        }

        println!("Set {} in environment '{}'", self.key, self.env);
        Ok(())
    }
}
