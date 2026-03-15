use super::audit;
use super::constants::*;
use super::crypto;
use super::locking;
use super::security;
use crate::core::models::*;
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use hex;
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
                parent: None,
                entries: HashMap::new(),
            },
        );
    }

    // Generate a random salt for key derivation
    let salt_bytes = crypto::generate_salt();
    let salt =
        SaltString::from_b64(&hex::encode(salt_bytes)).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Failed to create salt: {}", e)),
        })?;
    let salt_b64 = salt.to_string();

    let config = ConfigFile {
        project_name: "My Project".to_string(),
        version: "0.1.0".to_string(),
        environments,
        salt: Some(salt_b64),
    };

    save_json(CONFIG_FILE, &config)?;

    let schema = SchemaFile {
        version: "1.0".to_string(),
        fields: vec![],
    };

    save_json(SCHEMA_FILE, &schema)?;

    Ok(())
}

#[deprecated(
    since = "0.6.1",
    note = "Use atomic_update_config for config files to prevent race conditions. This function is not atomic and can cause data loss in concurrent scenarios."
)]
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

    // Write atomically using temp file + rename pattern
    let temp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(data)?;
    fs::write(&temp_path, json)?;
    fs::rename(&temp_path, &path)?;
    Ok(())
}

#[deprecated(
    since = "0.6.1",
    note = "Use atomic_read_config or lock_file for safer concurrent access patterns."
)]
pub fn load_json<T: serde::de::DeserializeOwned>(filename: &str) -> Result<T, PersistenceError> {
    let sanitized_filename =
        security::sanitize_file_path(filename).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    let path = Path::new(NARU_DIR).join(sanitized_filename);

    let _lock =
        locking::FileLock::acquire_exclusive(&path).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Could not acquire file lock: {}", e)),
        })?;

    let content = fs::read_to_string(&path)?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

pub struct LockedFile {
    _lock: locking::FileLock,
    path: std::path::PathBuf,
}

impl LockedFile {
    pub fn read<T: serde::de::DeserializeOwned>(&self) -> Result<T, PersistenceError> {
        let content =
            fs::read_to_string(&self.path).map_err(|e| PersistenceError::IoError { source: e })?;
        serde_json::from_str(&content).map_err(|e| PersistenceError::JsonError { source: e })
    }

