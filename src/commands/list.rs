use crate::core::constants::SCHEMA_FILE;
use crate::core::models::SchemaFile;
use crate::core::persistence;
use crate::core::security;
use anyhow::Result;
use console::{style, Emoji};

pub struct ListCommand {
    pub env: String,
}

impl ListCommand {
    pub fn new(env: String) -> Self {
        ListCommand { env }
    }

    pub fn execute(&self) -> Result<()> {
        security::validate_environment_name(&self.env)
            .map_err(|e| anyhow::anyhow!("Invalid environment name: {}", e))?;

        let schema: SchemaFile =
            persistence::atomic_read_json(SCHEMA_FILE, |s: &SchemaFile| s.clone()).unwrap_or_else(
                |_| SchemaFile {
                    version: "1.0".to_string(),
                    fields: vec![],
                },
            );

        persistence::atomic_read_config(|config| {
            let env_config = config
                .environments
                .get(&self.env)
                .ok_or_else(|| anyhow::anyhow!("Environment '{}' not found.", self.env))?;

            println!(
                "\n{} {}",
                Emoji("📁", ""),
                style(format!("Environment: {}", self.env)).bold().cyan()
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

                    if let Some(f) = field_def {
                        if let Some(desc) = &f.description {
                            println!("     {}", style(format!("└─ {}", desc)).dim().italic());
                        }
                    }
                }
            }
            println!("{}", style("=".repeat(60)).dim());
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("Read error: {}", e))?
    }
}
