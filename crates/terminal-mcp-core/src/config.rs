//! Configuration types for Terminal MCP Server.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Server configuration loaded from YAML file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ServerConfig {
    /// Server settings
    pub server: ServerSettings,
    /// Security settings
    pub security: SecuritySettings,
    /// Detection settings
    pub detection: DetectionSettings,
    /// Terminal settings
    pub terminal: TerminalSettings,
}

impl ServerConfig {
    /// Load configuration from a YAML file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Parse configuration from YAML string.
    pub fn from_yaml(yaml: &str) -> crate::Result<Self> {
        let config: ServerConfig = serde_yaml::from_str(yaml)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values.
    pub fn validate(&self) -> crate::Result<()> {
        // Validate max_sessions
        if self.server.max_sessions == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "server.max_sessions must be > 0",
            )
            .into());
        }

        // Validate terminal dimensions
        if self.terminal.default_rows == 0 || self.terminal.default_cols == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "terminal dimensions must be > 0",
            )
            .into());
        }

        // Validate custom patterns
        for pattern in &self.detection.custom_patterns {
            pattern.validate()?;
        }

        Ok(())
    }
}

/// Server settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    /// Transport type (stdio, tcp, etc.)
    pub transport: String,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Session timeout in seconds (0 = no timeout)
    pub session_timeout: u64,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            transport: "stdio".to_string(),
            max_sessions: 10,
            session_timeout: 3600,
            log_level: "info".to_string(),
        }
    }
}

/// Security settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecuritySettings {
    /// List of allowed commands (empty = allow all)
    pub allowed_commands: Vec<String>,
    /// Sandbox mode: none, container, namespace
    pub sandbox_mode: String,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            allowed_commands: vec![],
            sandbox_mode: "none".to_string(),
        }
    }
}

impl SecuritySettings {
    /// Check if a command is allowed.
    ///
    /// Returns true if allowed_commands is empty (allow all) or if the command
    /// matches one of the allowed commands.
    pub fn is_command_allowed(&self, command: &str) -> bool {
        if self.allowed_commands.is_empty() {
            return true;
        }
        self.allowed_commands
            .iter()
            .any(|allowed| allowed == command)
    }
}

/// Detection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DetectionSettings {
    /// Idle threshold in milliseconds
    pub idle_threshold_ms: u64,
    /// Maximum idle wait time in milliseconds
    pub max_idle_wait_ms: u64,
    /// Custom pattern definitions
    pub custom_patterns: Vec<CustomPatternConfig>,
}

impl Default for DetectionSettings {
    fn default() -> Self {
        Self {
            idle_threshold_ms: 100,
            max_idle_wait_ms: 5000,
            custom_patterns: vec![],
        }
    }
}

/// Custom pattern configuration for domain-specific element detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPatternConfig {
    /// Pattern name (identifier)
    pub name: String,
    /// Regular expression pattern
    pub pattern: String,
    /// Element type to generate
    pub element_type: String,
    /// Named captures
    #[serde(default)]
    pub captures: Vec<CaptureConfig>,
}

impl CustomPatternConfig {
    /// Validate the pattern configuration.
    pub fn validate(&self) -> crate::Result<()> {
        // Validate name is not empty
        if self.name.trim().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "custom pattern name cannot be empty",
            )
            .into());
        }

        // Validate regex pattern
        regex::Regex::new(&self.pattern).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid regex pattern '{}': {}", self.name, e),
            )
        })?;

        // Validate element_type
        if self.element_type.trim().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "custom pattern '{}' element_type cannot be empty",
                    self.name
                ),
            )
            .into());
        }

        Ok(())
    }
}

/// Named capture configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Capture name
    pub name: String,
}

/// Terminal settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalSettings {
    /// Default terminal rows
    pub default_rows: u16,
    /// Default terminal columns
    pub default_cols: u16,
    /// Scrollback buffer lines
    pub scrollback_lines: usize,
    /// TERM environment variable value
    pub term: String,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            default_rows: 24,
            default_cols: 80,
            scrollback_lines: 10000,
            term: "xterm-256color".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.server.transport, "stdio");
        assert_eq!(config.server.max_sessions, 10);
        assert_eq!(config.terminal.default_rows, 24);
        assert_eq!(config.terminal.default_cols, 80);
    }

    #[test]
    fn test_config_validation() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_max_sessions() {
        let mut config = ServerConfig::default();
        config.server.max_sessions = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_dimensions() {
        let mut config = ServerConfig::default();
        config.terminal.default_rows = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
server:
  transport: stdio
  max_sessions: 5
  session_timeout: 1800
  log_level: debug

security:
  allowed_commands:
    - /bin/bash
    - /usr/bin/htop
  sandbox_mode: none

detection:
  idle_threshold_ms: 200
  max_idle_wait_ms: 10000
  custom_patterns: []

terminal:
  default_rows: 30
  default_cols: 120
  scrollback_lines: 5000
  term: "xterm-256color"
"#;

        let config = ServerConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.server.max_sessions, 5);
        assert_eq!(config.server.session_timeout, 1800);
        assert_eq!(config.server.log_level, "debug");
        assert_eq!(config.security.allowed_commands.len(), 2);
        assert_eq!(config.detection.idle_threshold_ms, 200);
        assert_eq!(config.terminal.default_rows, 30);
        assert_eq!(config.terminal.default_cols, 120);
    }

    #[test]
    fn test_custom_patterns() {
        let yaml = r#"
detection:
  custom_patterns:
    - name: "service_status"
      pattern: "^(\\w+)\\s+(Running|Stopped)$"
      element_type: "table_row"
      captures:
        - name: "service"
        - name: "status"
"#;

        let config = ServerConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.detection.custom_patterns.len(), 1);
        assert_eq!(config.detection.custom_patterns[0].name, "service_status");
        assert_eq!(config.detection.custom_patterns[0].captures.len(), 2);
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let yaml = r#"
detection:
  custom_patterns:
    - name: "bad_pattern"
      pattern: "([unclosed"
      element_type: "text"
"#;

        let result = ServerConfig::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_security_command_allowed() {
        let mut settings = SecuritySettings::default();

        // Empty list = allow all
        assert!(settings.is_command_allowed("/bin/bash"));
        assert!(settings.is_command_allowed("anything"));

        // With allowlist
        settings.allowed_commands = vec!["/bin/bash".to_string(), "/usr/bin/htop".to_string()];
        assert!(settings.is_command_allowed("/bin/bash"));
        assert!(settings.is_command_allowed("/usr/bin/htop"));
        assert!(!settings.is_command_allowed("/bin/sh"));
    }

    #[test]
    fn test_empty_pattern_name() {
        let pattern = CustomPatternConfig {
            name: "".to_string(),
            pattern: "test".to_string(),
            element_type: "text".to_string(),
            captures: vec![],
        };
        assert!(pattern.validate().is_err());
    }

    #[test]
    fn test_empty_element_type() {
        let pattern = CustomPatternConfig {
            name: "test".to_string(),
            pattern: "test".to_string(),
            element_type: "".to_string(),
            captures: vec![],
        };
        assert!(pattern.validate().is_err());
    }
}