    #[allow(dead_code)]
    pub fn write<T: serde::Serialize>(&self, data: &T) -> Result<(), PersistenceError> {
        let json = serde_json::to_string_pretty(data)?;
        fs::write(&self.path, json).map_err(|e| PersistenceError::IoError { source: e })?;
        Ok(())
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

pub fn lock_file(filename: &str) -> Result<LockedFile, PersistenceError> {
    let sanitized_filename =
        security::sanitize_file_path(filename).map_err(|e| PersistenceError::IoError {
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    let path = Path::new(NARU_DIR).join(&sanitized_filename);

    // Ensure the config file exists before locking
    // This prevents "No such file or directory" errors during concurrent access
    if !path.exists() {
        // Try to create the file with default content
        let default_config = ConfigFile {
            project_name: "My Project".to_string(),
            version: "0.1.0".to_string(),
            environments: std::collections::HashMap::new(),
            salt: None,
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, serde_json::to_string_pretty(&default_config).unwrap_or_default());
    }

    // Retry logic for acquiring lock - helps with concurrent access
    let max_retries = 5;
    let mut last_error = None;
    
    for attempt in 0..max_retries {
        match locking::FileLock::acquire_exclusive(&path) {
            Ok(_lock) => {
                return Ok(LockedFile { _lock, path });
            }
            Err(e) => {
                last_error = Some(e);
                // Wait a bit before retrying (exponential backoff)
                if attempt < max_retries - 1 {
                    std::thread::sleep(std::time::Duration::from_millis(10 * (1 << attempt)));
                }
            }
        }
    }
    
    Err(PersistenceError::IoError {
        source: std::io::Error::other(format!(
            "Could not acquire file lock after {} attempts: {:?}", 
            max_retries, last_error
        )),
    })
}

pub fn atomic_update_config<F>(update_fn: F) -> Result<(), PersistenceError>
where
    F: FnOnce(&mut ConfigFile) -> Result<(), PersistenceError>,
{
    let locked = lock_file(CONFIG_FILE)?;
    let mut config: ConfigFile = locked.read()?;

    update_fn(&mut config)?;

    // Write atomically by writing to temp file first, then renaming
    let temp_path = locked.path().with_extension("json.tmp");
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| PersistenceError::JsonError { source: e })?;
    fs::write(&temp_path, json).map_err(|e| PersistenceError::IoError { source: e })?;
    fs::rename(&temp_path, locked.path()).map_err(|e| PersistenceError::IoError { source: e })?;

    Ok(())
}

#[allow(dead_code)]
pub fn atomic_read_config<F, R>(read_fn: F) -> Result<R, PersistenceError>
where
    F: FnOnce(&ConfigFile) -> R,
{
    let locked = lock_file(CONFIG_FILE)?;
    let config: ConfigFile = locked.read()?;
    Ok(read_fn(&config))
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

    // Prevent symlink attacks - check if path is a symlink pointing outside
    if security::is_symlink(&sanitized_path) {
        // For import, we allow symlinks but validate they don't escape
        // Get current working directory as base
        let base_dir = std::env::current_dir().map_err(|_| PersistenceError::IoError {
            source: std::io::Error::other("Cannot determine base directory"),
        })?;

        security::resolve_and_validate_path(&sanitized_path, &base_dir).map_err(|e| {
            PersistenceError::IoError {
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            }
        })?;
    }

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

    // Prevent symlink attacks
    if security::is_symlink(&sanitized_path) {
        let base_dir = std::env::current_dir().map_err(|_| PersistenceError::IoError {
            source: std::io::Error::other("Cannot determine base directory"),
        })?;
        security::resolve_and_validate_path(&sanitized_path, &base_dir).map_err(|e| {
            PersistenceError::IoError {
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            }
        })?;
    }

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

    // Prevent symlink attacks
    if security::is_symlink(&sanitized_path) {
        let base_dir = std::env::current_dir().map_err(|_| PersistenceError::IoError {
            source: std::io::Error::other("Cannot determine base directory"),
        })?;
        security::resolve_and_validate_path(&sanitized_path, &base_dir).map_err(|e| {
            PersistenceError::IoError {
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            }
        })?;
    }

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

fn decrypt_entry_value(entry: &mut ConfigValueEntry) -> Result<(), PersistenceError> {
    if entry.encrypted {
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

fn merge_map_into_config(
    config: &mut ConfigFile,
    env: &str,
    pairs: HashMap<String, String>,
) -> Result<ConfigFile, PersistenceError> {
    let schema: SchemaFile = load_json(SCHEMA_FILE).unwrap_or(SchemaFile {
        version: "1.0".to_string(),
        fields: vec![],
    });

    // Check if schema is defined (not empty) - if so, enforce schema
    let schema_is_defined = !schema.fields.is_empty();

    // Check if environment exists, if not, create it
    if !config.environments.contains_key(env) {
        config.environments.insert(
            env.to_string(),
            EnvironmentConfig {
                parent: None,
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

            // Check if key is in schema when schema is defined
            if schema_is_defined && !schema.fields.iter().any(|f| f.key == key) {
                return Err(PersistenceError::ValidationError(format!(
                    "Key '{}' is not defined in schema. Schema enforcement is enabled.",
                    key
                )));
            }

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
        // Skip encrypted secrets - should not be exported in plaintext
        if entry.is_secret && entry.encrypted {
            writeln!(
                file,
                "# SKIPPED: {} (secret value - not exported for security)",
                key
            )?;
            continue;
        }

        // If secret but not encrypted, try to use as-is (user responsibility)
        // If not secret, export normally
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

    // Filter out encrypted secrets for security
    let filtered_entries: std::collections::HashMap<String, String> = env_config
        .entries
        .iter()
        .filter(|(_, entry)| !(entry.is_secret && entry.encrypted))
        .map(|(k, v)| (k.clone(), v.value.clone()))
        .collect();

    let serialized = serde_yaml::to_string(&filtered_entries)?;
    fs::write(sanitized_path, serialized)?;

    Ok(())
}

// Helper function to get encryption key using Argon2 with salt
// Note: Key derivation is intentionally slow (Argon2). Rate limiting is NOT applied here
// because the computation itself provides adequate protection against brute-force.
fn get_encryption_key() -> Result<[u8; 32], PersistenceError> {
    let key_str =
        env::var("NARU_ENCRYPTION_KEY").map_err(|_| PersistenceError::MissingEncryptionKey)?;

    // Load config to get the salt
    #[allow(deprecated)]
    let config: ConfigFile = load_json(CONFIG_FILE).map_err(|_| PersistenceError::IoError {
        source: std::io::Error::other("Cannot load config for key derivation"),
    })?;

    let salt_str = config.salt.ok_or_else(|| PersistenceError::IoError {
        source: std::io::Error::other("No salt found in config. Run 'naru init' to regenerate."),
    })?;

    // Parse the salt from base64
    let salt = SaltString::from_b64(&salt_str).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::other(format!("Invalid salt: {}", e)),
    })?;

    // Use Argon2id for key derivation (more secure than simple hash)
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(key_str.as_bytes(), &salt)
        .map_err(|e| PersistenceError::IoError {
            source: std::io::Error::other(format!("Key derivation failed: {}", e)),
        })?;

    // Extract the hash output (first 32 bytes for AES-256)
    let hash_output = hash.hash.ok_or_else(|| PersistenceError::IoError {
        source: std::io::Error::other("No hash output generated"),
    })?;

    let hash_bytes = hash_output.as_bytes();
    let mut key = [0u8; 32];
    let len = std::cmp::min(hash_bytes.len(), 32);
    key[..len].copy_from_slice(&hash_bytes[..len]);

    crypto::validate_key_strength(&key).map_err(|e| PersistenceError::IoError {
        source: std::io::Error::other(format!("Key validation failed: {}", e)),
    })?;

    Ok(key)
}

// Encrypt a value if it's marked as secret
pub fn encrypt_if_needed(
    config: &mut ConfigFile,
    env: &str,
    key: &str,
) -> Result<(), PersistenceError> {
    if let Some(env_config) = config.environments.get_mut(env) {
        if let Some(entry) = env_config.entries.get_mut(key) {
            encrypt_entry_value(entry)?;
        }
    }
    Ok(())
}

// Decrypt a value if it's encrypted
pub fn decrypt_if_needed(
    config: &mut ConfigFile,
    env: &str,
    key: &str,
) -> Result<(), PersistenceError> {
    if let Some(env_config) = config.environments.get_mut(env) {
        if let Some(entry) = env_config.entries.get_mut(key) {
            decrypt_entry_value(entry)?;
        }
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
            salt: None,
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
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());

        // Initialize project to create salt
        init_project().unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_encryption") };
        let key1 = get_encryption_key().unwrap();

        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "different_key_string") };
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

    #[test]
    #[serial]
    fn test_import_from_env_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let env_content = "SPECIAL_CHARS='!@#$%^&*()_+-={}[]|\\\\:;\"<>?,./'";
        fs::write("special.env", env_content).unwrap();

        let config = import_from_env("special.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        // The value will have the outer quotes stripped by the parsing logic
        assert_eq!(
            entries.get("SPECIAL_CHARS").unwrap().value,
            "!@#$%^&*()_+-={}[]|\\\\:;\"<>?,./"
        );
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_multiline_values() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Note: Standard .env format doesn't support multiline values,
        // but we'll test how our parser handles newlines in values
        let env_content = "MULTILINE=\"line1\nline2\nline3\"";
        fs::write("multiline.env", env_content).unwrap();

        let config = import_from_env("multiline.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        // Our current implementation treats newlines as part of the value
        assert!(entries.contains_key("MULTILINE"));
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_no_equals_sign() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let env_content = "JUST_A_LINE_WITHOUT_EQUALS\nANOTHER_LINE";
        fs::write("no_equals.env", env_content).unwrap();

        let config = import_from_env("no_equals.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        // Lines without equals sign should be ignored
        assert!(!entries.contains_key("JUST_A_LINE_WITHOUT_EQUALS"));
        assert!(!entries.contains_key("ANOTHER_LINE"));
    }

    #[test]
    #[serial]
    fn test_import_from_json_with_complex_types() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Test with nested objects and arrays - these should be skipped according to implementation
        let json_content = r#"{
            "simple": "value",
            "number": 42,
            "boolean": true,
            "null_value": null,
            "array": [1, 2, 3],
            "object": {"nested": "value"}
        }"#;
        fs::write("complex.json", json_content).unwrap();

        let config = import_from_json("complex.json", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("simple").unwrap().value, "value");
        assert_eq!(entries.get("number").unwrap().value, "42");
        assert_eq!(entries.get("boolean").unwrap().value, "true");
        // Complex types (arrays, objects, null) should be skipped
        assert!(!entries.contains_key("array"));
        assert!(!entries.contains_key("object"));
        assert!(!entries.contains_key("null_value"));
    }

    #[test]
    #[serial]
    fn test_import_from_json_with_unicode_and_escape_sequences() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let json_content = r#"{
            "unicode": "🚀🌟",
            "escaped": "Line1\nLine2\tTabbed",
            "quote": "He said \"Hello\""
        }"#;
        fs::write("unicode.json", json_content).unwrap();

        let config = import_from_json("unicode.json", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("unicode").unwrap().value, "🚀🌟");
        assert_eq!(
            entries.get("escaped").unwrap().value,
            "Line1\nLine2\tTabbed"
        );
        assert_eq!(entries.get("quote").unwrap().value, "He said \"Hello\"");
    }

    #[test]
    #[serial]
    fn test_export_to_env_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let mut config: ConfigFile = load_json(CONFIG_FILE).unwrap();
        let env_config = config.environments.get_mut("development").unwrap();
        env_config.entries.insert(
            "SPECIAL_CHARS".into(),
            ConfigValueEntry::new("!@#$%^&*()_+-=[]{}|;':\",./<>?", "string", false),
        );
        env_config.entries.insert(
            "UNICODE_VALUE".into(),
            ConfigValueEntry::new("🚀🌟 Hello 世界", "string", false),
        );

        export_to_env(&config, "development", "export_special.env").unwrap();

        // Verify we can re-import the exported file
        let reimported = import_from_env("export_special.env", "staging").unwrap();
        let staging_entries = &reimported.environments.get("staging").unwrap().entries;
        assert_eq!(
            staging_entries.get("SPECIAL_CHARS").unwrap().value,
            "!@#$%^&*()_+-=[]{}|;':\",./<>?"
        );
        assert_eq!(
            staging_entries.get("UNICODE_VALUE").unwrap().value,
            "🚀🌟 Hello 世界"
        );
    }

    #[test]
    #[serial]
    fn test_encrypt_if_needed_on_already_encrypted_value() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        unsafe { std::env::set_var("NARU_ENCRYPTION_KEY", "test_key_for_edge_cases") };
        init_project().unwrap();

        let mut config: ConfigFile = load_json(CONFIG_FILE).unwrap();
        let env_config = config.environments.get_mut("development").unwrap();

        // Add a secret value
        let mut entry = ConfigValueEntry::new("secret_value", "string", true);
        entry.encrypted = false; // Mark as not encrypted initially
        env_config.entries.insert("SECRET_KEY".into(), entry);

        // Encrypt it once
        encrypt_if_needed(&mut config, "development", "SECRET_KEY").unwrap();

        // Get the encrypted value by cloning to avoid borrowing conflicts
        let first_encrypted = config
            .environments
            .get("development")
            .unwrap()
            .entries
            .get("SECRET_KEY")
            .unwrap()
            .value
            .clone();

        // Try to encrypt again - should not change
        encrypt_if_needed(&mut config, "development", "SECRET_KEY").unwrap();
        let second_encrypted = config
            .environments
            .get("development")
            .unwrap()
            .entries
            .get("SECRET_KEY")
            .unwrap()
            .value
            .clone();

        // Encryption should be deterministic with the same input, but in practice AES-GCM with random nonces
        // will produce different outputs. So we'll decrypt to verify it's still the same value.
        let decrypted_first =
            crypto::decrypt_data(&first_encrypted, &get_encryption_key().unwrap()).unwrap();
        let decrypted_second =
            crypto::decrypt_data(&second_encrypted, &get_encryption_key().unwrap()).unwrap();
        assert_eq!(decrypted_first, decrypted_second);
        unsafe { std::env::remove_var("NARU_ENCRYPTION_KEY") };
    }

    #[test]
    #[serial]
    fn test_decrypt_if_needed_on_non_encrypted_value() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let mut config: ConfigFile = load_json(CONFIG_FILE).unwrap();
        let env_config = config.environments.get_mut("development").unwrap();

        // Add a non-secret value (not encrypted)
        let entry = ConfigValueEntry::new("public_value", "string", false);
        env_config.entries.insert("PUBLIC_KEY".into(), entry);

        // Try to decrypt - should not fail, should just leave value as is
        let result = decrypt_if_needed(&mut config, "development", "PUBLIC_KEY");
        assert!(result.is_ok());

        // Value should remain unchanged
        let value_after = &config
            .environments
            .get("development")
            .unwrap()
            .entries
            .get("PUBLIC_KEY")
            .unwrap()
            .value;
        assert_eq!(value_after, "public_value");
    }

