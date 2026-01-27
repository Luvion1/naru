use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "naru")]
#[command(about = "A simple config manager CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Naru project
    Init,
    /// Set a configuration value
    Set {
        /// Format: key=value
        kv: String,
        /// The environment (e.g., development, production)
        #[arg(short, long, default_value = "development")]
        env: String,
        /// Mark as secret
        #[arg(short, long)]
        secret: bool,
    },
    /// Get a configuration value
    Get {
        key: String,
        /// The environment
        #[arg(short, long, default_value = "development")]
        env: String,
    },
    /// List all configurations in an environment
    List {
        /// The environment
        #[arg(short, long, default_value = "development")]
        env: String,
    },
    /// Import configurations from a file (.env or .yaml)
    Import {
        /// Path to the file to import
        file_path: String,
        /// The environment to import to
        #[arg(short, long, default_value = "development")]
        env: String,
    },
    /// Export configurations to a file (.env or .yaml)
    Export {
        /// Path to the file to export to
        file_path: String,
        /// The environment to export from
        #[arg(short, long, default_value = "development")]
        env: String,
        /// Format to export (env or yaml)
        #[arg(long, default_value = "env")]
        format: String,
    },
    /// Interactive schema editor
    Schema {
        /// Action to perform on schema (add, edit, remove, view)
        #[command(subcommand)]
        action: SchemaAction,
    },
    /// Manage environments
    Env {
        /// Action to perform on environments (add, remove, list)
        #[command(subcommand)]
        action: EnvAction,
    },
    /// Backup and restore configurations
    Backup {
        /// Action to perform (backup or restore)
        #[command(subcommand)]
        action: BackupAction,
    },
    /// Compare configurations between environments
    Diff {
        /// First environment to compare
        env1: String,
        /// Second environment to compare
        env2: String,
    },
    /// Convert configuration between different formats
    Convert {
        /// Source file path
        from_file: String,
        /// Destination file path
        to_file: String,
        /// Source format (json, yaml, toml, properties)
        #[arg(long, default_value = "json")]
        from_format: String,
        /// Destination format (json, yaml, toml, properties)
        #[arg(long, default_value = "json")]
        to_format: String,
    },
    /// Encrypt or decrypt configuration files
    Crypto {
        /// Action to perform (encrypt or decrypt)
        #[command(subcommand)]
        action: CryptoAction,
    },
    /// View or verify audit logs
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },
    /// Validate configurations against schema
    Validate,
    /// Show version information
    Version,
    /// 🐱 Meow! Show the Naru mascot
    Meow,
}

#[derive(Subcommand)]
pub enum AuditAction {
    /// Show recent audit logs
    Log {
        /// Number of recent entries to show
        #[arg(long, default_value = "10")]
        count: usize,
    },
    /// Verify the integrity of the audit log (Tamper-evident check)
    Verify,
}

#[derive(Subcommand)]
pub enum CryptoAction {
    /// Encrypt a configuration file
    Encrypt {
        /// Input file path
        input_file: String,
        /// Output file path
        output_file: String,
    },
    /// Decrypt a configuration file
    Decrypt {
        /// Input file path
        input_file: String,
        /// Output file path
        output_file: String,
    },
}

#[derive(Subcommand)]
pub enum BackupAction {
    /// Create a backup of all configurations
    Create {
        /// Path to the backup file
        file_path: String,
    },
    /// Restore configurations from a backup
    Restore {
        /// Path to the backup file
        file_path: String,
    },
}

#[derive(Subcommand)]
pub enum EnvAction {
    /// Add a new environment
    Add {
        /// Name of the environment to add
        name: String,
    },
    /// Remove an environment
    Remove {
        /// Name of the environment to remove
        name: String,
    },
    /// List all environments
    List,
}

#[derive(Subcommand)]
pub enum SchemaAction {
    /// Add a new field to the schema
    Add {
        /// Key name for the field (omit for interactive mode)
        key: Option<String>,
        /// Type of the field (string, integer, boolean)
        #[arg(long, default_value = "string")]
        r#type: String,
        /// Description for the field
        #[arg(long)]
        description: Option<String>,
        /// Mark as secret
        #[arg(short, long)]
        secret: bool,
    },
    /// Remove a field from the schema
    Remove {
        /// Key name to remove (omit for interactive mode)
        key: Option<String>,
    },
    /// View the current schema
    View,
    /// Edit field properties interactively
    Edit {
        /// Key name to edit (omit for interactive mode)
        key: Option<String>,
    },
}
