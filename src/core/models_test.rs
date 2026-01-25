#[cfg(test)]
mod tests {
    use crate::core::models::*;
    use std::collections::HashMap;

    #[test]
    fn test_config_file_creation() {
        let mut environments = HashMap::new();
        environments.insert(
            "development".to_string(), 
            EnvironmentConfig { entries: HashMap::new() }
        );

        let config = ConfigFile {
            project_name: "Test Project".to_string(),
            version: "1.0.0".to_string(),
            environments,
        };

        assert_eq!(config.project_name, "Test Project");
        assert_eq!(config.version, "1.0.0");
        assert!(config.environments.contains_key("development"));
    }

    #[test]
    fn test_config_value_entry() {
        let entry = ConfigValueEntry {
            value: "test_value".to_string(),
            r#type: "string".to_string(),
            is_secret: true,
            encrypted: false,
        };

        assert_eq!(entry.value, "test_value");
        assert_eq!(entry.r#type, "string");
        assert_eq!(entry.is_secret, true);
        assert_eq!(entry.encrypted, false);
    }

    #[test]
    fn test_field_definition() {
        let validation_rules = ValidationRules {
            min_length: Some(5),
            max_length: Some(100),
            min_value: None,
            max_value: None,
        };

        let field = FieldDefinition {
            key: "test_field".to_string(),
            r#type: "string".to_string(),
            description: Some("A test field".to_string()),
            validation: Some(validation_rules),
            is_secret: false,
        };

        assert_eq!(field.key, "test_field");
        assert_eq!(field.r#type, "string");
        assert_eq!(field.description, Some("A test field".to_string()));
        assert_eq!(field.validation.unwrap().min_length, Some(5));
    }
}