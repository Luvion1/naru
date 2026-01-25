# 🛡️ Naru

[![CI](https://github.com/Luvion1/naru/actions/workflows/ci.yml/badge.svg)](https://github.com/Luvion1/naru/actions/workflows/ci.yml)
[![Release](https://github.com/Luvion1/naru/actions/workflows/release.yml/badge.svg)](https://github.com/Luvion1/naru/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

**Naru** is a high-performance, security-first CLI tool for structured configuration management. Built with Rust for speed and safety, it provides an industrial-grade layer for handling sensitive environment variables and application settings with built-in schema enforcement and cryptographic protection.

[Explore Documentation](./docs/architecture.md) • [Report Bug](https://github.com/Luvion1/naru/issues) • [Request Feature](https://github.com/Luvion1/naru/issues)

---

## ✨ Core Pillars

### 🔒 Zero-Trust Security
All sensitive values are encrypted using **AES-256-GCM**. Naru ensures that secrets are never stored in plain text, even in your local configuration files.

### 📐 Schema-Driven Integrity
Forget about runtime crashes due to missing environment variables. Naru enforces strict schema validation (types, ranges, regex) before your configuration is ever deployed.

### 🕵️ Immutable Auditing
Every operation—from manual sets to bulk imports—is cryptographically hashed and logged. Know exactly **who** changed **what** and **when**, with sensitive data automatically masked in logs.

### 🔄 Seamless Interop
Native support for `.env`, `YAML`, and `JSON`. Import your existing configurations and Naru will automatically upgrade them with security and validation.

---

## 🚀 Quick Start

### One-Line Install (Linux & macOS)
```bash
curl -sSf https://raw.githubusercontent.com/Luvion1/naru/master/install.sh | bash
```

### Manual Build
Naru is built with Rust. Ensure you have the latest Rust toolchain installed.

### The 60-Second Workflow
1. **Initialize**: `naru init`
2. **Secure Schema**: `naru schema --interactive`
3. **Set Secrets**: `naru set DB_PASSWORD "p@ssword" --secret`
4. **Validation Check**: `naru validate`
5. **Export**: `naru export --format yaml`

---

## 🛠️ Technology Stack

- **Core**: Rust (2024 Edition)
- **Encryption**: `aes-gcm` (Authenticated Encryption)
- **CLI**: `clap` v4 (Derive API)
- **Serialization**: `serde` (High-performance JSON/YAML/TOML)
- **Terminal UI**: `dialoguer` & `console`

---

## 📖 Deep Dive Documentation

| Document | Description |
| :--- | :--- |
| [**CLI Guide**](./docs/cli-guide.md) | Comprehensive reference for all commands and flags. |
| [**Architecture**](./docs/architecture.md) | Detailed breakdown of the internal DDD-inspired design. |
| [**Security Model**](./docs/security-model.md) | Technical specs on the encryption and threat model. |
| [**Validation**](./docs/validation-schema.md) | How to write complex validation rules for your data. |
| [**Audit System**](./docs/audit-system.md) | Understanding the tamper-evident logging system. |

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) to get started.

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

---
<p align="center">Built with ❤️ for the DevOps community by <b>Luvion1</b></p>