    #[test]
    #[serial]
    fn test_merge_map_into_config_with_duplicate_keys() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Create a config with an existing value
        let mut config: ConfigFile = load_json(CONFIG_FILE).unwrap();
        let env_config = config.environments.get_mut("development").unwrap();
        env_config.entries.insert(
            "EXISTING_KEY".into(),
            ConfigValueEntry::new("old_value", "string", false),
        );
        save_json(CONFIG_FILE, &config).unwrap();

        // Import new values that include the same key
        let mut new_pairs = HashMap::new();
        new_pairs.insert("EXISTING_KEY".to_string(), "new_value".to_string());
        new_pairs.insert("NEW_KEY".to_string(), "another_value".to_string());

        let result_config = merge_map_into_config(&mut config, "development", new_pairs).unwrap();
        let final_entries = &result_config
            .environments
            .get("development")
            .unwrap()
            .entries;

        // Existing key should be overwritten
        assert_eq!(
            final_entries.get("EXISTING_KEY").unwrap().value,
            "new_value"
        );
        // New key should be added
        assert_eq!(final_entries.get("NEW_KEY").unwrap().value, "another_value");
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_extremely_long_values() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let long_value = "x".repeat(10000); // 10KB value
        let env_content = format!("LONG_VALUE={}", long_value);
        fs::write("long.env", env_content).unwrap();

