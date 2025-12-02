# Terminal MCP Server

MCP server enabling AI agents to interact with terminal applications through structured Terminal State Tree representation. Works with any AI assistant that supports the [Model Context Protocol](https://modelcontextprotocol.io/).

## Installation

### Via npm (Recommended)

```bash
npx mcp-server-terminal
```

### Via GitHub Releases

Download pre-built binaries from [Releases](https://github.com/aybelatchane/mcp-server-terminal/releases).

### Build from Source

```bash
git clone https://github.com/aybelatchane/mcp-server-terminal.git
cd mcp-server-terminal
cargo build --release
# Binary: ./target/release/terminal-mcp
```

## Configuration

### Claude Code

```bash
claude mcp add terminal -- npx mcp-server-terminal
```

### Claude Desktop

Add to `~/.claude.json` (macOS/Linux) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "terminal": {
      "command": "npx",
      "args": ["mcp-server-terminal"]
    }
  }
}
```

### Cursor

Add to MCP settings (Settings → MCP Servers):

```json
{
  "terminal": {
    "command": "npx",
    "args": ["mcp-server-terminal"]
  }
}
```

### Other MCP Clients

For any MCP-compatible client, configure the server with:
- **Command**: `npx`
- **Args**: `["mcp-server-terminal"]`

Or if using the binary directly:
- **Command**: `terminal-mcp`

## Usage

Ask your AI agent:
- *"Create a terminal session running htop"*
- *"Take a snapshot of the terminal"*
- *"Press the down arrow key"*
- *"Type 'ls -la' and press Enter"*

## MCP Tools

| Tool | Description |
|------|-------------|
| `terminal_session_create` | Start a terminal session |
| `terminal_session_list` | List active sessions |
| `terminal_session_close` | Close a session |
| `terminal_session_resize` | Resize terminal dimensions |
| `terminal_snapshot` | Capture terminal state with UI elements |
| `terminal_type` | Type text into terminal |
| `terminal_press_key` | Press keys (arrows, F-keys, Ctrl+X) |
| `terminal_click` | Click on detected UI element |
| `terminal_wait_for` | Wait for text, element, or idle state |
| `terminal_read_output` | Read raw terminal output |

## Visual Mode

By default, sessions spawn a visible terminal window (xterm). For headless operation:

```bash
npx mcp-server-terminal --headless
```

Or in your MCP config:

```json
{
  "mcpServers": {
    "terminal": {
      "command": "npx",
      "args": ["mcp-server-terminal", "--headless"]
    }
  }
}
```

### X11 Setup (Linux/WSL)

Visual mode requires X11. Add the DISPLAY environment variable to your MCP config:

```json
{
  "mcpServers": {
    "terminal": {
      "command": "npx",
      "args": ["mcp-server-terminal"],
      "env": {
        "DISPLAY": ":0"
      }
    }
  }
}
```

## Logging

Set the `RUST_LOG` environment variable:

```json
{
  "mcpServers": {
    "terminal": {
      "command": "npx",
      "args": ["mcp-server-terminal"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

Log levels: `error`, `warn`, `info`, `debug`, `trace`

Logs go to stderr (stdout is reserved for MCP protocol).

## Platform Support

| Platform | Architecture | Status |
|----------|--------------|--------|
| Linux | x64, arm64 | Supported |
| macOS | x64, arm64 | Supported |
| Windows | WSL | Supported |

## License

MIT
