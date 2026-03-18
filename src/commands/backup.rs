use crate::core::audit;
use crate::core::constants::{NARU_DIR, SCHEMA_FILE};
use crate::core::models::{BackupData, ConfigFile, SchemaFile};
use crate::core::persistence;
use anyhow::{anyhow, Result};

pub struct BackupCreateCommand {
    pub file_path: String,
}

pub struct BackupRestoreCommand {
    pub file_path: String,
}

pub fn execute_create(cmd: BackupCreateCommand) -> Result<()> {
    // Determine appropriate file extension based on actual format
    let file_path = if cmd.file_path.ends_with(".json") {
        cmd.file_path.clone()
    } else if cmd.file_path.ends_with(".tar.gz") || cmd.file_path.ends_with(".tgz") {
        // User specified tar.gz but we write JSON - warn and change extension
        eprintln!("Warning: Backup format is JSON, not tar.gz. Using .json extension.");
        cmd.file_path
            .trim_end_matches(".tar.gz")
            .trim_end_matches(".tgz")
            .to_string()
            + ".json"
    } else {
        // Add .json extension if no known extension
        cmd.file_path + ".json"
    };

    let sanitized_path = crate::core::security::sanitize_file_path(&file_path)
        .map_err(|e| anyhow!("Invalid file path: {}", e))?;

    let schema: SchemaFile = persistence::atomic_read_json(SCHEMA_FILE, |s: &SchemaFile| s.clone())
        .unwrap_or_else(|_| {
            eprintln!("Warning: Could not load schema file, using default schema");
            SchemaFile {
                version: "1.0".to_string(),
                fields: vec![],
            }
        });

    let config: ConfigFile = persistence::atomic_read_config(|config| {
        let mut config_clone = config.clone();
        for env_config in config_clone.environments.values_mut() {
            env_config
                .entries
                .retain(|_, entry| !entry.is_secret || !entry.encrypted);
        }
        config_clone
    })
    .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

    let backup_data = BackupData::new(config, schema);

    let json_data = serde_json::to_string_pretty(&backup_data)?;
    std::fs::write(
        sanitized_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid file path"))?,
        json_data,
    )?;
    println!("Backup created successfully at: {}", file_path);
    println!("Note: Encrypted secrets have been excluded from backup for security.");
    Ok(())
}

pub fn execute_restore(cmd: BackupRestoreCommand) -> Result<()> {
    let sanitized_path = crate::core::security::sanitize_file_path(&cmd.file_path)
        .map_err(|e| anyhow!("Invalid file path: {}", e))?;

    crate::core::security::check_file_size(&sanitized_path, 10 * 1024 * 1024)
        .map_err(|e| anyhow!("Backup file too large: {}", e))?;

    let content = std::fs::read_to_string(
        sanitized_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid file path"))?,
    )?;

    if content.len() > 10 * 1024 * 1024 {
        return Err(anyhow!("Backup file exceeds 10MB limit"));
    }

    let backup_data: BackupData =
        serde_json::from_str(&content).map_err(|e| anyhow!("Invalid backup file format: {}", e))?;

    if backup_data.config.environments.is_empty() {
        return Err(anyhow!("Backup file has no environments"));
    }
    for (env_name, env_config) in &backup_data.config.environments {
        if env_name.is_empty() {
            return Err(anyhow!("Backup contains empty environment name"));
        }
        for (key, entry) in &env_config.entries {
            if key.is_empty() {
                return Err(anyhow!(
                    "Backup contains empty key in environment {}",
                    env_name
                ));
            }
            if entry.value.len() > 1_000_000 {
                return Err(anyhow!(
                    "Backup contains excessively large value for key {}",
                    key
                ));
            }
        }
    }

    if backup_data.schema.version.is_empty() {
        return Err(anyhow!("Backup has invalid schema version"));
    }

    persistence::atomic_update_config(|config| {
        *config = backup_data.config.clone();
        Ok(())
    })?;

    persistence::atomic_update_json(SCHEMA_FILE, |schema: &mut SchemaFile| {
        *schema = backup_data.schema.clone();
        Ok(())
    })?;

    let log_path = format!("{}/audit.log", NARU_DIR);
    if let Err(e) = audit::log_action(
        "BACKUP_RESTORE",
        "all",
        None,
        None,
        Some(&cmd.file_path),
        &log_path,
    ) {
        eprintln!("Warning: Failed to log audit entry: {}", e);
    }

    println!(
        "Configuration restored successfully from: {}",
        cmd.file_path
    );
    Ok(())
}
