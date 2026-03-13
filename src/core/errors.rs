use thiserror::Error;

#[derive(Error, Debug)]
pub enum NaruError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Missing encryption key. Please set NARU_ENCRYPTION_KEY environment variable.")]
    MissingEncryptionKey,

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid environment name: {0}")]
    InvalidEnvironment(String),

    #[error("Environment '{0}' not found.")]
    EnvironmentNotFound(String),

    #[error("Key '{0}' not found in environment '{1}'.")]
    KeyNotFound(String, String),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Schema field '{0}' not found.")]
    SchemaFieldNotFound(String),

    #[error("File error: {0}")]
    File(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Audit log integrity check failed. The log may have been tampered with.")]
    AuditIntegrityFailure,

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("Restore error: {0}")]
    Restore(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("Cryptography error: {0}")]
    Cryptography(String),

    #[error("Project not initialized. Run 'naru init' first.")]
    ProjectNotInitialized,

    #[error("Project already initialized.")]
    ProjectAlreadyInitialized,

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

impl NaruError {
    pub fn config(msg: impl Into<String>) -> Self {
        NaruError::Config(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        NaruError::Validation(msg.into())
    }

    pub fn encryption(msg: impl Into<String>) -> Self {
        NaruError::Encryption(msg.into())
    }

    pub fn decryption(msg: impl Into<String>) -> Self {
        NaruError::Decryption(msg.into())
    }

    pub fn invalid_key(msg: impl Into<String>) -> Self {
        NaruError::InvalidKey(msg.into())
    }

    pub fn invalid_environment(msg: impl Into<String>) -> Self {
        NaruError::InvalidEnvironment(msg.into())
    }

    pub fn environment_not_found(env: impl Into<String>) -> Self {
        NaruError::EnvironmentNotFound(env.into())
    }

    pub fn key_not_found(key: impl Into<String>, env: impl Into<String>) -> Self {
        NaruError::KeyNotFound(key.into(), env.into())
    }

    pub fn schema(msg: impl Into<String>) -> Self {
        NaruError::Schema(msg.into())
    }

    pub fn schema_field_not_found(field: impl Into<String>) -> Self {
        NaruError::SchemaFieldNotFound(field.into())
    }

    pub fn file(msg: impl Into<String>) -> Self {
        NaruError::File(msg.into())
    }

    pub fn security(msg: impl Into<String>) -> Self {
        NaruError::Security(msg.into())
    }

    pub fn lock(msg: impl Into<String>) -> Self {
        NaruError::Lock(msg.into())
    }

    pub fn serialization(msg: impl Into<String>) -> Self {
        NaruError::Serialization(msg.into())
    }

    pub fn deserialization(msg: impl Into<String>) -> Self {
        NaruError::Deserialization(msg.into())
    }

    pub fn backup(msg: impl Into<String>) -> Self {
        NaruError::Backup(msg.into())
    }

    pub fn restore(msg: impl Into<String>) -> Self {
        NaruError::Restore(msg.into())
    }

    pub fn import(msg: impl Into<String>) -> Self {
        NaruError::Import(msg.into())
    }

    pub fn export(msg: impl Into<String>) -> Self {
        NaruError::Export(msg.into())
    }

    pub fn conversion(msg: impl Into<String>) -> Self {
        NaruError::Conversion(msg.into())
    }

    pub fn cryptography(msg: impl Into<String>) -> Self {
        NaruError::Cryptography(msg.into())
    }

    pub fn unsupported_format(format: impl Into<String>) -> Self {
        NaruError::UnsupportedFormat(format.into())
    }

    pub fn invalid_value(msg: impl Into<String>) -> Self {
        NaruError::InvalidValue(msg.into())
    }
}

impl From<NaruError> for std::io::Error {
    fn from(err: NaruError) -> Self {
        std::io::Error::other(err.to_string())
    }
}

impl serde::ser::Error for NaruError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        NaruError::Serialization(msg.to_string())
    }
}

impl serde::de::Error for NaruError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        NaruError::Deserialization(msg.to_string())
    }
}

#[allow(dead_code)]
pub type NaruResult<T> = Result<T, NaruError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = NaruError::environment_not_found("production");
        assert_eq!(err.to_string(), "Environment 'production' not found.");

        let err = NaruError::key_not_found("API_KEY", "development");
        assert_eq!(
            err.to_string(),
            "Key 'API_KEY' not found in environment 'development'."
        );

        let err = NaruError::validation("Invalid email format");
        assert_eq!(err.to_string(), "Validation error: Invalid email format");
    }

    #[test]
    fn test_error_conversion() {
        let io_err: std::io::Error = NaruError::config("test").into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn test_builder_methods() {
        assert_eq!(
            NaruError::config("msg").to_string(),
            "Configuration error: msg"
        );
        assert_eq!(
            NaruError::validation("msg").to_string(),
            "Validation error: msg"
        );
        assert_eq!(
            NaruError::encryption("msg").to_string(),
            "Encryption error: msg"
        );
        assert_eq!(
            NaruError::decryption("msg").to_string(),
            "Decryption error: msg"
        );
        assert_eq!(
            NaruError::invalid_key("msg").to_string(),
            "Invalid key: msg"
        );
        assert_eq!(
            NaruError::invalid_environment("msg").to_string(),
            "Invalid environment name: msg"
        );
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> NaruResult<String> {
            Ok("success".to_string())
        }

        assert!(returns_result().is_ok());
    }
}
