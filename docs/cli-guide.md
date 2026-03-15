# 💻 CLI Guide

**Version:** 0.6.0  
**Install:** `cargo install naru-config`  

This guide provides a comprehensive reference for all Naru commands and their options.

## 🏁 Project Initialization

### `naru init`
Initializes a new Naru project in the current directory.
- Creates `.naru/` directory.
- Generates default `config.json` and `schema.json`.

## ⚙️ Configuration Management

### `naru set <KEY=VALUE>`
Sets a configuration value in the default environment.
- `--env, -e`: Target environment (default: `development`).
- `--secret, -s`: Encrypt the value.

### `naru get <KEY>`
Retrieves a configuration value.
- `--env, -e`: Environment to read from.

### `naru list`
Lists all configuration keys and values in an environment.
- `--env, -e`: Target environment.

### `naru diff <ENV1> <ENV2>`
Compares configuration values between two environments and shows the differences.

## 📐 Schema Management

### `naru schema add`
Adds a new field to the schema. Can be used interactively or with flags:
- `--key`: Field name.
- `--type`: Data type (`string`, `integer`, `boolean`).
- `--pattern`: Regex pattern for strings.
- `--secret`: Mark as secret.

### `naru schema view`
Displays the current validation schema in a readable format.

## 🕵️ Audit & Security

### `naru audit log`
Shows the most recent configuration changes.
- `--count`: Number of entries to show (default: 10).

### `naru audit verify`
Performs a cryptographic integrity check on the entire audit log trail.

### `naru validate`
Validates all configuration values in the project against the defined schema rules.

## 🔄 Import & Export

### `naru import <FILE>`
Imports configurations from external files.
- Supports `.env`, `.json`, `.yaml`.
- `--env, -e`: Target environment.

### `naru export <ENV>`
Exports configurations from Naru to a file.
- `--file`: Output path.
- `--format`: `env` or `yaml`.

## 🔐 Cryptography Commands

### `naru crypto encrypt <INPUT> <OUTPUT>`
Encrypts a standalone file using the project's master key.

### `naru crypto decrypt <INPUT> <OUTPUT>`
Decrypts a previously encrypted file.

## 🧪 Testing & Development

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test suites
cargo test penetration_tests    # Penetration testing suite
cargo test security_tests       # Security validation tests
cargo test deep_security_tests  # Advanced security analysis

# Run tests with output
cargo test -- --nocapture
```

### Test Coverage
- **257 tests** covering all security aspects
- **Penetration tests** - 8 exploit scenarios
- **Security tests** - Encryption, validation, path traversal
- **Deep security analysis** - Race conditions, DoS, info leaks