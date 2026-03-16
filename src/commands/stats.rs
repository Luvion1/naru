use crate::core::constants::{CONFIG_FILE, NARU_DIR, SCHEMA_FILE};
use crate::core::models::{ConfigFile, SchemaFile};
use anyhow::{Context, Result};
use console::style;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct StatsCommand;

impl StatsCommand {
    pub fn new() -> Self {
        StatsCommand
    }

    pub fn execute(&self) -> Result<()> {
        println!("\n{}", style("Naru Statistics").bold().cyan());
        println!("{}\n", "=".repeat(50));

        let config_path = Path::new(NARU_DIR).join(CONFIG_FILE);
        let schema_path = Path::new(NARU_DIR).join(SCHEMA_FILE);

        if !config_path.exists() {
            println!(
                "{} Naru project not initialized. Run 'naru init' first.",
                style("⚠").yellow()
            );
            return Ok(());
        }

        let config_content =
            fs::read_to_string(&config_path).context("Failed to read configuration file")?;
        let config: ConfigFile =
            serde_json::from_str(&config_content).context("Failed to parse configuration file")?;

        self.show_project_info(&config);
        self.show_environment_stats(&config);
        self.show_value_type_stats(&config);
        self.show_secret_stats(&config);

        if schema_path.exists() {
            self.show_schema_stats(&schema_path)?;
        }

        self.show_audit_stats();

        println!("\n{}", "=".repeat(50));
        println!("{}", style("Statistics complete").green().bold());
        println!();

        Ok(())
    }

    fn show_project_info(&self, config: &ConfigFile) {
        println!("{}", style("Project Information").bold().white());
        println!("  Project Name: {}", style(&config.project_name).cyan());
        println!("  Version: {}", style(&config.version).cyan());
        println!(
            "  Salt Configured: {}",
            style(if config.salt.is_some() { "Yes" } else { "No" }).cyan()
        );
        println!();
    }

    fn show_environment_stats(&self, config: &ConfigFile) {
        println!("{}", style("Environment Statistics").bold().white());
        println!(
            "  Total Environments: {}",
            style(config.environments.len()).cyan()
        );

        let mut total_entries = 0;
        let mut env_details = Vec::new();

        for (env_name, env_config) in &config.environments {
            let entry_count = env_config.entries.len();
            total_entries += entry_count;
            let parent_info = env_config
                .parent
                .as_ref()
                .map(|p| format!(" (inherits: {})", p))
                .unwrap_or_default();
            env_details.push((env_name.clone(), entry_count, parent_info));
        }

        println!(
            "  Total Configuration Entries: {}",
            style(total_entries).cyan()
        );
        println!();

        println!("  Environment Breakdown:");
        env_details.sort_by(|a, b| a.0.cmp(&b.0));
        for (env_name, count, parent_info) in env_details {
            println!(
                "    {} {} entries{}",
                style(&env_name).yellow(),
                style(count).cyan(),
                parent_info
            );
        }
        println!();
    }

    fn show_value_type_stats(&self, config: &ConfigFile) {
        println!("{}", style("Value Type Distribution").bold().white());

        let mut type_counts: HashMap<&str, usize> = HashMap::new();

        for env_config in config.environments.values() {
            for entry in env_config.entries.values() {
                let type_name = entry.r#type.as_str();
                *type_counts.entry(type_name).or_insert(0) += 1;
            }
        }

        if type_counts.is_empty() {
            println!("  No configuration entries found.");
        } else {
            let mut types: Vec<_> = type_counts.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));

