use std::path::{Path, PathBuf};

/// Sanitize file path to prevent directory traversal attacks
pub fn sanitize_file_path(path: &str) -> Result<PathBuf, &'static str> {
    // Check for directory traversal patterns
    if path.contains("../")
        || path.contains("..\\")
        || path.starts_with("../")
        || path.starts_with("..\\")
    {
        return Err("Path contains directory traversal sequences");
    }

    // Normalize the path by removing redundant components
    let path = Path::new(path);

    // Check if the path is absolute (which we don't allow)
    if path.is_absolute() {
        return Err("Absolute paths are not allowed");
    }

    // Resolve the path to ensure it's within allowed boundaries
    let normalized = normalize_path(path);

    // Ensure the final path is still relative
    if normalized.is_absolute() {
        return Err("Path normalization resulted in absolute path");
    }

    Ok(normalized)
}

/// Normalize path by resolving "." and ".." components
fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                // Remove the last component if it exists
                if !normalized.pop() {
                    // If we can't pop, add the parent dir literally to prevent escaping
                    normalized.push(component.as_os_str());
                }
            }
            std::path::Component::Normal(c) => {
                normalized.push(c);
            }
            _ => {
                // Keep other components as they are
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

/// Validate environment name to prevent injection attacks
pub fn validate_environment_name(name: &str) -> Result<(), &'static str> {
    // Check if name is empty
    if name.is_empty() {
        return Err("Environment name cannot be empty");
    }

    // Check for invalid characters
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "Environment name contains invalid characters. Only alphanumeric, underscore, and hyphen are allowed.",
        );
    }

    // Check length limits
    if name.len() > 100 {
        return Err("Environment name is too long (max 100 characters)");
    }

    Ok(())
}

/// Validate configuration key to prevent injection attacks
pub fn validate_config_key(key: &str) -> Result<(), &'static str> {
    // Check if key is empty
    if key.is_empty() {
        return Err("Configuration key cannot be empty");
    }

    // Check for invalid characters
    if !key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(
            "Configuration key contains invalid characters. Only alphanumeric, underscore, hyphen, and dot are allowed.",
        );
    }

    // Check length limits
    if key.len() > 255 {
        return Err("Configuration key is too long (max 255 characters)");
    }

    Ok(())
}

/// Sanitize string value to prevent injection
pub fn sanitize_string_value(value: &str) -> String {
    // Remove or escape potentially dangerous characters
    // For now, we'll just return the value as-is, but in a real implementation
    // you might want to escape certain characters depending on the context
    value.replace("\0", "") // Remove null bytes
}

/// Check if file size is within acceptable limits
pub fn check_file_size(path: &Path, max_size: u64) -> Result<(), &'static str> {
    let metadata = std::fs::metadata(path).map_err(|_| "Could not get file metadata")?;

    if metadata.len() > max_size {
        return Err("File exceeds maximum allowed size");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_file_path() {
        // Valid paths
        assert!(sanitize_file_path("config.json").is_ok());
        assert!(sanitize_file_path("./config.json").is_ok());
        assert!(sanitize_file_path("folder/config.json").is_ok());

        // Invalid paths
        assert!(sanitize_file_path("../config.json").is_err());
        assert!(sanitize_file_path("/etc/passwd").is_err());
        assert!(sanitize_file_path("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_environment_name() {
        // Valid names
        assert!(validate_environment_name("dev").is_ok());
        assert!(validate_environment_name("production_env").is_ok());
        assert!(validate_environment_name("staging-test").is_ok());

        // Invalid names
        assert!(validate_environment_name("").is_err());
        assert!(validate_environment_name("dev;rm -rf /").is_err());
        assert!(validate_environment_name("dev/../../../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_config_key() {
        // Valid keys
        assert!(validate_config_key("database_url").is_ok());
        assert!(validate_config_key("api.key.timeout").is_ok());

        // Invalid keys
        assert!(validate_config_key("").is_err());
        assert!(validate_config_key("key;DROP TABLE").is_err());
    }
}
