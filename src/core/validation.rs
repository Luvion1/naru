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

    #[test]
    fn test_validate_empty_string_with_min_length() {
        let field = FieldDefinition {
            key: "empty_test".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(1),
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value("", &field).is_err());
    }

    #[test]
    fn test_validate_max_length_boundary() {
        let field = FieldDefinition {
            key: "boundary_test".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: Some(5),
                min_value: None,
                max_value: None,
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value("12345", &field).is_ok()); // Exactly max length
        assert!(validate_value("123456", &field).is_err()); // Over max length
    }

    #[test]
    fn test_validate_integer_zero() {
        let field = FieldDefinition {
            key: "zero_test".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: Some(-10),
                max_value: Some(10),
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value("0", &field).is_ok());
    }

    #[test]
    fn test_validate_integer_with_whitespace() {
        let field = FieldDefinition {
            key: "whitespace_test".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        assert!(validate_value(" 123 ", &field).is_err()); // Should fail due to whitespace
        assert!(validate_value("123", &field).is_ok()); // Should pass without whitespace
    }

    #[test]
    fn test_validate_special_float_strings() {
        let field = FieldDefinition {
            key: "float_test".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        assert!(validate_value("1.0", &field).is_err()); // Float string should fail for integer
        assert!(validate_value("inf", &field).is_err()); // Infinity should fail
        assert!(validate_value("nan", &field).is_err()); // NaN should fail
    }

    #[test]
    fn test_validate_boolean_case_sensitivity() {
        let field = FieldDefinition {
            key: "bool_case_test".to_string(),
            r#type: "boolean".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        assert!(validate_value("True", &field).is_err()); // Should be case-sensitive
        assert!(validate_value("TRUE", &field).is_err()); // Should be case-sensitive
        assert!(validate_value("False", &field).is_err()); // Should be case-sensitive
        assert!(validate_value("FALSE", &field).is_err()); // Should be case-sensitive
        assert!(validate_value("true", &field).is_ok()); // Only lowercase should pass
        assert!(validate_value("false", &field).is_ok()); // Only lowercase should pass
    }

    #[test]
    fn test_validate_integer_overflow_bounds() {
        let field = FieldDefinition {
            key: "overflow_test".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: Some(i64::MIN),
                max_value: Some(i64::MAX),
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value(&i64::MAX.to_string(), &field).is_ok());
        assert!(validate_value(&i64::MIN.to_string(), &field).is_ok());
    }

    #[test]
    fn test_validate_string_with_unicode() {
        let field = FieldDefinition {
            key: "unicode_test".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(1),
                max_length: Some(10),
                min_value: None,
                max_value: None,
                pattern: Some(r"^[a-zA-Zü]+$".to_string()), // Letters and accented chars
            }),
            is_secret: false,
        };
        assert!(validate_value("hello", &field).is_ok());
        assert!(validate_value("hullo", &field).is_ok()); // Simple letter combination
        assert!(validate_value("üüü", &field).is_ok()); // Only accented chars
        assert!(validate_value("", &field).is_err()); // Empty string too short
        assert!(validate_value("hello_world_that_is_way_too_long", &field).is_err());
        // Too long
    }

    #[test]
    fn test_validate_multiple_validation_errors() {
        let field = FieldDefinition {
            key: "multi_error_test".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(10),
                max_length: Some(5), // Impossible condition
                min_value: None,
                max_value: None,
                pattern: Some(r"^[A-Z]+$".to_string()), // Only uppercase
            }),
            is_secret: false,
        };
        // Value that satisfies one constraint but not others
        assert!(validate_value("ABC", &field).is_err()); // Too short and too long at the same time
        assert!(validate_value("abc", &field).is_err()); // Wrong case, too short, too long
    }

    #[test]
    fn test_validate_integer_with_extreme_values() {
        let field = FieldDefinition {
            key: "extreme_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_value: Some(i64::MIN),
                max_value: Some(i64::MAX),
                min_length: None,
                max_length: None,
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value(&i64::MIN.to_string(), &field).is_ok());
        assert!(validate_value(&i64::MAX.to_string(), &field).is_ok());
    }

    #[test]
    fn test_validate_integer_with_leading_zeros() {
        let field = FieldDefinition {
            key: "int_with_zeros".to_string(),
            r#type: "integer".to_string(),
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
        assert!(validate_value("007", &field).is_ok()); // Should parse as 7
        assert!(validate_value("000", &field).is_ok()); // Should parse as 0
        assert!(validate_value("010", &field).is_ok()); // Should parse as 10
    }

    #[test]
    fn test_validate_integer_with_negative_values() {
        let field = FieldDefinition {
            key: "negative_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_value: Some(-100),
                max_value: Some(-10),
                min_length: None,
                max_length: None,
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value("-50", &field).is_ok());
        assert!(validate_value("-101", &field).is_err()); // Below minimum
        assert!(validate_value("-9", &field).is_err()); // Above maximum
    }

    #[test]
    fn test_validate_string_with_extreme_lengths() {
        let field = FieldDefinition {
            key: "extreme_length".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(0),
                max_length: Some(10000),
                min_value: None,
                max_value: None,
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value("", &field).is_ok()); // Zero length
        let long_string = "x".repeat(10000);
        assert!(validate_value(&long_string, &field).is_ok()); // Max length
        let too_long_string = "x".repeat(10001);
        assert!(validate_value(&too_long_string, &field).is_err()); // Too long
    }

    #[test]
    fn test_validate_string_with_only_whitespace() {
        let field = FieldDefinition {
            key: "whitespace_only".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(1),
                max_length: Some(10),
                min_value: None,
                max_value: None,
                pattern: None,
            }),
            is_secret: false,
        };
        assert!(validate_value("   ", &field).is_ok()); // Whitespace is valid
        assert!(validate_value("\t\n\r", &field).is_ok()); // Various whitespace chars
    }

    #[test]
    fn test_validate_string_with_regex_edge_cases() {
        let field = FieldDefinition {
            key: "regex_edge".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some(r"^[a-zA-Z0-9_]*$".to_string()), // Alphanumeric and underscore only
            }),
            is_secret: false,
        };
        assert!(validate_value("valid123", &field).is_ok());
        assert!(validate_value("Valid_123", &field).is_ok());
        assert!(validate_value("123", &field).is_ok());
        assert!(validate_value("_valid_", &field).is_ok());
        assert!(validate_value("invalid-char", &field).is_err()); // Contains hyphen
        assert!(validate_value("invalid.char", &field).is_err()); // Contains dot
    }

    #[test]
    fn test_validate_string_with_unicode_regex() {
        let field = FieldDefinition {
            key: "unicode_regex".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some(r"^[a-zA-Z\u{00C0}-\u{017F}]*$".to_string()), // ASCII + Latin Extended-A
            }),
            is_secret: false,
        };
        assert!(validate_value("café", &field).is_ok()); // Contains accented character
        assert!(validate_value("naïve", &field).is_ok()); // Contains diaeresis
        assert!(validate_value("résumé", &field).is_ok()); // Multiple accents
        assert!(validate_value("你好", &field).is_err()); // Chinese characters outside range
    }

    #[test]
    fn test_validate_boolean_with_case_variations() {
        let field = FieldDefinition {
            key: "bool_case".to_string(),
            r#type: "boolean".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        assert!(validate_value("true", &field).is_ok());
        assert!(validate_value("false", &field).is_ok());
        assert!(validate_value("True", &field).is_err()); // Case sensitive
        assert!(validate_value("TRUE", &field).is_err()); // Case sensitive
        assert!(validate_value("False", &field).is_err()); // Case sensitive
        assert!(validate_value("FALSE", &field).is_err()); // Case sensitive
        assert!(validate_value("1", &field).is_err()); // Not valid boolean
        assert!(validate_value("0", &field).is_err()); // Not valid boolean
    }

    #[test]
    fn test_validate_integer_with_overflow_boundaries() {
        let field = FieldDefinition {
            key: "overflow_test".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_value: Some(i64::MIN),
                max_value: Some(i64::MAX),
                min_length: None,
                max_length: None,
                pattern: None,
            }),
            is_secret: false,
        };
        // Test values just inside and outside the i64 range
        assert!(validate_value(&(i64::MAX as i128).to_string(), &field).is_ok());
        assert!(validate_value(&(i64::MIN as i128).to_string(), &field).is_ok());

        // Values that would overflow when parsed
        assert!(validate_value(&(i128::MAX).to_string(), &field).is_err());
        assert!(validate_value(&(i128::MIN).to_string(), &field).is_err());
    }

    #[test]
    fn test_validate_string_with_control_characters() {
        let field = FieldDefinition {
            key: "control_chars".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(0),
                max_length: Some(100),
                min_value: None,
                max_value: None,
                pattern: Some(r"^[^\x00-\x1F\x7F]+$".to_string()), // No control characters
            }),
            is_secret: false,
        };
        assert!(validate_value("valid string", &field).is_ok());
        assert!(validate_value("string\x07with\x08control", &field).is_err()); // Contains control chars
        assert!(validate_value("string\x7Fwith\x7Fdel", &field).is_err()); // Contains DEL
    }

    #[test]
    fn test_validate_complex_pattern_with_backtracking() {
        let field = FieldDefinition {
            key: "backtrack_pattern".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some(r"^(a+)+$".to_string()), // Pathological regex that can backtrack
            }),
            is_secret: false,
        };
        // This test ensures the regex doesn't cause catastrophic backtracking
        let long_a_string = "a".repeat(1000);
        assert!(validate_value(&long_a_string, &field).is_ok());
    }

    #[test]
    fn test_validate_integer_with_scientific_notation() {
        let field = FieldDefinition {
            key: "scientific_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        // Scientific notation should not be accepted as integer
        assert!(validate_value("1e5", &field).is_err());
        assert!(validate_value("1.23e4", &field).is_err());
        assert!(validate_value("1E5", &field).is_err());
    }

    #[test]
    fn test_validate_integer_with_hexadecimal() {
        let field = FieldDefinition {
            key: "hex_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        // Hexadecimal should not be accepted as integer (only decimal)
        assert!(validate_value("0xFF", &field).is_err());
        assert!(validate_value("0x123", &field).is_err());
        assert!(validate_value("0xabcd", &field).is_err());
    }

    #[test]
    fn test_validate_integer_with_octal() {
        let field = FieldDefinition {
            key: "octal_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        // Octal should not be accepted as integer (only decimal)
        assert!(validate_value("0o755", &field).is_err());
        assert!(validate_value("0o123", &field).is_err());
    }

    #[test]
    fn test_validate_integer_with_binary() {
        let field = FieldDefinition {
            key: "binary_int".to_string(),
            r#type: "integer".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        // Binary should not be accepted as integer (only decimal)
        assert!(validate_value("0b1010", &field).is_err());
        assert!(validate_value("0b1111", &field).is_err());
    }

    #[test]
    fn test_validate_integer_with_leading_plus() {
        let field = FieldDefinition {
            key: "plus_int".to_string(),
            r#type: "integer".to_string(),
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
        assert!(validate_value("+42", &field).is_ok());
        assert!(validate_value("+0", &field).is_ok());
        assert!(validate_value("+100", &field).is_ok());
        assert!(validate_value("+101", &field).is_err()); // Above max
    }

    #[test]
    fn test_validate_string_with_extreme_regex_patterns() {
        let field = FieldDefinition {
            key: "extreme_regex".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some(r"(\d+)+".to_string()), // Potential ReDoS vulnerability
            }),
            is_secret: false,
        };
        // Test with a string that could cause ReDoS
        let malicious_string = "1".repeat(1000) + "a";
        assert!(validate_value(&malicious_string, &field).is_ok()); // Should not hang
    }

    #[test]
    fn test_validate_string_with_unicode_categories() {
        let field = FieldDefinition {
            key: "unicode_category".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: Some(1),
                max_length: Some(100),
                min_value: None,
                max_value: None,
                pattern: Some(r"\w+".to_string()), // Word characters
            }),
            is_secret: false,
        };
        // Test various Unicode categories
        assert!(validate_value("café", &field).is_ok()); // Latin with diacritics
        assert!(validate_value("Здравствуйте", &field).is_ok()); // Cyrillic
                                                                 // The behavior for Arabic/Japanese depends on the regex engine's handling of \w
                                                                 // In Rust's regex, \w typically includes Unicode word characters
                                                                 // So these might pass depending on the implementation
        assert!(validate_value("مرحبا", &field).is_ok()); // Arabic (may be accepted as \w in Rust)
        assert!(validate_value("こんにちは", &field).is_ok()); // Japanese (may be accepted as \w in Rust)
    }

    #[test]
    fn test_validate_string_with_extreme_length_patterns() {
        let field = FieldDefinition {
            key: "long_pattern".to_string(),
            r#type: "string".to_string(),
            description: None,
            validation: Some(ValidationRules {
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                pattern: Some(r"^[a-z]{1,10000}$".to_string()), // Long but limited repetition
            }),
            is_secret: false,
        };
        let long_string = "a".repeat(5000);
        assert!(validate_value(&long_string, &field).is_ok());
    }

    #[test]
    fn test_validate_boolean_with_numeric_strings() {
        let field = FieldDefinition {
            key: "numeric_bool".to_string(),
            r#type: "boolean".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        // Numeric strings should not be valid booleans
        assert!(validate_value("1", &field).is_err());
        assert!(validate_value("0", &field).is_err());
        assert!(validate_value("2", &field).is_err());
        assert!(validate_value("-1", &field).is_err());
    }

    #[test]
    fn test_validate_boolean_with_text_variations() {
        let field = FieldDefinition {
            key: "text_bool".to_string(),
            r#type: "boolean".to_string(),
            description: None,
            validation: None,
            is_secret: false,
        };
        // Various text representations should not be valid
        assert!(validate_value("yes", &field).is_err());
        assert!(validate_value("no", &field).is_err());
        assert!(validate_value("on", &field).is_err());
        assert!(validate_value("off", &field).is_err());
        assert!(validate_value("enable", &field).is_err());
        assert!(validate_value("disable", &field).is_err());
        assert!(validate_value("enabled", &field).is_err());
        assert!(validate_value("disabled", &field).is_err());
    }
}
