use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct FileStorage {
    base_path: PathBuf,
}

#[allow(dead_code)]
impl FileStorage {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    pub fn ensure_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_path)
    }

    pub fn exists(&self, filename: &str) -> bool {
        self.resolve_path(filename).exists()
    }

    pub fn read(&self, filename: &str) -> std::io::Result<String> {
        fs::read_to_string(self.resolve_path(filename))
    }

    pub fn write(&self, filename: &str, content: &str) -> std::io::Result<()> {
        let path = self.resolve_path(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
    }

    pub fn delete(&self, filename: &str) -> std::io::Result<()> {
        let path = self.resolve_path(filename);
        if path.exists() {
            fs::remove_file(path)
        } else {
            Ok(())
        }
    }

    pub fn list_files(&self, extension: Option<&str>) -> std::io::Result<Vec<String>> {
        let mut files = Vec::new();
        if self.base_path.is_dir() {
            for entry in fs::read_dir(&self.base_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = extension {
                        if let Some(path_ext) = path.extension() {
                            if path_ext.to_str() == Some(ext) {
                                if let Some(name) = path.file_name() {
                                    files.push(name.to_string_lossy().to_string());
                                }
                            }
                        }
                    } else if let Some(name) = path.file_name() {
                        files.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(files)
    }

    pub fn copy(&self, source: &str, dest: &str) -> std::io::Result<()> {
        fs::copy(self.resolve_path(source), self.resolve_path(dest))?;
        Ok(())
    }

    pub fn move_file(&self, source: &str, dest: &str) -> std::io::Result<()> {
        fs::rename(self.resolve_path(source), self.resolve_path(dest))
    }

    pub fn get_size(&self, filename: &str) -> std::io::Result<u64> {
        let metadata = fs::metadata(self.resolve_path(filename))?;
        Ok(metadata.len())
    }

    fn resolve_path(&self, filename: &str) -> PathBuf {
        self.base_path.join(filename)
    }

    pub fn path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_storage_new() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        assert!(storage.path().exists());
    }

    #[test]
    fn test_file_storage_write_read() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage.write("test.txt", "hello world").unwrap();
        assert!(storage.exists("test.txt"));

        let content = storage.read("test.txt").unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_file_storage_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage.write("delete_me.txt", "content").unwrap();
        assert!(storage.exists("delete_me.txt"));

        storage.delete("delete_me.txt").unwrap();
        assert!(!storage.exists("delete_me.txt"));
    }

    #[test]
    fn test_file_storage_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage.write("file1.txt", "content1").unwrap();
        storage.write("file2.txt", "content2").unwrap();
        storage.write("file3.json", "content3").unwrap();

        let txt_files = storage.list_files(Some("txt")).unwrap();
        assert_eq!(txt_files.len(), 2);

        let json_files = storage.list_files(Some("json")).unwrap();
        assert_eq!(json_files.len(), 1);

        let all_files = storage.list_files(None).unwrap();
        assert_eq!(all_files.len(), 3);
    }

    #[test]
    fn test_file_storage_get_size() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage.write("size_test.txt", "hello").unwrap();
        let size = storage.get_size("size_test.txt").unwrap();
        assert_eq!(size, 5);
    }

    #[test]
    fn test_file_storage_nested_path() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());

        storage
            .write("nested/dir/test.txt", "nested content")
            .unwrap();
        let content = storage.read("nested/dir/test.txt").unwrap();
        assert_eq!(content, "nested content");
    }
}
