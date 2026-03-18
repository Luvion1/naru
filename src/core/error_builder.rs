use crate::core::error_kind::NaruError;

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

    pub fn import_err(msg: impl Into<String>) -> Self {
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

    pub fn missing_encryption_key() -> Self {
        NaruError::MissingEncryptionKey
    }

    pub fn project_not_initialized() -> Self {
        NaruError::ProjectNotInitialized
    }

    pub fn project_already_initialized() -> Self {
        NaruError::ProjectAlreadyInitialized
    }

    pub fn audit_integrity_failure() -> Self {
        NaruError::AuditIntegrityFailure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_project_errors() {
        assert_eq!(
            NaruError::project_not_initialized().to_string(),
            "Project not initialized. Run 'naru init' first."
        );
        assert_eq!(
            NaruError::project_already_initialized().to_string(),
            "Project already initialized."
        );
    }

    #[test]
    fn test_audit_error() {
        assert_eq!(
            NaruError::audit_integrity_failure().to_string(),
            "Audit log integrity check failed. The log may have been tampered with."
        );
    }

    #[test]
    fn test_missing_key_error() {
        assert_eq!(
            NaruError::missing_encryption_key().to_string(),
            "Missing encryption key. Please set NARU_ENCRYPTION_KEY environment variable."
        );
    }
}
