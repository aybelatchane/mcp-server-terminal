# terminal-mcp-core

Core types and primitives for Terminal MCP Server.

## Overview

This crate provides the foundational types used across all Terminal MCP components:

- **Geometry**: `Dimensions`, `Position`, `Region` for terminal layout
- **Sessions**: `SessionId`, `SessionInfo`, `SessionMode` for session management
- **Elements**: `Element`, `ElementType`, `RefId` for detected UI components
- **Keys**: `Key`, `Modifier` for keyboard input handling
- **Errors**: `Error`, `Result` for unified error handling

## Usage

```rust
use terminal_mcp_core::{
    Dimensions, Position, Region,
    SessionId, SessionInfo, SessionMode,
    Element, ElementType, RefId,
    Key, Modifier,
    Error, Result,
};

// Create a session ID
let session_id = SessionId::new();

// Define terminal dimensions
let dims = Dimensions::new(24, 80);

// Parse a key
let key = Key::parse("Ctrl+c")?;
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
