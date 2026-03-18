//! Naru Core Library
//!
//! A security-first configuration manager with encryption and audit logging.
//!
//! ## Features
//!
//! - AES-256-GCM encryption
//! - Argon2 key derivation  
//! - Hash-chained audit logging
//! - Schema validation
//! - Multi-format support (JSON, YAML, TOML, .env)
//!
//! ## Quick Start
//!
//! ```rust
//! use naru_core::{ConfigFile, SchemaFile};
//!
//! let config = ConfigFile::new("MyProject");
//! let schema = SchemaFile::new("1.0.0");
//! ```

pub mod core;

pub use core::{
    validate_value, BackupData, ConfigFile, ConfigFormat, ConfigValueEntry, EnvironmentConfig,
    FieldDefinition, JsonFormat, NaruError, NaruResult, PropertiesFormat, SchemaFile, TomlFormat,
    ValidationRules,
};
