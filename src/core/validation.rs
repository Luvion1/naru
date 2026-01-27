use crate::core::models::FieldDefinition;

pub fn validate_value(value: &str, field: &FieldDefinition) -> Result<(), String> {
    match field.r#type.as_str() {
        "integer" => {
            value
                .parse::<i64>()
                .map_err(|_| format!("'{}' is not a valid integer.", value))?;
        }
        "boolean" => {
            if value != "true" && value != "false" {
                return Err(format!("'{}' is not a valid boolean (true/false).", value));
            }
        }
        _ => {}
    }

    if let Some(rules) = &field.validation {
        if field.r#type == "string" {
            if let Some(min) = rules.min_length
                && value.len() < min
            {
                return Err(format!("Too short (min: {})", min));
            }
            if let Some(max) = rules.max_length
                && value.len() > max
            {
                return Err(format!("Too long (max: {})", max));
            }
            if let Some(pattern) = &rules.pattern {
                let re = regex::Regex::new(pattern)
                    .map_err(|e| format!("Invalid regex pattern: {}", e))?;
                if !re.is_match(value) {
                    return Err(format!("Value does not match pattern: {}", pattern));
                }
            }
        }
        if field.r#type == "integer" {
            let val = value
                .parse::<i64>()
                .map_err(|_| format!("'{}' is not a valid integer.", value))?;
            if let Some(min) = rules.min_value
                && val < min
            {
                return Err(format!("Less than minimum {}", min));
            }
            if let Some(max) = rules.max_value
                && val > max
            {
                return Err(format!("Greater than maximum {}", max));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ValidationRules;

    #[test]
    fn test_validate_integer() {
        let field = FieldDefinition {
            key: "age".to_string(),
            r#type: "integer".to_string(),
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
            key: "username".to_string(),
            r#type: "string".to_string(),
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
            key: "email".to_string(),
            r#type: "string".to_string(),
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
            key: "active".to_string(),
            r#type: "boolean".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };

        assert!(validate_value("true", &field).is_ok());
        assert!(validate_value("false", &field).is_ok());
        assert!(validate_value("1", &field).is_err());
        assert!(validate_value("yes", &field).is_err());
        assert!(
            validate_value("True", &field).is_err(),
            "Should be case sensitive for now or explicitly handled"
        );
    }

    #[test]
    fn test_validate_integer_limits() {
        let field = FieldDefinition {
            key: "big_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };

        assert!(
            validate_value("9223372036854775807", &field).is_ok(),
            "Max i64"
        );
        assert!(
            validate_value("-9223372036854775808", &field).is_ok(),
            "Min i64"
        );
        assert!(
            validate_value("9223372036854775808", &field).is_err(),
            "Overflow i64"
        );
    }

    #[test]
    fn test_validate_combined_rules() {
        let field = FieldDefinition {
            key: "restricted_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: Some(10),
                max_value: Some(100),
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value("10", &field).is_ok());
        assert!(validate_value("100", &field).is_ok());
        assert!(validate_value("9", &field).is_err());
        assert!(validate_value("101", &field).is_err());
    }

    #[test]
    fn test_validate_string_with_regex_and_length() {
        let field = FieldDefinition {
            key: "username".to_string(),
            r#type: "string".to_string(),
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
        assert!(validate_value("adm", &field).is_err(), "Too short");
        assert!(validate_value("ADMIN", &field).is_err(), "Regex mismatch");
        assert!(validate_value("administrator", &field).is_err(), "Too long");
    }

    #[test]
    fn test_validate_unsupported_type() {
        let field = FieldDefinition {
            key: "float".to_string(),
            r#type: "float".to_string(), // Not supported
            description: None,
            validation: None,
            is_secret: false,
        };
        // Should just be Ok because we don't know how to validate it (default to string-like)
        assert!(validate_value("1.23", &field).is_ok());
    }

    #[test]
    fn test_validate_integer_with_string_rules() {
        let field = FieldDefinition {
            key: "age".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(10), // Should be ignored for integers
                max_length: None,
                min_value: Some(0),
                max_value: None,
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(
            validate_value("25", &field).is_ok(),
            "min_length should be ignored for integers"
        );
    }

    #[test]
    fn test_validate_empty_pattern() {
        let field = FieldDefinition {
            key: "any".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some("".to_string()),
            }),
            is_secret: false,
        };
        assert!(validate_value("", &field).is_ok());
        assert!(validate_value("anything", &field).is_ok());
    }

    #[test]
    fn test_validate_invalid_regex_logic() {
        let field = FieldDefinition {
            key: "bad_regex".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some("[".to_string()), // Invalid regex
            }),
            is_secret: false,
        };
        assert!(validate_value("test", &field).is_err());
    }
}
