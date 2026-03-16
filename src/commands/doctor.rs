use crate::core::constants::{CONFIG_FILE, NARU_DIR, SCHEMA_FILE};
use crate::core::models::{ConfigFile, SchemaFile};
use crate::core::persistence;
use anyhow::Result;
use console::style;
use std::fs;
use std::path::Path;

pub struct DoctorCommand;

impl DoctorCommand {
    pub fn new() -> Self {
        DoctorCommand
    }

    pub fn execute(&self) -> Result<()> {
        println!(
            "\n{}",
            style("Naru Doctor - Diagnostic Report").bold().cyan()
        );
        println!("{}\n", "=".repeat(50));

        let mut issues_found = 0;

        issues_found += self.check_naru_directory()?;
        issues_found += self.check_config_file()?;
        issues_found += self.check_schema_file()?;
        issues_found += self.check_encryption_key()?;
        issues_found += self.check_audit_log()?;
        issues_found += self.check_file_permissions()?;
        issues_found += self.check_project_integrity()?;

        println!("\n{}", "=".repeat(50));
        if issues_found == 0 {
            println!(
                "{}",
                style("✓ All checks passed! Your Naru setup is healthy.")
                    .green()
                    .bold()
            );
        } else {
            println!(
                "{}",
                style(format!(
                    "⚠ Found {} issue(s) that need attention.",
                    issues_found
                ))
                .yellow()
                .bold()
            );
        }
        println!();

        Ok(())
    }

    fn check_naru_directory(&self) -> Result<usize> {
        println!("{}", style("Checking Naru directory...").bold());

        let naru_path = Path::new(NARU_DIR);

        if !naru_path.exists() {
            println!(
                "  {} Naru directory not found. Run 'naru init' to initialize.",
                style("✗").red()
            );
            return Ok(1);
        }

        if !naru_path.is_dir() {
            println!(
                "  {} {} exists but is not a directory.",
                style("✗").red(),
                NARU_DIR
            );
            return Ok(1);
        }

        println!("  {} Naru directory exists", style("✓").green());
        Ok(0)
    }

