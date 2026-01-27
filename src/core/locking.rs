use fs2::FileExt;
use std::fs::File;
use std::io::Error;
use std::path::Path;

/// Represents a file lock using OS-level advisory locks
pub struct FileLock {
    _file: File,
    pub path: std::path::PathBuf,
}

impl FileLock {
    /// Acquire an exclusive lock on a file
    pub fn acquire_exclusive<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path_ref = path.as_ref();

        // Append .lock to the filename for the lock file
        let lock_path = path_ref.with_extension("lock");

        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_path)?;

        // Acquire exclusive lock (blocks until acquired)
        file.lock_exclusive()?;

        Ok(FileLock {
            _file: file,
            path: lock_path,
        })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Unlocking and removing the lock file is handled automatically when the File object is dropped
        // and when we explicitly remove the file.
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_lock() {
        let temp_dir = TempDir::new().unwrap();
        let target_file = temp_dir.path().join("test.json");

        // First lock
        let lock1 = FileLock::acquire_exclusive(&target_file).expect("Should acquire first lock");
        assert!(lock1.path.exists());

        // Second lock attempt in another scope or handled correctly
        // (fs2 lock_exclusive is blocking, so this would block indefinitely in a single thread)
        // Instead, we just verify the file exists and is dropped correctly.
        drop(lock1);
        assert!(!target_file.with_extension("lock").exists());
    }

    #[test]
    fn test_lock_non_existent_directory() {
        let target_file = Path::new("/non/existent/path/file.json");
        let result = FileLock::acquire_exclusive(target_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_rapid_relock() {
        let temp_dir = TempDir::new().unwrap();
        let target_file = temp_dir.path().join("test.json");

        for _ in 0..10 {
            let lock = FileLock::acquire_exclusive(&target_file).unwrap();
            assert!(lock.path.exists());
            drop(lock);
            assert!(!target_file.with_extension("lock").exists());
        }
    }
}
