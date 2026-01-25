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
}
