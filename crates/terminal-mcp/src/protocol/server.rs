//! Terminal MCP Server Implementation
//!
//! This module implements the MCP server using rmcp 0.9's #[tool_router] pattern.
//! It routes MCP tool calls to the underlying terminal manipulation library.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ErrorData as McpError,
};

use tracing::{debug, error, info, instrument, warn};

use terminal_mcp_core::{Dimensions, SessionId};
use terminal_mcp_detector::{
    BorderDetector, ButtonDetector, CheckboxDetector, DetectionPipeline, InputDetector,
    MenuDetector, ProgressDetector, StatusBarDetector, TableDetector,
};
use terminal_mcp_session::Session;

use crate::tools::*;

/// Create a fully configured detection pipeline with all detectors
fn create_detection_pipeline() -> DetectionPipeline {
    let mut pipeline = DetectionPipeline::new();

    // Add all detectors in priority order (pipeline will auto-sort)
    pipeline.add_detector(Arc::new(BorderDetector::new())); // Priority 100
    pipeline.add_detector(Arc::new(MenuDetector::new())); // Priority 80
    pipeline.add_detector(Arc::new(TableDetector::new())); // Priority 80
    pipeline.add_detector(Arc::new(InputDetector::new())); // Priority 70
    pipeline.add_detector(Arc::new(ButtonDetector::new())); // Priority 60
    pipeline.add_detector(Arc::new(ProgressDetector::new())); // Priority 60
    pipeline.add_detector(Arc::new(StatusBarDetector::new())); // Priority 50
    pipeline.add_detector(Arc::new(CheckboxDetector::new())); // Priority 60

    pipeline
}

/// Terminal MCP Server
///
/// Manages terminal sessions and exposes them via MCP tools.
#[derive(Clone)]
pub struct TerminalMcpServer {
    /// Active terminal sessions (using Arc for shared access)
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Session>>>>,
    /// Tool router for handling MCP tool calls
    tool_router: ToolRouter<Self>,
    /// Whether to default to headless mode (no visual terminal windows)
    /// When false (default), visual mode is used unless explicitly disabled per-session
    headless_mode: bool,
}

#[tool_router]
impl TerminalMcpServer {
    /// Create a new Terminal MCP Server with visual mode as default
    pub fn new() -> Self {
        Self::with_headless_mode(false)
    }

