//! # Terminal MCP Server
//!
//! Model Context Protocol server for AI agents to interact with terminal-based
//! applications (TUI/CLI tools).
//!
//! ## Overview
//!
//! This server provides MCP tools for:
//! - Session management (create, list, terminate)
//! - State capture (get Terminal State Tree)
//! - Input handling (send keys, send text)
//! - Waiting mechanisms (wait for output, wait for element)
//!
//! ## Architecture
//!
//! This is Layer 1 - the main MCP server binary that ties together:
//! - terminal-mcp-core: Core types
//! - terminal-mcp-emulator: Terminal emulation
//! - terminal-mcp-session: Session lifecycle
//! - terminal-mcp-detector: Element detection

use rmcp::{transport::stdio, ServiceExt};
use terminal_mcp::TerminalMcpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let headless_mode = args.iter().any(|arg| arg == "--headless");

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let mode_str = if headless_mode { "headless" } else { "visual" };
    tracing::info!(
        "Terminal MCP Server v0.1.0 starting in {} mode...",
        mode_str
    );

    // Create MCP server instance with headless mode flag
    let server = TerminalMcpServer::with_headless_mode(headless_mode);

    tracing::info!("Server initialized, starting stdio transport...");

    // Serve the MCP server over stdio
    let service = server.serve(stdio()).await.map_err(|e| {
        tracing::error!("Error starting server: {}", e);
        e
    })?;

    tracing::info!(
        "Terminal MCP Server running on stdio (default mode: {})",
        mode_str
    );

    // Wait for the service to complete
    service.waiting().await?;

    tracing::info!("Terminal MCP Server shutting down");

    Ok(())
}