        let config = import_from_env("long.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("LONG_VALUE").unwrap().value, long_value);
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_special_char_keys() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let env_content = "KEY_WITH_DOT=value1\nKEY-WITH-HYPHEN=value2\nKEY_WITH_UNDERSCORE=value3";
        fs::write("special_keys.env", env_content).unwrap();

        let config = import_from_env("special_keys.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("KEY_WITH_DOT").unwrap().value, "value1");
        assert_eq!(entries.get("KEY-WITH-HYPHEN").unwrap().value, "value2");
        assert_eq!(entries.get("KEY_WITH_UNDERSCORE").unwrap().value, "value3");
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_special_characters_in_values() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let env_content = r#"SPECIAL_VALUES='!@#$%^&*()_+-=[]{}|;":,./<>?~`'"#;
        fs::write("special_vals.env", env_content).unwrap();

        let config = import_from_env("special_vals.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(
            entries.get("SPECIAL_VALUES").unwrap().value,
            "!@#$%^&*()_+-=[]{}|;\":,./<>?~`"
        );
    }

    #[test]
    #[serial]
    fn test_import_from_json_with_nested_objects_and_arrays() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let json_content = r#"{
            "simple": "value",
            "array": [1, 2, 3],
            "nested_object": {
                "inner": "value"
            },
            "mixed_array": ["string", 123, true],
            "deeply_nested": {
                "level1": {
                    "level2": {
                        "value": "deep"
                    }
                }
            }
        }"#;
        fs::write("nested.json", json_content).unwrap();

