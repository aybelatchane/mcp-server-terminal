//! Platform detection for cross-platform terminal emulator support.
//!
//! This module provides runtime platform detection to enable
//! platform-specific terminal emulator selection.

use serde::{Deserialize, Serialize};

/// Supported platforms for visual terminal mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Native Linux (not WSL)
    Linux,
    /// macOS
    MacOS,
    /// Native Windows
    Windows,
    /// Windows Subsystem for Linux
    WSL,
}

impl Platform {
    /// Detect the current platform at runtime.
    ///
    /// # Platform Detection Logic
    ///
    /// - **WSL**: Checks `/proc/version` for "microsoft" or "Microsoft" string
    /// - **Linux**: target_os = "linux" and not WSL
    /// - **macOS**: target_os = "macos"
    /// - **Windows**: target_os = "windows"
    ///
    /// # Examples
    ///
    /// ```
    /// use terminal_mcp_core::Platform;
    ///
    /// let platform = Platform::detect();
    /// println!("Running on: {:?}", platform);
    /// ```
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            // Check if running under WSL
            if Self::is_wsl() {
                return Platform::WSL;
            }
            Platform::Linux
        }

        #[cfg(target_os = "macos")]
        {
            Platform::MacOS
        }

        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            // Fallback for unsupported platforms
            compile_error!("Unsupported platform - only Linux, macOS, and Windows are supported")
        }
    }

    /// Check if running under Windows Subsystem for Linux (WSL).
    ///
    /// Detection strategies:
    /// 1. Check `/proc/version` for "microsoft" or "Microsoft"
    /// 2. Check for `/proc/sys/fs/binfmt_misc/WSLInterop` file
    #[cfg(target_os = "linux")]
    fn is_wsl() -> bool {
        // Strategy 1: Check /proc/version
        if let Ok(version) = std::fs::read_to_string("/proc/version") {
            if version.to_lowercase().contains("microsoft") {
                return true;
            }
        }

        // Strategy 2: Check for WSLInterop
        if std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists() {
            return true;
        }

        false
    }

    /// Get the platform name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Linux => "Linux",
            Platform::MacOS => "macOS",
            Platform::Windows => "Windows",
            Platform::WSL => "WSL",
        }
    }

    /// Check if this is a Unix-like platform.
    pub fn is_unix(&self) -> bool {
        matches!(self, Platform::Linux | Platform::MacOS | Platform::WSL)
    }

    /// Check if this is Windows or WSL (Windows-based).
    pub fn is_windows_based(&self) -> bool {
        matches!(self, Platform::Windows | Platform::WSL)
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detect() {
        let platform = Platform::detect();

        // Should return one of the valid platforms
        assert!(matches!(
            platform,
            Platform::Linux | Platform::MacOS | Platform::Windows | Platform::WSL
        ));
    }

    #[test]
    fn test_platform_name() {
        assert_eq!(Platform::Linux.name(), "Linux");
        assert_eq!(Platform::MacOS.name(), "macOS");
        assert_eq!(Platform::Windows.name(), "Windows");
        assert_eq!(Platform::WSL.name(), "WSL");
    }

    #[test]
    fn test_is_unix() {
        assert!(Platform::Linux.is_unix());
        assert!(Platform::MacOS.is_unix());
        assert!(Platform::WSL.is_unix());
        assert!(!Platform::Windows.is_unix());
    }

    #[test]
    fn test_is_windows_based() {
        assert!(!Platform::Linux.is_windows_based());
        assert!(!Platform::MacOS.is_windows_based());
        assert!(Platform::WSL.is_windows_based());
        assert!(Platform::Windows.is_windows_based());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_wsl_detection() {
        // This test will pass differently on WSL vs native Linux
        let is_wsl = Platform::is_wsl();
        let platform = Platform::detect();

        if is_wsl {
            assert_eq!(platform, Platform::WSL);
        } else {
            assert_eq!(platform, Platform::Linux);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Platform::Linux), "Linux");
        assert_eq!(format!("{}", Platform::MacOS), "macOS");
        assert_eq!(format!("{}", Platform::Windows), "Windows");
        assert_eq!(format!("{}", Platform::WSL), "WSL");
    }
}
