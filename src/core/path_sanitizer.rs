use std::path::{Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            std::path::Component::Normal(c) => {
                normalized.push(c);
            }
            _ => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

pub fn sanitize_file_path(path: &str) -> Result<PathBuf, &'static str> {
    if path.contains('\0') {
        return Err("Path contains null bytes");
    }

    let unified_path = path.replace('\\', "/");

    if unified_path.starts_with('/') || unified_path.starts_with("//") {
        return Err("Absolute paths are not allowed");
    }

    for component in unified_path.split('/') {
        if component == ".." {
            return Err("Path contains directory traversal sequences");
        }
    }

    let original_path_obj = Path::new(path);
    let normalized = normalize_path(original_path_obj);

    for component in normalized.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path attempts to escape parent directory");
        }
    }

    Ok(normalized)
}

pub fn sanitize_file_path_internal(path: &str) -> Result<PathBuf, &'static str> {
    if path.contains('\0') {
        return Err("Path contains null bytes");
    }

    let unified_path = path.replace('\\', "/");

    for component in unified_path.split('/') {
        if component == ".." {
            return Err("Path contains directory traversal sequences");
        }
    }

    let original_path_obj = Path::new(path);
    let normalized = normalize_path(original_path_obj);

    for component in normalized.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path attempts to escape parent directory");
        }
    }

    Ok(normalized)
}

pub fn is_path_traversal(path: &str) -> bool {
    let unified = path.replace('\\', "/");

    if unified.contains("../") || unified.starts_with("..") {
        return true;
    }

    if unified.contains("/..") {
        return true;
    }

    false
}

pub fn is_absolute_path(path: &str) -> bool {
    let unified = path.replace('\\', "/");
    unified.starts_with('/') || unified.starts_with("//")
}

pub fn has_null_byte(path: &str) -> bool {
    path.contains('\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_valid_paths() {
        assert!(sanitize_file_path("config.json").is_ok());
        assert!(sanitize_file_path("./config.json").is_ok());
        assert!(sanitize_file_path("folder/config.json").is_ok());
    }

    #[test]
    fn test_sanitize_traversal() {
        assert!(sanitize_file_path("../config.json").is_err());
        assert!(sanitize_file_path("/etc/passwd").is_err());
        assert!(sanitize_file_path("../../../etc/passwd").is_err());
        assert!(sanitize_file_path("folder/../config.json").is_err());
    }

    #[test]
    fn test_sanitize_null_bytes() {
        assert!(sanitize_file_path("config.json\0.txt").is_err());
    }

    #[test]
    fn test_sanitize_windows_traversal() {
        assert!(sanitize_file_path("folder\\..\\config.json").is_err());
    }

    #[test]
    fn test_is_path_traversal() {
        assert!(is_path_traversal("../etc/passwd"));
        assert!(is_path_traversal("folder/../etc/passwd"));
        assert!(!is_path_traversal("folder/file.txt"));
    }

    #[test]
    fn test_is_absolute_path() {
        assert!(is_absolute_path("/etc/passwd"));
        assert!(is_absolute_path("//etc/passwd"));
        assert!(!is_absolute_path("relative/path"));
    }

    #[test]
    fn test_has_null_byte() {
        assert!(has_null_byte("file\0.txt"));
        assert!(!has_null_byte("file.txt"));
    }

    #[test]
    fn test_normalize_path() {
        let path = Path::new("a/b/c/../d/./e");
        let normalized = normalize_path(path);
        assert!(normalized.to_string_lossy().contains("e"));
    }
}
