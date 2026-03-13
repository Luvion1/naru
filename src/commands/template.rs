use crate::core::constants;
use crate::core::models::{
    ConfigFile, ConfigValueEntry, EnvironmentConfig, FieldDefinition, SchemaFile, ValidationRules,
};
use crate::core::persistence;
use crate::core::persistence::PersistenceError;
use anyhow::Result;
use console::style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub name: String,
    pub version: String,
    pub description: String,
    pub variables: Vec<TemplateVariable>,
    pub schema_fields: Vec<FieldDefinition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateVariable {
    pub key: String,
    pub kind: String,
    pub default: Option<String>,
    pub note: Option<String>,
    pub secret: bool,
    pub rules: Option<ValidationRules>,
}

pub fn template_create(name: &str, include_secrets: bool) -> Result<(), PersistenceError> {
    let config: ConfigFile = persistence::load_json(constants::CONFIG_FILE)?;
    let schema: SchemaFile = persistence::load_json(constants::SCHEMA_FILE).unwrap_or(SchemaFile {
        version: "1.0".to_string(),
        fields: vec![],
    });

    let templates_dir = PathBuf::from(constants::NARU_DIR).join("templates");
    fs::create_dir_all(&templates_dir).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::other(format!("Cannot create templates directory: {}", e)),
    })?;

    let mut variables = Vec::new();

    for (env_name, env_config) in &config.environments {
        for (key, entry) in &env_config.entries {
            if !include_secrets && entry.is_secret {
                continue;
            }

            if !variables.iter().any(|v: &TemplateVariable| v.key == *key) {
                let rules = schema
                    .fields
                    .iter()
                    .find(|f| f.key == *key)
                    .and_then(|f| f.validation.clone());

                variables.push(TemplateVariable {
                    key: key.clone(),
                    kind: entry.r#type.clone(),
                    default: if entry.is_secret {
                        None
                    } else {
                        Some(entry.value.clone())
                    },
                    note: schema
                        .fields
                        .iter()
                        .find(|f| f.key == *key)
                        .and_then(|f| f.description.clone()),
                    secret: entry.is_secret,
                    rules,
                });
            }
        }
    }

    let template = Template {
        name: name.to_string(),
        version: config.version.clone(),
        description: format!("Template created from {}", config.project_name),
        variables,
        schema_fields: schema.fields,
    };

    let template_path = templates_dir.join(format!("{}.toml", name));
    let toml_str = toml::to_string_pretty(&template).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::other(format!("Cannot serialize template: {}", e)),
    })?;

    fs::write(&template_path, toml_str).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::other(format!("Cannot write template: {}", e)),
    })?;

    println!(
        "{} Template '{}' created with {} variables",
        style("✓").green(),
        style(name).cyan(),
        style(template.variables.len()).yellow()
    );

    Ok(())
}

pub fn template_apply(name: &str, env: &str) -> Result<(), PersistenceError> {
    let templates_dir = PathBuf::from(constants::NARU_DIR).join("templates");
    let template_path = templates_dir.join(format!("{}.toml", name));

    if !template_path.exists() {
        return Err(PersistenceError::IoError {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Template '{}' not found", name),
            ),
        });
    }

    let template_content =
        fs::read_to_string(&template_path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Cannot read template: {}", e)),
        })?;

    let template: Template =
        toml::from_str(&template_content).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Invalid template format: {}", e)),
        })?;

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

    let mut applied_count = 0;

    if let Some(env_config) = config.environments.get_mut(env) {
        for var in &template.variables {
            if !env_config.entries.contains_key(&var.key) {
                let value = var.default.clone().unwrap_or_else(|| {
                    if var.secret {
                        String::new()
                    } else {
                        format!("TODO_{}", var.key)
                    }
                });

                env_config.entries.insert(
                    var.key.clone(),
                    ConfigValueEntry {
                        value,
                        r#type: var.kind.clone(),
                        is_secret: var.secret,
                        encrypted: false,
                    },
                );
                applied_count += 1;
            }
        }
    }

    if !template.schema_fields.is_empty() {
        let mut current_schema: SchemaFile = persistence::load_json(constants::SCHEMA_FILE)
            .unwrap_or(SchemaFile {
                version: "1.0".to_string(),
                fields: vec![],
            });

        for field in &template.schema_fields {
            if !current_schema.fields.iter().any(|f| f.key == field.key) {
                current_schema.fields.push(field.clone());
            }
        }

        persistence::save_json(constants::SCHEMA_FILE, &current_schema)?;
    }

    persistence::save_json(constants::CONFIG_FILE, &config)?;

    println!(
        "{} Template '{}' applied: {} variables added to '{}'",
        style("✓").green(),
        style(name).cyan(),
        style(applied_count).yellow(),
        style(env).cyan()
    );

    Ok(())
}

pub fn template_list() -> Result<(), PersistenceError> {
    let templates_dir = PathBuf::from(constants::NARU_DIR).join("templates");

    if !templates_dir.exists() {
        println!("{} No templates directory found", style("ℹ").blue());
        return Ok(());
    }

    let mut templates = Vec::new();

    for entry in fs::read_dir(&templates_dir).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::other(format!("Cannot read templates directory: {}", e)),
    })? {
        let entry = entry.map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Error reading directory: {}", e)),
        })?;

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                templates.push(name.to_string());
            }
        }
    }

    if templates.is_empty() {
        println!("{} No templates found", style("ℹ").blue());
    } else {
        println!("{} Available templates:", style("✓").green());
        for template in templates {
            println!("  • {}", style(&template).cyan());
        }
    }

    Ok(())
}