    fn check_config_file(&self) -> Result<usize> {
        println!("{}", style("Checking configuration file...").bold());

        let config_path = Path::new(NARU_DIR).join(CONFIG_FILE);

        if !config_path.exists() {
            println!(
                "  {} Configuration file not found. Run 'naru init' to create.",
                style("✗").red()
            );
            return Ok(1);
        }

        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<ConfigFile>(&content) {
                Ok(config) => {
                    println!(
                        "  {} Configuration file is valid (project: {})",
                        style("✓").green(),
                        style(&config.project_name).cyan()
                    );

                    let env_count = config.environments.len();
                    if env_count == 0 {
                        println!(
                                "  {} No environments configured. Consider adding one with 'naru env add'.",
                                style("⚠").yellow()
                            );
                        return Ok(1);
                    }

                    println!(
                        "  {} {} environment(s) configured",
                        style("✓").green(),
                        env_count
                    );
                    Ok(0)
                }
                Err(e) => {
                    println!(
                        "  {} Configuration file is corrupted: {}",
                        style("✗").red(),
                        e
                    );
                    Ok(1)
                }
            },
            Err(e) => {
                println!(
                    "  {} Cannot read configuration file: {}",
                    style("✗").red(),
                    e
                );
                Ok(1)
            }
        }
    }

    fn check_schema_file(&self) -> Result<usize> {
        println!("{}", style("Checking schema file...").bold());

        let schema_path = Path::new(NARU_DIR).join(SCHEMA_FILE);

        if !schema_path.exists() {
            println!(
                "  {} Schema file not found. Use 'naru schema add' to define schema.",
                style("ℹ").blue()
            );
            return Ok(0);
        }

        match fs::read_to_string(&schema_path) {
            Ok(content) => match serde_json::from_str::<SchemaFile>(&content) {
                Ok(schema) => {
                    println!(
                        "  {} Schema file is valid ({} fields defined)",
                        style("✓").green(),
                        schema.fields.len()
                    );
                    Ok(0)
                }
                Err(e) => {
                    println!("  {} Schema file is corrupted: {}", style("✗").red(), e);
                    Ok(1)
                }
            },
            Err(e) => {
                println!("  {} Cannot read schema file: {}", style("✗").red(), e);
                Ok(1)
            }
        }
    }

    fn check_encryption_key(&self) -> Result<usize> {
        println!("{}", style("Checking encryption key...").bold());

        match std::env::var("NARU_ENCRYPTION_KEY") {
            Ok(key) => {
                if key.is_empty() {
                    println!(
                        "  {} NARU_ENCRYPTION_KEY is set but empty.",
                        style("✗").red()
                    );
                    println!(
                        "  {} Set a strong password: export NARU_ENCRYPTION_KEY=\"your-strong-password\"",
                        style("→").yellow()
                    );
                    return Ok(1);
                }

                if key.len() < 8 {
                    println!(
                        "  {} Encryption key is too weak ({} characters).",
                        style("⚠").yellow(),
                        key.len()
                    );
                    println!(
                        "  {} Use at least 8 characters for security.",
                        style("→").yellow()
                    );
                    return Ok(1);
                }

                println!(
                    "  {} Encryption key is set ({} characters)",
                    style("✓").green(),
                    key.len()
                );
                Ok(0)
            }
            Err(_) => {
                println!(
                    "  {} NARU_ENCRYPTION_KEY environment variable not set.",
                    style("✗").red()
                );
                println!(
                    "  {} Set it: export NARU_ENCRYPTION_KEY=\"your-strong-password\"",
                    style("→").yellow()
                );
                Ok(1)
            }
        }
    }

    fn check_audit_log(&self) -> Result<usize> {
        println!("{}", style("Checking audit log...").bold());

        let audit_path = Path::new(NARU_DIR).join("audit.log");

        if !audit_path.exists() {
            println!(
                "  {} Audit log not found. Will be created on first operation.",
                style("ℹ").blue()
            );
            return Ok(0);
        }

        match fs::metadata(&audit_path) {
            Ok(metadata) => {
                if metadata.len() == 0 {
                    println!("  {} Audit log exists but is empty.", style("⚠").yellow());
                    return Ok(0);
                }

                println!(
                    "  {} Audit log exists ({} bytes)",
                    style("✓").green(),
                    metadata.len()
                );

                match fs::read_to_string(&audit_path) {
                    Ok(content) => {
                        let line_count = content.lines().count();
                        println!(
                            "  {} {} audit entries recorded",
                            style("✓").green(),
                            line_count
                        );
                        Ok(0)
                    }
                    Err(e) => {
                        println!("  {} Cannot read audit log: {}", style("✗").red(), e);
                        Ok(1)
                    }
                }
            }
            Err(e) => {
                println!("  {} Cannot access audit log: {}", style("✗").red(), e);
                Ok(1)
            }
        }
    }

    fn check_file_permissions(&self) -> Result<usize> {
        println!("{}", style("Checking file permissions...").bold());

        let mut issues = 0;
        let files_to_check = [
            (CONFIG_FILE, "config"),
            (SCHEMA_FILE, "schema"),
            ("audit.log", "audit"),
        ];

        for (file_path, file_type) in &files_to_check {
            let path = Path::new(NARU_DIR).join(file_path);
            if path.exists() {
                match fs::metadata(&path) {
                    Ok(metadata) => {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let mode = metadata.permissions().mode();
                            let world_readable = mode & 0o004 != 0;

                            if world_readable && file_type.contains("config") {
                                println!(
                                    "  {} {} is world-readable (consider chmod 600)",
                                    style("⚠").yellow(),
                                    file_path
                                );
                                issues += 1;
                            } else {
                                println!(
                                    "  {} {} permissions are secure",
                                    style("✓").green(),
                                    file_path
                                );
                            }
                        }

                        #[cfg(not(unix))]
                        {
                            println!("  {} {} exists", style("✓").green(), file_path);
                        }
                    }
                    Err(e) => {
                        println!(
                            "  {} Cannot check permissions for {}: {}",
                            style("✗").red(),
                            file_path,
                            e
                        );
                        issues += 1;
                    }
                }
            }
        }

        Ok(issues)
    }

    fn check_project_integrity(&self) -> Result<usize> {
        println!("{}", style("Checking project integrity...").bold());

        let mut issues = 0;

        let config_path = Path::new(NARU_DIR).join(CONFIG_FILE);
        if config_path.exists() {
            match persistence::lock_file(CONFIG_FILE) {
                Ok(_lock) => {
                    println!(
                        "  {} Configuration file is not locked by another process",
                        style("✓").green()
                    );
                }
                Err(e) => {
                    println!(
                        "  {} Cannot access configuration file: {}",
                        style("✗").red(),
                        e
                    );
                    issues += 1;
                }
            }
        }

        Ok(issues)
    }
}

impl Default for DoctorCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    struct EnvVarGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvVarGuard {
        fn new(key: &'static str, value: &str) -> Self {
            let original = env::var(key).ok();
            env::set_var(key, value);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(val) => env::set_var(self.key, val),
                None => env::remove_var(self.key),
            }
        }
    }

    #[test]
    #[serial]
    fn test_doctor_command_creation() {
        let _command = DoctorCommand::new();
        assert!(true);
    }

    #[test]
    #[serial]
    fn test_doctor_with_encryption_key() {
        let _guard = EnvVarGuard::new("NARU_ENCRYPTION_KEY", "test-password-123");
        let command = DoctorCommand::new();
        let result = command.execute();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_doctor_without_encryption_key() {
        env::remove_var("NARU_ENCRYPTION_KEY");
        let command = DoctorCommand::new();
        let result = command.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_doctor_command_default() {
        let _command = DoctorCommand::default();
        assert!(true);
    }
}
