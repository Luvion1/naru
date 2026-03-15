use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

fn normalize_unicode(input: &str) -> String {
    input.nfc().collect()
}

/// Sanitize file path to prevent directory traversal attacks
/// This version rejects absolute paths - suitable for user input
pub fn sanitize_file_path(path: &str) -> Result<PathBuf, &'static str> {
    // Check for null bytes which are used in many exploits
    if path.contains('\0') {
        return Err("Path contains null bytes");
    }

    // Unify separators for traversal check
    let unified_path = path.replace('\\', "/");

    // Reject absolute paths
    if unified_path.starts_with('/') || unified_path.starts_with("//") {
        return Err("Absolute paths are not allowed");
    }

    // Check for traversal patterns in the unified path
    // Split by separator and check each component
    for component in unified_path.split('/') {
        if component == ".." {
            return Err("Path contains directory traversal sequences");
        }
    }

    // For the actual path object we return, we use the original path but normalized
    // This is because normalize_path handles the platform-specific separator logic
    let original_path_obj = Path::new(path);
    let normalized = normalize_path(original_path_obj);

    // Final check on normalized path for directory traversal
    for component in normalized.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path attempts to escape parent directory");
        }
    }

    Ok(normalized)
}

/// Sanitize file path allowing absolute paths - for internal use
/// This version allows absolute paths but still prevents directory traversal
pub fn sanitize_file_path_internal(path: &str) -> Result<PathBuf, &'static str> {
    // Check for null bytes which are used in many exploits
    if path.contains('\0') {
        return Err("Path contains null bytes");
    }

    // Unify separators for traversal check
    let unified_path = path.replace('\\', "/");

    // Check for traversal patterns in the unified path
    // Split by separator and check each component
    for component in unified_path.split('/') {
        if component == ".." {
            return Err("Path contains directory traversal sequences");
        }
    }

    // For the actual path object we return, we use the original path but normalized
    let original_path_obj = Path::new(path);
    let normalized = normalize_path(original_path_obj);

    // Final check on normalized path for directory traversal
    for component in normalized.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path attempts to escape parent directory");
        }
    }

    Ok(normalized)
}

/// Check if a path is a symlink (to prevent symlink attacks)
pub fn is_symlink(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => meta.file_type().is_symlink(),
        Err(_) => false,
    }
}

/// Resolve symlink and check if it points outside allowed directory
pub fn resolve_and_validate_path(path: &Path, base_dir: &Path) -> Result<PathBuf, &'static str> {
    // First check if it's a symlink
    if is_symlink(path) {
        // Resolve the symlink
        let resolved = std::fs::read_link(path).map_err(|_| "Cannot resolve symlink")?;

        // Check if resolved path is outside base directory
        let resolved_abs = if resolved.is_absolute() {
            resolved
        } else {
            // Make it absolute relative to the parent directory of the original path
            if let Some(parent) = path.parent() {
                parent.join(&resolved)
            } else {
                resolved
            }
        };

        // Normalize and check if it's within base_dir
        let canonical_base =
            std::fs::canonicalize(base_dir).map_err(|_| "Cannot access base directory")?;
        let canonical_resolved =
            std::fs::canonicalize(&resolved_abs).map_err(|_| "Cannot resolve symlink target")?;

        // Check if resolved path starts with base directory
        if !canonical_resolved.starts_with(&canonical_base) {
            return Err("Symlink points outside allowed directory");
        }

        return Ok(canonical_resolved);
    }

    // Not a symlink, just return canonicalized path
    std::fs::canonicalize(path).map_err(|_| "Cannot access path")
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
/// Returns the normalized form for consistent storage
pub fn validate_environment_name(name: &str) -> Result<(), &'static str> {
    // Normalize Unicode to NFC form to prevent bypass via different encodings
    let normalized = normalize_unicode(name);

    // Check if name is empty
    if normalized.is_empty() {
        return Err("Environment name cannot be empty");
    }

    // Check for null bytes (must check before normalization)
    if name.contains('\0') {
        return Err("Environment name contains null bytes");
    }

    // Check for invalid characters
    if !normalized
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "Environment name contains invalid characters. Only alphanumeric, underscore, and hyphen are allowed.",
        );
    }

    // Check length limits
    if normalized.len() > 100 {
        return Err("Environment name is too long (max 100 characters)");
    }

    Ok(())
}

