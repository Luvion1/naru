use unicode_normalization::UnicodeNormalization;

fn normalize_unicode(input: &str) -> String {
    input.nfc().collect()
}

pub fn validate_environment_name(name: &str) -> Result<(), &'static str> {
    let normalized = normalize_unicode(name);

    if normalized.is_empty() {
        return Err("Environment name cannot be empty");
    }

    if name.contains('\0') {
        return Err("Environment name contains null bytes");
    }

    if !normalized
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "Environment name contains invalid characters. Only alphanumeric, underscore, and hyphen are allowed.",
        );
    }

    if normalized.len() > 100 {
        return Err("Environment name is too long (max 100 characters)");
    }

    Ok(())
}

pub fn validate_config_key(key: &str) -> Result<(), &'static str> {
    if key.contains('\0') {
        return Err("Configuration key contains null bytes");
    }

    let normalized = normalize_unicode(key);

    if normalized.is_empty() {
        return Err("Configuration key cannot be empty");
    }

    if !normalized
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(
            "Configuration key contains invalid characters. Only alphanumeric, underscore, hyphen, and dot are allowed.",
        );
    }

    if normalized.len() > 255 {
        return Err("Configuration key is too long (max 255 characters)");
    }

    Ok(())
}

pub fn normalize_environment_name(name: &str) -> String {
    normalize_unicode(name)
}

pub fn normalize_config_key(key: &str) -> String {
    normalize_unicode(key)
}

pub fn sanitize_string_value(value: &str) -> String {
    value.replace("\0", "")
}

pub fn is_valid_environment_name(name: &str) -> bool {
    validate_environment_name(name).is_ok()
}

pub fn is_valid_config_key(key: &str) -> bool {
    validate_config_key(key).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_environment_name_valid() {
        assert!(validate_environment_name("dev").is_ok());
        assert!(validate_environment_name("production_env").is_ok());
        assert!(validate_environment_name("staging-test").is_ok());
    }

    #[test]
    fn test_validate_environment_name_invalid() {
        assert!(validate_environment_name("").is_err());
        assert!(validate_environment_name("dev;rm -rf /").is_err());
        assert!(validate_environment_name("dev/../../../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_environment_name_length() {
        assert!(validate_environment_name(&"a".repeat(100)).is_ok());
        assert!(validate_environment_name(&"a".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_config_key_valid() {
        assert!(validate_config_key("DB_HOST").is_ok());
        assert!(validate_config_key("api.key").is_ok());
        assert!(validate_config_key("my-key_123").is_ok());
    }

    #[test]
    fn test_validate_config_key_invalid() {
        assert!(validate_config_key("").is_err());
        assert!(validate_config_key("key with spaces").is_err());
        assert!(validate_config_key("key!@#").is_err());
    }

    #[test]
    fn test_validate_config_key_length() {
        assert!(validate_config_key(&"a".repeat(255)).is_ok());
        assert!(validate_config_key(&"a".repeat(256)).is_err());
    }

    #[test]
    fn test_normalize_environment_name() {
        let normalized = normalize_environment_name("test-env");
        assert_eq!(normalized, "test-env");
    }

    #[test]
    fn test_normalize_config_key() {
        let normalized = normalize_config_key("my.key");
        assert_eq!(normalized, "my.key");
    }

    #[test]
    fn test_sanitize_string_value() {
        assert_eq!(sanitize_string_value("hello"), "hello");
        assert_eq!(sanitize_string_value("hel\0lo"), "hello");
    }

    #[test]
    fn test_is_valid_environment_name() {
        assert!(is_valid_environment_name("dev"));
        assert!(!is_valid_environment_name("dev env"));
    }

    #[test]
    fn test_is_valid_config_key() {
        assert!(is_valid_config_key("API_KEY"));
        assert!(!is_valid_config_key("key with space"));
    }

    #[test]
    fn test_validate_environment_injection() {
        assert!(validate_environment_name("dev;rm -rf /").is_err());
        assert!(validate_environment_name("production\n").is_err());
        assert!(validate_environment_name("staging\0").is_err());
    }
}
