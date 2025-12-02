# Security Policy

## Supported Versions

Currently supported versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please report security vulnerabilities through one of these methods:

1. **GitHub Security Advisories** (Preferred)
   - Go to: https://github.com/aybelatchane/mcp-server-terminal/security/advisories/new
   - Provide details about the vulnerability

2. **Private Issue**
   - Email the maintainers directly via GitHub
   - Include "SECURITY" in the subject line

### What to Include

When reporting a vulnerability, please include:

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if any)
- Your contact information

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: Within 7 days
  - High: Within 30 days
  - Medium/Low: Within 90 days

### Disclosure Policy

- We will acknowledge your report within 48 hours
- We will provide regular updates on our progress
- We will credit you in the security advisory (unless you prefer to remain anonymous)
- We will coordinate disclosure timing with you

### Security Best Practices

When using Terminal MCP Server:

- **Command Whitelist**: Always configure `allowed_commands` in your config file
- **Session Limits**: Set appropriate `max_sessions` to prevent resource exhaustion
- **Network Isolation**: Run in isolated environments for untrusted workloads
- **Regular Updates**: Keep Terminal MCP updated to the latest version
- **Audit Logs**: Monitor terminal sessions for suspicious activity

## Security Features

Terminal MCP is designed with security in mind:

- ✅ **Zero unsafe code** - Memory-safe Rust implementation
- ✅ **Command whitelist** - Restrict allowed commands via configuration
- ✅ **Input validation** - All inputs validated (UUIDs, regex patterns, keys)
- ✅ **Session isolation** - Each session runs in separate PTY
- ✅ **No command injection** - Arguments passed securely via PTY
- ✅ **Resource limits** - Configurable session and timeout limits

## Known Security Considerations

- Terminal MCP provides direct terminal access - use appropriate access controls
- Visual mode may expose terminal content - use in trusted environments only
- Session recording stores terminal output - handle recordings securely

## Contact

For security-related questions: Use GitHub Security Advisories

For general questions: See [SUPPORT.md](./SUPPORT.md)
