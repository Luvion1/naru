# 🕵️ Audit System & Hash Chaining

Naru features an industrial-grade auditing system that provides a transparent and immutable history of all configuration changes.

## 🔗 How Hash Chaining Works

Naru's audit log behaves like a cryptographic ledger. Each entry is linked to the one before it using a SHA-256 hash.

### 🧬 Anatomy of a Log Entry
```json
{
  "timestamp": "2026-01-27T10:00:00Z",
  "action": "SET",
  "environment": "production",
  "key": "API_KEY",
  "old_value": "********",
  "new_value": "********",
  "user": "admin",
  "previous_hash": "a1b2c3d4...",
  "hash": "e5f6g7h8..."
}
```

1. **Genesis**: The first entry uses a hardcoded "Genesis Hash" (`0000...0000`) as its `previous_hash`.
2. **Linking**: Every subsequent entry reads the `hash` of the last line in `audit.log` and stores it in its `previous_hash` field.
3. **Hashing**: The current entry's `hash` is then calculated by hashing all its contents *including* the `previous_hash`.

## 🛡️ Tamper Detection

If an attacker modifies a previous log entry (e.g., to hide an unauthorized change):
1. The `hash` of the modified entry will no longer match its content.
2. The `previous_hash` of the next entry will no longer match the recalculated hash of the modified entry.
3. The entire chain from the point of tampering becomes invalid.

### Verifying Integrity
You can verify the audit trail at any time using:
```bash
naru audit verify
```
Naru will iterate through the entire log, recalculate hashes, and ensure the chain is unbroken.

## 🔒 Automatic Secret Masking

Naru protects your secrets even in the logs. If a configuration key contains sensitive keywords (like `pass`, `secret`, `key`, `token`, `auth`), Naru automatically:
- Detects the sensitivity.
- Replaces the `old_value` and `new_value` in the audit log with `********`.
- **Note**: The hash is still calculated using the *masked* value, ensuring that log verification doesn't require access to the actual plaintext secrets.

## 👤 User Identification

Naru automatically identifies the person making changes by reading the `$USER` (Linux/macOS) or `$USERNAME` (Windows) environment variables, ensuring accountability in team environments.