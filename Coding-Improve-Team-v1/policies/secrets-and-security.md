# Secrets and Security Policy

## Never Commit

- API keys
- Tokens
- Passwords
- SSH private keys
- Cloud credentials
- `.env` files with secrets

## Review Points

- Input validation
- File path handling
- Shell command injection
- Network calls
- Dependency changes
- Unsafe deserialization

## Response

If a secret is found, stop and warn the user. Do not repeat the secret in full.
