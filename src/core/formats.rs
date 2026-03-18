pub use crate::core::format_trait::ConfigFormat;
pub use crate::core::json_format::JsonFormat;
pub use crate::core::properties_format::PropertiesFormat;
pub use crate::core::toml_format::TomlFormat;

pub use crate::core::format_trait::save_config_as_format;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_re_exports() {
        let format = JsonFormat;
        assert_eq!(format.name(), "JSON");
    }
}