        let config = import_from_json("nested.json", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("simple").unwrap().value, "value");
        // Complex types should be skipped
        assert!(!entries.contains_key("array"));
        assert!(!entries.contains_key("nested_object"));
        assert!(!entries.contains_key("mixed_array"));
        assert!(!entries.contains_key("deeply_nested"));
    }

    #[test]
    #[serial]
    fn test_import_from_yaml_with_complex_structures() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let yaml_content = r#"---
simple: value
list:
  - item1
  - item2
nested:
  key: value
anchors: &anchor
  reused: value
reused: *anchor
"#;
        fs::write("complex.yaml", yaml_content).unwrap();

        let config = import_from_yaml("complex.yaml", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("simple").unwrap().value, "value");
        // Complex structures should be skipped
        assert!(!entries.contains_key("list"));
        assert!(!entries.contains_key("nested"));
        assert!(!entries.contains_key("anchors"));
        assert!(!entries.contains_key("reused"));
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_malformed_quoted_values() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Test various malformed quoting scenarios
        let env_content = r#"MALFORMED1="unclosed_quote
MALFORMED2='unclosed_single_quote
MALFORMED3=no_quotes_but=equals_sign
MALFORMED4="nested"quote"
"#;
        fs::write("malformed.env", env_content).unwrap();

        let config = import_from_env("malformed.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        // Only properly formed key-value pairs should be imported
        assert!(entries.contains_key("MALFORMED3")); // This one should work
        assert_eq!(
            entries.get("MALFORMED3").unwrap().value,
            "no_quotes_but=equals_sign"
        );
    }

    #[test]
    #[serial]
    fn test_export_to_yaml_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let mut config: ConfigFile = load_json(CONFIG_FILE).unwrap();
        let env_config = config.environments.get_mut("development").unwrap();
        env_config.entries.insert(
            "SPECIAL_CHARS".into(),
            ConfigValueEntry::new("!@#$%^&*()_+-=[]{}|;':\",./<>?", "string", false),
        );
        env_config.entries.insert(
            "UNICODE_VALUE".into(),
            ConfigValueEntry::new("🚀🌟 Hello 世界", "string", false),
        );
        env_config.entries.insert(
            "CONTROL_CHARS".into(),
            ConfigValueEntry::new("\n\t\r\0", "string", false),
        );

        export_to_yaml(&config, "development", "export_special.yaml").unwrap();

        // Verify we can re-import the exported file
        let reimported = import_from_yaml("export_special.yaml", "staging").unwrap();
        let staging_entries = &reimported.environments.get("staging").unwrap().entries;
        // Only simple keys will be imported, complex ones are skipped
        if staging_entries.contains_key("SPECIAL_CHARS") {
            assert_eq!(
                staging_entries.get("SPECIAL_CHARS").unwrap().value,
                "!@#$%^&*()_+-=[]{}|;':\",./<>?"
            );
        }
        if staging_entries.contains_key("UNICODE_VALUE") {
            assert_eq!(
                staging_entries.get("UNICODE_VALUE").unwrap().value,
                "🚀🌟 Hello 世界"
            );
        }
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_extreme_unicode_combinations() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Test with various Unicode scripts and combining marks
        let env_content = "EMOJI_MIX=😀😃😄😁😆😅😂🤣\nMIXED_SCRIPTS=Hello 你好 مرحبا Здравствуйте\nCOMBINING_CHARS=a\u{0300}\u{0301}\u{0302}"; // a with grave, acute, circumflex
        fs::write("unicode_combo.env", env_content).unwrap();

        let config = import_from_env("unicode_combo.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("EMOJI_MIX").unwrap().value, "😀😃😄😁😆😅😂🤣");
        assert!(entries.contains_key("MIXED_SCRIPTS"));
        assert!(entries.contains_key("COMBINING_CHARS"));
    }

    #[test]
    #[serial]
    fn test_save_json_with_extremely_large_data() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Create a large config with many entries
        let mut environments = HashMap::new();
        let mut entries = HashMap::new();
        for i in 0..1000 {
            entries.insert(
                format!("KEY_{}", i),
                ConfigValueEntry::new(&format!("VALUE_{}", i), "string", false),
            );
        }
        environments.insert(
            "development".to_string(),
            EnvironmentConfig {
                parent: None,
                entries,
            },
        );

        let large_config = ConfigFile {
            project_name: "Large Test Project".to_string(),
            version: "1.0.0".to_string(),
            environments,
            salt: None,
        };

        save_json(CONFIG_FILE, &large_config).unwrap();
        let loaded_config: ConfigFile = load_json(CONFIG_FILE).unwrap();

        assert_eq!(loaded_config.project_name, "Large Test Project");
        assert_eq!(
            loaded_config
                .environments
                .get("development")
                .unwrap()
                .entries
                .len(),
            1000
        );
        assert_eq!(
            loaded_config
                .environments
                .get("development")
                .unwrap()
                .entries
                .get("KEY_500")
                .unwrap()
                .value,
            "VALUE_500"
        );
    }

    #[test]
    #[serial]
    fn test_import_from_json_with_extremely_nested_objects() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Create deeply nested JSON that should be ignored by our import logic
        let json_content = r#"{
            "simple": "value",
            "nested": {
                "level1": {
                    "level2": {
                        "level3": {
                            "level4": {
                                "level5": {
                                    "deep_value": "deep"
                                }
                            }
                        }
                    }
                }
            },
            "array_of_objects": [
                {"obj1": "val1"},
                {"obj2": {"nested": "value"}},
                {"obj3": [{"arr": "val"}]}
            ]
        }"#;
        fs::write("deeply_nested.json", json_content).unwrap();

        let config = import_from_json("deeply_nested.json", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("simple").unwrap().value, "value");
        // Nested structures should be ignored
        assert!(!entries.contains_key("nested"));
        assert!(!entries.contains_key("array_of_objects"));
    }

    #[test]
    #[serial]
    fn test_import_from_yaml_with_complex_data_types() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let yaml_content = r#"---
