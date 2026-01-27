# 🚀 Modern Integration Guide

Naru is designed to be a "glue" tool in modern DevOps pipelines. This guide explains how to integrate Naru into your automated workflows.

## 🛠️ CI/CD Pipeline Integration (GitHub Actions)

To use Naru in your pipelines, you can define your encryption key as a GitHub Secret.

```yaml
name: Deploy
on: [push]

jobs:
  validate-configs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Naru
        run: |
          curl -L https://github.com/Luvion1/naru/releases/latest/download/naru-linux -o naru
          chmod +x naru
          sudo mv naru /usr/local/bin/
      
      - name: Validate & Verify Audit
        env:
          NARU_ENCRYPTION_KEY: ${{ secrets.NARU_KEY }}
        run: |
          naru audit verify
          naru validate
```

## 🐳 Docker Integration

You can use Naru to safely inject environment variables during the container build process or at entry point.

**Example Entrypoint:**
```bash
#!/bin/bash
# Decrypt production secrets into a temporary .env file
naru export production --file /tmp/.env --format env
# Source the file and start the app
source /tmp/.env && rm /tmp/.env
exec npm start
```

## ☸️ Kubernetes Secret Management

Naru can bridge the gap between human-managed secrets and Kubernetes Secrets.

```bash
# Export secrets from Naru
naru export staging --file secrets.yaml --format yaml

# Apply to Kubernetes
kubectl create secret generic app-secrets --from-file=config=secrets.yaml
```

## 📦 Automation API (Future)

Naru is designed to be machine-readable. Many commands support silent output for scripts.
- Use `naru get KEY --env prod` to retrieve a single value.
- Use the exit codes for flow control:
    - `0`: Success / Valid
    - `1`: Error
    - `101`: Integrity Failure (CRITICAL)

## 🛡️ Best Practices for Integration

1. **Key Rotation**: Rotate your `NARU_ENCRYPTION_KEY` periodically.
2. **Audit Verification**: Always run `naru audit verify` before any critical deployment to ensure your configuration history hasn't been tampered with.
3. **No Key in Logs**: Never print the `NARU_ENCRYPTION_KEY` in CI logs.
4. **Cleanup**: If you export a `.env` file during a build, ensure it is deleted immediately after use.
