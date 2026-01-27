# 📐 Validation Schema

Naru uses a schema-driven approach to ensure that your configurations are always correct and your application is safe from invalid inputs.

## 📋 Field Definitions

A schema consists of multiple field definitions. Each field defines the rules for a specific configuration key.

### Basic Schema Structure
```json
{
  "key": "APP_PORT",
  "type": "integer",
  "description": "Port the server listens on",
  "is_secret": false,
  "validation": {
    "min_value": 1024,
    "max_value": 65535
  }
}
```

## 🛠️ Supported Data Types

| Type | Description | 
| :--- | :--- |
| `string` | UTF-8 text strings. |
| `integer` | 64-bit signed integers (i64). |
| `boolean` | `true` or `false`. |

## ✅ Validation Rules

### String Validation
- `min_length`: Minimum number of characters.
- `max_length`: Maximum number of characters.
- `pattern`: A **Regular Expression (Regex)** the value must match.

### Integer Validation
- `min_value`: Minimum numeric value.
- `max_value`: Maximum numeric value.

## 🧩 Advanced: Regex Validation

Naru supports standard Rust-flavored regex for complex string validation.

**Example: Email Validation**
```bash
naru schema add --key ADMIN_EMAIL --type string --pattern "^[\\w\\.-]+@[\\w\\.-]+\\.\\w+$"
```

**Example: Database URL Format**
```bash
naru schema add --key DB_URL --type string --pattern "^postgres://.*:.*@.*:.*$"
```

## 🔒 The `is_secret` Flag

When a field is marked as `is_secret: true` in the schema:
1. **Auto-Encryption**: Any value set for this key will be automatically encrypted using AES-256-GCM.
2. **Masking**: Values will be masked in audit logs.
3. **Safety**: Naru will refuse to export these values in plaintext unless explicitly requested.

## 🚀 Usage

Define your schema interactively:
```bash
naru schema view
naru schema add
```

Or validate your current configuration against the schema:
```bash
naru validate
```