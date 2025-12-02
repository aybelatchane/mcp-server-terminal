//! Windows terminal emulator implementations.

use super::{VisualTerminal, VisualTerminalHandle};
use std::process::Command;
use terminal_mcp_core::{Dimensions, Error, Result};

/// Check if a command exists in PATH (Windows version).
fn command_exists(cmd: &str) -> bool {
    Command::new("where")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Windows Terminal (modern terminal for Windows 10/11).
pub struct WindowsTerminal;

impl VisualTerminal for WindowsTerminal {
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

        let child = Command::new("wt.exe")
            .arg("new-tab")
            .arg("--")
            .arg("cmd.exe")
            .arg("/k")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn Windows Terminal: {}", e)))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        command_exists("wt.exe")
    }

    fn name(&self) -> &'static str {
        "Windows Terminal"
    }

    fn priority(&self) -> u8 {
        100 // Highest priority (modern, recommended)
    }
}

/// PowerShell terminal.
pub struct PowerShell;

impl VisualTerminal for PowerShell {
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

        let script = format!(
            "Start-Process powershell -ArgumentList '-NoExit', '-Command', '{}'",
            full_command.replace('\'', "''")
        );

        let child = Command::new("powershell.exe")
            .arg("-Command")
            .arg(&script)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn PowerShell: {}", e)))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        command_exists("powershell.exe")
    }

    fn name(&self) -> &'static str {
        "PowerShell"
    }

    fn priority(&self) -> u8 {
        60 // Medium priority
    }
}

/// cmd.exe (classic Windows command prompt, always available).
pub struct CmdExe;

impl VisualTerminal for CmdExe {
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

        let child = Command::new("cmd.exe")
            .arg("/c")
            .arg("start")
            .arg("cmd.exe")
            .arg("/k")
            .arg(&full_command)
            .spawn()
            .map_err(|e| Error::Other(format!("Failed to spawn cmd.exe: {}", e)))?;

        Ok(VisualTerminalHandle::new(child.id(), self.name()))
    }

    fn is_available(&self) -> bool {
        // cmd.exe is always available on Windows
        true
    }

    fn name(&self) -> &'static str {
        "cmd.exe"
    }

    fn priority(&self) -> u8 {
        50 // Low priority, fallback only
    }
}
