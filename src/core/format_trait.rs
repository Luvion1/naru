use crate::core::config_model::ConfigFile;
use anyhow::Result;

pub trait ConfigFormat: Send + Sync {
    fn name(&self) -> &str;
    fn extension(&self) -> &str;
    fn serialize(&self, config: &ConfigFile) -> Result<String>;
    fn deserialize(&self, data: &str) -> Result<ConfigFile>;
}

pub trait FormatDetector {
    fn detect_format(data: &str) -> Option<&'static str>;
}

pub fn detect_format_from_extension(extension: &str) -> Option<&'static str> {
    match extension.to_lowercase().as_str() {
        "json" => Some("json"),
        "toml" => Some("toml"),
        "yaml" | "yml" => Some("yaml"),
        "env" => Some("env"),
        "properties" => Some("properties"),
        _ => None,
    }
}

pub fn detect_format_from_content(content: &str) -> Option<&'static str> {
    let trimmed = content.trim();

    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Some("json");
    }

    if trimmed.starts_with("---") || trimmed.contains(": ") {
        return Some("yaml");
    }

    if trimmed.contains('=')
        && trimmed
            .lines()
            .all(|l| l.contains('=') || l.trim().is_empty() || l.trim().starts_with('#'))
    {
        return Some("env");
    }

    if trimmed.contains('[') && trimmed.contains(']') {
        return Some("toml");
    }

    None
}

use std::fs;
use std::path::Path;

pub fn save_config_as_format<P: AsRef<Path>>(
    path: P,
    config: &ConfigFile,
    format: &dyn ConfigFormat,
) -> Result<()> {
    let serialized = format.serialize(config)?;
    fs::write(path, serialized)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_from_extension() {
        assert_eq!(detect_format_from_extension("json"), Some("json"));
        assert_eq!(detect_format_from_extension("JSON"), Some("json"));
        assert_eq!(detect_format_from_extension("toml"), Some("toml"));
        assert_eq!(detect_format_from_extension("yaml"), Some("yaml"));
        assert_eq!(detect_format_from_extension("yml"), Some("yaml"));
        assert_eq!(detect_format_from_extension("env"), Some("env"));
        assert_eq!(detect_format_from_extension("unknown"), None);
    }

    #[test]
    fn test_detect_format_from_content_json() {
        let json = r#"{"key": "value"}"#;
        assert_eq!(detect_format_from_content(json), Some("json"));

        let json_array = r#"[1, 2, 3]"#;
        assert_eq!(detect_format_from_content(json_array), Some("json"));
    }

    #[test]
    fn test_detect_format_from_content_yaml() {
        let yaml = "key: value\nfoo: bar";
        assert_eq!(detect_format_from_content(yaml), Some("yaml"));

        let yaml_with_doc = "---\nkey: value";
        assert_eq!(detect_format_from_content(yaml_with_doc), Some("yaml"));
    }

    #[test]
    fn test_detect_format_from_content_env() {
        let env = "KEY=value\nFOO=bar";
        assert_eq!(detect_format_from_content(env), Some("env"));

        let env_with_comments = "# Comment\nKEY=value";
        assert_eq!(detect_format_from_content(env_with_comments), Some("env"));
    }

    #[test]
    fn test_detect_format_from_content_toml() {
        let toml = "[section]\nkey = \"value\"";
        assert_eq!(detect_format_from_content(toml), Some("toml"));
    }
}
