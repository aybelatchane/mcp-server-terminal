# Visual Terminal Mode

Visual mode spawns a visible terminal window when creating sessions. Useful for debugging and interactive applications.

## Enable Visual Mode

Visual mode is enabled by default. Sessions spawn an xterm window.

```json
{
  "command": "bash",
  "visual": true
}
```

## Headless Mode

Run without visible windows (for automation/CI):

```bash
terminal-mcp --headless
```

Or per-session:

```json
{
  "command": "bash",
  "visual": false
}
```

## X11 Setup (WSL/Linux)

Visual mode requires X11.

### WSL2 with WSLg (Windows 11)

WSLg provides built-in X11. Set DISPLAY in MCP config:

```json
{
  "mcpServers": {
    "terminal-mcp": {
      "type": "stdio",
      "command": "terminal-mcp",
      "env": {
        "DISPLAY": ":0"
      }
    }
  }
}
```

### WSL2 without WSLg (Windows 10)

Install an X server (VcXsrv, Xming) and set DISPLAY to Windows host IP:

```bash
export DISPLAY=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}'):0
```

### Native Linux

```bash
export DISPLAY=:0
```

## Supported Terminals

| Platform | Terminals |
|----------|-----------|
| Linux | gnome-terminal, konsole, xterm, alacritty, kitty |
| macOS | iTerm2, Terminal.app |
| Windows | Windows Terminal, PowerShell, cmd.exe |
| WSL | wt.exe, tmux |

Auto-detection selects the best available terminal.

## Troubleshooting

**Window doesn't appear**
- Check DISPLAY is set: `echo $DISPLAY`
- Test xterm: `DISPLAY=:0 xterm`
- Install xterm: `sudo apt install xterm`

**Wrong terminal opens**

Specify explicitly:

```json
{
  "command": "vim",
  "visual": true,
  "terminal_emulator": "iTerm2"
}
```
