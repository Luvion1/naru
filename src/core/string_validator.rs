use crate::core::schema_model::ValidationRules;

pub fn validate_string_length(value: &str, rules: &ValidationRules) -> Result<(), String> {
    if let Some(min) = rules.min_length {
        if value.len() < min {
            return Err(format!("Value is too short. Minimum length is {}", min));
        }
    }

    if let Some(max) = rules.max_length {
        if value.len() > max {
            return Err(format!("Value is too long. Maximum length is {}", max));
        }
    }

    Ok(())
}

pub fn validate_string_pattern(value: &str, pattern: &str) -> Result<(), String> {
    let re = regex::Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;

    if re.is_match(value) {
        Ok(())
    } else {
        Err(format!(
            "Value does not match required pattern: {}",
            pattern
        ))
    }
}

pub fn validate_string_with_rules(value: &str, rules: &ValidationRules) -> Result<(), String> {
    validate_string_length(value, rules)?;

    if let Some(pattern) = &rules.pattern {
        validate_string_pattern(value, pattern)?;
    }

    Ok(())
}

pub fn is_empty_string(value: &str) -> bool {
    value.is_empty()
}

pub fn is_whitespace_only(value: &str) -> bool {
    value.chars().all(|c| c.is_whitespace())
}

pub fn is_valid_string_length(value: &str, min: usize, max: usize) -> bool {
    let len = value.len();
    len >= min && len <= max
}

pub fn contains_control_characters(value: &str) -> bool {
    value.chars().any(|c| c.is_control())
}

pub fn contains_null_byte(value: &str) -> bool {
    value.contains('\0')
}

pub fn count_characters(value: &str) -> usize {
    value.chars().count()
}

pub fn count_bytes(value: &str) -> usize {
    value.as_bytes().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_string_length_min() {
        let rules = ValidationRules::new().with_min_length(3);

        assert!(validate_string_length("abc", &rules).is_ok());
        assert!(validate_string_length("ab", &rules).is_err());
        assert!(validate_string_length("", &rules).is_err());
    }

    #[test]
    fn test_validate_string_length_max() {
        let rules = ValidationRules::new().with_max_length(5);

        assert!(validate_string_length("abc", &rules).is_ok());
        assert!(validate_string_length("abcdef", &rules).is_err());
    }

    #[test]
    fn test_validate_string_length_range() {
        let rules = ValidationRules::new().with_length_range(3, 5);

        assert!(validate_string_length("ab", &rules).is_err());
        assert!(validate_string_length("abc", &rules).is_ok());
        assert!(validate_string_length("abcdef", &rules).is_err());
    }

    #[test]
    fn test_validate_string_pattern() {
        assert!(validate_string_pattern("test@example.com", r"^[\w\.-]+@[\w\.-]+\.\w+$").is_ok());
        assert!(validate_string_pattern("invalid-email", r"^[\w\.-]+@[\w\.-]+\.\w+$").is_err());
    }

    #[test]
    fn test_validate_string_pattern_invalid_regex() {
        assert!(validate_string_pattern("test", "[").is_err());
    }

    #[test]
    fn test_validate_string_with_rules() {
        let rules = ValidationRules::new()
            .with_length_range(5, 10)
            .with_pattern(r"^[a-z]+$");

        assert!(validate_string_with_rules("hello", &rules).is_ok());
        assert!(validate_string_with_rules("hi", &rules).is_err());
        assert!(validate_string_with_rules("hello123", &rules).is_err());
    }

    #[test]
    fn test_empty_and_whitespace() {
        assert!(is_empty_string(""));
        assert!(!is_empty_string("a"));

        assert!(is_whitespace_only("   "));
        assert!(is_whitespace_only("\t\n\r"));
        assert!(!is_whitespace_only(" abc "));
    }

    #[test]
    fn test_valid_string_length() {
        assert!(is_valid_string_length("hello", 3, 10));
        assert!(is_valid_string_length("hi", 1, 5));
        assert!(!is_valid_string_length("hello", 10, 20));
    }

    #[test]
    fn test_control_characters() {
        assert!(!contains_control_characters("hello"));
        assert!(contains_control_characters("hello\x00world"));
        assert!(contains_control_characters("hello\nworld"));
    }

    #[test]
    fn test_null_byte() {
        assert!(!contains_null_byte("hello"));
        assert!(contains_null_byte("hello\0world"));
    }

    #[test]
    fn test_character_counting() {
        assert_eq!(count_characters("hello"), 5);
        assert_eq!(count_characters("héllo"), 5);
        assert_eq!(count_bytes("héllo"), 6);
    }

    #[test]
    fn test_string_length_edge_cases() {
        let rules = ValidationRules::new().with_min_length(1);
        assert!(validate_string_length("", &rules).is_err());

        let rules2 = ValidationRules::new().with_max_length(0);
        assert!(validate_string_length("", &rules2).is_ok());
        assert!(validate_string_length("a", &rules2).is_err());
    }

    #[test]
    fn test_pattern_with_unicode() {
        assert!(validate_string_pattern("café", r"^[\w\s]+$").is_ok());
        assert!(validate_string_pattern("Здравствуйте", r"^[\w\s]+$").is_ok());
    }

    #[test]
    fn test_complex_patterns() {
        let rules = ValidationRules::new().with_pattern(r"^[a-zA-Z0-9_]+$");

        assert!(validate_string_with_rules("valid_123", &rules).is_ok());
        assert!(validate_string_with_rules("valid-123", &rules).is_err());
    }
}
