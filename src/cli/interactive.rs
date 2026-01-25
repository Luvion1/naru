use dialoguer::{Input, Select, Confirm};
use anyhow::Result;
use crate::core::models::{FieldDefinition, ValidationRules, SchemaFile};

/// Menjalankan wizard interaktif untuk menambahkan field baru ke skema
pub fn prompt_add_field() -> Result<FieldDefinition> {
    let key: String = Input::new()
        .with_prompt("Nama kunci (Key name)")
        .validate_with(|input: &String| -> Result<(), &str> {
            crate::core::security::validate_config_key(input)
        })
        .interact_text()?;

    let types = vec!["string", "integer", "boolean"];
    let type_index = Select::new()
        .with_prompt("Tipe data")
        .items(&types)
        .default(0)
        .interact()?;
    
    let r#type = types[type_index].to_string();

    let description: String = Input::new()
        .with_prompt("Deskripsi (opsional)")
        .allow_empty(true)
        .interact_text()?;
    
    let description_opt = if description.is_empty() { None } else { Some(description) };

    let mut validation = None;
    if Confirm::new()
        .with_prompt("Tambahkan aturan validasi?")
        .default(false)
        .interact()? 
    {
        let mut rules = ValidationRules {
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
        };

        if r#type == "string" {
            let min: String = Input::new().with_prompt("Panjang minimum (biarkan kosong jika tidak ada)").allow_empty(true).interact_text()?;
            rules.min_length = min.parse().ok();
            
            let max: String = Input::new().with_prompt("Panjang maksimum (biarkan kosong jika tidak ada)").allow_empty(true).interact_text()?;
            rules.max_length = max.parse().ok();
        } else if r#type == "integer" {
            let min: String = Input::new().with_prompt("Nilai minimum (biarkan kosong jika tidak ada)").allow_empty(true).interact_text()?;
            rules.min_value = min.parse().ok();
            
            let max: String = Input::new().with_prompt("Nilai maksimum (biarkan kosong jika tidak ada)").allow_empty(true).interact_text()?;
            rules.max_value = max.parse().ok();
        }
        validation = Some(rules);
    }

    let is_secret = Confirm::new()
        .with_prompt("Tandai sebagai rahasia (secret)?")
        .default(false)
        .interact()?;

    Ok(FieldDefinition {
        key,
        r#type,
        description: description_opt,
        validation,
        is_secret,
    })
}

/// Memilih field dari skema yang ada
pub fn prompt_select_field(schema: &SchemaFile) -> Result<String> {
    if schema.fields.is_empty() {
        return Err(anyhow::anyhow!("Skema kosong, tidak ada field untuk dipilih."));
    }

    let keys: Vec<String> = schema.fields.iter().map(|f| f.key.clone()).collect();
    let selection = Select::new()
        .with_prompt("Pilih field")
        .items(&keys)
        .default(0)
        .interact()?;
    
    Ok(keys[selection].clone())
}

/// Menjalankan wizard interaktif untuk mengedit field yang sudah ada
pub fn prompt_edit_field(existing: &FieldDefinition) -> Result<FieldDefinition> {
    let key: String = Input::new()
        .with_prompt("Nama kunci (Key name)")
        .with_initial_text(&existing.key)
        .validate_with(|input: &String| -> Result<(), &str> {
            crate::core::security::validate_config_key(input)
        })
        .interact_text()?;

    let types = vec!["string", "integer", "boolean"];
    let default_type_index = types.iter().position(|&t| t == existing.r#type).unwrap_or(0);
    
    let type_index = Select::new()
        .with_prompt("Tipe data")
        .items(&types)
        .default(default_type_index)
        .interact()?;
    
    let r#type = types[type_index].to_string();

    let description: String = Input::new()
        .with_prompt("Deskripsi (opsional)")
        .allow_empty(true)
        .with_initial_text(existing.description.as_deref().unwrap_or(""))
        .interact_text()?;
    
    let description_opt = if description.is_empty() { None } else { Some(description) };

    let mut validation = None;
    
    let has_validation = existing.validation.is_some();
    if Confirm::new()
        .with_prompt("Edit/Tambahkan aturan validasi?")
        .default(has_validation)
        .interact()? 
    {
        let mut rules = existing.validation.clone().unwrap_or(ValidationRules {
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
        });

        if r#type == "string" {
            let min: String = Input::new()
                .with_prompt("Panjang minimum (biarkan kosong jika tidak ada)")
                .allow_empty(true)
                .with_initial_text(rules.min_length.map(|v| v.to_string()).unwrap_or_default())
                .interact_text()?;
            rules.min_length = min.parse().ok();
            
            let max: String = Input::new()
                .with_prompt("Panjang maksimum (biarkan kosong jika tidak ada)")
                .allow_empty(true)
                .with_initial_text(rules.max_length.map(|v| v.to_string()).unwrap_or_default())
                .interact_text()?;
            rules.max_length = max.parse().ok();
            
            rules.min_value = None;
            rules.max_value = None;
        } else if r#type == "integer" {
            let min: String = Input::new()
                .with_prompt("Nilai minimum (biarkan kosong jika tidak ada)")
                .allow_empty(true)
                .with_initial_text(rules.min_value.map(|v| v.to_string()).unwrap_or_default())
                .interact_text()?;
            rules.min_value = min.parse().ok();
            
            let max: String = Input::new()
                .with_prompt("Nilai maksimum (biarkan kosong jika tidak ada)")
                .allow_empty(true)
                .with_initial_text(rules.max_value.map(|v| v.to_string()).unwrap_or_default())
                .interact_text()?;
            rules.max_value = max.parse().ok();
            
            rules.min_length = None;
            rules.max_length = None;
        } else {
            rules.min_length = None;
            rules.max_length = None;
            rules.min_value = None;
            rules.max_value = None;
        }
        validation = Some(rules);
    }

    let is_secret = Confirm::new()
        .with_prompt("Tandai sebagai rahasia (secret)?")
        .default(existing.is_secret)
        .interact()?;

    Ok(FieldDefinition {
        key,
        r#type,
        description: description_opt,
        validation,
        is_secret,
    })
}
