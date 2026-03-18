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

#[allow(dead_code)]
pub type NaruResult<T> = Result<T, NaruError>;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = NaruError::EnvironmentNotFound("production".to_string());
        assert_eq!(err.to_string(), "Environment 'production' not found.");

        let err = NaruError::KeyNotFound("API_KEY".to_string(), "development".to_string());
        assert_eq!(
            err.to_string(),
            "Key 'API_KEY' not found in environment 'development'."
        );

        let err = NaruError::Validation("Invalid email format".to_string());
        assert_eq!(err.to_string(), "Validation error: Invalid email format");
    }

    #[test]
    fn test_error_conversion() {
        let io_err: std::io::Error = NaruError::Config("test".to_string()).into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> NaruResult<String> {
            Ok("success".to_string())
        }

        assert!(returns_result().is_ok());
    }
}
