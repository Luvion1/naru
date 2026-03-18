use crate::core::schema_model::FieldDefinition;

pub mod type_validator {
    pub use crate::core::type_validator::*;
}

pub mod string_validator {
    pub use crate::core::string_validator::*;
}

pub fn validate_value(value: &str, field: &FieldDefinition) -> Result<(), String> {
    match field.r#type.as_str() {
        "integer" => {
            type_validator::validate_integer_with_rules(
                value,
                field
                    .validation
                    .as_ref()
                    .unwrap_or(&crate::core::schema_model::ValidationRules::new()),
            )?;
        }
        "boolean" => {
            type_validator::validate_boolean(value).map(|_| ())?;
        }
        "string" | _ => {
            if let Some(rules) = &field.validation {
                string_validator::validate_string_with_rules(value, rules)?;
            }
        }
    }

    Ok(())
}

pub fn is_valid_value(value: &str, field: &FieldDefinition) -> bool {
    validate_value(value, field).is_ok()
}

pub fn get_validation_error(value: &str, field: &FieldDefinition) -> Option<String> {
    match validate_value(value, field) {
        Ok(()) => None,
        Err(e) => Some(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::schema_model::ValidationRules;

    #[test]
    fn test_validate_integer() {
        let field = FieldDefinition {
            key: "age".into(),
            r#type: "integer".into(),
            description: None,
            validation: Some(ValidationRules {
                min_value: Some(0),
                max_value: Some(120),
                min_length: None,
                max_length: None,
                pattern: None,
            }),
            is_secret: false,
        };

        assert!(validate_value("25", &field).is_ok());
        assert!(validate_value("-1", &field).is_err());
        assert!(validate_value("121", &field).is_err());
        assert!(validate_value("abc", &field).is_err());
    }

    #[test]
    fn test_validate_string() {
        let field = FieldDefinition {
            key: "username".into(),
            r#type: "string".into(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(3),
                max_length: Some(10),
                min_value: None,
                max_value: None,
                pattern: None,
            }),
            is_secret: false,
        };

        assert!(validate_value("admin", &field).is_ok());
        assert!(validate_value("ab", &field).is_err());
        assert!(validate_value("verylongusername", &field).is_err());
    }

    #[test]
    fn test_validate_pattern() {
        let field = FieldDefinition {
            key: "email".into(),
            r#type: "string".into(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some(r"^[\w\.-]+@[\w\.-]+\.\w+$".to_string()),
            }),
            is_secret: false,
        };

        assert!(validate_value("test@example.com", &field).is_ok());
        assert!(validate_value("invalid-email", &field).is_err());
    }

    #[test]
    fn test_validate_boolean_strict() {
        let field = FieldDefinition {
            key: "active".into(),
            r#type: "boolean".into(),
            description: None,
            validation: None,
            is_secret: false,
        };

        assert!(validate_value("true", &field).is_ok());
        assert!(validate_value("false", &field).is_ok());
        assert!(validate_value("1", &field).is_err());
        assert!(validate_value("yes", &field).is_err());
    }

    #[test]
    fn test_is_valid_value() {
        let field = FieldDefinition::new("test", "integer");

        assert!(is_valid_value("123", &field));
        assert!(!is_valid_value("abc", &field));
    }

    #[test]
    fn test_get_validation_error() {
        let field = FieldDefinition {
            key: "age".into(),
            r#type: "integer".into(),
            description: None,
            validation: Some(ValidationRules {
                min_value: Some(0),
                max_value: Some(100),
                min_length: None,
                max_length: None,
                pattern: None,
            }),
            is_secret: false,
        };

        assert!(get_validation_error("50", &field).is_none());
        assert!(get_validation_error("-1", &field).is_some());
    }

    #[test]
    fn test_combined_rules() {
        let field = FieldDefinition {
            key: "username".into(),
            r#type: "string".into(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(5),
                max_length: Some(10),
                min_value: None,
                max_value: None,
                pattern: Some(r"^[a-z]+$".to_string()),
            }),
            is_secret: false,
        };

        assert!(validate_value("admin", &field).is_ok());
        assert!(validate_value("adm", &field).is_err());
        assert!(validate_value("ADMIN", &field).is_err());
        assert!(validate_value("administrator", &field).is_err());
    }
}
