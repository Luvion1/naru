use crate::core::audit;
use crate::core::constants::{CONFIG_FILE, NARU_DIR};
use crate::core::models::{ConfigFile, EnvironmentConfig};
use crate::core::persistence;
use crate::core::security;
use anyhow::{anyhow, Result};
use dialoguer::Confirm;

pub struct EnvAddCommand {
    pub name: String,
}

pub struct EnvRemoveCommand {
    pub name: String,
}

pub struct EnvSetParentCommand {
    pub name: String,
    pub parent: String,
}

pub fn execute_add(cmd: EnvAddCommand) -> Result<()> {
    security::validate_environment_name(&cmd.name)
        .map_err(|e| anyhow!("Invalid environment name: {}", e))?;

    let mut config: ConfigFile = persistence::load_json(CONFIG_FILE)
        .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

    if config.environments.contains_key(&cmd.name) {
        return Err(anyhow!("Environment '{}' already exists.", cmd.name));
    }

    config.environments.insert(
        cmd.name.clone(),
        EnvironmentConfig {
            parent: None,
            entries: std::collections::HashMap::new(),
        },
    );
    persistence::save_json(CONFIG_FILE, &config)?;

    let log_path = format!("{}/audit.log", NARU_DIR);
    if let Err(e) = audit::log_action("ENV_ADD", &cmd.name, None, None, None, &log_path) {
        eprintln!("Warning: Failed to log audit entry: {}", e);
    }

    println!("Added environment '{}'", cmd.name);
    Ok(())
}

pub fn execute_remove(cmd: EnvRemoveCommand) -> Result<()> {
    security::validate_environment_name(&cmd.name)
        .map_err(|e| anyhow!("Invalid environment name: {}", e))?;

    let mut config: ConfigFile = persistence::load_json(CONFIG_FILE)
        .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

    if ["development", "staging", "production"].contains(&cmd.name.as_str()) {
        return Err(anyhow!("Cannot remove default environment: {}", cmd.name));
    }

    if config.environments.remove(&cmd.name).is_none() {
        return Err(anyhow!("Environment '{}' not found.", cmd.name));
    }

    let confirmed = Confirm::new()
        .with_prompt(format!(
            "Are you sure you want to remove environment '{}'? This cannot be undone.",
            cmd.name
        ))
        .default(false)
        .interact()
        .map_err(|e| anyhow!("Confirmation prompt failed: {}", e))?;

    if !confirmed {
        return Ok(());
    }

    persistence::save_json(CONFIG_FILE, &config)?;

    let log_path = format!("{}/audit.log", NARU_DIR);
    if let Err(e) = audit::log_action("ENV_REMOVE", &cmd.name, None, None, None, &log_path) {
        eprintln!("Warning: Failed to log audit entry: {}", e);
    }

    println!("Removed environment '{}'", cmd.name);
    Ok(())
}

pub fn execute_list() -> Result<()> {
    let config: ConfigFile = persistence::load_json(CONFIG_FILE)
        .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

    println!("Available environments:");
    for (name, env_config) in &config.environments {
        if let Some(parent) = &env_config.parent {
            println!("  - {} (inherits from: {})", name, parent);
        } else {
            println!("  - {}", name);
        }
    }
    Ok(())
}

pub fn execute_set_parent(cmd: EnvSetParentCommand) -> Result<()> {
    security::validate_environment_name(&cmd.name)
        .map_err(|e| anyhow!("Invalid environment name: {}", e))?;
    security::validate_environment_name(&cmd.parent)
        .map_err(|e| anyhow!("Invalid parent environment name: {}", e))?;

    let mut config: ConfigFile = persistence::load_json(CONFIG_FILE)
        .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

    if !config.environments.contains_key(&cmd.name) {
        return Err(anyhow!("Environment '{}' not found.", cmd.name));
    }
    if !config.environments.contains_key(&cmd.parent) {
        return Err(anyhow!("Parent environment '{}' not found.", cmd.parent));
    }

    if cmd.name == cmd.parent {
        return Err(anyhow!("An environment cannot be its own parent."));
    }

    // Check for circular inheritance
    let mut current_parent = Some(cmd.parent.clone());
    while let Some(p) = current_parent {
        if p == cmd.name {
            return Err(anyhow!("Circular inheritance detected."));
        }
        current_parent = config.environments.get(&p).and_then(|e| e.parent.clone());
    }

    if let Some(env_config) = config.environments.get_mut(&cmd.name) {
        env_config.parent = Some(cmd.parent.clone());
    } else {
        return Err(anyhow!("Environment '{}' not found.", cmd.name));
    }

    persistence::save_json(CONFIG_FILE, &config)?;

    let log_path = format!("{}/audit.log", NARU_DIR);
    if let Err(e) = audit::log_action(
        "ENV_SET_PARENT",
        &cmd.name,
        Some("parent"),
        None,
        Some(&cmd.parent),
        &log_path,
    ) {
        eprintln!("Warning: Failed to log audit entry: {}", e);
    }

    println!(
        "Environment '{}' now inherits from '{}'",
        cmd.name, cmd.parent
    );
    Ok(())
}
