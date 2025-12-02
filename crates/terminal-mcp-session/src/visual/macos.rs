//! macOS terminal emulator implementations.

use super::{VisualTerminal, VisualTerminalHandle};
use std::process::Command;
use terminal_mcp_core::{Dimensions, Error, Result};

/// macOS Terminal.app (built-in terminal).
pub struct MacOSTerminal;

impl VisualTerminal for MacOSTerminal {
    fn spawn(
        &self,
        command: &str,
        args: &[String],
        _dimensions: Dimensions,
    ) -> Result<VisualTerminalHandle> {
        let full_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };

        // Use AppleScript to open Terminal.app with the command
        let script = format!(
            "tell application \"Terminal\" to do script \"{}\"",
            full_command.replace('"', "\\\"")
        );

        let child = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn Terminal.app: {}", e)))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        // Terminal.app is always available on macOS
        true
    }

    fn name(&self) -> &'static str {
        "Terminal.app"
    }

    fn priority(&self) -> u8 {
        80 // Good default for macOS
    }
}

/// iTerm2 terminal emulator (popular third-party terminal for macOS).
pub struct ITerm2;

impl VisualTerminal for ITerm2 {
    fn spawn(
        &self,
        command: &str,
        args: &[String],
        _dimensions: Dimensions,
    ) -> Result<VisualTerminalHandle> {
        let full_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };

        // Use AppleScript to open iTerm2 with the command
        let script = format!(
            r#"tell application "iTerm2"
                create window with default profile
                tell current session of current window
                    write text "{}"
                end tell
            end tell"#,
            full_command.replace('"', "\\\"")
        );

        let child = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn iTerm2: {}", e)))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        // Check if iTerm2 is installed
        std::path::Path::new("/Applications/iTerm.app").exists()
    }

    fn name(&self) -> &'static str {
        "iTerm2"
    }

    fn priority(&self) -> u8 {
        100 // Highest priority if available (better than Terminal.app)
    }
}
