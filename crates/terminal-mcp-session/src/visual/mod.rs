//! Visual terminal support for spawning visible terminal windows.
//!
//! This module provides cross-platform support for spawning visible terminal
//! emulator windows, similar to how Playwright opens visible browser windows.

use terminal_mcp_core::{Dimensions, Result};

pub mod registry;

// Platform-specific implementations
#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

/// Handle to a visual terminal window.
#[derive(Debug, Clone)]
pub struct VisualTerminalHandle {
    /// Process ID of the terminal window
    pub pid: u32,

    /// Platform-specific window identifier (optional)
    pub window_id: Option<String>,

    /// Name of the terminal emulator used
    pub terminal_name: String,
}

impl VisualTerminalHandle {
    /// Create a new visual terminal handle.
    pub fn new(pid: u32, terminal_name: impl Into<String>) -> Self {
        Self {
            pid,
            window_id: None,
            terminal_name: terminal_name.into(),
        }
    }

    /// Create a new visual terminal handle with window ID.
    pub fn with_window_id(
        pid: u32,
        terminal_name: impl Into<String>,
        window_id: impl Into<String>,
    ) -> Self {
        Self {
            pid,
            window_id: Some(window_id.into()),
            terminal_name: terminal_name.into(),
        }
    }
}

/// Platform-agnostic visual terminal interface.
///
/// This trait abstracts over different terminal emulators across platforms,
/// providing a unified interface for spawning visible terminal windows.
pub trait VisualTerminal: Send + Sync {
    /// Spawn a visible terminal window.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to run in the terminal
    /// * `args` - Command arguments
    /// * `dimensions` - Terminal dimensions (rows, columns)
    ///
    /// # Returns
    ///
    /// A handle to the spawned terminal window, or an error if spawning failed.
    fn spawn(
        &self,
        command: &str,
        args: &[String],
        dimensions: Dimensions,
    ) -> Result<VisualTerminalHandle>;

    /// Check if this terminal type is available on the current platform.
    ///
    /// This typically checks if the terminal emulator executable exists
    /// in the system PATH or at a known location.
    fn is_available(&self) -> bool;

    /// Get the terminal emulator name.
    ///
    /// Returns a human-readable name for logging and user feedback.
    fn name(&self) -> &'static str;

    /// Get the priority of this terminal emulator.
    ///
    /// Higher values = higher priority. Used for selecting the preferred
    /// terminal when multiple are available.
    ///
    /// Typical priorities:
    /// - 100: Primary terminal (gnome-terminal, Terminal.app, Windows Terminal)
    /// - 80: Alternative quality terminal (konsole, iTerm2)
    /// - 70: Modern terminals (alacritty, kitty)
    /// - 60: PowerShell
    /// - 50: Universal fallback (xterm, cmd.exe)
    /// - 40: Multiplexer fallback (tmux, screen)
    fn priority(&self) -> u8;
}

/// Session mode for terminal sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionMode {
    /// Headless PTY (current behavior, no visible window)
    #[default]
    Headless,

    /// Visual terminal window (pops up visible window)
    Visual,
}

impl std::fmt::Display for SessionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionMode::Headless => write!(f, "headless"),
            SessionMode::Visual => write!(f, "visual"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_mode_default() {
        assert_eq!(SessionMode::default(), SessionMode::Headless);
    }

    #[test]
    fn test_session_mode_display() {
        assert_eq!(format!("{}", SessionMode::Headless), "headless");
        assert_eq!(format!("{}", SessionMode::Visual), "visual");
    }

    #[test]
    fn test_visual_terminal_handle() {
        let handle = VisualTerminalHandle::new(1234, "test-terminal");
        assert_eq!(handle.pid, 1234);
        assert_eq!(handle.terminal_name, "test-terminal");
        assert_eq!(handle.window_id, None);

        let handle_with_id =
            VisualTerminalHandle::with_window_id(5678, "test-terminal", "window-123");
        assert_eq!(handle_with_id.pid, 5678);
        assert_eq!(handle_with_id.terminal_name, "test-terminal");
        assert_eq!(handle_with_id.window_id, Some("window-123".to_string()));
    }
}
