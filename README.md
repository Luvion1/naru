# Naru - Secure Configuration Manager

Naru is a CLI tool designed for secure, structured, and schema-aware application configuration management. It ensures the integrity of your configuration data across environments (development, staging, production) with automatic encryption and a strict auditing system.

## 🚀 Key Features

- **AES-GCM Encryption**: Automatically secures sensitive data using industry standards.
- **Schema Validation**: Ensures configuration values match expected data types (string, integer, boolean) and rules (min/max).
- **Multi-Environment**: Separate management for different development environments.
- **Audit System**: Records every change (Set, Import, Env, Schema) with sensitive value masking.
- **Flexible Import/Export**: Supports `.env`, `YAML`, and `JSON` formats.
- **Interactive Wizard**: Built-in interactive schema editor for easy data rule maintenance.

## 🛠️ Installation

Naru is built with Rust. Ensure you have the latest Rust toolchain installed.

```bash
# Clone the repository
git clone https://github.com/Luvion1/naru.git
cd naru

# Build the project
cargo build --release
```

The binary will be available at `target/release/naru`.

## 📖 Usage Quick Start

### 1. Initialize a Project
```bash
naru init
```

### 2. Create a Schema (Interactive)
```bash
naru schema --interactive
```

### 3. Set a Configuration
```bash
naru set KEY VALUE --env production --secret
```

### 4. Export Configurations
```bash
naru export --format yaml --output config.yaml
```

## 📁 Detailed Documentation

For more in-depth information, please refer to the documents in the `docs/` directory:

1. [**CLI Reference**](./docs/cli-guide.md) - Complete command reference.
2. [**Core Architecture**](./docs/architecture.md) - Internal design and structure.
3. [**Security Model**](./docs/security-model.md) - Encryption and data protection details.
4. [**Audit System**](./docs/audit-system.md) - How Naru tracks activities.
5. [**Validation Schema**](./docs/validation-schema.md) - Creating data validation rules.

---
© 2026 Naru Project. Built for DevOps security and efficiency.