            for (type_name, count) in types {
                println!(
                    "    {}: {} entries",
                    style(type_name).yellow(),
                    style(count).cyan()
                );
            }
        }
        println!();
    }

    fn show_secret_stats(&self, config: &ConfigFile) {
        println!("{}", style("Security Statistics").bold().white());

        let mut total_secrets = 0;
        let mut total_non_secrets = 0;
        let mut secrets_per_env: HashMap<String, usize> = HashMap::new();

        for (env_name, env_config) in &config.environments {
            let mut env_secrets = 0;
            for entry in env_config.entries.values() {
                if entry.is_secret {
                    total_secrets += 1;
                    env_secrets += 1;
                } else {
                    total_non_secrets += 1;
                }
            }
            if env_secrets > 0 {
                secrets_per_env.insert(env_name.clone(), env_secrets);
            }
        }

        println!("  Total Secrets: {}", style(total_secrets).cyan());
        println!("  Total Non-Secrets: {}", style(total_non_secrets).cyan());

        if total_secrets + total_non_secrets > 0 {
            let secret_percentage =
                (total_secrets as f64) / ((total_secrets + total_non_secrets) as f64) * 100.0;
            println!(
                "  Secret Ratio: {}%",
                style(format!("{:.1}", secret_percentage)).cyan()
            );
        }

        if !secrets_per_env.is_empty() {
            println!("  Secrets by Environment:");
            let mut envs: Vec<_> = secrets_per_env.iter().collect();
            envs.sort_by(|a, b| b.1.cmp(a.1));
            for (env_name, count) in envs {
                println!(
                    "    {}: {} secrets",
                    style(env_name).yellow(),
                    style(count).cyan()
                );
            }
        }
        println!();
    }

    fn show_schema_stats(&self, schema_path: &Path) -> Result<()> {
        println!("{}", style("Schema Statistics").bold().white());

        let schema_content =
            fs::read_to_string(schema_path).context("Failed to read schema file")?;
        let schema: SchemaFile =
            serde_json::from_str(&schema_content).context("Failed to parse schema file")?;

        println!("  Schema Version: {}", style(&schema.version).cyan());
        println!(
            "  Total Fields Defined: {}",
            style(schema.fields.len()).cyan()
        );

        if !schema.fields.is_empty() {
            let secret_fields = schema.fields.iter().filter(|f| f.is_secret).count();
            let public_fields = schema.fields.len() - secret_fields;

            println!("  Public Fields: {}", style(public_fields).cyan());
            println!("  Secret Fields: {}", style(secret_fields).cyan());

            let mut type_counts: HashMap<&str, usize> = HashMap::new();
            for field in &schema.fields {
                let type_name = field.r#type.as_str();
                *type_counts.entry(type_name).or_insert(0) += 1;
            }

            println!("  Field Types:");
            let mut types: Vec<_> = type_counts.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            for (type_name, count) in types {
                println!(
                    "    {}: {} fields",
                    style(type_name).yellow(),
                    style(count).cyan()
                );
            }

            let fields_with_validation = schema
                .fields
                .iter()
                .filter(|f| f.validation.is_some())
                .count();
            println!(
                "  Fields with Validation: {}",
                style(fields_with_validation).cyan()
            );
        }
        println!();

        Ok(())
    }

    fn show_audit_stats(&self) {
        println!("{}", style("Audit Log Statistics").bold().white());

        let audit_path = Path::new(NARU_DIR).join("audit.log");

        if !audit_path.exists() {
            println!("  Audit log not yet created.");
        } else {
            match fs::metadata(&audit_path) {
                Ok(metadata) => {
                    println!("  Audit Log Size: {} bytes", style(metadata.len()).cyan());

                    if let Ok(content) = fs::read_to_string(&audit_path) {
                        let line_count = content.lines().count();
                        println!("  Total Audit Entries: {}", style(line_count).cyan());

                        if line_count > 0 {
                            if let Some(last_line) = content.lines().last() {
                                if let Ok(entry) =
                                    serde_json::from_str::<serde_json::Value>(last_line)
                                {
                                    if let Some(timestamp) =
                                        entry.get("timestamp").and_then(|t| t.as_str())
                                    {
                                        println!("  Last Activity: {}", style(timestamp).cyan());
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("  Unable to read audit log: {}", style(e).red());
                }
            }
        }
        println!();
    }
}

impl Default for StatsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_stats_command_creation() {
        let _command = StatsCommand::new();
        assert!(true);
    }

    #[test]
    #[serial]
    fn test_stats_command_execute_uninitialized() {
        let command = StatsCommand::new();
        let result = command.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_stats_command_default() {
        let _command = StatsCommand::default();
        assert!(true);
    }
}
