# Publishing Guide for Naru

This document outlines the steps to publish Naru to GitHub and crates.io.

## GitHub Release (Automated)

The GitHub release is **automatically triggered** when pushing a version tag:

```bash
# Tag is already pushed: v0.6.6
# GitHub Actions will:
# 1. Build binaries for Linux, macOS (x64 & ARM), and Windows
# 2. Create a GitHub Release with CHANGELOG.md as description
# 3. Attach all compiled binaries to the release
```

### Monitor GitHub Actions
Visit: https://github.com/Luvion1/naru/actions

## crates.io Publishing (Manual)

### Prerequisites

1. **Create crates.io account** (if not already done):
   ```bash
   cargo login
   # Visit https://crates.io/me to get your API token
   # Paste the token when prompted
   ```

2. **Verify package readiness**:
   ```bash
   cargo package
   # Should complete without errors
   ```

### Publish to crates.io

```bash
# Dry run first (recommended)
cargo publish --dry-run

# Actual publish
cargo publish
```

### Post-Publish Verification

1. **Check crates.io page**: https://crates.io/crates/naru-config
2. **Verify installation**:
   ```bash
   cargo install naru-config
   naru-config --version
   ```

## Version Management

### Current Version: 0.6.6

### Next Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with new features
- [ ] Update test count in `README.md`
- [ ] Run `cargo test` - all tests must pass
- [ ] Run `cargo clippy -- -D warnings` - no warnings
- [ ] Run `cargo fmt --all -- --check` - properly formatted
- [ ] Commit changes with descriptive message
- [ ] Create annotated tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
- [ ] Push to GitHub: `git push origin master && git push origin vX.Y.Z`
- [ ] Publish to crates.io: `cargo publish`

## GitHub Release Notes

The release workflow automatically:
- Builds for 4 platforms (Linux x64, macOS x64, macOS ARM, Windows x64)
- Creates a GitHub Release with the tag name
- Uses `CHANGELOG.md` as the release description
- Attaches all compiled binaries

## crates.io Metadata

Current package info (from `Cargo.toml`):
- **Name**: naru-config
- **License**: MIT
- **Repository**: https://github.com/Luvion1/naru
- **Keywords**: config, encryption, security, aes, cli
- **Categories**: command-line-utilities, cryptography

## Troubleshooting

### Package validation fails
```bash
# Check for uncommitted changes
git status

# Verify all files are tracked
cargo package --list
```

### crates.io publish fails
```bash
# Check if version already exists
# Visit: https://crates.io/crates/naru-config/versions

# Increment version in Cargo.toml if needed
```

### GitHub Actions fails
1. Check the Actions tab on GitHub
2. Review error logs
3. Fix issues locally (test, clippy, fmt)
4. Push fix and re-tag if necessary
