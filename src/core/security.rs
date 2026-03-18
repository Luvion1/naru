pub mod path_sanitizer {
    pub use crate::core::path_sanitizer::*;
}

pub mod input_validator {
    pub use crate::core::input_validator::*;
}

pub mod file_security {
    pub use crate::core::file_security::*;
}

pub use path_sanitizer::normalize_path;
pub use path_sanitizer::sanitize_file_path;
pub use path_sanitizer::sanitize_file_path_internal;

pub use input_validator::is_valid_config_key;
pub use input_validator::is_valid_environment_name;
pub use input_validator::normalize_config_key;
pub use input_validator::normalize_environment_name;
pub use input_validator::sanitize_string_value;
pub use input_validator::validate_config_key;
pub use input_validator::validate_environment_name;

pub use file_security::check_file_size;
pub use file_security::get_file_size;
pub use file_security::is_file_readable;
pub use file_security::is_file_writable;
pub use file_security::is_symlink;
pub use file_security::resolve_and_validate_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backward_compatibility() {
        assert!(sanitize_file_path("config.json").is_ok());
        assert!(validate_environment_name("dev").is_ok());
        assert!(validate_config_key("API_KEY").is_ok());
    }
}
