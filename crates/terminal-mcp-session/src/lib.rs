//! # terminal-mcp-session
//!
//! Session lifecycle management for the Terminal MCP Server.
//!
//! This crate provides:
//! - Session creation and initialization
//! - Session state tracking
//! - Session cleanup and termination
//! - Session registry management
//!
//! ## Architecture
//!
//! This is Layer 2 in the architecture - it depends on terminal-mcp-core
//! and terminal-mcp-emulator to manage terminal session lifecycles.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod manager;
pub mod navigation;
pub mod output;
pub mod session;
pub mod snapshot;
pub mod visual;
pub mod wait;

// Re-export commonly used types
pub use manager::{SessionInfo, SessionManager, SessionManagerConfig};
pub use navigation::NavigationCalculator;
pub use output::{OutputBuffer, OutputRead};
pub use session::{Session, SessionStatus};
pub use snapshot::SnapshotConfig;
pub use visual::{SessionMode, VisualTerminal, VisualTerminalHandle};
pub use wait::{WaitCondition, WaitResult};
