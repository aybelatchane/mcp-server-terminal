//! Linux terminal emulator implementations.

use super::{VisualTerminal, VisualTerminalHandle};
use std::process::Command;
use terminal_mcp_core::{Dimensions, Error, Result};

/// Check if a command exists in PATH.
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// GNOME Terminal emulator.
pub struct GnomeTerminal;

impl VisualTerminal for GnomeTerminal {
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

        let child = Command::new("gnome-terminal")
            .arg("--")
            .arg("bash")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn gnome-terminal: {e}")))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        command_exists("gnome-terminal")
    }

    fn name(&self) -> &'static str {
        "gnome-terminal"
    }

    fn priority(&self) -> u8 {
        100 // Highest priority for GNOME desktops
    }
}

/// KDE Konsole terminal emulator.
pub struct Konsole;

impl VisualTerminal for Konsole {
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

        let child = Command::new("konsole")
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn konsole: {e}")))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        command_exists("konsole")
    }

    fn name(&self) -> &'static str {
        "konsole"
    }

    fn priority(&self) -> u8 {
        100 // Highest priority for KDE desktops
    }
}

/// XTerm terminal emulator (universal fallback).
pub struct XTerm;

impl VisualTerminal for XTerm {
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

        let child = Command::new("xterm")
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn xterm: {e}")))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        command_exists("xterm")
    }

    fn name(&self) -> &'static str {
        "xterm"
    }

    fn priority(&self) -> u8 {
        50 // Low priority, fallback only
    }
}

/// Alacritty terminal emulator (modern, GPU-accelerated).
pub struct Alacritty;

impl VisualTerminal for Alacritty {
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

        let child = Command::new("alacritty")
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn alacritty: {e}")))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        command_exists("alacritty")
    }

    fn name(&self) -> &'static str {
        "alacritty"
    }

    fn priority(&self) -> u8 {
        70 // Modern terminal, good priority
    }
}

/// Kitty terminal emulator (modern, GPU-accelerated).
pub struct Kitty;

impl VisualTerminal for Kitty {
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

        let child = Command::new("kitty")
            .arg("bash")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn kitty: {e}")))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        command_exists("kitty")
    }

    fn name(&self) -> &'static str {
        "kitty"
    }

    fn priority(&self) -> u8 {
        70 // Modern terminal, good priority
    }
}

/// Windows Terminal accessed from WSL.
pub struct WindowsTerminalWSL;

impl VisualTerminal for WindowsTerminalWSL {
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

        // Try to find wt.exe in Windows path
        let child = Command::new("wt.exe")
            .arg("--")
            .arg("bash")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn wt.exe from WSL: {e}")))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        // Check if wt.exe is accessible from WSL
        Command::new("wt.exe")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        "wt.exe"
    }

    fn priority(&self) -> u8 {
        100 // Highest priority for WSL (best experience)
    }
}

/// Tmux terminal multiplexer (fallback for WSL).
pub struct Tmux;

impl VisualTerminal for Tmux {
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

        // Generate unique session name
        let session_name = format!("terminal-mcp-{}", uuid::Uuid::new_v4());

        // Create detached tmux session
        Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(&session_name)
            .arg("bash")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to create tmux session: {e}")))?;

        // Set remain-on-exit so session stays alive after command exits
        // This prevents "session no longer exists" errors when commands complete
        let _ = Command::new("tmux")
            .arg("set-option")
            .arg("-t")
            .arg(&session_name)
            .arg("remain-on-exit")
            .arg("on")
            .spawn();

        // Return handle with session name as window_id
        Ok(VisualTerminalHandle::with_window_id(
            0,
            self.name(),
            session_name,
        ))
    }

    fn is_available(&self) -> bool {
        command_exists("tmux")
    }

    fn name(&self) -> &'static str {
        "tmux"
    }

    fn priority(&self) -> u8 {
        40 // Low priority, multiplexer fallback
    }
}
