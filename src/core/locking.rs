use std::fs::File;
use std::io::Error;
use std::path::Path;
use fs2::FileExt;

/// Represents a file lock using OS-level advisory locks
pub struct FileLock {
    _file: File,
    path: std::path::PathBuf,
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
            .open(&lock_path)?;
        
        // Acquire exclusive lock (blocks until acquired)
        file.lock_exclusive()?;
        
        Ok(FileLock { 
            _file: file,
            path: lock_path,
        })
    }

    /// Release the lock (optional, as it's released on drop)
    pub fn release(self) -> Result<(), Error> {
        // The lock is automatically released when the file is closed (on drop)
        drop(self);
        Ok(())
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Lock is released by the OS when the file handle is closed
        // Best effort to remove the lock file
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_lock() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Acquire the first lock
        let lock1 = FileLock::acquire_exclusive(path).expect("Should acquire first lock");

        // Release the first lock
        lock1.release().expect("Should release lock");

        // Now acquiring a new lock should succeed
        let lock2 = FileLock::acquire_exclusive(path).expect("Should acquire second lock");
        lock2.release().expect("Should release second lock");
    }
}