use crate::core::constants::SCHEMA_FILE;
use crate::core::models::{FieldDefinition, SchemaFile};
use crate::core::persistence;
use anyhow::Result;

/// Menambahkan field ke skema
pub fn add_field(field: FieldDefinition) -> Result<()> {
    let mut schema: SchemaFile =
        persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| SchemaFile {
            version: "1.0".to_string(),
            fields: vec![],
        });

    if schema.fields.iter().any(|f| f.key == field.key) {
        return Err(anyhow::anyhow!(
            "Field '{}' already exists in schema",
            field.key
        ));
    }

    let key = field.key.clone();
    schema.fields.push(field);
    persistence::save_json(SCHEMA_FILE, &schema)?;
    println!("Field '{}' added to schema.", key);
    Ok(())
}

/// Menghapus field dari skema
pub fn remove_field(key: &str) -> Result<()> {
    let mut schema: SchemaFile =
        persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| SchemaFile {
            version: "1.0".to_string(),
            fields: vec![],
        });

    let initial_len = schema.fields.len();
    schema.fields.retain(|f| f.key != key);

    if schema.fields.len() == initial_len {
        return Err(anyhow::anyhow!("Field '{}' not found in schema", key));
    }

    persistence::save_json(SCHEMA_FILE, &schema)?;
    println!("Field '{}' removed from schema.", key);
    Ok(())
}

/// Memperbarui field di skema
pub fn update_field(key: &str, updated_field: FieldDefinition) -> Result<()> {
    let mut schema: SchemaFile =
        persistence::load_json(SCHEMA_FILE).unwrap_or_else(|_| SchemaFile {
            version: "1.0".to_string(),
            fields: vec![],
        });

    if let Some(field) = schema.fields.iter_mut().find(|f| f.key == key) {
        *field = updated_field;
        persistence::save_json(SCHEMA_FILE, &schema)?;
        println!("Field '{}' updated in schema.", key);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Field '{}' not found in schema", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Guard to revert current directory on drop
    struct TestDirGuard {
        original_dir: std::path::PathBuf,
    }

    impl TestDirGuard {
        fn new(temp_path: &Path) -> Self {
            let original_dir = std::env::current_dir().unwrap();
            std::env::set_current_dir(temp_path).unwrap();
            Self { original_dir }
        }
    }

    impl Drop for TestDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    #[test]
    fn test_schema_operations() {
        let temp_dir = TempDir::new().unwrap();
        let _guard = TestDirGuard::new(temp_dir.path());

        persistence::init_project().unwrap();

        // 1. Test Add
        add_field(FieldDefinition {
            key: "test_key".to_string(),
            r#type: "string".to_string(),
            description: Some("desc".to_string()),
            validation: None,
            is_secret: false,
        })
        .unwrap();

        let schema: SchemaFile = persistence::load_json(SCHEMA_FILE).unwrap();
        assert_eq!(schema.fields.len(), 1);
        assert_eq!(schema.fields[0].key, "test_key");

        // 2. Test Update
        update_field(
            "test_key",
            FieldDefinition {
                key: "test_key".to_string(),
                r#type: "integer".to_string(),
                description: Some("updated desc".to_string()),
                validation: None,
                is_secret: false,
            },
        )
        .unwrap();

        let schema: SchemaFile = persistence::load_json(SCHEMA_FILE).unwrap();
        assert_eq!(schema.fields[0].r#type, "integer");
        assert_eq!(
            schema.fields[0].description,
            Some("updated desc".to_string())
        );

        // 3. Test Remove
        remove_field("test_key").unwrap();
        let schema: SchemaFile = persistence::load_json(SCHEMA_FILE).unwrap();
        assert_eq!(schema.fields.len(), 0);
    }
}
