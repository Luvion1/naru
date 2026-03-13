use crate::cli::interactive;
use crate::core::audit;
use crate::core::constants::{NARU_DIR, SCHEMA_FILE};
use crate::core::models::{FieldDefinition, SchemaFile};
use crate::core::persistence;
use crate::core::schema;
use crate::core::security;
use anyhow::{anyhow, Result};

pub struct SchemaAddCommand {
    pub key: Option<String>,
    pub r#type: String,
    pub description: Option<String>,
    pub secret: bool,
}

pub struct SchemaRemoveCommand {
    pub key: Option<String>,
}

pub struct SchemaEditCommand {
    pub key: Option<String>,
}

pub fn execute_add(cmd: SchemaAddCommand) -> Result<()> {
    let field_def = if let Some(key) = cmd.key {
        security::validate_config_key(&key).map_err(|e| anyhow!("Invalid config key: {}", e))?;

        FieldDefinition {
            key: key.clone(),
            r#type: cmd.r#type,
            description: cmd.description,
            validation: None,
            is_secret: cmd.secret,
        }
    } else {
        interactive::prompt_add_field()?
    };

    let key_name = field_def.key.clone();
    schema::add_field(field_def)?;

    let log_path = format!("{}/audit.log", NARU_DIR);
    if let Err(e) = audit::log_action(
        "SCHEMA_ADD",
        "schema",
        Some(&key_name),
        None,
        None,
        &log_path,
    ) {
        eprintln!("Warning: Failed to log audit entry: {}", e);
    }

    Ok(())
}

pub fn execute_remove(cmd: SchemaRemoveCommand) -> Result<()> {
    let key = if let Some(key) = cmd.key {
        key
    } else {
        let schema: SchemaFile =
            persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| SchemaFile {
                version: "1.0".to_string(),
                fields: vec![],
            });
        interactive::prompt_select_field(&schema)?
    };

    schema::remove_field(&key)?;

    let log_path = format!("{}/audit.log", NARU_DIR);
    if let Err(e) = audit::log_action("SCHEMA_REMOVE", "schema", Some(&key), None, None, &log_path)
    {
        eprintln!("Warning: Failed to log audit entry: {}", e);
    }

    Ok(())
}

pub fn execute_view() -> Result<()> {
    let schema: SchemaFile = persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| {
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
    Ok(())
}

pub fn execute_edit(cmd: SchemaEditCommand) -> Result<()> {
    let schema: SchemaFile =
        persistence::load_json(SCHEMA_FILE).map_err(|e| anyhow!("Failed to load schema: {}", e))?;

    let key = if let Some(key) = cmd.key {
        key
    } else {
        interactive::prompt_select_field(&schema)?
    };

    security::validate_config_key(&key).map_err(|e| anyhow!("Invalid field key: {}", e))?;

    let existing_field = schema
        .fields
        .iter()
        .find(|f| f.key == key)
        .ok_or_else(|| anyhow!("Field '{}' not found in schema", key))?;

    let updated_field = interactive::prompt_edit_field(existing_field)?;
    let key_name = updated_field.key.clone();
    schema::update_field(&key, updated_field)?;

    let log_path = format!("{}/audit.log", NARU_DIR);
    if let Err(e) = audit::log_action(
        "SCHEMA_EDIT",
        "schema",
        Some(&key_name),
        None,
        None,
        &log_path,
    ) {
        eprintln!("Warning: Failed to log audit entry: {}", e);
    }

    Ok(())
}
