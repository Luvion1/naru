use anyhow::Result;
use clap::Parser;

mod cli;
mod core;

use cli::parser::{Cli, Commands};
use core::constants::*;
use core::models::*;
use core::persistence;
use std::path::Path;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            println!("Naru version {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Init => {
            if Path::new(NARU_DIR).exists() {
                println!("Project already initialized.");
            } else {
                persistence::init_project()?;
                println!("Project initialized successfully.");
            }
        }
        Commands::Set { kv, env, secret } => {
            let (key, value) = kv
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Invalid format. Use key=value"))?;

            // Validate environment name and key
            crate::core::security::validate_environment_name(&env)
                .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;
            crate::core::security::validate_config_key(key)
                .map_err(|e| anyhow::anyhow!("Invalid config key: {}", e))?;

            let mut config: ConfigFile = persistence::load_json(CONFIG_FILE).map_err(|e| {
                anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
            })?;

            let schema: SchemaFile = persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| {
                eprintln!("Warning: Could not load schema file, using default schema");
                SchemaFile {
                    version: "1.0".to_string(),
                    fields: vec![],
                }
            });

            let mut target_type = "string".to_string();
            let mut is_secret = secret;

            // Apply schema validation if field exists
            if let Some(field) = schema.fields.iter().find(|f| f.key == key) {
                target_type = field.r#type.clone();
                is_secret = secret || field.is_secret; // Use schema if it's secret, or flag

                let entry_to_validate = ConfigValueEntry::new(value, &target_type, is_secret);
                entry_to_validate
                    .validate(field)
                    .map_err(|e| anyhow::anyhow!("Validation error: {}", e))?;
            }

            let env_config = config
                .environments
                .get_mut(&env)
                .ok_or_else(|| anyhow::anyhow!("Environment '{}' not found.", env))?;

            // Get the old value if it exists
            let old_entry = env_config.entries.get(key);
            let old_value = old_entry.map(|e| e.value.clone());
            let is_previously_secret = old_entry.is_some_and(|e| e.is_secret);

            // Insert the new value
            env_config.entries.insert(
                key.to_string(),
                ConfigValueEntry {
                    value: value.to_string(),
                    r#type: target_type,
                    is_secret,
                    encrypted: false, // Will be set to true during encryption if needed
                },
            );

            // Encrypt the value if it's marked as secret
            if is_secret {
                persistence::encrypt_if_needed(&mut config, &env, key)
                    .map_err(|e| anyhow::anyhow!("Failed to encrypt value: {}", e))?;
            }

            persistence::save_json(CONFIG_FILE, &config)?;

            // Log the change
            let log_path = format!("{}/audit.log", NARU_DIR);
            let log_value = if is_secret { "********" } else { value };
            let log_old = if is_secret || is_previously_secret {
                if old_value.is_some() {
                    Some("********")
                } else {
                    None
                }
            } else {
                old_value.as_deref()
            };

            if let Err(e) = crate::core::audit::log_action(
                "SET",
                &env,
                Some(key),
                log_old,
                Some(log_value),
                &log_path,
            ) {
                eprintln!("Warning: Failed to log audit entry: {}", e);
            }

            println!("Set {} in environment '{}'", key, env);
        }
        Commands::Get { key, env } => {
            // Validate environment name and key
            crate::core::security::validate_environment_name(&env)
                .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;
            crate::core::security::validate_config_key(&key)
                .map_err(|e| anyhow::anyhow!("Invalid config key: {}", e))?;

            let mut config: ConfigFile = persistence::load_json(CONFIG_FILE).map_err(|e| {
                anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
            })?;

            // Decrypt the value if it's encrypted
            persistence::decrypt_if_needed(&mut config, &env, &key)
                .map_err(|e| anyhow::anyhow!("Failed to decrypt value: {}", e))?;

            let env_config = config
                .environments
                .get(&env)
                .ok_or_else(|| anyhow::anyhow!("Environment '{}' not found.", env))?;

            let entry = env_config
                .entries
                .get(&key)
                .ok_or_else(|| anyhow::anyhow!("Key '{}' not found.", key))?;

            println!("{}", entry.value);
        }
        Commands::List { env } => {
            use console::{Emoji, style};
            // Validate environment name
            crate::core::security::validate_environment_name(&env)
                .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;

            let config: ConfigFile = persistence::load_json(CONFIG_FILE).map_err(|e| {
                anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
            })?;

            let schema: SchemaFile =
                persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| SchemaFile {
                    version: "1.0".to_string(),
                    fields: vec![],
                });

            let env_config = config
                .environments
                .get(&env)
                .ok_or_else(|| anyhow::anyhow!("Environment '{}' not found.", env))?;

            println!(
                "\n{} {}",
                Emoji("📁", ""),
                style(format!("Environment: {}", env)).bold().cyan()
            );
            println!("{}", style("=".repeat(60)).dim());

            let mut all_keys: std::collections::BTreeSet<String> =
                env_config.entries.keys().cloned().collect();
            for field in &schema.fields {
                all_keys.insert(field.key.clone());
            }

            if all_keys.is_empty() {
                println!("  (empty)");
            } else {
                for key in all_keys {
                    let entry = env_config.entries.get(&key);
                    let field_def = schema.fields.iter().find(|f| f.key == key);

                    let key_style = if entry.is_some() {
                        style(&key).bold()
                    } else {
                        style(&key).dim()
                    };
                    let secret_icon = if entry.is_some_and(|e| e.is_secret) {
                        Emoji("🔒 ", "")
                    } else {
                        Emoji("   ", "")
                    };

                    let value_str = match entry {
                        Some(e) => {
                            if e.is_secret {
                                style("********").dim().italic().to_string()
                            } else {
                                style(&e.value).green().to_string()
                            }
                        }
                        None => style("MISSING").red().italic().to_string(),
                    };

                    let type_str = match field_def {
                        Some(f) => style(format!("[{}]", f.r#type)).dim().to_string(),
                        _ => "".to_string(),
                    };

                    println!(
                        "  {} {:<20} = {:<15} {}",
                        secret_icon, key_style, value_str, type_str
                    );

                    if let Some(f) = field_def
                        && let Some(desc) = &f.description
                    {
                        println!("     {}", style(format!("└─ {}", desc)).dim().italic());
                    }
                }
            }
            println!("{}", style("=".repeat(60)).dim());
        }
        Commands::Import { file_path, env } => {
            let file_path_lower = file_path.to_lowercase();
            let config = if file_path_lower.ends_with(".env") {
                persistence::import_from_env(&file_path, &env)?
            } else if file_path_lower.ends_with(".yaml") || file_path_lower.ends_with(".yml") {
                persistence::import_from_yaml(&file_path, &env)?
            } else if file_path_lower.ends_with(".json") {
                persistence::import_from_json(&file_path, &env)?
            } else {
                return Err(anyhow::anyhow!(
                    "Unsupported file format. Supported formats: .env, .yaml, .yml, .json"
                ));
            };

            persistence::save_json(CONFIG_FILE, &config)?;

            // Audit Log
            let log_path = format!("{}/audit.log", NARU_DIR);
            if let Err(e) = crate::core::audit::log_action(
                "IMPORT",
                &env,
                None,
                None,
                Some(&file_path),
                &log_path,
            ) {
                eprintln!("Warning: Failed to log audit entry: {}", e);
            }

            println!(
                "Successfully imported from {} to environment '{}'",
                file_path, env
            );
        }
        Commands::Export {
            file_path,
            env,
            format,
        } => {
            // Validate environment name
            crate::core::security::validate_environment_name(&env)
                .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;

            let config: ConfigFile = persistence::load_json(CONFIG_FILE).map_err(|e| {
                anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
            })?;

            match format.as_str() {
                "env" => {
                    persistence::export_to_env(&config, &env, &file_path)?;
                    println!(
                        "Successfully exported environment '{}' to {} in .env format",
                        env, file_path
                    );
                }
                "yaml" | "yml" => {
                    persistence::export_to_yaml(&config, &env, &file_path)?;
                    println!(
                        "Successfully exported environment '{}' to {} in YAML format",
                        env, file_path
                    );
                }
                _ => {
                    eprintln!("Unsupported format: {}. Use 'env' or 'yaml'.", format);
                }
            }
        }
        Commands::Schema { action } => {
            let log_path = format!("{}/audit.log", NARU_DIR);
            match action {
                cli::parser::SchemaAction::Add {
                    key,
                    r#type,
                    description,
                    secret,
                } => {
                    let field_def = if let Some(key) = key {
                        // Validate key
                        crate::core::security::validate_config_key(&key)
                            .map_err(|e| anyhow::anyhow!("Invalid config key: {}", e))?;

                        FieldDefinition {
                            key: key.clone(),
                            r#type: r#type.clone(),
                            description: description.clone(),
                            validation: None,
                            is_secret: secret,
                        }
                    } else {
                        cli::interactive::prompt_add_field()?
                    };

                    let key_name = field_def.key.clone();
                    core::schema::add_field(field_def)?;

                    // Audit Log
                    if let Err(e) = crate::core::audit::log_action(
                        "SCHEMA_ADD",
                        "schema",
                        Some(&key_name),
                        None,
                        None,
                        &log_path,
                    ) {
                        eprintln!("Warning: Failed to log audit entry: {}", e);
                    }
                }
                cli::parser::SchemaAction::Remove { key } => {
                    let key = if let Some(key) = key {
                        key
                    } else {
                        let schema: SchemaFile = persistence::load_json(SCHEMA_FILE)
                            .unwrap_or_else(|_| SchemaFile {
                                version: "1.0".to_string(),
                                fields: vec![],
                            });
                        cli::interactive::prompt_select_field(&schema)?
                    };

                    core::schema::remove_field(&key)?;

                    // Audit Log
                    if let Err(e) = crate::core::audit::log_action(
                        "SCHEMA_REMOVE",
                        "schema",
                        Some(&key),
                        None,
                        None,
                        &log_path,
                    ) {
                        eprintln!("Warning: Failed to log audit entry: {}", e);
                    }
                }
                cli::parser::SchemaAction::View => {
                    let schema: SchemaFile =
                        persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| {
                            eprintln!("Warning: Could not load schema file, using default schema");
                            SchemaFile {
                                version: "1.0".to_string(),
                                fields: vec![],
                            }
                        });

                    println!("Schema version: {}", schema.version);
                    if schema.fields.is_empty() {
                        println!("No fields defined in schema.");
                    } else {
                        println!("Fields:");
                        for field in &schema.fields {
                            println!(
                                "  - {}: {} ({})",
                                field.key,
                                field.r#type,
                                field.description.as_deref().unwrap_or("no description")
                            );
                        }
                    }
                }
                cli::parser::SchemaAction::Edit { key } => {
                    let schema: SchemaFile = persistence::load_json(SCHEMA_FILE)
                        .map_err(|e| anyhow::anyhow!("Failed to load schema: {}", e))?;

                    let key = if let Some(key) = key {
                        key
                    } else {
                        cli::interactive::prompt_select_field(&schema)?
                    };

                    // Validate key
                    crate::core::security::validate_config_key(&key)
                        .map_err(|e| anyhow::anyhow!("Invalid field key: {}", e))?;

                    let existing_field =
                        schema.fields.iter().find(|f| f.key == key).ok_or_else(|| {
                            anyhow::anyhow!("Field '{}' not found in schema", key)
                        })?;

                    let updated_field = cli::interactive::prompt_edit_field(existing_field)?;
                    let key_name = updated_field.key.clone();
                    core::schema::update_field(&key, updated_field)?;

                    // Audit Log
                    if let Err(e) = crate::core::audit::log_action(
                        "SCHEMA_EDIT",
                        "schema",
                        Some(&key_name),
                        None,
                        None,
                        &log_path,
                    ) {
                        eprintln!("Warning: Failed to log audit entry: {}", e);
                    }
                }
            }
        }
        Commands::Env { action } => {
            match action {
                cli::parser::EnvAction::Add { name } => {
                    // Validate environment name
                    crate::core::security::validate_environment_name(&name)
                        .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;

                    let mut config: ConfigFile =
                        persistence::load_json(CONFIG_FILE).map_err(|e| {
                            anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
                        })?;

                    // Check if environment already exists
                    if config.environments.contains_key(&name) {
                        return Err(anyhow::anyhow!("Environment '{}' already exists.", name));
                    }

                    // Add new environment
                    config.environments.insert(
                        name.clone(),
                        EnvironmentConfig {
                            entries: std::collections::HashMap::new(),
                        },
                    );
                    persistence::save_json(CONFIG_FILE, &config)?;

                    // Audit Log
                    let log_path = format!("{}/audit.log", NARU_DIR);
                    if let Err(e) = crate::core::audit::log_action(
                        "ENV_ADD", &name, None, None, None, &log_path,
                    ) {
                        eprintln!("Warning: Failed to log audit entry: {}", e);
                    }

                    println!("Added environment '{}'", name);
                }
                cli::parser::EnvAction::Remove { name } => {
                    // Validate environment name
                    crate::core::security::validate_environment_name(&name)
                        .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;

                    let mut config: ConfigFile =
                        persistence::load_json(CONFIG_FILE).map_err(|e| {
                            anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
                        })?;

                    // Check if trying to remove default environments
                    if ["development", "staging", "production"].contains(&name.as_str()) {
                        return Err(anyhow::anyhow!(
                            "Cannot remove default environment: {}",
                            name
                        ));
                    }

                    if config.environments.remove(&name).is_none() {
                        return Err(anyhow::anyhow!("Environment '{}' not found.", name));
                    }

                    persistence::save_json(CONFIG_FILE, &config)?;

                    // Audit Log
                    let log_path = format!("{}/audit.log", NARU_DIR);
                    if let Err(e) = crate::core::audit::log_action(
                        "ENV_REMOVE",
                        &name,
                        None,
                        None,
                        None,
                        &log_path,
                    ) {
                        eprintln!("Warning: Failed to log audit entry: {}", e);
                    }

                    println!("Removed environment '{}'", name);
                }
                cli::parser::EnvAction::List => {
                    let config: ConfigFile = persistence::load_json(CONFIG_FILE).map_err(|e| {
                        anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
                    })?;

                    println!("Available environments:");
                    for name in config.environments.keys() {
                        println!("  - {}", name);
                    }
                }
            }
        }
        Commands::Backup { action } => {
            match action {
                cli::parser::BackupAction::Create { file_path } => {
                    // Sanitize file path
                    let sanitized_path = crate::core::security::sanitize_file_path(&file_path)
                        .map_err(|e| anyhow::anyhow!("Invalid file path: {}", e))?;

                    let config: ConfigFile = persistence::load_json(CONFIG_FILE).map_err(|e| {
                        anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
                    })?;

                    let schema: SchemaFile =
                        persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| {
                            eprintln!("Warning: Could not load schema file, using default schema");
                            SchemaFile {
                                version: "1.0".to_string(),
                                fields: vec![],
                            }
                        });

                    // Create a backup object with both config and schema
                    let backup_data = BackupData { config, schema };

                    // Serialize to JSON and save to file
                    let json_data = serde_json::to_string_pretty(&backup_data)?;
                    std::fs::write(
                        sanitized_path
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?,
                        json_data,
                    )?;
                    println!("Backup created successfully at: {}", file_path);
                }
                cli::parser::BackupAction::Restore { file_path } => {
                    // Sanitize file path to prevent directory traversal
                    let sanitized_path = crate::core::security::sanitize_file_path(&file_path)
                        .map_err(|e| anyhow::anyhow!("Invalid file path: {}", e))?;

                    // Read the backup file
                    let content = std::fs::read_to_string(
                        sanitized_path
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?,
                    )?;
                    let backup_data: BackupData = serde_json::from_str(&content)?;

                    // Save the restored config and schema
                    persistence::save_json(CONFIG_FILE, &backup_data.config)?;
                    persistence::save_json(SCHEMA_FILE, &backup_data.schema)?;

                    // Audit Log
                    let log_path = format!("{}/audit.log", NARU_DIR);
                    if let Err(e) = crate::core::audit::log_action(
                        "BACKUP_RESTORE",
                        "all",
                        None,
                        None,
                        Some(&file_path),
                        &log_path,
                    ) {
                        eprintln!("Warning: Failed to log audit entry: {}", e);
                    }

                    println!("Configuration restored successfully from: {}", file_path);
                }
            }
        }
        Commands::Diff { env1, env2 } => {
            // Validate environment names
            crate::core::security::validate_environment_name(&env1)
                .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;
            crate::core::security::validate_environment_name(&env2)
                .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;

            let config: ConfigFile = persistence::load_json(CONFIG_FILE).map_err(|e| {
                anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
            })?;

            // Check if both environments exist
            if !config.environments.contains_key(&env1) {
                return Err(anyhow::anyhow!("Environment '{}' not found.", env1));
            }
            if !config.environments.contains_key(&env2) {
                return Err(anyhow::anyhow!("Environment '{}' not found.", env2));
            }

            let env1_config = config.environments.get(&env1).unwrap();
            let env2_config = config.environments.get(&env2).unwrap();

            println!("\nDiff between '{}' and '{}':", env1, env2);
            println!("{}", "-".repeat(60));

            let mut different_values = false;
            let mut only_in_env1 = false;
            let mut only_in_env2 = false;
            let mut same_values = false;

            // Check values in env1 and compare with env2
            for (key, entry1) in &env1_config.entries {
                if let Some(entry2) = env2_config.entries.get(key)
                    && entry1.value != entry2.value
                {
                    println!(
                        "  ~ {}: {} -> {} ({} type)",
                        key, entry1.value, entry2.value, entry1.r#type
                    );
                    different_values = true;
                }
            }

            // Check values only in env1
            for key in env1_config.entries.keys() {
                if !env2_config.entries.contains_key(key) {
                    println!("  - {} (only in {})", key, env1);
                    only_in_env1 = true;
                }
            }

            // Check values only in env2
            for key in env2_config.entries.keys() {
                if !env1_config.entries.contains_key(key) {
                    println!("  + {} (only in {})", key, env2);
                    only_in_env2 = true;
                }
            }

            // Check same values
            for (key, entry1) in &env1_config.entries {
                if let Some(entry2) = env2_config.entries.get(key)
                    && entry1.value == entry2.value
                {
                    println!("  = {}: {} ({})", key, entry1.value, entry1.r#type);
                    same_values = true;
                }
            }

            if !different_values && !only_in_env1 && !only_in_env2 && !same_values {
                println!("  (none)");
            }
        }
        Commands::Convert {
            from_file,
            to_file,
            from_format,
            to_format,
        } => {
            use crate::core::formats::{ConfigFormat, JsonFormat, PropertiesFormat, TomlFormat};

            // Sanitize file paths to prevent directory traversal
            let sanitized_from_file = crate::core::security::sanitize_file_path(&from_file)
                .map_err(|e| anyhow::anyhow!("Invalid source file path: {}", e))?;
            let sanitized_to_file = crate::core::security::sanitize_file_path(&to_file)
                .map_err(|e| anyhow::anyhow!("Invalid destination file path: {}", e))?;

            // Determine source format and load config
            let config = match from_format.as_str() {
                "json" => {
                    let content = std::fs::read_to_string(
                        sanitized_from_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid source file path"))?,
                    )?;
                    let format = JsonFormat;
                    format.deserialize(&content)?
                }
                "toml" => {
                    let content = std::fs::read_to_string(
                        sanitized_from_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid source file path"))?,
                    )?;
                    let format = TomlFormat;
                    format.deserialize(&content)?
                }
                "properties" => {
                    let content = std::fs::read_to_string(
                        sanitized_from_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid source file path"))?,
                    )?;
                    let format = PropertiesFormat;
                    format.deserialize(&content)?
                }
                "yaml" | "yml" => {
                    let content = std::fs::read_to_string(
                        sanitized_from_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid source file path"))?,
                    )?;
                    serde_yaml::from_str(&content)?
                }
                _ => {
                    eprintln!("Unsupported source format: {}", from_format);
                    return Ok(());
                }
            };

            // Determine destination format and save config
            match to_format.as_str() {
                "json" => {
                    let format = JsonFormat;
                    crate::core::formats::save_config_as_format(
                        sanitized_to_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid destination file path"))?,
                        &config,
                        &format,
                    )?;
                }
                "toml" => {
                    let format = TomlFormat;
                    crate::core::formats::save_config_as_format(
                        sanitized_to_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid destination file path"))?,
                        &config,
                        &format,
                    )?;
                }
                "properties" => {
                    let format = PropertiesFormat;
                    crate::core::formats::save_config_as_format(
                        sanitized_to_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid destination file path"))?,
                        &config,
                        &format,
                    )?;
                }
                "yaml" | "yml" => {
                    let yaml_content = serde_yaml::to_string(&config)?;
                    std::fs::write(
                        sanitized_to_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid destination file path"))?,
                        yaml_content,
                    )?;
                }
                _ => {
                    eprintln!("Unsupported destination format: {}", to_format);
                    return Ok(());
                }
            };

            println!(
                "Configuration converted from {} to {} and saved to {}",
                from_format, to_format, to_file
            );
        }
        Commands::Crypto { action } => {
            use crate::core::crypto;
            use std::env;

            // Get encryption key from environment variable
            let key_str = env::var("NARU_ENCRYPTION_KEY")
                .map_err(|_| anyhow::anyhow!("NARU_ENCRYPTION_KEY environment variable not set"))?;

            if key_str.len() < 32 {
                return Err(anyhow::anyhow!(
                    "Encryption key must be at least 32 characters long"
                ));
            }

            let mut key = [0u8; 32];
            let bytes = key_str.as_bytes();
            let len = std::cmp::min(bytes.len(), 32);
            key[..len].copy_from_slice(&bytes[..len]);

            match action {
                cli::parser::CryptoAction::Encrypt {
                    input_file,
                    output_file,
                } => {
                    // Sanitize file paths to prevent directory traversal
                    let sanitized_input_file =
                        crate::core::security::sanitize_file_path(&input_file)
                            .map_err(|e| anyhow::anyhow!("Invalid input file path: {}", e))?;

                    let sanitized_output_file =
                        crate::core::security::sanitize_file_path(&output_file)
                            .map_err(|e| anyhow::anyhow!("Invalid output file path: {}", e))?;

                    crypto::encrypt_file(
                        sanitized_input_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid input file path"))?,
                        sanitized_output_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid output file path"))?,
                        &key,
                    )
                    .map_err(|e| anyhow::anyhow!("Failed to encrypt file: {}", e))?;
                    println!(
                        "File encrypted successfully from {} to {}",
                        input_file, output_file
                    );
                }
                cli::parser::CryptoAction::Decrypt {
                    input_file,
                    output_file,
                } => {
                    // Sanitize file paths to prevent directory traversal
                    let sanitized_input_file =
                        crate::core::security::sanitize_file_path(&input_file)
                            .map_err(|e| anyhow::anyhow!("Invalid input file path: {}", e))?;

                    let sanitized_output_file =
                        crate::core::security::sanitize_file_path(&output_file)
                            .map_err(|e| anyhow::anyhow!("Invalid output file path: {}", e))?;

                    crypto::decrypt_file(
                        sanitized_input_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid input file path"))?,
                        sanitized_output_file
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid output file path"))?,
                        &key,
                    )
                    .map_err(|e| anyhow::anyhow!("Failed to decrypt file: {}", e))?;
                    println!(
                        "File decrypted successfully from {} to {}",
                        input_file, output_file
                    );
                }
            }
        }
        Commands::Audit { count } => {
            let log_path = format!("{}/audit.log", NARU_DIR);
            let logs = crate::core::audit::AuditLogEntry::get_recent_logs(&log_path, count)
                .map_err(|e| anyhow::anyhow!("Failed to read audit logs: {}", e))?;

            if logs.is_empty() {
                println!("No audit logs found.");
            } else {
                println!("\nRecent Audit Logs (lastЧто?):");
                println!("{}", "-".repeat(80));
                for log in logs {
                    let key_str = log.key.clone().unwrap_or_else(|| "-".to_string());
                    let user_str = log.user.clone().unwrap_or_else(|| "unknown".to_string());
                    println!(
                        "[{}] {} - {} - Key: {} - User: {}",
                        log.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        log.action,
                        log.environment,
                        key_str,
                        user_str
                    );
                }
                println!("{}", "-".repeat(80));
            }
        }
        Commands::Validate => {
            let config: ConfigFile = persistence::load_json(CONFIG_FILE).map_err(|e| {
                anyhow::anyhow!("Failed to load config: {}. Run 'naru init' first.", e)
            })?;

            let schema: SchemaFile = persistence::load_json(SCHEMA_FILE)
                .map_err(|e| anyhow::anyhow!("Failed to load schema: {}", e))?;

            println!("\nValidating configuration against schema...");
            let mut errors = 0;

            for (env_name, env_config) in &config.environments {
                println!("  Environment: {}", env_name);
                for (key, entry) in &env_config.entries {
                    if let Some(field) = schema.fields.iter().find(|f| f.key == *key)
                        && let Err(e) = entry.validate(field)
                    {
                        println!("  [ERROR] {}: {}", key, e);
                        errors += 1;
                    }
                }
            }

            if errors == 0 {
                println!("✅ All configurations are valid!");
            } else {
                println!("\n❌ Found {} validation errors.", errors);
                return Err(anyhow::anyhow!("Validation failed."));
            }
        }
    }

    Ok(())
}
