# terminal-mcp-session

Session lifecycle management for Terminal MCP Server.

## Overview

This crate manages terminal sessions including:

- Session creation and destruction
- PTY process lifecycle
- Output buffering and retrieval
- Snapshot capture with element detection
- Keyboard input handling
- Wait conditions for synchronization

## Key Components

- **SessionManager**: Central registry for all active sessions
- **Session**: Individual terminal session with PTY handle
- **SessionMode**: Headless vs visual terminal mode
- **SnapshotOptions**: Configuration for terminal snapshots

## Usage

```rust
use terminal_mcp_session::{SessionManager, SessionConfig};

// Create a session manager
let manager = SessionManager::new();

// Create a new session
let config = SessionConfig {
    command: "bash".to_string(),
    args: vec![],
    dimensions: Dimensions::new(24, 80),
    ..Default::default()
};
let session_id = manager.create(config)?;

// Capture a snapshot
let snapshot = manager.snapshot(&session_id, Default::default())?;
println!("Elements: {}", snapshot.elements.len());

// Send keyboard input
manager.press_key(&session_id, "Enter")?;

// Close the session
manager.close(&session_id)?;
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
