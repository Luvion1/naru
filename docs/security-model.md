# 🛡️ Security Model

Naru is designed with a **Zero-Trust** philosophy. This document outlines the cryptographic standards and safety measures implemented to protect your sensitive data.

## 🔑 Cryptography

### 1. Encryption Algorithm
Naru uses **AES-256-GCM** (Advanced Encryption Standard with Galois/Counter Mode).
- **Confidentiality**: Data is encrypted using a 256-bit key.
- **Integrity**: GCM provides an authentication tag that ensures the ciphertext has not been tampered with.
- **Randomness**: Every encryption operation generates a unique 12-byte **Nonce** (number used once) to prevent pattern recognition.

### 2. Key Derivation (KDF)
To ensure strong keys regardless of user password length, Naru employs a **SHA-256 KDF**:
- The `NARU_ENCRYPTION_KEY` environment variable is hashed using SHA-256.
- The resulting 32-byte (256-bit) digest is used as the master encryption key.
- This prevents "short key" vulnerabilities.

## 🛡️ Input Sanitization

### 1. Directory Traversal Protection
Naru implements a multi-stage path sanitization logic:
- **Null Byte Check**: Rejects any path containing `\0`.
- **Component Analysis**: Every path is broken into components. Any component containing `..` (Parent Directory) is strictly rejected.
- **Absolute Path Rejection**: Naru only operates within relative paths to prevent access to system files (e.g., `/etc/passwd`).
- **Separator Unification**: Handles both `/` and `\` to ensure security across Linux, macOS, and Windows.

### 2. Environment Injection
Environment names and configuration keys are validated against strict alphanumeric patterns:
- **Allowed**: `[a-zA-Z0-0_.-]`
- **Rejected**: Any character that could be used for shell injection (`;`, `&`, `|`, `>`, etc.).

## 🕵️ Audit Log Integrity
The audit system uses **Hash Chaining**:
- Each log entry includes `previous_hash` and its own `hash`.
- `hash = SHA256(timestamp + action + env + key + values + previous_hash)`.
- Changing a single bit in the past logs invalidates the entire subsequent chain.

## 🛑 Threat Model & Mitigations

| Threat | Mitigation |
| :--- | :--- |
| **Physical Access** | Config files are encrypted at rest. Audit logs are tamper-evident. |
| **Process Spoofing** | Advisory file locking prevents concurrent process manipulation. |
| **Dictionary Attacks** | Mitigated by SHA-256 KDF (though strong passwords are recommended). |
| **Improper Validation** | Schema enforcement prevents invalid/malicious data types from being stored. |
| **Data Corruption** | Atomic writes and checksums (via GCM tags) detect and prevent corruption. |