use crate::core::persistence;
use anyhow::{anyhow, Result};
use console::Style;

pub struct SearchCommand {
    pub query: String,
    pub env: Option<String>,
    pub show_values: bool,
}

impl SearchCommand {
    pub fn new(query: String, env: Option<String>, show_values: bool) -> Self {
        Self {
            query,
            env,
            show_values,
        }
    }

    pub fn execute(&self) -> Result<()> {
        let config = persistence::atomic_read_config(|c| c.clone())
            .map_err(|e| anyhow!("Failed to load config: {}. Run 'naru init' first.", e))?;

        let query_lower = self.query.to_lowercase();
        let mut found = false;

        let environments: Vec<_> = if let Some(ref env_name) = self.env {
            vec![env_name.clone()]
        } else {
            config.environments.keys().cloned().collect()
        };

        for env_name in environments {
            if let Some(env_config) = config.environments.get(&env_name) {
                println!(
                    "\n{}Environment: {}{}",
                    Style::new().cyan().apply_to("["),
                    Style::new().green().apply_to(&env_name),
                    Style::new().cyan().apply_to("]")
                );

                for (key, entry) in &env_config.entries {
                    let key_lower = key.to_lowercase();
                    let value_lower = entry.value.to_lowercase();
                    let value_matches = self.show_values && value_lower.contains(&query_lower);

                    if key_lower.contains(&query_lower) || value_matches {
                        found = true;
                        if self.show_values {
                            let display_value = if entry.is_secret && !entry.encrypted {
                                Style::new().dim().apply_to("********").to_string()
                            } else if entry.encrypted {
                                Style::new().yellow().apply_to("<encrypted>").to_string()
                            } else {
                                entry.value.clone()
                            };
                            println!(
                                "  {} = {}",
                                Style::new().cyan().apply_to(key),
                                display_value
                            );
                        } else {
                            println!("  {}", Style::new().cyan().apply_to(key));
                        }
                    }
                }
            }
        }

        if !found {
            println!("No matches found for '{}'", self.query);
        }

        Ok(())
    }
}
