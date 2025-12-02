//! MCP Tool Types and Handlers
//!
//! This module defines all MCP tool parameter and response types,
//! ready for integration with the rmcp SDK when available.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terminal_mcp_core::{Dimensions, TerminalStateTree};

// =============================================================================
// Session Management Tools
// =============================================================================

/// Parameters for terminal_session_create
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionCreateParams {
    /// Command to execute (e.g., "bash", "vim", "htop")
    pub command: String,

    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Terminal dimensions
    #[serde(default)]
    pub dimensions: Option<Dimensions>,

    /// Working directory
    #[serde(default)]
    pub cwd: Option<String>,

    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Enable visual mode (spawn visible terminal window)
    /// If not specified, defaults to server mode (visual unless --headless flag is set)
    #[serde(default)]
    pub visual: Option<bool>,

    /// Preferred terminal emulator (e.g., "gnome-terminal", "iTerm2", "auto")
    /// If not specified or "auto", will use the best available terminal for the platform
    #[serde(default)]
    pub terminal_emulator: Option<String>,
}

/// Response for terminal_session_create
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionCreateResponse {
    /// Unique session identifier
    pub session_id: String,

    /// Terminal dimensions
    pub dimensions: Dimensions,

    /// Success message
    pub message: String,

    /// Session mode (headless or visual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// Terminal emulator used (only for visual mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_emulator: Option<String>,

    /// Window ID (platform-specific, only for visual mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<String>,
}

/// Parameters for terminal_session_list
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionListParams {}

/// Response for terminal_session_list
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionListResponse {
    /// List of active sessions
    pub sessions: Vec<SessionInfo>,

    /// Total count
    pub count: usize,
}

/// Information about a session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionInfo {
    /// Session identifier
    pub session_id: String,

    /// Command being run
    pub command: String,

    /// Terminal dimensions
    pub dimensions: Dimensions,

    /// Session age in seconds
    pub age_seconds: u64,
}

/// Parameters for terminal_session_close
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionCloseParams {
    /// Session to close
    pub session_id: String,

    /// Force close (send SIGKILL instead of SIGTERM)
    #[serde(default)]
    pub force: bool,
}

/// Response for terminal_session_close
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionCloseResponse {
    /// Session that was closed
    pub session_id: String,

    /// Success message
    pub message: String,
}

/// Parameters for terminal_session_resize
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionResizeParams {
    /// Session to resize
    pub session_id: String,

    /// New dimensions
    pub dimensions: Dimensions,
}

/// Response for terminal_session_resize
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionResizeResponse {
    /// Session that was resized
    pub session_id: String,

    /// New dimensions
    pub dimensions: Dimensions,

    /// Success message
    pub message: String,
}

// =============================================================================
// State Capture Tools
// =============================================================================

/// Parameters for terminal_snapshot
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SnapshotParams {
    /// Session to snapshot
    pub session_id: String,

    /// Include raw text output
    #[serde(default = "default_true")]
    pub include_raw_text: bool,

    /// Idle threshold in milliseconds (wait for terminal to be idle)
    #[serde(default)]
    pub idle_threshold_ms: Option<u64>,
}

fn default_true() -> bool {
    true
}

/// Response for terminal_snapshot (returns Terminal State Tree)
pub type SnapshotResponse = TerminalStateTree;

/// Parameters for terminal_read_output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadOutputParams {
    /// Session to read from
    pub session_id: String,

    /// Maximum bytes to read (default: read all available)
    #[serde(default)]
    pub max_bytes: Option<usize>,
}

/// Response for terminal_read_output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadOutputResponse {
    /// Raw text output
    pub output: String,

    /// Number of bytes read
    pub bytes_read: usize,

    /// Whether more output is available
    pub more_available: bool,
}

// =============================================================================
// Input Tools
// =============================================================================

/// Parameters for terminal_press_key
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PressKeyParams {
    /// Session to send key to
    pub session_id: String,

    /// Key to press (e.g., "Enter", "Up", "Ctrl+c", "F1")
    pub key: String,
}

/// Response for terminal_press_key
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PressKeyResponse {
    /// Session that received the key
    pub session_id: String,

    /// Key that was sent
    pub key: String,

    /// Escape sequence that was sent
    pub escape_sequence: String,

    /// Success message
    pub message: String,
}

/// Parameters for terminal_type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TypeParams {
    /// Session to type into
    pub session_id: String,

    /// Text to type
    pub text: String,

    /// Delay between characters in milliseconds
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