/// Validate configuration key to prevent injection attacks
/// Returns the normalized form for consistent storage
pub fn validate_config_key(key: &str) -> Result<(), &'static str> {
    // Check for null bytes first (before any processing)
    if key.contains('\0') {
        return Err("Configuration key contains null bytes");
    }

    // Normalize Unicode to NFC form to prevent bypass via different encodings
    let normalized = normalize_unicode(key);

    // Check if key is empty
    if normalized.is_empty() {
        return Err("Configuration key cannot be empty");
    }

    // Check for invalid characters
    if !normalized
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(
            "Configuration key contains invalid characters. Only alphanumeric, underscore, hyphen, and dot are allowed.",
        );
    }

    // Check length limits (using normalized string)
    if normalized.len() > 255 {
        return Err("Configuration key is too long (max 255 characters)");
    }

    Ok(())
}

/// Sanitize and normalize a configuration key for storage
/// Returns the NFC-normalized form for consistent comparison
#[allow(dead_code)]
pub fn normalize_config_key(key: &str) -> String {
    normalize_unicode(key)
}

/// Sanitize and normalize an environment name for storage
/// Returns the NFC-normalized form for consistent comparison
#[allow(dead_code)]
pub fn normalize_environment_name(name: &str) -> String {
    normalize_unicode(name)
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

    #[test]
    fn test_validate_environment_name_with_unicode() {
        // Note: is_alphanumeric() in Rust accepts some Unicode letters but not symbols like emojis
        assert!(validate_environment_name("env_🚀").is_err()); // Emoji not allowed by is_alphanumeric
        assert!(validate_environment_name("env_ümlaut").is_ok()); // Accented characters allowed by is_alphanumeric
        assert!(validate_environment_name("env_123").is_ok()); // Numbers allowed
    }

    #[test]
    fn test_validate_config_key_edge_cases() {
        assert!(validate_config_key(&"a".repeat(255)).is_ok()); // Max length
        assert!(validate_config_key(&"a".repeat(256)).is_err()); // Over max length
        assert!(validate_config_key("key with spaces").is_err()); // Spaces not allowed
        assert!(validate_config_key("key!@#").is_err()); // Special chars not allowed
        assert!(validate_config_key("valid.key-name_123").is_ok()); // Valid chars
    }

    #[test]
    fn test_sanitize_file_path_unicode_and_special_chars() {
        assert!(sanitize_file_path("config_🚀.json").is_ok()); // Unicode in filename is OK
        assert!(sanitize_file_path("file\nwith\tnewline.json").is_ok()); // Control chars in filename OK
        assert!(sanitize_file_path("file\0with_null.json").is_err()); // Null byte not OK
    }

    #[test]
    fn test_sanitize_file_path_deeply_nested_traversal() {
        let deep_path = (0..100).map(|_| "dir").collect::<Vec<_>>().join("/") + "/file.txt";
        assert!(sanitize_file_path(&deep_path).is_ok()); // Deep nesting without traversal is OK

        let traversal_path = (0..50).map(|_| "../").collect::<Vec<_>>().join("") + "etc/passwd";
        assert!(sanitize_file_path(&traversal_path).is_err()); // Deep traversal should fail
    }

    #[test]
    fn test_sanitize_file_path_mixed_separators_traversal() {
        assert!(sanitize_file_path("a/b\\c/../d").is_err()); // Mixed separators with traversal
        assert!(sanitize_file_path("a/b\\c/../../d").is_err()); // More complex mixed traversal
        assert!(sanitize_file_path("a/b\\c/d").is_ok()); // Mixed separators without traversal
    }

    #[test]
    fn test_validate_environment_name_unicode_normalization() {
        // Test homograph attacks or similar
        assert!(validate_environment_name("аpple").is_ok()); // Cyrillic 'a' - should be OK since we allow unicode chars in names
        assert!(validate_environment_name("test-env_123").is_ok()); // Normal case
    }

    #[test]
    fn test_sanitize_file_path_url_encoded_traversal() {
        // These should be treated as literal strings, not decoded
        assert!(sanitize_file_path("config%2Ejson").is_ok()); // Literal %2E should be OK
        assert!(sanitize_file_path("normal/config.json").is_ok()); // Normal path
    }

    #[test]
    fn test_validate_config_key_with_unicode() {
        // Note: is_alphanumeric() in Rust accepts some Unicode letters but not symbols like emojis
        assert!(validate_config_key("key_🚀").is_err()); // Emoji not allowed by is_alphanumeric
        assert!(validate_config_key("key_ümlaut").is_ok()); // Accented characters allowed by is_alphanumeric
        assert!(validate_config_key("valid.key-name_123").is_ok()); // Valid chars only
    }

    #[test]
    fn test_sanitize_file_path_symlink_like_patterns() {
        // These are file names, not actual symlinks, so they should be OK if no traversal
        assert!(sanitize_file_path("symlink_target").is_ok());
        assert!(sanitize_file_path("link->target").is_ok()); // Arrow chars are OK as literals
        assert!(sanitize_file_path("../link->target").is_err()); // But with traversal not OK
    }

    #[test]
    fn test_validate_config_key_with_extreme_unicode() {
        // Test with various Unicode categories
        assert!(validate_config_key("key_αβγδε").is_ok()); // Greek letters
        assert!(validate_config_key("key_Здравствуйте").is_ok()); // Cyrillic (alphanumeric in Rust)
        assert!(validate_config_key("key_🚀").is_err()); // Emoji (not alphanumeric)
        assert!(validate_config_key("key_café_naïve").is_ok()); // Accented Latin characters
                                                                // Arabic letters are not considered alphanumeric in Rust's is_alphanumeric
                                                                // so they should be rejected
    }

    #[test]
    fn test_validate_environment_name_with_extreme_cases() {
        // Test with maximum allowed length
        let max_len_name = "a".repeat(100);
        assert!(validate_environment_name(&max_len_name).is_ok());

        // Test with one character over the limit
        let over_limit_name = "a".repeat(101);
        assert!(validate_environment_name(&over_limit_name).is_err());

        // Test with various valid characters
        assert!(validate_environment_name("dev_env-test_123").is_ok());

        // Test with invalid characters
        assert!(validate_environment_name("dev env").is_err()); // Space
        assert!(validate_environment_name("dev/env").is_err()); // Forward slash
        assert!(validate_environment_name("dev\\env").is_err()); // Backslash
        assert!(validate_environment_name("dev@env").is_err()); // At symbol
    }

    #[test]
    fn test_sanitize_file_path_with_extreme_unicode() {
        // Test with Unicode filenames
        assert!(sanitize_file_path("config_🚀.json").is_ok());
        assert!(sanitize_file_path("file_αβγ.txt").is_ok());
        assert!(sanitize_file_path("你好世界.cfg").is_ok());

        // Unicode traversal attempts
        assert!(sanitize_file_path("config_🚀/../passwd").is_err());
        assert!(sanitize_file_path("αβγ/../file.txt").is_err());
    }

    #[test]
    fn test_sanitize_file_path_with_percent_encoding() {
        // Test percent-encoded traversal (should be treated literally)
        assert!(sanitize_file_path("config%2Ejson").is_ok()); // %2E is not processed as .
        assert!(sanitize_file_path("%2E%2E%2Fetc%2Fpasswd").is_ok()); // Should be treated as literal
        assert!(sanitize_file_path("normal/file.txt").is_ok());
    }

    #[test]
    fn test_sanitize_file_path_with_alternative_encodings() {
        // Test various alternative encodings that might be used for traversal
        assert!(sanitize_file_path("config.json").is_ok());
        // These should be treated as literal strings, not processed as traversal
        assert!(sanitize_file_path("..%2ftest").is_ok()); // ..%2f should be literal
        assert!(sanitize_file_path("..%5Ctest").is_ok()); // ..%5C should be literal
    }

    #[test]
    fn test_validate_config_key_with_edge_character_combinations() {
        // Test various combinations of allowed characters
        assert!(validate_config_key("a_b.c-d").is_ok()); // All allowed separators
        assert!(validate_config_key("a..b").is_ok()); // Multiple dots
        assert!(validate_config_key("a__b").is_ok()); // Multiple underscores
        assert!(validate_config_key("a--b").is_ok()); // Multiple hyphens
        assert!(validate_config_key("a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z").is_ok()); // Many dots

        // Test with disallowed characters
        assert!(validate_config_key("a@b").is_err()); // At symbol
        assert!(validate_config_key("a!b").is_err()); // Exclamation
        assert!(validate_config_key("a b").is_err()); // Space
        assert!(validate_config_key("a[b]").is_err()); // Brackets
    }

    #[test]
    fn test_sanitize_string_value_with_extreme_cases() {
        // Test with various control characters
        let input_with_controls =
            "value\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
        let sanitized = sanitize_string_value(input_with_controls);
        assert!(!sanitized.contains('\0')); // Null byte should be removed
                                            // Other control characters should remain
        assert!(sanitized.contains('\x01'));
        assert!(sanitized.contains('\x02'));
        assert!(sanitized.contains('\t')); // Tab should remain
        assert!(sanitized.contains('\n')); // Newline should remain

        // Test with Unicode and special characters
        let unicode_input = "value_with_🚀_and_üñíçødé";
        let sanitized_unicode = sanitize_string_value(unicode_input);
        assert_eq!(sanitized_unicode, unicode_input); // Should remain unchanged
    }

    #[test]
    fn test_validate_environment_name_with_homoglyphs() {
        // Test potential homograph attacks
        assert!(validate_environment_name("аpple").is_ok()); // Cyrillic 'a' - should be OK
        assert!(validate_environment_name("аdmin").is_ok()); // Another Cyrillic test
        assert!(validate_environment_name("test-env_123").is_ok()); // Normal case
    }

    #[test]
    fn test_sanitize_file_path_with_mixed_encoding_tricks() {
        // Test various encoding tricks that might bypass filters
        assert!(sanitize_file_path("././config.json").is_ok()); // Double current dir
        assert!(sanitize_file_path("./../config.json").is_err()); // Current + parent
        assert!(sanitize_file_path(".../config.json").is_ok()); // Triple dot (valid filename)
        assert!(sanitize_file_path("....//config.json").is_ok()); // Double dot followed by slash - this resolves to parent
        assert!(sanitize_file_path("config.json.").is_ok()); // Trailing dot
        assert!(sanitize_file_path("config.json..").is_ok()); // Trailing double dot
    }

    #[test]
    fn test_check_file_size_with_edge_cases() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Test with exactly the max size
        let exact_size_file = temp_dir.path().join("exact_size.txt");
        std::fs::write(&exact_size_file, &vec![0u8; 1000]).unwrap();
        assert!(check_file_size(&exact_size_file, 1000).is_ok());

        // Test with one byte over
        let over_size_file = temp_dir.path().join("over_size.txt");
        std::fs::write(&over_size_file, &vec![0u8; 1001]).unwrap();
        assert!(check_file_size(&over_size_file, 1000).is_err());

        // Test with zero size
        let zero_size_file = temp_dir.path().join("zero_size.txt");
        std::fs::write(&zero_size_file, "").unwrap();
        assert!(check_file_size(&zero_size_file, 1000).is_ok());

        // Test with non-existent file
        let non_existent = temp_dir.path().join("non_existent.txt");
        assert!(check_file_size(&non_existent, 1000).is_err());
    }

    #[test]
    fn test_sanitize_file_path_with_extreme_path_depth() {
        // Test with extremely deep paths (but without traversal)
        let deep_path = (0..1000).map(|_| "dir").collect::<Vec<_>>().join("/") + "/file.txt";
        assert!(sanitize_file_path(&deep_path).is_ok()); // Deep nesting without traversal is OK
    }

    #[test]
    fn test_sanitize_file_path_with_extreme_traversal_attempts() {
        // Test with extremely deep traversal attempts
        let traversal_path = (0..500).map(|_| "../").collect::<Vec<_>>().join("") + "etc/passwd";
        assert!(sanitize_file_path(&traversal_path).is_err()); // Deep traversal should fail
    }

    #[test]
    fn test_validate_environment_name_with_extreme_length() {
        // Test with maximum allowed length (100 chars)
        let max_len_name = "a".repeat(100);
        assert!(validate_environment_name(&max_len_name).is_ok());

        // Test with one character over the limit (101 chars)
        let over_limit_name = "a".repeat(101);
        assert!(validate_environment_name(&over_limit_name).is_err());

        // Test with minimum length (empty string)
        assert!(validate_environment_name("").is_err());

        // Test with single character
        assert!(validate_environment_name("a").is_ok());
    }

    #[test]
    fn test_validate_config_key_with_extreme_length() {
        // Test with maximum allowed length (255 chars)
        let max_len_key = "a".repeat(255);
        assert!(validate_config_key(&max_len_key).is_ok());

        // Test with one character over the limit (256 chars)
        let over_limit_key = "a".repeat(256);
        assert!(validate_config_key(&over_limit_key).is_err());

        // Test with minimum length (empty string)
        assert!(validate_config_key("").is_err());

        // Test with single character
        assert!(validate_config_key("a").is_ok());
    }

    #[test]
    fn test_validate_config_key_with_all_allowed_characters() {
        // Test with all allowed characters combined
        let mixed_key = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-.";
        assert!(validate_config_key(mixed_key).is_ok());

        // Test with repeated allowed characters
        assert!(validate_config_key("a_0.z-9").is_ok());
    }

    #[test]
    fn test_validate_config_key_with_various_invalid_characters() {
        // Test with various invalid characters
        assert!(validate_config_key("key@value").is_err()); // @ symbol
        assert!(validate_config_key("key!value").is_err()); // exclamation
        assert!(validate_config_key("key#value").is_err()); // hash
        assert!(validate_config_key("key%value").is_err()); // percent
        assert!(validate_config_key("key^value").is_err()); // caret
        assert!(validate_config_key("key&value").is_err()); // ampersand
        assert!(validate_config_key("key*value").is_err()); // asterisk
        assert!(validate_config_key("key(value").is_err()); // parenthesis
        assert!(validate_config_key("key)value").is_err()); // parenthesis
        assert!(validate_config_key("key[value").is_err()); // brackets
        assert!(validate_config_key("key]value").is_err()); // brackets
        assert!(validate_config_key("key{value").is_err()); // braces
        assert!(validate_config_key("key}value").is_err()); // braces
        assert!(validate_config_key("key|value").is_err()); // pipe
        assert!(validate_config_key("key\\value").is_err()); // backslash
        assert!(validate_config_key("key/value").is_err()); // forward slash
        assert!(validate_config_key("key:value").is_err()); // colon
        assert!(validate_config_key("key;value").is_err()); // semicolon
        assert!(validate_config_key("key'value").is_err()); // single quote
        assert!(validate_config_key("key\"value").is_err()); // double quote
        assert!(validate_config_key("key<value").is_err()); // less than
        assert!(validate_config_key("key>value").is_err()); // greater than
        assert!(validate_config_key("key,value").is_err()); // comma
        assert!(validate_config_key("key?value").is_err()); // question mark
        assert!(validate_config_key("key space").is_err()); // space
    }

    #[test]
    fn test_sanitize_string_value_with_extreme_control_sequences() {
        // Test with various control characters
        let input_with_controls = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F";
        let sanitized = sanitize_string_value(input_with_controls);
        assert!(!sanitized.contains('\0')); // Null byte should be removed
                                            // Other control characters should remain
        assert_eq!(sanitized.chars().count(), 31); // All except null byte
    }

    #[test]
    fn test_validate_environment_name_with_unicode_confusables() {
        // Test potential confusable characters that might be used in attacks
        assert!(validate_environment_name("аdmin").is_ok()); // Cyrillic 'a' - should be OK but could be confusing
        assert!(validate_environment_name("аpple").is_ok()); // Another Cyrillic example
        assert!(validate_environment_name("admin").is_ok()); // Regular ASCII
        assert!(validate_environment_name("рhish").is_ok()); // Cyrillic 'p' in "phish"
    }

    #[test]
    fn test_sanitize_file_path_with_multiple_encodings() {
        // Test paths that might try to exploit encoding differences
        assert!(sanitize_file_path("normal/path/file.txt").is_ok());
        assert!(sanitize_file_path("path/with/special_chars.txt").is_ok());
        assert!(sanitize_file_path("path/with spaces/file.txt").is_ok());
        assert!(sanitize_file_path("path/with.123/file.txt").is_ok());
        assert!(sanitize_file_path("path/with-dashes/file.txt").is_ok());
        assert!(sanitize_file_path("path/with_underscores/file.txt").is_ok());
    }
}
