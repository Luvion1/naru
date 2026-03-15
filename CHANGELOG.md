# Changelog

All notable changes to this project will be documented in this file.

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
