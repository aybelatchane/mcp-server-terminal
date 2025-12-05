# Terminal MCP Server

[![npm version](https://img.shields.io/npm/v/mcp-server-terminal.svg)](https://www.npmjs.com/package/mcp-server-terminal)
[![npm downloads](https://img.shields.io/npm/dm/mcp-server-terminal.svg)](https://www.npmjs.com/package/mcp-server-terminal)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

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

### Claude

#### Claude Code (CLI)

```bash
claude mcp add terminal -- npx mcp-server-terminal
```

#### Claude Desktop

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

---

### OpenAI Codex

#### Codex CLI

```bash
codex mcp add terminal -- npx mcp-server-terminal
```

#### Codex Configuration File

Add to `~/.codex/config.toml`:

```toml
[mcp_servers.terminal]
command = "npx"
args = ["mcp-server-terminal"]
```

---

### Google Gemini

#### Gemini CLI

```bash
gemini mcp add terminal npx mcp-server-terminal
```

---

### VS Code / GitHub Copilot

VS Code 1.101+ supports MCP. Add to your VS Code settings (`settings.json`):

```json
{
  "mcp.servers": {
    "terminal": {
      "command": "npx",
      "args": ["mcp-server-terminal"]
    }
  }
}
```

---

### Cursor

Add to `~/.cursor/mcp.json` or `.cursor/mcp.json` in your project:

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

---

### Windsurf

Add to `~/.codeium/windsurf/mcp_config.json`:

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

---

### Zed

Add to your Zed settings (Preferences → Settings):

```json
{
  "context_servers": {
    "terminal": {
      "command": {
        "path": "npx",
        "args": ["mcp-server-terminal"]
      }
    }
  }
}
```

---

### Cline (VS Code Extension)

Click MCP Servers icon → Configure → Advanced MCP Settings, then add:

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

---

### AWS Bedrock

Add to your Bedrock agent MCP configuration:

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

---

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
