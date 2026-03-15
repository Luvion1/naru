# Changelog

All notable changes to this project will be documented in this file.

## [0.6.2] - 2026-03-15
### Bug Fixes
- **Race Condition**: Fixed data loss during concurrent writes by adding global mutex lock in `locking.rs`
- **Audit Log Integrity**: Fixed audit verification failure after concurrent operations by properly scoping file locks
- **Config File Creation**: Auto-create config file if not exists before locking to prevent "No such file" errors
- **Secret Encryption Hang**: Removed blocking rate limiter from key derivation (Argon2 provides sufficient protection)
- **Backup Extension**: Fixed misleading `.tar.gz` extension - now correctly uses `.json` with warning

### Improvements
- Added retry logic with exponential backoff for file locking
- Enhanced error messages for better debugging

## [0.6.1] - 2026-03-15
### Security Fixes
- **Race Condition Prevention**: Added deprecation warnings to `save_json()` and `load_json()` in favor of atomic operations (`atomic_update_config()`, `lock_file()`)
- **Key Zeroization**: Implemented secure memory zeroization for encryption keys using `zeroize` crate in `derive_key()`, `derive_key_secure()`, `encrypt_data()`, and `decrypt_data()`
- **Weak Key Detection**: Enhanced `is_key_too_weak()` with comprehensive checks for sequential patterns, alternating patterns, and low entropy keys
- **Secure Memory Allocation**: Added `Zeroizing` wrapper for plaintext buffers during encryption/decryption operations
- **Unicode Normalization**: Added early null byte detection before Unicode normalization in `validate_config_key()` and `validate_environment_name()`
- **Helper Functions**: Added `normalize_config_key()` and `normalize_environment_name()` for consistent Unicode storage

### Fixed
- Integer overflow test setup issue in penetration tests
- All 257 tests passing (100% test suite success rate)

### Deprecated
- `save_json()`: Use `atomic_update_config()` for config files to prevent race conditions
- `load_json()`: Use `atomic_read_config()` or `lock_file()` for safer concurrent access

## [0.6.0] - 2026-03-15
### Added
- Path traversal protection with Unicode normalization
- Argon2 key derivation for enhanced security
- Rate limiting for decryption operations
- Internal `sanitize_file_path_internal` for test compatibility
- Comprehensive penetration testing suite (8 exploit scenarios)
- Deep security analysis tests (race conditions, DoS, info leaks)

### Fixed
- **CRITICAL**: Fixed 10 failing tests in penetration and security test suites
- Removed unused `TooManyAttempts` variant from `RateLimitError`
- Dead code cleanup in test files
- Clippy warnings and formatting issues
- Race condition test failures with proper error handling
- Directory traversal test false positives
- Schema test edge cases with many fields

### Test Improvements
- Replaced binary command calls with direct API calls for reliability
- Added proper `NARU_ENCRYPTION_KEY` setup for integration tests
- Fixed directory cleanup to prevent use-after-move errors
- Implemented safe option chaining pattern across all tests
- Test result: 257 passed, 0 failed (was: 247 passed, 10 failed)

## [0.5.0] - 2026-03-13
### Security
- Enhanced path sanitization to prevent directory traversal attacks
- Improved error handling for security-critical operations

## [0.4.0] - 2026-01-29
### Added
- Advanced validation rules for configuration values
- Tamper-evident audit logging system with integrity checks
- Sensitive value masking in audit logs

### Fixed
- Test race conditions in concurrent environments
- Core module stability and error handling improvements

## [0.3.0] - 2026-01-29
### Added
- New brand identity with a modern SVG logo and mascot
- Comprehensive documentation suite (Guides, Components, Reference)

## [0.2.0] - 2026-01-25
### Added
- Advanced validation rules for configuration values.
- Tamper-evident audit logging system with integrity checks.
- Sensitive value masking in audit logs.
- New brand identity with a modern SVG logo and mascot.
- Comprehensive documentation suite (Guides, Components, Reference).

### Fixed
- Test race conditions in concurrent environments.
- Core module stability and error handling improvements.

## [0.1.0] - 2026-01-25
### Added
- Initial release of Naru.
- AES-256-GCM encryption for secret values.
- Schema validation support (String, Integer, Boolean).
- Multi-environment support (dev, staging, prod).
- Audit logging system with sensitive value masking.
- Import/Export functionality for .env, JSON, and YAML.
- Interactive schema wizard.
- Professional GitHub CI/CD workflows and documentation.
