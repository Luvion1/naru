use super::audit;
use super::constants::*;
use super::crypto;
use super::locking;
use super::security;
use crate::core::models::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("JSON error: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },
    #[error("YAML error: {source}")]
    YamlError {
        #[from]
        source: serde_yaml::Error,
    },
    #[error("Missing encryption key. Please set NARU_ENCRYPTION_KEY environment variable.")]
    MissingEncryptionKey,
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Audit log integrity check failed. The log may have been tampered with.")]
    AuditIntegrityFailure,
}

pub fn init_project() -> Result<(), PersistenceError> {
    if Path::new(NARU_DIR).exists() {
        // Check audit log integrity if it exists
        let log_path = format!("{}/audit.log", NARU_DIR);
        if Path::new(&log_path).exists() {
            match audit::AuditLogEntry::verify_log_integrity(&log_path) {
                Ok(true) => {}
                _ => return Err(PersistenceError::AuditIntegrityFailure),
            }
        }
        return Ok(());
    }

    fs::create_dir_all(NARU_DIR)?;

    let mut environments = HashMap::new();
    for env in &["development", "staging", "production"] {
        environments.insert(
            env.to_string(),
            EnvironmentConfig {
                entries: HashMap::new(),
            },
        );
    }

    let config = ConfigFile {
        project_name: "My Project".to_string(),
        version: "0.1.0".to_string(),
        environments,
    };

    save_json(CONFIG_FILE, &config)?;

    let schema = SchemaFile {
        version: "1.0".to_string(),
        fields: vec![],
    };

    save_json(SCHEMA_FILE, &schema)?;

    Ok(())
}

pub fn save_json<T: serde::Serialize>(filename: &str, data: &T) -> Result<(), PersistenceError> {
    // Sanitize filename to prevent directory traversal
    let sanitized_filename =
        security::sanitize_file_path(filename).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    let path = Path::new(NARU_DIR).join(sanitized_filename);

    // Acquire a lock before writing
    let _lock =
        locking::FileLock::acquire_exclusive(&path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Could not acquire file lock: {}", e)),
        })?;

    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_json<T: serde::de::DeserializeOwned>(filename: &str) -> Result<T, PersistenceError> {
    // Sanitize filename to prevent directory traversal
    let sanitized_filename =
        security::sanitize_file_path(filename).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    let path = Path::new(NARU_DIR).join(sanitized_filename);

    // Acquire a lock before reading
    let _lock =
        locking::FileLock::acquire_exclusive(&path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Could not acquire file lock: {}", e)),
        })?;

    let content = fs::read_to_string(path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn import_from_env(file_path: &str, env: &str) -> Result<ConfigFile, PersistenceError> {
    // Sanitize file path to prevent directory traversal
    let sanitized_path =
        security::sanitize_file_path(file_path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    // Validate environment name
    security::validate_environment_name(env).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
    })?;

    // Check file size before reading (max 1MB)
    security::check_file_size(&sanitized_path, 1024 * 1024) // 1MB limit
        .map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })?;

    let content = fs::read_to_string(sanitized_path)?;
    let mut config: ConfigFile = load_json(CONFIG_FILE)?;

    let mut dotenv_pairs = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_string();
            let mut value = line[pos + 1..].trim().to_string();

            // Handle quoted values (double quotes or single quotes)
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = value[1..value.len() - 1].to_string();
            }

            dotenv_pairs.insert(key, value);
        }
    }

    merge_map_into_config(&mut config, env, dotenv_pairs)
}

pub fn import_from_yaml(file_path: &str, env: &str) -> Result<ConfigFile, PersistenceError> {
    let sanitized_path =
        security::sanitize_file_path(file_path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    security::validate_environment_name(env).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
    })?;

    security::check_file_size(&sanitized_path, 1024 * 1024).map_err(|e| {
        PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        }
    })?;

    let content = fs::read_to_string(sanitized_path)?;
    let yaml_pairs: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&content)?;
    let mut config: ConfigFile = load_json(CONFIG_FILE)?;

    let mut string_pairs: HashMap<String, String> = HashMap::new();
    for (k, v) in yaml_pairs {
        let val_str = match v {
            serde_yaml::Value::String(s) => s,
            serde_yaml::Value::Number(n) => n.to_string(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            _ => continue, // Skip complex types
        };
        string_pairs.insert(k, val_str);
    }

    merge_map_into_config(&mut config, env, string_pairs)
}

