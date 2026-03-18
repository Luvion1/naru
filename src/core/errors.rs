#[cfg(test)]
mod tests {
    use crate::core::error_kind::NaruError;

    #[test]
    fn test_error_re_exports() {
        let err = NaruError::config("test");
        assert_eq!(err.to_string(), "Configuration error: test");
    }
}
