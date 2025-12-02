//! MCP Protocol Layer
//!
//! This module implements the Model Context Protocol server using rmcp 0.9.
//! It exposes the terminal manipulation capabilities as MCP tools.

pub mod server;

pub use server::TerminalMcpServer;