pub fn import_from_json(file_path: &str, env: &str) -> Result<ConfigFile, PersistenceError> {
    let sanitized_path =
        security::sanitize_file_path(file_path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    security::validate_environment_name(env).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
    })?;

    security::check_file_size(&sanitized_path, 1024 * 1024).map_err(|e| {
        PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        }
    })?;

    let content = fs::read_to_string(sanitized_path)?;
    let json_pairs: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
    let mut config: ConfigFile = load_json(CONFIG_FILE)?;

    let mut string_pairs: HashMap<String, String> = HashMap::new();
    for (k, v) in json_pairs {
        let val_str = match v {
            serde_json::Value::String(s) => s,
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => continue, // Skip complex types
        };
        string_pairs.insert(k, val_str);
    }

    merge_map_into_config(&mut config, env, string_pairs)
}

fn encrypt_entry_value(entry: &mut ConfigValueEntry) -> Result<(), PersistenceError> {
    if entry.is_secret && !entry.encrypted {
        let encryption_key = get_encryption_key()?;
        let encrypted_value = crypto::encrypt_data(&entry.value, &encryption_key).map_err(|e| {
            PersistenceError::IoError {
                source: std::io::Error::other(e.to_string()),
            }
        })?;

        entry.value = encrypted_value;
        entry.encrypted = true;
    }
    Ok(())
}

fn merge_map_into_config(
    config: &mut ConfigFile,
    env: &str,
    pairs: HashMap<String, String>,
) -> Result<ConfigFile, PersistenceError> {
    let schema: SchemaFile = load_json(SCHEMA_FILE).unwrap_or(SchemaFile {
        version: "1.0".to_string(),
        fields: vec![],
    });

    // Check if environment exists, if not, create it
    if !config.environments.contains_key(env) {
        config.environments.insert(
            env.to_string(),
            EnvironmentConfig {
                entries: HashMap::new(),
            },
        );
    }

    // Now get the mutable reference to the environment
    if let Some(env_config) = config.environments.get_mut(env) {
        for (key, value) in pairs {
            // Validate the key
            security::validate_config_key(&key).map_err(|e| PersistenceError::IoError {
                source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
            })?;

            // Sanitize the value
            let sanitized_value = security::sanitize_string_value(&value);

            let mut is_secret = false;
            let mut target_type = "string".to_string();

            if let Some(field) = schema.fields.iter().find(|f| f.key == key) {
                target_type = field.r#type.clone();
                is_secret = field.is_secret;

                // Validate value against schema
                crate::core::validation::validate_value(&sanitized_value, field).map_err(|e| {
                    PersistenceError::ValidationError(format!("Field '{}': {}", key, e))
                })?;
            }

            let mut entry = ConfigValueEntry {
                value: sanitized_value,
                r#type: target_type,
                is_secret,
                encrypted: false,
            };

            encrypt_entry_value(&mut entry)?;

            env_config.entries.insert(key, entry);
        }
    }

    Ok(config.clone())
}

pub fn export_to_env(
    config: &ConfigFile,
    env: &str,
    file_path: &str,
) -> Result<(), PersistenceError> {
    // Validate environment name
    security::validate_environment_name(env).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
    })?;

    // Sanitize file path
    let sanitized_path =
        security::sanitize_file_path(file_path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    let env_config = config
        .environments
        .get(env)
        .ok_or_else(|| PersistenceError::IoError {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Environment '{}' not found", env),
            ),
        })?;

    let mut file = fs::File::create(sanitized_path)?;
    for (key, entry) in &env_config.entries {
        writeln!(file, "{}={}", key, entry.value)?;
    }

    Ok(())
}

