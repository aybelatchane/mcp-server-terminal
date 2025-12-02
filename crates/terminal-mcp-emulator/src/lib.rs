//! # terminal-mcp-emulator
//!
//! Terminal emulator implementation for the Terminal MCP Server.
//!
//! This crate provides:
//! - VTE parser for ANSI/VT escape sequences
//! - Terminal grid state management
//! - PTY (pseudo-terminal) lifecycle management
//! - Cell and color types for terminal rendering
//!
//! ## Architecture
//!
//! This is Layer 3 in the architecture - it depends on terminal-mcp-core
//! and provides terminal emulation functionality.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod grid;
pub mod parser;
pub mod pty;
pub mod recording;

// Re-export commonly used types
pub use grid::{Cursor, CursorStyle, Grid};
pub use parser::Parser;
pub use pty::PtyHandle;
pub use recording::{AsciinemaHeader, RecordEvent, SessionRecorder};
