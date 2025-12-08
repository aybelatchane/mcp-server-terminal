//! Terminal MCP Server Library
//!
//! This library contains the MCP protocol layer types and handlers.
//! The actual server binary is in main.rs.

pub mod protocol;
pub mod schema;
pub mod tools;

// Re-export commonly used types
pub use protocol::TerminalMcpServer;
pub use schema::SchemaTransformer;
pub use tools::*;
