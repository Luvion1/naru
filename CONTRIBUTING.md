# Contributing to Naru

Thank you for your interest in Naru! We welcome contributions from everyone to help make our security ecosystem stronger.

## 🛠️ Development Setup

1. **Prerequisites**: Ensure you have the latest stable Rust toolchain installed.
2. **Clone**: `git clone https://github.com/Luvion1/naru.git`
3. **Test**: Run `cargo test` to ensure all 257+ tests pass.
4. **Lint**: Use `cargo clippy -- -D warnings` and `cargo fmt --all -- --check`.

## 🧪 Testing Requirements

All contributions must maintain our test coverage:

### Running Tests
```bash
# Full test suite
cargo test

# Specific test suites
cargo test penetration_tests      # Penetration testing (8 exploits)
cargo test security_tests         # Security validation
cargo test deep_security_tests    # Advanced security analysis
cargo test core                   # Core module tests

# With output for debugging
cargo test -- --nocapture
```

### Test Coverage Expectations
- **Bug fixes**: Must include regression tests
- **New features**: Must include unit and integration tests
- **Security changes**: Must pass all penetration tests
- **Performance changes**: Should include benchmark comparisons

## 📜 Contribution Guidelines

### Bug Reports
- Use the provided GitHub issue template.
- Include reproduction steps and your environment details (OS, Naru version).

### Feature Requests
- Please open an issue to discuss the feature before implementing it.
- Security-related features require a brief design proposal.

### Pull Requests
- Keep PRs focused on a single change.
- **Tests are mandatory**. Every bug fix or feature must include unit tests.
- Ensure your code follows the existing style and is "Clippy-clean".

## 🛡️ Security Vulnerabilities
**Do not open a public issue.** Please report security vulnerabilities privately to the maintainers at `security@luvion.io`.

## 🏗️ Commit Message Convention
We follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` for new features.
- `fix:` for bug fixes.
- `docs:` for documentation changes.
- `refactor:` for code changes that neither fix a bug nor add a feature.

---
*Happy Hacking!*