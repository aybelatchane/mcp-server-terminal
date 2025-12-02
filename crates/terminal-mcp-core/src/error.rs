//! Error types for the Terminal MCP Server.

use thiserror::Error;

use crate::SessionId;

/// Main error type for Terminal MCP operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    /// Element not found by reference ID
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    /// PTY-related errors
    #[error("PTY error: {0}")]
    PtyError(String),

    /// Command not allowed (security/policy violation)
    #[error("Command not allowed: {0}")]
    CommandNotAllowed(String),

    /// Timeout waiting for condition
    #[error("Timeout waiting for condition after {0}ms")]
    WaitTimeout(u64),

    /// Invalid key string
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Session limit reached
    #[error("Session limit reached (max: {0})")]
    SessionLimitReached(usize),

    /// Invalid terminal dimensions
    #[error("Invalid dimensions: {rows}x{cols}")]
    InvalidDimensions {
        /// Number of rows
        rows: u16,
        /// Number of columns
        cols: u16,
    },

    /// Session already terminated
    #[error("Session already terminated")]
    SessionTerminated,

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid input or parameters (generic)
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Generic error with custom message
    #[error("{0}")]
    Other(String),
}

/// Result type alias using our Error type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_not_found_error() {
        let session_id = SessionId::new();
        let err = Error::SessionNotFound(session_id);
        let display = err.to_string();
        assert!(display.starts_with("Session not found:"));
    }

    #[test]
    fn test_element_not_found_error() {
        let err = Error::ElementNotFound("elem_123".to_string());
        assert_eq!(err.to_string(), "Element not found: elem_123");
    }

    #[test]
    fn test_pty_error() {
        let err = Error::PtyError("spawn failed".to_string());
        assert_eq!(err.to_string(), "PTY error: spawn failed");
    }

    #[test]
    fn test_command_not_allowed_error() {
        let err = Error::CommandNotAllowed("rm -rf /".to_string());
        assert_eq!(err.to_string(), "Command not allowed: rm -rf /");
    }

    #[test]
    fn test_wait_timeout_error() {
        let err = Error::WaitTimeout(5000);
        assert_eq!(
            err.to_string(),
            "Timeout waiting for condition after 5000ms"
        );
    }

    #[test]
    fn test_invalid_key_error() {
        let err = Error::InvalidKey("Ctrl+".to_string());
        assert_eq!(err.to_string(), "Invalid key: Ctrl+");
    }

    #[test]
    fn test_session_limit_reached_error() {
        let err = Error::SessionLimitReached(10);
        assert_eq!(err.to_string(), "Session limit reached (max: 10)");
    }

    #[test]
    fn test_invalid_dimensions_error() {
        let err = Error::InvalidDimensions { rows: 0, cols: 100 };
        assert_eq!(err.to_string(), "Invalid dimensions: 0x100");
    }

    #[test]
    fn test_session_terminated_error() {
        let err = Error::SessionTerminated;
        assert_eq!(err.to_string(), "Session already terminated");
    }

    #[test]
    fn test_parse_error() {
        let err = Error::ParseError("unexpected EOF".to_string());
        assert_eq!(err.to_string(), "Parse error: unexpected EOF");
    }

    #[test]
    fn test_config_error() {
        let err = Error::Config("missing field: shell".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing field: shell");
    }

    #[test]
    fn test_invalid_input_error() {
        let err = Error::InvalidInput("missing parameter".to_string());
        assert_eq!(err.to_string(), "Invalid input: missing parameter");
    }

    #[test]
    fn test_other_error() {
        let err = Error::Other("unknown error".to_string());
        assert_eq!(err.to_string(), "unknown error");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_serialization_error_conversion() {
        let json_err = serde_json::from_str::<i32>("invalid json").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn test_result_type() {
        let success: Result<i32> = Ok(42);
        assert!(success.is_ok());
        if let Ok(value) = success {
            assert_eq!(value, 42);
        }

        let failure: Result<i32> = Err(Error::Other("test error".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_debug() {
        let err = Error::InvalidInput("test".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("InvalidInput"));
    }
}
