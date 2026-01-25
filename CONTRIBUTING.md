# Contributing to Naru

First off, thank you for considering contributing to Naru! It's people like you that make Naru such a great tool.

## 🌈 Code of Conduct
By participating in this project, you agree to abide by our terms. Please be respectful and professional in all interactions.

## 🚀 How Can I Contribute?

### Reporting Bugs
- Use the **Bug Report** template on GitHub.
- Describe the expected behavior vs actual behavior.
- Include your OS and Naru version.

### Suggesting Enhancements
- Check if the feature has already been suggested.
- Provide a clear use-case for the enhancement.

### Pull Requests
1. Fork the repo and create your branch from `master`.
2. If you've added code that should be tested, add tests.
3. Ensure the test suite passes (`cargo test`).
4. Format your code with `cargo fmt`.
5. Run `cargo clippy` to check for common mistakes.
6. Issue the PR!

## 💻 Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone your fork
git clone https://github.com/YOUR_USERNAME/naru.git
cd naru

# Build and test
cargo build
cargo test
```

## 📜 Style Guide
- Follow standard Rust naming conventions (`snake_case` for functions/variables, `PascalCase` for types).
- Use `anyhow` for application-level error handling.
- Keep functions small and focused on a single responsibility.

Thank you for your support!