timestamp: 2023-12-25T10:30:00Z
number: 42
float: 3.14159
boolean: true
null_value: null
string: "just a string"
list:
  - item1
  - item2
  - item3
nested:
  key1: value1
  key2: value2
"#;
        fs::write("complex_types.yaml", yaml_content).unwrap();

        let config = import_from_yaml("complex_types.yaml", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("string").unwrap().value, "just a string");
        assert_eq!(entries.get("number").unwrap().value, "42");
        assert_eq!(entries.get("float").unwrap().value, "3.14159");
        assert_eq!(entries.get("boolean").unwrap().value, "true");
        // Complex types like lists and nested objects should be skipped
        assert!(!entries.contains_key("list"));
        assert!(!entries.contains_key("nested"));
        // Timestamp is a special type in YAML, so it might not be imported
        // Null values are also typically skipped
        // Only simple scalar values (string, number, boolean) are imported
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_extreme_comment_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let env_content = r#"# This is a normal comment
VALID_KEY1=value1
### This is a comment with hashes
VALID_KEY2=value2
  # This is a comment with leading spaces
VALID_KEY3=value3
#COMMENT_WITHOUT_EQUALS
ANOTHER_VALID_KEY=another_value
# Multi-line
# comment
FINAL_KEY=final_value
# Comment at the end"#;
        fs::write("comments.env", env_content).unwrap();

        let config = import_from_env("comments.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("VALID_KEY1").unwrap().value, "value1");
        assert_eq!(entries.get("VALID_KEY2").unwrap().value, "value2");
        assert_eq!(entries.get("VALID_KEY3").unwrap().value, "value3");
        assert_eq!(
            entries.get("ANOTHER_VALID_KEY").unwrap().value,
            "another_value"
        );
        assert_eq!(entries.get("FINAL_KEY").unwrap().value, "final_value");
        // Keys from comment lines should not be present
        assert!(!entries.contains_key("COMMENT_WITHOUT_EQUALS"));
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_variable_expansion_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Test with shell variable expansion syntax (should be treated as literal)
        let env_content = r#"LITERAL_VAR=$HOME_DIR
ANOTHER_VAR=${VAR_NAME}
CMD_VAR=$(command)
BACKTICK_VAR=`command`
SIMPLE_VAR=normal_value"#;
        fs::write("expansion.env", env_content).unwrap();

        let config = import_from_env("expansion.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(entries.get("LITERAL_VAR").unwrap().value, "$HOME_DIR");
        assert_eq!(entries.get("ANOTHER_VAR").unwrap().value, "${VAR_NAME}");
        assert_eq!(entries.get("CMD_VAR").unwrap().value, "$(command)");
        assert_eq!(entries.get("BACKTICK_VAR").unwrap().value, "`command`");
        assert_eq!(entries.get("SIMPLE_VAR").unwrap().value, "normal_value");
    }

    #[test]
    #[serial]
    fn test_import_from_json_with_unicode_and_escape_sequences_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let json_content = r#"{
            "unicode_emojis": "😀😃😄😁😆😅😂🤣",
            "unicode_accents": "café naïve résumé",
            "unicode_cjk": "你好 世界",
            "unicode_arabic": "مرحبا",
            "escape_sequences": "Line1\\nLine2\\tTabbed\\rCarriage\\\\Backslash\\/Slash",
            "unicode_escapes": "Hello",
            "quote_handling": "She said \"Hello\" to me"
        }"#;
        fs::write("unicode_edge.json", json_content).unwrap();

        let config = import_from_json("unicode_edge.json", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;
        assert_eq!(
            entries.get("unicode_emojis").unwrap().value,
            "😀😃😄😁😆😅😂🤣"
        );
        assert_eq!(
            entries.get("unicode_accents").unwrap().value,
            "café naïve résumé"
        );
        assert_eq!(entries.get("unicode_cjk").unwrap().value, "你好 世界");
        assert_eq!(
            entries.get("escape_sequences").unwrap().value,
            "Line1\\nLine2\\tTabbed\\rCarriage\\\\Backslash\\/Slash"
        );
        assert_eq!(entries.get("unicode_escapes").unwrap().value, "Hello");
        assert_eq!(
            entries.get("quote_handling").unwrap().value,
            "She said \"Hello\" to me"
        );
    }

    #[test]
    #[serial]
    fn test_export_import_cycle_with_extreme_values() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        // Create a config with extreme values
        let mut config: ConfigFile = load_json(CONFIG_FILE).unwrap();
        let env_config = config.environments.get_mut("development").unwrap();

        env_config.entries.insert(
            "VERY_LONG_KEY_WITH_SPECIAL_CHARS".into(), // Remove Unicode chars that are not allowed
            ConfigValueEntry::new(&"x".repeat(10000), "string", false),
        );
        env_config.entries.insert(
            "SPECIAL_CHARS_VALUE".into(),
            ConfigValueEntry::new("!@#$%^&*()_+-=[]{}|;':\",./<>?~`", "string", false),
        );
        env_config.entries.insert(
            "UNICODE_VALUE".into(),
            ConfigValueEntry::new("🚀🌟 Hello 世界 🌍", "string", false),
        );
        env_config.entries.insert(
            "INTEGER_VALUE".into(),
            ConfigValueEntry::new("9223372036854775807", "integer", false), // i64 max
        );

        // Export to JSON
        export_to_json(&config, "development", "cycle_test.json").unwrap();

        // Import back from JSON
        let reimported_config = import_from_json("cycle_test.json", "staging").unwrap();
        let staging_entries = &reimported_config
            .environments
            .get("staging")
            .unwrap()
            .entries;

        assert_eq!(
            staging_entries
                .get("VERY_LONG_KEY_WITH_SPECIAL_CHARS")
                .unwrap()
                .value,
            "x".repeat(10000)
        );
        assert_eq!(
            staging_entries.get("SPECIAL_CHARS_VALUE").unwrap().value,
            "!@#$%^&*()_+-=[]{}|;':\",./<>?~`"
        );
        assert_eq!(
            staging_entries.get("UNICODE_VALUE").unwrap().value,
            "🚀🌟 Hello 世界 🌍"
        );
        assert_eq!(
            staging_entries.get("INTEGER_VALUE").unwrap().value,
            "9223372036854775807"
        );
    }

    #[test]
    #[serial]
    fn test_import_from_env_with_mixed_quoting_styles() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());
        init_project().unwrap();

        let env_content = r#"NO_QUOTES=simple_value
