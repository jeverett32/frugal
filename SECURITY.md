# Security Policy

## Supported Versions

Security fixes target the latest `main` branch state first.

## Reporting A Vulnerability

If you find a security issue:

1. do not open a public issue with exploit details
2. send a private report if possible
3. if private reporting is unavailable, open a minimal public issue and note that details were withheld for security reasons

Include:

- affected version or commit
- reproduction steps
- impact assessment
- suggested mitigation if known

## Scope Notes

`frugal` is a local CLI. Main security concerns are:

- unsafe file handling
- path traversal or repo boundary mistakes
- accidental destructive behavior
- misleading context output in automation workflows

The project intentionally avoids networked proxy behavior to keep risk surface smaller.
