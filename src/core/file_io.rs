use std::fs;
use std::path::Path;

pub fn read_json_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

pub fn write_json_file<P: AsRef<Path>>(path: P, content: &str) -> Result<(), std::io::Error> {
    fs::write(path, content)
}

pub fn read_json_file_or_default<P: AsRef<Path>, T: serde::de::DeserializeOwned + Default>(
    path: P,
) -> T {
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => T::default(),
    }
}

pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn create_directory<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    fs::create_dir_all(path)
}

pub fn delete_file<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    fs::remove_file(path)
}

pub fn read_directory<P: AsRef<Path>>(path: P) -> Result<Vec<String>, std::io::Error> {
    let entries = fs::read_dir(path)?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_write_json_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        let content = r#"{"key": "value"}"#;
        write_json_file(&file_path, content).unwrap();

        let read_content = read_json_file(&file_path).unwrap();
        assert_eq!(content, read_content);
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("exists.txt");

        assert!(!file_exists(&file_path));

        std::fs::write(&file_path, "content").unwrap();

        assert!(file_exists(&file_path));
    }

    #[test]
    fn test_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new/nested/dir");

        create_directory(&new_dir).unwrap();

        assert!(file_exists(&new_dir));
    }

    #[test]
    fn test_read_json_file_or_default() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("data.json");

        #[derive(serde::Deserialize, Default, PartialEq)]
        struct Data {
            value: String,
        }

        let data: Data = read_json_file_or_default(&file_path);
        assert_eq!(data, Data::default());

        std::fs::write(&file_path, r#"{"value": "test"}"#).unwrap();

        let data: Data = read_json_file_or_default(&file_path);
        assert_eq!(data.value, "test");
    }
}
