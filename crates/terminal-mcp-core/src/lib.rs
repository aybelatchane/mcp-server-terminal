//! # terminal-mcp-core
//!
//! Core types for the Terminal MCP Server.
//!
//! This crate contains all fundamental types with **no internal dependencies**
//! on other terminal-mcp crates. It provides:
//!
//! - Geometry types (Position, Bounds, Dimensions)
//! - Session types (SessionId, SessionStatus, SessionConfig)
//! - Cell and color types for terminal grid
//! - Element types for Terminal State Tree (TST)
//! - Key types for input handling
//! - Error types
//!
//! ## Architecture
//!
//! This is Layer 0 in the architecture - all other crates depend on this one,
//! but this crate has no dependencies on other terminal-mcp crates.

#![warn(missing_docs)]
#![warn(clippy::all)]

// Re-export all modules
pub mod cell;
pub mod config;
pub mod element;
pub mod error;
pub mod geometry;
pub mod key;
pub mod platform;
pub mod session;

// Re-export commonly used types
pub use cell::{Cell, CellAttributes, Color};
pub use config::{
    CaptureConfig, CustomPatternConfig, DetectionSettings, SecuritySettings, ServerConfig,
    ServerSettings, TerminalSettings,
};
pub use element::{Element, MenuItem, TerminalStateTree};
pub use error::{Error, Result};
pub use geometry::{Bounds, Dimensions, Position};
pub use key::Key;
pub use platform::Platform;
pub use session::{SessionConfig, SessionId, SessionInfo, SessionStatus};
