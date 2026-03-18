use std::fs;
use std::io::Write;
use std::path::Path;

use crate::core::locking::FileLock;

pub struct AtomicWriter {
    temp_path: std::path::PathBuf,
    target_path: std::path::PathBuf,
}

impl AtomicWriter {
    pub fn new(target_path: &Path) -> Self {
        let temp_path = target_path.with_extension("tmp");
        AtomicWriter {
            temp_path,
            target_path: target_path.to_path_buf(),
        }
    }

    pub fn write<F>(target_path: &Path, mut f: F) -> Result<(), std::io::Error>
    where
        F: FnMut(&mut std::fs::File) -> std::io::Result<()>,
    {
        let lock_path = target_path.with_extension("lock");

        let _lock = FileLock::acquire_exclusive(&lock_path)?;

        let temp_path = target_path.with_extension("tmp");

        {
            let mut temp_file = fs::File::create(&temp_path)?;
            f(&mut temp_file)?;
            temp_file.sync_all()?;
        }

        fs::rename(&temp_path, target_path)?;

        Ok(())
    }

    pub fn write_with_content(&self, content: &str) -> Result<(), std::io::Error> {
        Self::write(&self.target_path, |file| file.write_all(content.as_bytes()))
    }

    pub fn commit(self) -> Result<(), std::io::Error> {
        fs::rename(&self.temp_path, &self.target_path)?;
        Ok(())
    }

    pub fn rollback(self) -> Result<(), std::io::Error> {
        if self.temp_path.exists() {
            fs::remove_file(&self.temp_path)?;
        }
        Ok(())
    }
}

pub fn atomic_write<P: AsRef<Path>>(path: P, content: &str) -> Result<(), std::io::Error> {
    let writer = AtomicWriter::new(path.as_ref());
    writer.write_with_content(content)
}

pub fn atomic_read<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

pub fn atomic_update<P: AsRef<Path>, F>(path: P, mut f: F) -> Result<(), std::io::Error>
where
    F: FnMut(&str) -> String,
{
    let content = fs::read_to_string(&path)?;
    let new_content = f(&content);
    atomic_write(&path, &new_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("atomic.txt");

        atomic_write(&file_path, "test content").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_atomic_update() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("update.txt");

        fs::write(&file_path, "Hello").unwrap();

        atomic_update(&file_path, |s| format!("{} World", s)).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello World");
    }

    #[test]
    fn test_atomic_writer() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("writer.txt");

        let writer = AtomicWriter::new(&file_path);
        writer.write_with_content("direct content").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "direct content");
    }
}