pub fn export_to_yaml(
    config: &ConfigFile,
    env: &str,
    file_path: &str,
) -> Result<(), PersistenceError> {
    // Validate environment name
    security::validate_environment_name(env).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
    })?;

    // Sanitize file path
    let sanitized_path =
        security::sanitize_file_path(file_path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    let env_config = config
        .environments
        .get(env)
        .ok_or_else(|| PersistenceError::IoError {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Environment '{}' not found", env),
            ),
        })?;

    let serialized = serde_yaml::to_string(&env_config.entries)?;
    fs::write(sanitized_path, serialized)?;

    Ok(())
}

// Helper function to get encryption key
fn get_encryption_key() -> Result<[u8; 32], PersistenceError> {
    let key_str =
        env::var("NARU_ENCRYPTION_KEY").map_err(|_| PersistenceError::MissingEncryptionKey)?;

    let mut hasher = Sha256::new();
    hasher.update(key_str.as_bytes());
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    Ok(key)
}

// Encrypt a value if it's marked as secret
pub fn encrypt_if_needed(
    config: &mut ConfigFile,
    env: &str,
    key: &str,
) -> Result<(), PersistenceError> {
    if let Some(env_config) = config.environments.get_mut(env)
        && let Some(entry) = env_config.entries.get_mut(key)
    {
        encrypt_entry_value(entry)?;
    }
    Ok(())
}