    /// Create a new Terminal MCP Server with specified headless mode
    ///
    /// When headless_mode is true, sessions default to headless (no xterm window)
    /// When headless_mode is false (default), sessions spawn visible xterm windows
    pub fn with_headless_mode(headless_mode: bool) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            tool_router: Self::tool_router(),
            headless_mode,
        }
    }

    /// Get a session by ID (helper method)
    async fn get_session(&self, session_id: &str) -> Result<Arc<Session>, McpError> {
        let sessions = self.sessions.read().await;
        // Parse UUID string to SessionId
        use uuid::Uuid;
        let uuid = Uuid::parse_str(session_id).map_err(|_| {
            McpError::new(
                ErrorCode(-32602), // Invalid params
                format!("Invalid session ID format: {session_id}"),
                None,
            )
        })?;
        let session_id_key = SessionId::from(uuid);
        sessions.get(&session_id_key).cloned().ok_or_else(|| {
            McpError::new(
                ErrorCode(-32602), // Invalid params
                format!("Session '{session_id}' not found"),
                None,
            )
        })
    }

    /// Create a new terminal session
    #[tool(description = "Create a new terminal session with the specified command")]
    #[instrument(skip_all)]
    async fn terminal_session_create(
        &self,
        Parameters(params): Parameters<SessionCreateParams>,
    ) -> Result<CallToolResult, McpError> {
        // Determine visual mode: use explicit param if set, otherwise use server default
        // Server default is visual (true) unless --headless flag was passed
        let use_visual = params.visual.unwrap_or(!self.headless_mode);

        info!(
            "Creating terminal session: command='{}', visual={} (explicit: {:?}, server_headless: {}), emulator={:?}",
            params.command, use_visual, params.visual, self.headless_mode, params.terminal_emulator
        );

        use terminal_mcp_session::SessionMode;

        let dimensions = params.dimensions.unwrap_or_else(|| Dimensions::new(24, 80));

        // Determine session mode
        let mode = if use_visual {
            SessionMode::Visual
        } else {
            SessionMode::Headless
        };

        debug!("Session mode determined: {:?}", mode);

        // Create session with mode
        let session = Session::create_with_mode(
            params.command.clone(),
            params.args.clone(),
            dimensions,
            mode,
            params.terminal_emulator.clone(),
            params.cwd.clone(),
        )
        .map_err(|e| {
            error!("Failed to create session: {}", e);
            McpError::new(
                ErrorCode(-32603), // Internal error
                format!("Failed to create session: {e}"),
                None,
            )
        })?;

        let session_id = session.id().to_string();
        let session_id_clone = *session.id();

        // Extract visual terminal info if available
        let (terminal_emulator, window_id) = if let Some(handle) = session.visual_handle() {
            info!(
                "Visual terminal created: emulator={}, window_id={:?}",
                handle.terminal_name, handle.window_id
            );
            (Some(handle.terminal_name.clone()), handle.window_id.clone())
        } else {
            debug!("Headless session created");
            (None, None)
        };

        self.sessions
            .write()
            .await
            .insert(session_id_clone, Arc::new(session));

        info!(
            "Session created successfully: session_id={}, dimensions={}x{}",
            session_id, dimensions.rows, dimensions.cols
        );

        let response = SessionCreateResponse {
            session_id: session_id.clone(),
            dimensions,
            message: format!(
                "Session created for command '{}' in {} mode{}",
                params.command,
                if use_visual { "visual" } else { "headless" },
                terminal_emulator
                    .as_ref()
                    .map(|t| format!(" using {t}"))
                    .unwrap_or_default()
            ),
            mode: Some(if use_visual { "visual" } else { "headless" }.to_string()),
            terminal_emulator,
            window_id,
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or(session_id),
        )]))
    }

    /// List all active terminal sessions
    #[tool(description = "List all active terminal sessions")]
    #[instrument(skip_all)]
    async fn terminal_session_list(
        &self,
        Parameters(_params): Parameters<SessionListParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Listing all active terminal sessions");

        let sessions = self.sessions.read().await;

        let session_infos: Vec<SessionInfo> = sessions
            .iter()
            .map(|(id, session)| SessionInfo {
                session_id: id.to_string(),
                command: session.command().to_string(),
                dimensions: Dimensions::new(24, 80), // TODO: Get actual dimensions from session
                age_seconds: 0,                      // TODO: Track session creation time
            })
            .collect();

        let count = session_infos.len();

        info!("Found {} active session(s)", count);

        let response = SessionListResponse {
            sessions: session_infos,
            count,
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response)
                .unwrap_or_else(|_| format!("{count} sessions active")),
        )]))
    }

    /// Close a terminal session
    #[tool(description = "Close and terminate a terminal session")]
    #[instrument(skip_all)]
    async fn terminal_session_close(
        &self,
        Parameters(params): Parameters<SessionCloseParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("Closing terminal session: session_id={}", params.session_id);

        let mut sessions = self.sessions.write().await;

        // Parse UUID string to SessionId
        use uuid::Uuid;
        let uuid = Uuid::parse_str(&params.session_id).map_err(|_| {
            warn!("Invalid session ID format: {}", params.session_id);
            McpError::new(
                ErrorCode(-32602),
                format!("Invalid session ID: {}", params.session_id),
                None,
            )
        })?;
        let session_id = SessionId::from(uuid);

        if let Some(session) = sessions.remove(&session_id) {
            // Actually terminate the session (kills xterm and tmux)
            if let Err(e) = session.terminate() {
                warn!("Error terminating session {}: {}", params.session_id, e);
            }

            info!(
                "Session closed successfully: session_id={}",
                params.session_id
            );
            let response = SessionCloseResponse {
                session_id: params.session_id.clone(),
                message: format!("Session '{}' closed", params.session_id),
            };
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| format!("Session {} closed", params.session_id)),
            )]))
        } else {
            warn!("Session not found: session_id={}", params.session_id);
            Err(McpError::new(
                ErrorCode(-32602),
                format!("Session '{}' not found", params.session_id),
                None,
            ))
        }
    }

    /// Capture terminal state as Terminal State Tree (TST)
    #[tool(
        description = "Capture the current terminal state as a structured Terminal State Tree with detected UI elements"
    )]
    #[instrument(skip_all)]
    async fn terminal_snapshot(
        &self,
        Parameters(params): Parameters<SnapshotParams>,
    ) -> Result<CallToolResult, McpError> {
        info!(
            "Capturing terminal snapshot: session_id={}, idle_threshold_ms={:?}",
            params.session_id, params.idle_threshold_ms
        );

        let session = self.get_session(&params.session_id).await?;

        // Check if session is still alive before processing
        if !session.is_alive() {
            error!("Session {} is no longer alive", params.session_id);
            return Err(McpError::new(
                ErrorCode(-32603),
                format!(
                    "Session '{}' is no longer alive. It may have been terminated.",
                    params.session_id
                ),
                None,
            ));
        }

        // Process any pending output with a quick check
        debug!("Processing pending output");
        session.process_output().map_err(|e| {
            error!("Failed to process output: {}", e);
            McpError::new(
                ErrorCode(-32603),
                format!("Failed to process output: {e}"),
                None,
            )
        })?;

        // Get snapshot (requires DetectionPipeline and SnapshotConfig)
        use std::time::Duration;
        use terminal_mcp_session::SnapshotConfig;

        let pipeline = create_detection_pipeline();

        // Use idle_threshold_ms from params if provided, otherwise use defaults
        let mut config = SnapshotConfig::default();
        if let Some(idle_ms) = params.idle_threshold_ms {
            debug!(
                "Using custom idle threshold: {}ms (timeout: {}ms)",
                idle_ms,
                idle_ms * 2 + 1000
            );
            config.idle_threshold = Duration::from_millis(idle_ms);
            // Set idle_timeout to 2x threshold + 1 second for safety margin
            config.idle_timeout = Duration::from_millis(idle_ms * 2 + 1000);
        }

        debug!("Capturing snapshot with detection pipeline");

        // Wrap snapshot in a timeout to prevent indefinite hangs
        let snapshot_timeout = Duration::from_secs(10);
        let session_clone = session.clone();
        let pipeline_clone = pipeline;
        let config_clone = config;

        let snapshot_result = tokio::time::timeout(
            snapshot_timeout,
            tokio::task::spawn_blocking(move || {
                session_clone.snapshot(&pipeline_clone, &config_clone)
            }),
        )
        .await;

        let snapshot = match snapshot_result {
            Ok(Ok(Ok(snap))) => snap,
            Ok(Ok(Err(e))) => {
                error!("Failed to capture snapshot: {}", e);
                return Err(McpError::new(
                    ErrorCode(-32603),
                    format!("Failed to capture snapshot: {e}"),
                    None,
                ));
            }
            Ok(Err(e)) => {
                error!("Snapshot task panicked: {}", e);
                return Err(McpError::new(
                    ErrorCode(-32603),
                    format!("Snapshot task failed: {e}"),
                    None,
                ));
            }
            Err(_) => {
                error!("Snapshot timed out after {:?}", snapshot_timeout);
                return Err(McpError::new(
                    ErrorCode(-32603),
                    format!("Snapshot timed out after {} seconds. The tmux session may be invalid or unresponsive.", snapshot_timeout.as_secs()),
                    None,
                ));
            }
        };

        info!(
            "Snapshot captured successfully: {} elements detected",
            snapshot.elements.len()
        );

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&snapshot)
                .unwrap_or_else(|_| "Snapshot captured".to_string()),
        )]))
    }

    /// Type text into the terminal
    #[tool(description = "Type text into a terminal session")]
    #[instrument(skip_all)]
    async fn terminal_type(
        &self,
        Parameters(params): Parameters<TypeParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!(
            "Typing text into session: session_id={}, length={} chars",
            params.session_id,
            params.text.chars().count()
        );

        let session = self.get_session(&params.session_id).await?;

        session.write(params.text.as_bytes()).map_err(|e| {
            error!("Failed to write text: {}", e);
            McpError::new(
                ErrorCode(-32603),
                format!("Failed to write text: {e}"),
                None,
            )
        })?;

        info!(
            "Text typed successfully: {} chars",
            params.text.chars().count()
        );

        let response = TypeResponse {
            session_id: params.session_id.clone(),
            chars_typed: params.text.chars().count(),
            message: "Text typed successfully".to_string(),
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response)
                .unwrap_or_else(|_| "Text typed successfully".to_string()),
        )]))
    }

    /// Read raw terminal output
    #[tool(description = "Read raw output from a terminal session")]
    #[instrument(skip_all)]
    async fn terminal_read_output(
        &self,
        Parameters(params): Parameters<ReadOutputParams>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Reading terminal output: session_id={}", params.session_id);

        let session = self.get_session(&params.session_id).await?;

        // Read output (since_last_read=true, include_ansi=false)
        let output_read = session.read_output(true, false).map_err(|e| {
            error!("Failed to read output: {}", e);
            McpError::new(
                ErrorCode(-32603),
                format!("Failed to read output: {e}"),
                None,
            )
        })?;

        info!(
            "Output read successfully: {} bytes, has_more={}",
            output_read.bytes, output_read.has_more
        );

        // Return the actual output text as primary content
        Ok(CallToolResult::success(vec![
            Content::text(output_read.output.clone()),
            Content::text(format!(
                "\n(Read {} bytes, more_available: {})",
                output_read.bytes, output_read.has_more
            )),
        ]))
    }

    /// Press a key (send special keys, arrows, function keys, Ctrl combinations)
    #[tool(description = "Press a special key or key combination (arrows, F-keys, Ctrl+X, etc.)")]
    #[instrument(skip_all)]
    async fn terminal_press_key(
        &self,
        Parameters(params): Parameters<PressKeyParams>,
    ) -> Result<CallToolResult, McpError> {
        info!(
            "Pressing key: session_id={}, key='{}'",
            params.session_id, params.key
        );

        let session = self.get_session(&params.session_id).await?;

        // Parse and send the key
        session.press_key(&params.key).map_err(|e| {
            error!("Failed to press key '{}': {}", params.key, e);
            McpError::new(
                ErrorCode(-32603),
                format!("Failed to press key '{}': {}", params.key, e),
                None,
            )
        })?;

        // Get the escape sequence for response
        use terminal_mcp_core::Key;
        let key_enum = Key::parse(&params.key).map_err(|e| {
            error!("Invalid key format: {}", e);
            McpError::new(ErrorCode(-32602), format!("Invalid key format: {e}"), None)
        })?;
        let escape_sequence = key_enum.to_escape_sequence();
        let escape_str = escape_sequence
            .iter()
            .map(|b| format!("\\x{b:02x}"))
            .collect::<String>();

        debug!("Key escape sequence: {}", escape_str);
        info!("Key '{}' pressed successfully", params.key);

        let response = PressKeyResponse {
            session_id: params.session_id.clone(),
            key: params.key.clone(),
            escape_sequence: escape_str,
            message: format!("Key '{}' pressed successfully", params.key),
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response)
                .unwrap_or_else(|_| format!("Key '{}' pressed", params.key)),
        )]))
    }

    /// Resize terminal dimensions
    #[tool(description = "Resize a terminal session to new dimensions")]
    #[instrument(skip_all)]
    async fn terminal_session_resize(
        &self,
        Parameters(params): Parameters<SessionResizeParams>,
    ) -> Result<CallToolResult, McpError> {
        info!(
            "Resizing session: session_id={}, dimensions={}x{}",
            params.session_id, params.dimensions.rows, params.dimensions.cols
        );

        let session = self.get_session(&params.session_id).await?;

        session.resize(params.dimensions).map_err(|e| {
            error!("Failed to resize session: {}", e);
            McpError::new(
                ErrorCode(-32603),
                format!("Failed to resize session: {e}"),
                None,
            )
        })?;

        info!(
            "Session resized successfully to {}x{}",
            params.dimensions.rows, params.dimensions.cols
        );

        let response = SessionResizeResponse {
            session_id: params.session_id.clone(),
            dimensions: params.dimensions,
            message: format!(
                "Session resized to {}x{}",
                params.dimensions.rows, params.dimensions.cols
            ),
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| {
                format!(
                    "Session resized to {}x{}",
                    params.dimensions.rows, params.dimensions.cols
                )
            }),
        )]))
    }

    /// Click on an element by navigating to it
    #[tool(description = "Click on a UI element by its ref_id (navigates and activates)")]
    #[instrument(skip_all)]
    async fn terminal_click(
        &self,
        Parameters(params): Parameters<ClickParams>,
    ) -> Result<CallToolResult, McpError> {
        info!(
            "Clicking element: session_id={}, ref_id='{}'",
            params.session_id, params.ref_id
        );

        let session = self.get_session(&params.session_id).await?;

        // Setup detection pipeline and config
        use terminal_mcp_session::SnapshotConfig;

        let pipeline = create_detection_pipeline();
        let config = SnapshotConfig::default();

        debug!("Navigating to element '{}'", params.ref_id);

        // Click on the element
        let keys_sent = session
            .click(
                &params.ref_id,
                &pipeline,
                &config,
                params.inter_key_delay_ms,
            )
            .map_err(|e| {
                error!("Failed to click element '{}': {}", params.ref_id, e);
                McpError::new(
                    ErrorCode(-32603),
                    format!("Failed to click element '{}': {}", params.ref_id, e),
                    None,
                )
            })?;

        info!(
            "Element clicked successfully: {} keystrokes sent",
            keys_sent.len()
        );
        debug!("Keys sent: {:?}", keys_sent);

        let response = ClickResponse {
            session_id: params.session_id.clone(),
            ref_id: params.ref_id.clone(),
            keys_sent: keys_sent.clone(),
            message: format!(
                "Clicked element '{}' with {} keystrokes",
                params.ref_id,
                keys_sent.len()
            ),
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response)
                .unwrap_or_else(|_| format!("Clicked element '{}'", params.ref_id)),
        )]))
    }

    /// Wait for a condition to be met
    #[tool(description = "Wait for text to appear, element to show, or terminal to be idle")]
    #[instrument(skip_all)]
    async fn terminal_wait_for(
        &self,
        Parameters(params): Parameters<WaitForParams>,
    ) -> Result<CallToolResult, McpError> {
        info!(
            "Waiting for condition: session_id={}, text={:?}, element_type={:?}, gone={}, idle={}, timeout={}ms",
            params.session_id, params.text, params.element_type, params.gone, params.idle, params.timeout_ms
        );

        let session = self.get_session(&params.session_id).await?;

        // Setup detection pipeline and config
        use std::time::Duration;
        use terminal_mcp_session::{SnapshotConfig, WaitCondition};

        let pipeline = create_detection_pipeline();
        let config = SnapshotConfig::default();

        // Build wait condition
        let mut condition = WaitCondition::new();
        if let Some(text) = &params.text {
            condition.text = Some(text.clone());
        }
        if let Some(elem_type) = &params.element_type {
            condition.element_type = Some(elem_type.clone());
        }
        condition.gone = params.gone;
        condition.idle = params.idle;
        condition.timeout = Duration::from_millis(params.timeout_ms);
        condition.poll_interval = Duration::from_millis(params.poll_interval_ms);

        debug!(
            "Starting wait with poll_interval={}ms",
            params.poll_interval_ms
        );

        // Wait for condition
        let wait_result = session
            .wait_for(&condition, &pipeline, &config)
            .map_err(|e| {
                error!("Failed to wait for condition: {}", e);
                McpError::new(
                    ErrorCode(-32603),
                    format!("Failed to wait for condition: {e}"),
                    None,
                )
            })?;

        if wait_result.condition_met {
            info!("Condition met after {}ms", wait_result.waited_ms);
        } else {
            warn!(
                "Condition not met, timeout after {}ms",
                wait_result.waited_ms
            );
        }

        let response = WaitForResponse {
            session_id: params.session_id.clone(),
            condition_met: wait_result.condition_met,
            waited_ms: wait_result.waited_ms,
            snapshot: Some(wait_result.snapshot),
            message: if wait_result.condition_met {
                format!("Condition met after {}ms", wait_result.waited_ms)
            } else {
                format!("Timeout after {}ms", wait_result.waited_ms)
            },
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| {
                format!(
                    "Condition {}, waited {}ms",
                    if wait_result.condition_met {
                        "met"
                    } else {
                        "not met"
                    },
                    wait_result.waited_ms
                )
            }),
        )]))
    }
}

impl Default for TerminalMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the ServerHandler trait to define server capabilities
#[tool_handler]
impl rmcp::ServerHandler for TerminalMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Terminal MCP Server - Interact with terminal-based applications (TUI/CLI) \
                 through structured Terminal State Tree representation. \
                 Use terminal_session_create to start a session, terminal_snapshot to capture UI state, \
                 terminal_type to send input, and terminal_read_output to read text output."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
