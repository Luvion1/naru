<p align="center">
  <img src="assets/logo.svg" width="350" alt="Naru Logo">
</p>

# 🛡️ Naru

### *Securing the Backbone of Modern Applications*

[![Rust 2021](https://img.shields.io/badge/Rust-2021-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![AES-256-GCM](https://img.shields.io/badge/Security-AES--256--GCM-green?style=for-the-badge)](docs/security-model.md)
[![Audit Chained](https://img.shields.io/badge/Audit-Hash--Chained-blue?style=for-the-badge)](docs/audit-system.md)
[![Tests](https://img.shields.io/badge/Tests-264%20passed-brightgreen?style=for-the-badge)]()
[![License](https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge)](LICENSE)

**Naru** (naru-config) is an industrial-grade, security-first configuration engine. Built with Rust 2021 edition, it provides a tamper-evident, schema-enforced ecosystem for managing application secrets and environment variables in high-stakes production environments.

[**Explore Documentation**](docs/cli-guide.md) • [**Report an Issue**](https://github.com/Luvion1/naru/issues) • [**Request a Feature**](https://github.com/Luvion1/naru/issues)

---

## 🔥 Why Naru?

| Feature | Description |
| :--- | :--- |
| **Zero-Trust Encryption** | All sensitive data is protected by **AES-256-GCM** with **SHA-256** key derivation. |
| **Immutable Audit Trail** | Every mutation is cryptographically signed and chained, creating a tamper-evident history. |
| **Industrial Validation** | Enforce strict types, numeric ranges, and **Regex patterns** before data ever leaves the CLI. |
| **Atomic & Thread-Safe** | OS-level advisory locking ensures zero data corruption during concurrent operations. |
| **Modern Interop** | Native handling of `.env`, `YAML`, `JSON`, and `TOML` with intelligent merging logic. |
| **Battle-Tested** | 264+ automated tests including penetration testing and security analysis. |

---

## 🚀 Getting Started in 3 Steps

### 1. Installation

**From crates.io:**
```bash
cargo install naru-config
```

**From source:**
```bash
git clone https://github.com/Luvion1/naru.git
cd naru
cargo build --release --locked
sudo cp target/release/naru /usr/local/bin/
```

### 2. Initialize your Vault
```bash
export NARU_ENCRYPTION_KEY="your-strong-master-password"
naru init
```

### 3. Secure a Configuration
```bash
# Define the validation rule
naru schema add --key STRIPE_KEY --type string --secret --pattern "^sk_live_.*$"

# Set the value (automatically encrypted and validated)
naru set STRIPE_KEY=sk_live_51Pq... --env production
```

---

## 🏛️ Architecture at a Glance

Naru follows a **Clean Architecture** pattern, isolating its cryptographic core from external I/O.

- **`src/core`**: The Stateless Engine. Pure business logic, validation, and crypto.
- **`src/cli`**: The Interface. High-performance command parsing and TUI.
- **`src/persistence`**: The Safe. Atomic file operations and OS-level locking.

---

## 🧪 Testing & Development

Naru includes a comprehensive testing suite with **257+ automated tests**:

```bash
# Run all tests
cargo test

# Run penetration tests
cargo test penetration_tests

# Run security analysis
cargo test deep_security_tests

# Run with output
cargo test -- --nocapture
```

### Test Coverage
- ✅ **Penetration Tests** - 8 exploit scenarios (race conditions, path traversal, injection)
- ✅ **Security Tests** - Encryption, validation, audit integrity
- ✅ **Deep Security Analysis** - DoS, timing attacks, information leaks
- ✅ **Integration Tests** - End-to-end workflow validation

---

## 🤝 Contributing

We believe in open security. Check our [Contributing Guide](CONTRIBUTING.md) to see how you can help strengthen the Naru ecosystem.

---
<p align="center">
  <b>Developed with precision for the security-conscious engineer.</b><br>
  Released under the MIT License.
</p>
