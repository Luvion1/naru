use std::path::Path;

pub fn is_symlink(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => meta.file_type().is_symlink(),
        Err(_) => false,
    }
}

pub fn resolve_and_validate_path(path: &Path, base_dir: &Path) -> Result<PathBuf, &'static str> {
    if is_symlink(path) {
        let resolved = std::fs::read_link(path).map_err(|_| "Cannot resolve symlink")?;

        let resolved_abs = if resolved.is_absolute() {
            resolved
        } else {
            if let Some(parent) = path.parent() {
                parent.join(&resolved)
            } else {
                resolved
            }
        };

        let canonical_base =
            std::fs::canonicalize(base_dir).map_err(|_| "Cannot access base directory")?;
        let canonical_resolved =
            std::fs::canonicalize(&resolved_abs).map_err(|_| "Cannot resolve symlink target")?;

        if !canonical_resolved.starts_with(&canonical_base) {
            return Err("Symlink points outside allowed directory");
        }

        return Ok(canonical_resolved);
    }

    std::fs::canonicalize(path).map_err(|_| "Cannot access path")
}

pub fn check_file_size(path: &Path, max_size: u64) -> Result<(), &'static str> {
    let metadata = std::fs::metadata(path).map_err(|_| "Could not get file metadata")?;

    if metadata.len() > max_size {
        return Err("File exceeds maximum allowed size");
    }

    Ok(())
}

pub fn is_file_readable(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok()
}

pub fn is_file_writable(path: &Path) -> bool {
    if path.exists() {
        std::fs::metadata(path)
            .map(|m| m.permissions().readonly() == false)
            .unwrap_or(false)
    } else {
        if let Some(parent) = path.parent() {
            parent.exists() && is_file_writable(parent)
        } else {
            false
        }
    }
}

pub fn get_file_size(path: &Path) -> Result<u64, &'static str> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|_| "Could not get file metadata")
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_check_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "some data").unwrap();

        assert!(check_file_size(&file_path, 100).is_ok());
        assert!(check_file_size(&file_path, 5).is_err());
    }

    #[test]
    fn test_get_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test data").unwrap();

        let size = get_file_size(&file_path).unwrap();
        assert!(size > 0);
    }

    #[test]
    fn test_is_file_readable() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        assert!(is_file_readable(&file_path));
    }

    #[test]
    fn test_is_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        let link_path = temp_dir.path().join("link.txt");

        std::fs::write(&file_path, "content").unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink(&file_path, &link_path).unwrap();

        #[cfg(not(unix))]
        {
            use std::process::Command;
            let _ = Command::new("cmd")
                .args(&[
                    "/c",
                    "mklink",
                    &link_path.to_string_lossy(),
                    &file_path.to_string_lossy(),
                ])
                .output();
        }

        assert!(is_symlink(&link_path) || !link_path.exists());
    }
}
