use crate::core::constants;
use crate::core::crypto;
use crate::core::models::{ConfigFile, ConfigValueEntry, EnvironmentConfig, SchemaFile};
use crate::core::persistence;
use crate::core::persistence::PersistenceError;
use crate::core::security;
use anyhow::Result;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use console::style;
use std::collections::HashMap;
use std::env;
use std::fs;

fn get_encryption_key() -> Result<[u8; 32], PersistenceError> {
    let key_str =
        env::var("NARU_ENCRYPTION_KEY").map_err(|_| PersistenceError::MissingEncryptionKey)?;

    let config: ConfigFile =
        persistence::load_json(constants::CONFIG_FILE).map_err(|_| PersistenceError::IoError {
            source: std::io::Error::other("Cannot load config for key derivation"),
        })?;

    let salt_str = config.salt.ok_or_else(|| PersistenceError::IoError {
        source: std::io::Error::other("No salt found in config"),
    })?;

    let salt = SaltString::from_b64(&salt_str).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::other(format!("Invalid salt: {}", e)),
    })?;

    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(key_str.as_bytes(), &salt)
        .map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Key derivation failed: {}", e)),
        })?;

    let hash_output = hash.hash.ok_or_else(|| PersistenceError::IoError {
        source: std::io::Error::other("No hash output generated"),
    })?;

    let hash_bytes = hash_output.as_bytes();
    let mut key = [0u8; 32];
    let len = std::cmp::min(hash_bytes.len(), 32);
    key[..len].copy_from_slice(&hash_bytes[..len]);
    Ok(key)
}

pub fn batch_set(file: &str, env: &str) -> Result<(), PersistenceError> {
    let sanitized_path =
        security::sanitize_file_path(file).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    security::validate_environment_name(env).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
    })?;

    security::check_file_size(&sanitized_path, 1024 * 1024).map_err(|e| {
        PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        }
    })?;

    let content = fs::read_to_string(&sanitized_path)?;
    let schema: SchemaFile = persistence::load_json(constants::SCHEMA_FILE).unwrap_or(SchemaFile {
        version: "1.0".to_string(),
        fields: vec![],
    });

    let schema_defined = !schema.fields.is_empty();
    let mut config: ConfigFile = persistence::load_json(constants::CONFIG_FILE)?;

    if !config.environments.contains_key(env) {
        config.environments.insert(
            env.to_string(),
            EnvironmentConfig {
                parent: None,
                entries: HashMap::new(),
            },
        );
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let mut value = line[pos + 1..].trim().to_string();

            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = value[1..value.len() - 1].to_string();
            }

            if schema_defined && !schema.fields.iter().any(|f| f.key == key) {
                eprintln!(
                    "{} Line {}: Key '{}' not in schema",
                    style("⚠").yellow(),
                    line_num + 1,
                    key
                );
                error_count += 1;
                continue;
            }

            let mut is_secret = false;
            let mut target_type = "string".to_string();

            if let Some(field) = schema.fields.iter().find(|f| f.key == key) {
                target_type = field.r#type.clone();
                is_secret = field.is_secret;

                if let Err(e) = crate::core::validation::validate_value(&value, field) {
                    eprintln!(
                        "{} Line {}: {} - {}",
                        style("✗").red(),
                        line_num + 1,
                        key,
                        e
                    );
                    error_count += 1;
                    continue;
                }
            }

            let entry = ConfigValueEntry {
                value: value.clone(),
                r#type: target_type,
                is_secret,
                encrypted: false,
            };

            if let Some(env_config) = config.environments.get_mut(env) {
                env_config.entries.insert(key.to_string(), entry);
                success_count += 1;
            }
        } else {
            eprintln!(
                "{} Line {}: Invalid format (expected key=value)",
                style("⚠").yellow(),
                line_num + 1
            );
            error_count += 1;
        }
    }

    if success_count > 0 {
        persistence::save_json(constants::CONFIG_FILE, &config)?;
        println!(
            "{} Batch set complete: {} succeeded, {} failed",
            style("✓").green(),
            style(success_count).green(),
            style(error_count).red()
        );
    } else {
        eprintln!("{} No valid entries found in file", style("✗").red());
    }

    Ok(())
}

pub fn batch_get(keys: &[String], env: &str) -> Result<(), PersistenceError> {
    security::validate_environment_name(env).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
    })?;

    let config: ConfigFile = persistence::load_json(constants::CONFIG_FILE)?;

    let env_config = config
        .environments
        .get(env)
        .ok_or_else(|| PersistenceError::IoError {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Environment '{}' not found", env),
            ),
        })?;

    let mut found_count = 0;
    let mut missing_count = 0;

    for key in keys {
        if let Some(entry) = env_config.entries.get(key) {
            if entry.is_secret && entry.encrypted {
                let encryption_key = get_encryption_key()?;
                match crypto::decrypt_data(&entry.value, &encryption_key) {
                    Ok(decrypted) => {
                        println!("{}={}", key, decrypted);
                        found_count += 1;
                    }
                    Err(_) => {
                        eprintln!("{} Failed to decrypt {}", style("✗").red(), key);
                        missing_count += 1;
                    }
                }
            } else {
                println!("{}={}", key, entry.value);
                found_count += 1;
            }
        } else {
            eprintln!("{} Key '{}' not found in {}", style("✗").red(), key, env);
            missing_count += 1;
        }
    }

    if missing_count > 0 && found_count == 0 {
        return Err(PersistenceError::IoError {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "None of the requested keys were found",
            ),
        });
    }

    Ok(())
}

pub fn batch_get_all(env: &str) -> Result<(), PersistenceError> {
    security::validate_environment_name(env).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
    })?;

    let config: ConfigFile = persistence::load_json(constants::CONFIG_FILE)?;

    let env_config = config
        .environments
        .get(env)
        .ok_or_else(|| PersistenceError::IoError {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Environment '{}' not found", env),
            ),
        })?;

    if env_config.entries.is_empty() {
        println!("{} No values in environment '{}'", style("ℹ").blue(), env);
        return Ok(());
    }

    let encryption_key = if env_config
        .entries
        .values()
        .any(|e| e.is_secret && e.encrypted)
    {
        Some(get_encryption_key()?)
    } else {
        None
    };

    for (key, entry) in &env_config.entries {
        let value = if entry.is_secret && entry.encrypted {
            if let Some(key) = &encryption_key {
                crypto::decrypt_data(&entry.value, key)
                    .unwrap_or_else(|_| "[decryption failed]".to_string())
            } else {
                "[encrypted]".to_string()
            }
        } else {
            entry.value.clone()
        };

        println!("{}={}", key, value);
    }

    Ok(())
}
