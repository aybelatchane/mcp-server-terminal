//! Terminal emulator registry for managing available terminals.

use super::{VisualTerminal, VisualTerminalHandle};
use terminal_mcp_core::{Dimensions, Error, Platform, Result};

/// Registry for managing and selecting terminal emulators.
pub struct TerminalRegistry {
    terminals: Vec<Box<dyn VisualTerminal>>,
}

impl TerminalRegistry {
    /// Create a new terminal registry for the current platform.
    pub fn for_platform(platform: Platform) -> Self {
        let terminals = Self::get_terminals_for_platform(platform);
        Self { terminals }
    }

    /// Create a registry with the default platform (auto-detected).
    pub fn new() -> Self {
        Self::for_platform(Platform::detect())
    }

    /// Get available terminals for the specified platform.
    fn get_terminals_for_platform(platform: Platform) -> Vec<Box<dyn VisualTerminal>> {
        match platform {
            #[cfg(target_os = "linux")]
            Platform::Linux => Self::linux_terminals(),

            #[cfg(target_os = "linux")]
            Platform::WSL => Self::wsl_terminals(),

            #[cfg(target_os = "macos")]
            Platform::MacOS => Self::macos_terminals(),

            #[cfg(target_os = "windows")]
            Platform::Windows => Self::windows_terminals(),

            // Fallback for platforms not matching compile target
            #[allow(unreachable_patterns)]
            _ => Vec::new(),
        }
    }

    #[cfg(target_os = "linux")]
    fn linux_terminals() -> Vec<Box<dyn VisualTerminal>> {
        use super::linux::*;

        vec![
            Box::new(GnomeTerminal),
            Box::new(Konsole),
            Box::new(Alacritty),
            Box::new(Kitty),
            Box::new(XTerm),
        ]
    }

    #[cfg(target_os = "linux")]
    fn wsl_terminals() -> Vec<Box<dyn VisualTerminal>> {
        use super::linux::*;

        vec![
            // Try Windows Terminal first (best experience)
            Box::new(WindowsTerminalWSL),
            // Fall back to X11 terminals if X server available
            Box::new(GnomeTerminal),
            Box::new(Konsole),
            Box::new(XTerm),
            // Tmux as last resort
            Box::new(Tmux),
        ]
    }

    #[cfg(target_os = "macos")]
    fn macos_terminals() -> Vec<Box<dyn VisualTerminal>> {
        use super::macos::*;

        vec![Box::new(ITerm2), Box::new(MacOSTerminal)]
    }

    #[cfg(target_os = "windows")]
    fn windows_terminals() -> Vec<Box<dyn VisualTerminal>> {
        use super::windows::*;

        vec![
            Box::new(WindowsTerminal),
            Box::new(PowerShell),
            Box::new(CmdExe),
        ]
    }

    /// Find the best available terminal emulator.
    ///
    /// Selects the highest-priority terminal that is currently available.
    pub fn find_best_terminal(&self) -> Option<&dyn VisualTerminal> {
        self.terminals
            .iter()
            .filter(|t| t.is_available())
            .max_by_key(|t| t.priority())
            .map(|t| t.as_ref())
    }

    /// Find a specific terminal by name.
    pub fn find_terminal_by_name(&self, name: &str) -> Option<&dyn VisualTerminal> {
        self.terminals
            .iter()
            .find(|t| t.name().eq_ignore_ascii_case(name))
            .map(|t| t.as_ref())
    }

    /// Get all available terminals, sorted by priority (highest first).
    pub fn available_terminals(&self) -> Vec<&dyn VisualTerminal> {
        let mut terminals: Vec<_> = self
            .terminals
            .iter()
            .filter(|t| t.is_available())
            .map(|t| t.as_ref())
            .collect();

        terminals.sort_by_key(|t| std::cmp::Reverse(t.priority()));
        terminals
    }

    /// Spawn a terminal using the best available emulator.
    pub fn spawn_best(
        &self,
        command: &str,
        args: &[String],
        dimensions: Dimensions,
    ) -> Result<VisualTerminalHandle> {
        let terminal = self
            .find_best_terminal()
            .ok_or_else(|| Error::Other("No visual terminal emulator available".to_string()))?;

        terminal.spawn(command, args, dimensions)
    }

    /// Spawn a terminal using a specific emulator by name.
    pub fn spawn_with(
        &self,
        terminal_name: &str,
        command: &str,
        args: &[String],
        dimensions: Dimensions,
    ) -> Result<VisualTerminalHandle> {
        let terminal = self.find_terminal_by_name(terminal_name).ok_or_else(|| {
            Error::Other(format!("Terminal emulator '{terminal_name}' not found"))
        })?;

        if !terminal.is_available() {
            return Err(Error::Other(format!(
                "Terminal emulator '{terminal_name}' is not available"
            )));
        }

        terminal.spawn(command, args, dimensions)
    }
}

impl Default for TerminalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = TerminalRegistry::new();
        // Should create without panicking
        assert!(!registry.terminals.is_empty());
    }

    #[test]
    fn test_platform_specific_terminals() {
        let platform = Platform::detect();
        let registry = TerminalRegistry::for_platform(platform);

        // Should have at least one terminal for the current platform
        assert!(!registry.terminals.is_empty());
    }

    #[test]
    fn test_available_terminals_sorted() {
        let registry = TerminalRegistry::new();
        let available = registry.available_terminals();

        // Check that terminals are sorted by priority (descending)
        for i in 1..available.len() {
            assert!(available[i - 1].priority() >= available[i].priority());
        }
    }
}
