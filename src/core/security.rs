use std::path::{Path, PathBuf};

/// Sanitize file path to prevent directory traversal attacks
pub fn sanitize_file_path(path: &str) -> Result<PathBuf, &'static str> {
    // Check for null bytes which are used in many exploits
    if path.contains('\0') {
        return Err("Path contains null bytes");
    }

    // Unify separators for traversal check
    let unified_path = path.replace('\\', "/");
    let path_obj = Path::new(&unified_path);

    // Check if the path is absolute (which we don't allow)
    if path_obj.is_absolute() {
        return Err("Absolute paths are not allowed");
    }

    // Reject any path that contains ".." as a component
    for component in path_obj.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path contains directory traversal sequences");
        }
    }

    // For the actual path object we return, we use the original path but normalized
    // This is because normalize_path handles the platform-specific separator logic
    let original_path_obj = Path::new(path);
    let normalized = normalize_path(original_path_obj);

    // Final check on normalized path
    for component in normalized.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path attempts to escape parent directory");
        }
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
    fn test_sanitize_file_path_advanced() {
        // Sneaky traversal attempts
        assert!(
            sanitize_file_path("folder/../config.json").is_err(),
            "Should catch internal traversal"
        );
        assert!(sanitize_file_path("./../config.json").is_err());
        assert!(
            sanitize_file_path(".../config.json").is_ok(),
            "Triple dot is technically a valid filename"
        );

        // Null bytes
        assert!(
            sanitize_file_path("config.json\0.txt").is_err(),
            "Null bytes are dangerous"
        );

        // Windows-style traversal
        assert!(sanitize_file_path("folder\\..\\config.json").is_err());
    }

    #[test]
    fn test_validate_environment_name_edge_cases() {
        assert!(validate_environment_name(&"a".repeat(100)).is_ok());
        assert!(
            validate_environment_name(&"a".repeat(101)).is_err(),
            "Too long"
        );
        assert!(
            validate_environment_name("env name").is_err(),
            "Spaces not allowed"
        );
        assert!(
            validate_environment_name("env!").is_err(),
            "Special chars not allowed"
        );
    }

    #[test]
    fn test_sanitize_file_path_extreme() {
        // Deeply nested valid paths
        assert!(sanitize_file_path("a/b/c/d/e/f/g/h/i/j/k/l/m/n/config.json").is_ok());

        // Mixed separators and weird characters
        assert!(sanitize_file_path("my config file (1).json").is_ok());
        assert!(sanitize_file_path("config-2024.01.27.json").is_ok());

        // Attempts to use absolute-like paths in relative form
        assert!(sanitize_file_path("/config.json").is_err());
        assert!(sanitize_file_path("//config.json").is_err());

        // Tilde expansion prevention
        assert!(
            sanitize_file_path("~/config.json").is_ok(),
            "Tilde is a literal character in this context, not expanded"
        );

        // Windows reserved names (though we are on linux, we should check if we handle them)
        assert!(
            sanitize_file_path("CON.json").is_ok(),
            "On Linux CON is just a filename"
        );
    }

    #[test]
    fn test_validate_config_key_extreme() {
        assert!(validate_config_key("a.b.c.d.e.f").is_ok());
        assert!(
            validate_config_key("-").is_ok(),
            "Current logic allows hyphen"
        );
        assert!(validate_config_key("_").is_ok());
        assert!(validate_config_key(".").is_ok(), "Current logic allows dot");
    }

    #[test]
    fn test_check_file_size_logic() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "some data").unwrap();

        assert!(check_file_size(&file_path, 100).is_ok());
        assert!(
            check_file_size(&file_path, 5).is_err(),
            "Should fail if file is larger than max_size"
        );
    }

    #[test]
    fn test_validate_environment_name_injection() {
        assert!(validate_environment_name("dev;rm -rf /").is_err());
        assert!(validate_environment_name("production\n").is_err());
        assert!(validate_environment_name("staging\0").is_err());
    }

    #[test]
    fn test_sanitize_string_value_control_chars() {
        // Only null byte is removed currently. Let's see if we should add others.
        let input = "val\r\n\twith\x07bell";
        assert_eq!(
            sanitize_string_value(input),
            input,
            "Control characters other than null are kept for now"
        );
    }
}
