# Logging

Terminal MCP Server uses `RUST_LOG` for logging. All logs go to stderr (stdout is reserved for MCP protocol).

## Quick Start

```bash
RUST_LOG=info claude          # Standard logging
RUST_LOG=debug claude         # Detailed diagnostics
RUST_LOG=terminal_mcp=trace   # Maximum verbosity
```

## Log Levels

| Level | Usage |
|-------|-------|
| `error` | Critical failures only |
| `warn` | Warnings and errors |
| `info` | Standard operations |
| `debug` | Detailed diagnostics |
| `trace` | Maximum verbosity |

## Module-Specific Logging

```bash
# MCP tool handlers
RUST_LOG=terminal_mcp::protocol=debug

# Session lifecycle
RUST_LOG=terminal_mcp_session=debug

# PTY operations
RUST_LOG=terminal_mcp_emulator=debug

# Multiple modules
RUST_LOG=terminal_mcp::protocol=info,terminal_mcp_session=debug
```

## Troubleshooting

**Session hangs**

```bash
RUST_LOG=terminal_mcp_session=debug,terminal_mcp_emulator=debug claude
```

Look for: `Processing PTY output: 0 bytes`

**Visual mode fails**

```bash
RUST_LOG=terminal_mcp_session=info claude
```

Look for: `Failed to spawn visual terminal`

## Log to File

```bash
RUST_LOG=info claude 2>&1 | tee terminal-mcp.log
```