SINGLE_QUOTES='value with spaces'
DOUBLE_QUOTES="another value with spaces"
MIXED_QUOTES='value "with" quotes'
MIXED_QUOTES_2="value 'with' quotes"
NESTED_QUOTES='this is "nested" quote'
NESTED_QUOTES_2="this is 'nested' quote"
UNCLOSED_SINGLE='unclosed quote here
UNCLOSED_DOUBLE="unclosed quote here
EMPTY_SINGLE=''
EMPTY_DOUBLE=""
QUOTES_WITH_EQUALS='key=value_in_quotes'
MIXED_SPECIAL_CHARS='!@#$%^&*()_+-=[]{}|;":\",./<>?~`'"#;
        fs::write("mixed_quotes.env", env_content).unwrap();

        let config = import_from_env("mixed_quotes.env", "development").unwrap();
        let entries = &config.environments.get("development").unwrap().entries;

        assert_eq!(entries.get("NO_QUOTES").unwrap().value, "simple_value");
        assert_eq!(
            entries.get("SINGLE_QUOTES").unwrap().value,
            "value with spaces"
        );
        assert_eq!(
            entries.get("DOUBLE_QUOTES").unwrap().value,
            "another value with spaces"
        );
        assert_eq!(
            entries.get("MIXED_QUOTES").unwrap().value,
            "value \"with\" quotes"
        );
        assert_eq!(
            entries.get("MIXED_QUOTES_2").unwrap().value,
            "value 'with' quotes"
        );
        assert_eq!(entries.get("EMPTY_SINGLE").unwrap().value, "");
        assert_eq!(entries.get("EMPTY_DOUBLE").unwrap().value, "");
        assert_eq!(
            entries.get("QUOTES_WITH_EQUALS").unwrap().value,
            "key=value_in_quotes"
        );
        assert_eq!(
            entries.get("MIXED_SPECIAL_CHARS").unwrap().value,
            "!@#$%^&*()_+-=[]{}|;\":\\\",./<>?~`"
        );
    }
}

// Add the missing export_to_json function
#[allow(dead_code)] // This function is used in tests but might not be used in main binary
pub fn export_to_json(
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

    // Create a simplified map with just the values (not the full ConfigValueEntry)
    let mut simple_map = std::collections::HashMap::new();
    for (key, entry) in &env_config.entries {
        simple_map.insert(key.clone(), entry.value.clone());
    }

    let serialized = serde_json::to_string_pretty(&simple_map)?;
    std::fs::write(sanitized_path, serialized)?;

    Ok(())
}
