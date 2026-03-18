use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaFile {
    pub version: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldDefinition {
    pub key: String,
    pub r#type: String,
    pub description: Option<String>,
    pub validation: Option<ValidationRules>,
    pub is_secret: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationRules {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub pattern: Option<String>,
}

impl SchemaFile {
    pub fn new(version: &str) -> Self {
        SchemaFile {
            version: version.to_string(),
            fields: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field: FieldDefinition) {
        self.fields.push(field);
    }

    pub fn get_field(&self, key: &str) -> Option<&FieldDefinition> {
        self.fields.iter().find(|f| f.key == key)
    }

    pub fn get_field_mut(&mut self, key: &str) -> Option<&mut FieldDefinition> {
        self.fields.iter_mut().find(|f| f.key == key)
    }

    pub fn remove_field(&mut self, key: &str) -> Option<FieldDefinition> {
        if let Some(pos) = self.fields.iter().position(|f| f.key == key) {
            Some(self.fields.remove(pos))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &FieldDefinition> {
        self.fields.iter()
    }
}

impl FieldDefinition {
    pub fn new(key: &str, r#type: &str) -> Self {
        FieldDefinition {
            key: key.to_string(),
            r#type: r#type.to_string(),
            description: None,
            validation: None,
            is_secret: false,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_validation(mut self, validation: ValidationRules) -> Self {
        self.validation = Some(validation);
        self
    }

    pub fn secret(mut self) -> Self {
        self.is_secret = true;
        self
    }

    pub fn is_string(&self) -> bool {
        self.r#type == "string"
    }

    pub fn is_integer(&self) -> bool {
        self.r#type == "integer"
    }

    pub fn is_boolean(&self) -> bool {
        self.r#type == "boolean"
    }
}

impl ValidationRules {
    pub fn new() -> Self {
        ValidationRules {
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            pattern: None,
        }
    }

    pub fn with_min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    pub fn with_length_range(mut self, min: usize, max: usize) -> Self {
        self.min_length = Some(min);
        self.max_length = Some(max);
        self
    }

    pub fn with_min_value(mut self, min: i64) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn with_max_value(mut self, max: i64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn with_value_range(mut self, min: i64, max: i64) -> Self {
        self.min_value = Some(min);
        self.max_value = Some(max);
        self
    }

    pub fn with_pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_file_new() {
        let schema = SchemaFile::new("1.0.0");
        assert_eq!(schema.version, "1.0.0");
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_schema_file_add_field() {
        let mut schema = SchemaFile::new("1.0.0");
        let field = FieldDefinition::new("username", "string");
        schema.add_field(field);
        assert_eq!(schema.len(), 1);
    }

    #[test]
    fn test_schema_file_get_field() {
        let mut schema = SchemaFile::new("1.0.0");
        let field = FieldDefinition::new("username", "string");
        schema.add_field(field);

        assert!(schema.get_field("username").is_some());
        assert!(schema.get_field("nonexistent").is_none());
    }

    #[test]
    fn test_schema_file_remove_field() {
        let mut schema = SchemaFile::new("1.0.0");
        let field = FieldDefinition::new("username", "string");
        schema.add_field(field);

        let removed = schema.remove_field("username");
        assert!(removed.is_some());
        assert!(schema.get_field("username").is_none());
    }

    #[test]
    fn test_field_definition_new() {
        let field = FieldDefinition::new("db_host", "string");
        assert_eq!(field.key, "db_host");
        assert_eq!(field.r#type, "string");
        assert!(!field.is_secret);
    }

    #[test]
    fn test_field_definition_builder() {
        let field = FieldDefinition::new("email", "string")
            .with_description("User email address")
            .with_validation(ValidationRules::new().with_pattern(r"^[\w\.-]+@[\w\.-]+\.\w+$"))
            .secret();

        assert_eq!(field.key, "email");
        assert!(field.description.is_some());
        assert!(field.validation.is_some());
        assert!(field.is_secret);
    }

    #[test]
    fn test_field_type_checkers() {
        let string_field = FieldDefinition::new("name", "string");
        let int_field = FieldDefinition::new("age", "integer");
        let bool_field = FieldDefinition::new("active", "boolean");

        assert!(string_field.is_string());
        assert!(!string_field.is_integer());
        assert!(!string_field.is_boolean());

        assert!(int_field.is_integer());
        assert!(bool_field.is_boolean());
    }

    #[test]
    fn test_validation_rules_builder() {
        let rules = ValidationRules::new()
            .with_min_length(3)
            .with_max_length(10)
            .with_min_value(0)
            .with_max_value(120)
            .with_pattern(r"^[a-z]+$");

        assert_eq!(rules.min_length, Some(3));
        assert_eq!(rules.max_length, Some(10));
        assert_eq!(rules.min_value, Some(0));
        assert_eq!(rules.max_value, Some(120));
        assert!(rules.pattern.is_some());
    }

    #[test]
    fn test_validation_rules_length_range() {
        let rules = ValidationRules::new().with_length_range(5, 15);
        assert_eq!(rules.min_length, Some(5));
        assert_eq!(rules.max_length, Some(15));
    }

    #[test]
    fn test_validation_rules_value_range() {
        let rules = ValidationRules::new().with_value_range(-100, 100);
        assert_eq!(rules.min_value, Some(-100));
        assert_eq!(rules.max_value, Some(100));
    }

    #[test]
    fn test_schema_iter() {
        let mut schema = SchemaFile::new("1.0.0");
        schema.add_field(FieldDefinition::new("field1", "string"));
        schema.add_field(FieldDefinition::new("field2", "integer"));

        let keys: Vec<_> = schema.iter().map(|f| f.key.clone()).collect();
        assert_eq!(keys, vec!["field1", "field2"]);
    }
}
