# Contributing to Terminal MCP Server

Thank you for your interest in contributing!

## Getting Started

### Prerequisites

- Rust 1.75+: Install from [rustup.rs](https://rustup.rs/)
- Git

### Setup

```bash
git clone https://github.com/aybelatchane/terminal-mcp.git
cd terminal-mcp
cargo build
cargo test
```

## Development Workflow

### Branch Strategy

- `main` - Stable production releases
- `dev` - Active development (PRs merge here)
- `feature/*` - New features
- `fix/*` - Bug fixes

**All PRs should target the `dev` branch.**

### Making Changes

```bash
git checkout dev
git pull origin dev
git checkout -b feature/your-feature-name

# Make changes, then:
cargo test
cargo clippy -- -D warnings
cargo fmt

git push origin feature/your-feature-name
```

## Architecture

```
terminal-mcp/
├── crates/
│   ├── terminal-mcp/           # MCP Protocol layer
│   ├── terminal-mcp-session/   # Session Manager
│   ├── terminal-mcp-emulator/  # Terminal Emulator
│   ├── terminal-mcp-detector/  # Element Detection
│   └── terminal-mcp-core/      # Core Types
```

See crate READMEs for details.

## Coding Standards

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Document public APIs with `///` comments
- Use `Result<T, Error>` for fallible operations

## Testing

```bash
cargo test                           # All tests
cargo test -p terminal-mcp-detector  # Specific crate
cargo test -- --nocapture            # With output
```

## Pull Request Process

1. Ensure tests pass: `cargo test --workspace`
2. Run clippy: `cargo clippy --workspace -- -D warnings`
3. Format code: `cargo fmt --all`
4. Update CHANGELOG.md if needed

### PR Title Format

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `refactor:` - Code refactoring
- `chore:` - Maintenance

## Reporting Issues

- Check existing issues first
- Include reproduction steps
- Include environment info (OS, Rust version)

## License

Contributions are licensed under MIT.
