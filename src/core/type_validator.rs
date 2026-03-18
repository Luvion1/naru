use crate::core::schema_model::ValidationRules;

pub fn validate_integer(value: &str) -> Result<i64, String> {
    value
        .parse::<i64>()
        .map_err(|_| format!("'{}' is not a valid integer.", value))
}

pub fn validate_integer_with_rules(value: &str, rules: &ValidationRules) -> Result<(), String> {
    let val = validate_integer(value)?;

    if let Some(min) = rules.min_value {
        if val < min {
            return Err(format!("Value {} is less than minimum {}", val, min));
        }
    }

    if let Some(max) = rules.max_value {
        if val > max {
            return Err(format!("Value {} is greater than maximum {}", val, max));
        }
    }

    Ok(())
}

pub fn validate_boolean(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("'{}' is not a valid boolean (true/false).", value)),
    }
}

pub fn validate_boolean_strict(value: &str) -> Result<(), String> {
    if value == "true" || value == "false" {
        Ok(())
    } else {
        Err(format!(
            "'{}' is not a valid boolean. Expected 'true' or 'false' (case-sensitive).",
            value
        ))
    }
}

pub fn is_valid_integer(value: &str) -> bool {
    value.parse::<i64>().is_ok()
}

pub fn is_valid_boolean(value: &str) -> bool {
    value == "true" || value == "false"
}

pub fn is_hex_string(value: &str) -> bool {
    value.starts_with("0x") || value.starts_with("0X")
}

pub fn is_octal_string(value: &str) -> bool {
    value.starts_with("0o") || value.starts_with("0O")
}

pub fn is_binary_string(value: &str) -> bool {
    value.starts_with("0b") || value.starts_with("0B")
}

pub fn is_scientific_notation(value: &str) -> bool {
    value.contains('e') || value.contains('E')
}

pub fn has_leading_plus(value: &str) -> bool {
    value.starts_with('+')
}

pub fn has_leading_zeros(value: &str) -> bool {
    value.len() > 1
        && value.starts_with('0')
        && value.chars().nth(1).map_or(false, |c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_integer() {
        assert!(validate_integer("0").is_ok());
        assert!(validate_integer("123").is_ok());
        assert!(validate_integer("-456").is_ok());
        assert!(validate_integer("+789").is_ok());
        assert!(validate_integer("abc").is_err());
    }

    #[test]
    fn test_validate_integer_bounds() {
        assert!(validate_integer("9223372036854775807").is_ok());
        assert!(validate_integer("-9223372036854775808").is_ok());
        assert!(validate_integer("9223372036854775808").is_err());
    }

    #[test]
    fn test_validate_integer_with_rules() {
        let rules = ValidationRules::new().with_value_range(0, 100);

        assert!(validate_integer_with_rules("50", &rules).is_ok());
        assert!(validate_integer_with_rules("0", &rules).is_ok());
        assert!(validate_integer_with_rules("100", &rules).is_ok());
        assert!(validate_integer_with_rules("-1", &rules).is_err());
        assert!(validate_integer_with_rules("101", &rules).is_err());
    }

    #[test]
    fn test_validate_integer_with_negative_range() {
        let rules = ValidationRules::new().with_value_range(-50, -10);

        assert!(validate_integer_with_rules("-30", &rules).is_ok());
        assert!(validate_integer_with_rules("-50", &rules).is_ok());
        assert!(validate_integer_with_rules("-10", &rules).is_ok());
        assert!(validate_integer_with_rules("-9", &rules).is_err());
        assert!(validate_integer_with_rules("-51", &rules).is_err());
    }

    #[test]
    fn test_validate_boolean() {
        assert_eq!(validate_boolean("true").unwrap(), true);
        assert_eq!(validate_boolean("false").unwrap(), false);
        assert!(validate_boolean("1").is_err());
        assert!(validate_boolean("0").is_err());
        assert!(validate_boolean("yes").is_err());
    }

    #[test]
    fn test_validate_boolean_case_sensitivity() {
        assert!(validate_boolean_strict("true").is_ok());
        assert!(validate_boolean_strict("false").is_ok());
        assert!(validate_boolean_strict("True").is_err());
        assert!(validate_boolean_strict("FALSE").is_err());
        assert!(validate_boolean_strict("TRUE").is_err());
    }

    #[test]
    fn test_is_valid_integer() {
        assert!(is_valid_integer("0"));
        assert!(is_valid_integer("-123"));
        assert!(is_valid_integer("+456"));
        assert!(!is_valid_integer("abc"));
        assert!(!is_valid_integer("1.5"));
    }

    #[test]
    fn test_is_valid_boolean() {
        assert!(is_valid_boolean("true"));
        assert!(is_valid_boolean("false"));
        assert!(!is_valid_boolean("True"));
        assert!(!is_valid_boolean("1"));
    }

    #[test]
    fn test_integer_format_detection() {
        assert!(!is_hex_string("123"));
        assert!(is_hex_string("0xFF"));
        assert!(is_hex_string("0Xff"));

        assert!(!is_octal_string("123"));
        assert!(is_octal_string("0o755"));
        assert!(is_octal_string("0O123"));

        assert!(!is_binary_string("123"));
        assert!(is_binary_string("0b1010"));
        assert!(is_binary_string("0B1111"));

        assert!(is_scientific_notation("1e5"));
        assert!(is_scientific_notation("1.23e4"));
        assert!(!is_scientific_notation("123"));

        assert!(!has_leading_plus("123"));
        assert!(has_leading_plus("+456"));

        assert!(!has_leading_zeros("123"));
        assert!(has_leading_zeros("007"));
    }

    #[test]
    fn test_integer_with_leading_plus() {
        let rules = ValidationRules::new().with_value_range(0, 100);
        assert!(validate_integer_with_rules("+42", &rules).is_ok());
        assert!(validate_integer_with_rules("+0", &rules).is_ok());
        assert!(validate_integer_with_rules("+101", &rules).is_err());
    }

    #[test]
    fn test_integer_with_leading_zeros() {
        let rules = ValidationRules::new().with_value_range(0, 100);
        assert!(validate_integer_with_rules("007", &rules).is_ok());
        assert!(validate_integer_with_rules("000", &rules).is_ok());
    }

    #[test]
    fn test_integer_rejects_hex() {
        assert!(validate_integer("0xFF").is_err());
        assert!(validate_integer("0x123").is_err());
    }

    #[test]
    fn test_integer_rejects_octal() {
        assert!(validate_integer("0o755").is_err());
        assert!(validate_integer("0o123").is_err());
    }

    #[test]
    fn test_integer_rejects_binary() {
        assert!(validate_integer("0b1010").is_err());
        assert!(validate_integer("0b1111").is_err());
    }

    #[test]
    fn test_integer_rejects_scientific() {
        assert!(validate_integer("1e5").is_err());
        assert!(validate_integer("1.23e4").is_err());
    }

    #[test]
    fn test_integer_rejects_float() {
        assert!(validate_integer("1.0").is_err());
        assert!(validate_integer("3.14").is_err());
    }

    #[test]
    fn test_integer_rejects_special() {
        assert!(validate_integer("inf").is_err());
        assert!(validate_integer("nan").is_err());
    }

    #[test]
    fn test_integer_rejects_whitespace() {
        assert!(validate_integer(" 123 ").is_err());
        assert!(validate_integer("\t456\n").is_err());
    }
}
