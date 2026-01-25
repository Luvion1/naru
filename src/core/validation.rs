use crate::core::models::*;

pub fn validate_value(value: &str, field: &FieldDefinition) -> Result<(), String> {
    match field.r#type.as_str() {
        "integer" => {
            value.parse::<i64>()
                .map_err(|_| format!("'{}' is not a valid integer.", value))?;
        }
        "boolean" => {
            if value != "true" && value != "false" {
                return Err(format!("'{}' is not a valid boolean.", value));
            }
        }
        _ => {}
    }

    if let Some(rules) = &field.validation {
        if field.r#type == "string" {
            if let Some(min) = rules.min_length {
                if value.len() < min {
                    return Err(format!("Too short (min: {})", min));
                }
            }
            if let Some(max) = rules.max_length {
                if value.len() > max {
                    return Err(format!("Too long (max: {})", max));
                }
            }
        }
        if field.r#type == "integer" {
            let val = value.parse::<i64>().map_err(|_| format!("'{}' is not a valid integer.", value))?;
            if let Some(min) = rules.min_value {
                if val < min {
                    return Err(format!("Less than minimum {}", min));
                }
            }
            if let Some(max) = rules.max_value {
                if val > max {
                    return Err(format!("Greater than maximum {}", max));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{FieldDefinition, ValidationRules};

    #[test]
    fn test_integer_validation() {
        let field = FieldDefinition {
            key: "test_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };

        assert!(validate_value("123", &field).is_ok());
        assert!(validate_value("abc", &field).is_err());
    }

    #[test]
    fn test_boolean_validation() {
        let field = FieldDefinition {
            key: "test_bool".to_string(),
            r#type: "boolean".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };

        assert!(validate_value("true", &field).is_ok());
        assert!(validate_value("false", &field).is_ok());
        assert!(validate_value("yes", &field).is_err());
        assert!(validate_value("no", &field).is_err());
    }

    #[test]
    fn test_string_min_length_validation() {
        let field = FieldDefinition {
            key: "test_str".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(5),
                max_length: None,
                min_value: None,
                max_value: None,
            }),
            is_secret: false,
        };

        assert!(validate_value("hello", &field).is_ok());
        assert!(validate_value("hi", &field).is_err());
    }

    #[test]
    fn test_integer_range_validation() {
        let field = FieldDefinition {
            key: "test_int_range".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: Some(10),
                max_value: Some(20),
            }),
            is_secret: false,
        };

        assert!(validate_value("15", &field).is_ok());
        assert!(validate_value("5", &field).is_err());
        assert!(validate_value("25", &field).is_err());
    }
}