// Decrypt a value if it's encrypted
pub fn decrypt_if_needed(
    config: &mut ConfigFile,
    env: &str,
    key: &str,
) -> Result<(), PersistenceError> {
    if let Some(env_config) = config.environments.get_mut(env)
        && let Some(entry) = env_config.entries.get_mut(key)
        && entry.encrypted
    {
        let encryption_key = get_encryption_key()?;
        let decrypted_value = crypto::decrypt_data(&entry.value, &encryption_key).map_err(|e| {
            PersistenceError::IoError {
                source: std::io::Error::other(e.to_string()),
            }
        })?;

        entry.value = decrypted_value;
        entry.encrypted = false;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    /// Guard to revert current directory on drop
    struct TestDirGuard {
        original_dir: std::path::PathBuf,
    }

    impl TestDirGuard {
        fn new(temp_path: &Path) -> Self {
            let original_dir = std::env::current_dir().unwrap();
            std::env::set_current_dir(temp_path).unwrap();
            Self { original_dir }
        }
    }

    impl Drop for TestDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    #[test]
    #[serial]
    fn test_save_and_load_json() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());

        fs::create_dir_all(NARU_DIR).unwrap();

        let test_config = ConfigFile {
            project_name: "Test Project".to_string(),
            version: "1.0.0".to_string(),
            environments: std::collections::HashMap::new(),
        };

        save_json(CONFIG_FILE, &test_config).unwrap();
        let loaded_config: ConfigFile = load_json(CONFIG_FILE).unwrap();

        assert_eq!(test_config.project_name, loaded_config.project_name);
        assert_eq!(test_config.version, loaded_config.version);
    }

    #[test]
    #[serial]
    fn test_import_from_env() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());

        init_project().unwrap();

        let env_content = "APP_PORT=8080\nDB_PASS=\"quoted_secret\"\nEMPTY=";
        fs::write("test.env", env_content).unwrap();

        let config = import_from_env("test.env", "development").unwrap();
        let dev_entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(dev_entries.get("APP_PORT").unwrap().value, "8080");
        assert_eq!(dev_entries.get("DB_PASS").unwrap().value, "quoted_secret");
        assert_eq!(dev_entries.get("EMPTY").unwrap().value, "");
    }

    #[test]
    #[serial]
    fn test_import_from_json() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());

        init_project().unwrap();

        let json_content = r###"{"API_KEY": "12345", "DEBUG": true}"###;
        fs::write("test.json", json_content).unwrap();

        let config = import_from_json("test.json", "staging").unwrap();
        let staging_entries = &config.environments.get("staging").unwrap().entries;
        assert_eq!(staging_entries.get("API_KEY").unwrap().value, "12345");
        assert_eq!(staging_entries.get("DEBUG").unwrap().value, "true");
    }

    #[test]
    #[serial]
    fn test_import_from_env_messy() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let env_content = "  KEY1 = val1  \n#comment\nKEY2=\"quoted value\"\n  KEY3='single quoted'  \nINVALID_LINE\nKEY4=value=with=equals";
        fs::write("messy.env", env_content).unwrap();

        let config = import_from_env("messy.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("KEY1").unwrap().value, "val1");
        assert_eq!(entries.get("KEY2").unwrap().value, "quoted value");
        assert_eq!(entries.get("KEY3").unwrap().value, "single quoted");
        assert_eq!(entries.get("KEY4").unwrap().value, "value=with=equals");
        assert!(!entries.contains_key("INVALID_LINE"));
    }

    #[test]
    #[serial]
    fn test_import_unicode() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let json_content = r###"{"GREETING": "你好", "EMOJI": "🚀"}"###;
        fs::write("unicode.json", json_content).unwrap();

        let config = import_from_json("unicode.json", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("GREETING").unwrap().value, "你好");
        assert_eq!(entries.get("EMOJI").unwrap().value, "🚀");
    }

    #[test]
    #[serial]
    fn test_export_to_env_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let mut config: ConfigFile = load_json(CONFIG_FILE).unwrap();
        let env_config = config.environments.get_mut("development").unwrap();
        env_config
            .entries
            .insert("K1".into(), ConfigValueEntry::new("V1", "string", false));
        env_config
            .entries
            .insert("K2".into(), ConfigValueEntry::new("V2", "string", false));

        export_to_env(&config, "development", "exported.env").unwrap();
        let imported = import_from_env("exported.env", "staging").unwrap();
        let staging_entries = &imported.environments.get("staging").unwrap().entries;
        assert_eq!(staging_entries.get("K1").unwrap().value, "V1");
        assert_eq!(staging_entries.get("K2").unwrap().value, "V2");
    }

    #[test]
    #[serial]
    fn test_import_large_file_error() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let large_content = "A".repeat(1024 * 1024 + 1); // 1MB + 1 byte
        fs::write("large.env", large_content).unwrap();

        let result = import_from_env("large.env", "development");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_get_encryption_key_derivation() {
        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "short") };
        let key1 = get_encryption_key().unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "different_longer_key_string") };
        let key2 = get_encryption_key().unwrap();

        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
    }

    #[test]
    #[serial]
    fn test_save_json_sanitization() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Should fail because of traversal attempt in filename
        let result = save_json("../traversal.json", &"data");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_load_non_existent_json() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let result: Result<ConfigFile, _> = load_json("non_existent.json");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_import_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        fs::write("invalid.json", "{ invalid json }").unwrap();
        let result = import_from_json("invalid.json", "development");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_import_from_yaml_complex() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let yaml_content = "KEY1: value1\nKEY2: 123\nKEY3: true\nKEY4: [1, 2, 3]\nKEY5: {a: b}";
        fs::write("complex.yaml", yaml_content).unwrap();

        let config = import_from_yaml("complex.yaml", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("KEY1").unwrap().value, "value1");
        assert_eq!(entries.get("KEY2").unwrap().value, "123");
        assert_eq!(entries.get("KEY3").unwrap().value, "true");
        // Complex types should be skipped according to implementation match
        assert!(!entries.contains_key("KEY4"));
        assert!(!entries.contains_key("KEY5"));
    }

    #[test]
    #[serial]
    fn test_import_to_new_environment() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let json_content = r###"{"NEW_KEY": "val"}"###;
        fs::write("new.json", json_content).unwrap();

        let config = import_from_json("new.json", "totally_new_env").unwrap();
        assert!(config.environments.contains_key("totally_new_env"));
    }

    #[test]
    #[serial]
    fn test_persistence_io_error_simulation() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Try to load a directory as a file
        let dir_path = temp_dir.path().join("some_dir");
        fs::create_dir(&dir_path).unwrap();

        // This should trigger an IO error
        let result: Result<ConfigFile, _> = load_json("some_dir");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_init_project_already_exists_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());

        // First init
        init_project().unwrap();

        // Tamper with audit log
        let log_path = format!("{}/audit.log", NARU_DIR);
        fs::write(&log_path, "corrupted data").unwrap();

        // Second init should fail because of integrity check
        let result = init_project();
        assert!(result.is_err());
    }
}
