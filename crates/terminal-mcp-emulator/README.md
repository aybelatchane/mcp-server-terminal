# terminal-mcp-emulator

Terminal emulator implementation with VTE parser and PTY management.

## Overview

This crate provides a high-fidelity terminal emulator that:

- Parses ANSI escape sequences using the VTE parser (from Alacritty)
- Manages PTY sessions with cross-platform support via portable-pty
- Maintains a character grid with cell attributes (colors, styles)
- Supports both headless and visual terminal modes

## Key Components

- **Grid**: Terminal character grid with row/column access
- **Cell**: Individual character with styling attributes
- **VteHandler**: ANSI escape sequence processor
- **PtyManager**: Cross-platform PTY lifecycle management

## Usage

```rust
use terminal_mcp_emulator::{Grid, PtyManager, VteHandler};

// Create a grid
let mut grid = Grid::new(24, 80);

// Process terminal output
let mut handler = VteHandler::new(&mut grid);
handler.process(output_bytes);

// Access cell content
let cell = grid.cell(0, 0);
println!("Character: {}", cell.character);
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