/// Response for terminal_type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TypeResponse {
    /// Session that received the text
    pub session_id: String,

    /// Number of characters typed
    pub chars_typed: usize,

    /// Success message
    pub message: String,
}

// =============================================================================
// Navigation Tools
// =============================================================================

/// Parameters for terminal_click
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClickParams {
    /// Session to interact with
    pub session_id: String,

    /// Element reference ID to click
    pub ref_id: String,

    /// Delay between navigation keys in milliseconds
    #[serde(default)]
    pub inter_key_delay_ms: Option<u64>,
}

/// Response for terminal_click
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClickResponse {
    /// Session that was interacted with
    pub session_id: String,

    /// Element that was clicked
    pub ref_id: String,

    /// Keys that were sent for navigation
    pub keys_sent: Vec<String>,

    /// Success message
    pub message: String,
}

/// Parameters for terminal_wait_for
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaitForParams {
    /// Session to wait on
    pub session_id: String,

    /// Text to wait for (regex pattern)
    #[serde(default)]
    pub text: Option<String>,

    /// Element type to wait for
    #[serde(default)]
    pub element_type: Option<String>,

    /// Wait for text/element to disappear (gone)
    #[serde(default)]
    pub gone: bool,

    /// Wait for terminal to be idle
    #[serde(default)]
    pub idle: bool,

    /// Timeout in milliseconds (default: 5000)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Polling interval in milliseconds (default: 100)
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
}

fn default_timeout() -> u64 {
    5000
}

fn default_poll_interval() -> u64 {
    100
}

/// Response for terminal_wait_for
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaitForResponse {
    /// Session that was waited on
    pub session_id: String,

    /// Whether the condition was met
    pub condition_met: bool,

    /// Time waited in milliseconds
    pub waited_ms: u64,

    /// Current snapshot (if condition met)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<TerminalStateTree>,

    /// Message describing the result
    pub message: String,
}

// =============================================================================
// Tool Handler Trait (Ready for rmcp integration)
// =============================================================================

/// Tool handler trait for MCP tools
///
/// When rmcp becomes available, implement this for each tool
pub trait ToolHandler: Send + Sync {
    /// Tool name
    fn name(&self) -> &'static str;

    /// Tool description
    fn description(&self) -> &'static str;

    /// JSON schema for parameters
    fn parameter_schema(&self) -> schemars::Schema;

    /// Handle tool execution (will be async when rmcp is available)
    ///
    /// TODO: When rmcp is available, change signature to:
    /// async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, Error>
    fn execute_stub(&self, params: serde_json::Value) -> Result<serde_json::Value, String>;
}

// =============================================================================
// Stub Implementations (Ready for rmcp)
// =============================================================================

/// Example stub handler for terminal_session_create
///
/// TODO: When rmcp is available:
/// 1. Implement actual ToolHandler trait
/// 2. Add SessionManager dependency
/// 3. Implement async execute() method
/// 4. Wire up to MCP server
pub struct SessionCreateHandler {
    // TODO: Add SessionManager when implementing
    // manager: Arc<Mutex<SessionManager>>,
}

impl SessionCreateHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SessionCreateHandler {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Implement ToolHandler when rmcp is available
// impl ToolHandler for SessionCreateHandler {
//     fn name(&self) -> &'static str {
//         "terminal_session_create"
//     }
//
//     fn description(&self) -> &'static str {
//         "Create a new terminal session"
//     }
//
//     async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, Error> {
//         let params: SessionCreateParams = serde_json::from_value(params)?;
//         // Implementation here
//     }
// }

// =============================================================================
// Server State (Ready for rmcp integration)
// =============================================================================

/// MCP Server state
///
/// TODO: When rmcp is available, use this in server initialization
pub struct McpServerState {
    // TODO: Uncomment when implementing
    // Session manager
    // pub session_manager: Arc<Mutex<SessionManager>>,

    // Server configuration
    // pub config: ServerConfig,

    // Detection pipeline
    // pub detection_pipeline: Arc<DetectionPipeline>,
}

impl McpServerState {
    /// Create new server state
    ///
    /// TODO: When rmcp is available, use this in main()
    pub fn new() -> Self {
        Self {
            // TODO: Initialize with actual values
        }
    }
}

impl Default for McpServerState {
    fn default() -> Self {
        Self::new()
    }
}
