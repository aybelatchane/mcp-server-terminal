//! Session types for terminal session management.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Dimensions;

/// Unique identifier for a terminal session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new random session ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<Uuid> for SessionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of a terminal session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum SessionStatus {
    /// Session is initializing
    Initializing,
    /// Session is active and ready
    Active,
    /// Session is paused
    Paused,
    /// Session has terminated
    Terminated,
}

/// Configuration for creating a new terminal session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SessionConfig {
    /// Terminal dimensions (rows, columns)
    pub dimensions: Dimensions,
    /// Shell command to execute (e.g., "/bin/bash", "powershell.exe")
    pub shell: String,
    /// Working directory for the session
    pub working_directory: Option<String>,
    /// Environment variables
    pub env: Vec<(String, String)>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            dimensions: Dimensions::default(),
            shell: if cfg!(windows) {
                "powershell.exe".to_string()
            } else {
                "/bin/bash".to_string()
            },
            working_directory: None,
            env: Vec::new(),
        }
    }
}

/// Information about an active session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SessionInfo {
    /// Session identifier
    pub id: SessionId,
    /// Current status
    pub status: SessionStatus,
    /// Session configuration
    pub config: SessionConfig,
}

impl SessionInfo {
    /// Create new session info.
    pub fn new(id: SessionId, status: SessionStatus, config: SessionConfig) -> Self {
        Self { id, status, config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_creation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2); // Should generate different IDs
    }

    #[test]
    fn test_session_id_display() {
        let id = SessionId::new();
        let display = format!("{id}");
        assert!(!display.is_empty());
        assert_eq!(display.len(), 36); // UUID format length
    }

    #[test]
    fn test_session_status_variants() {
        let statuses = [
            SessionStatus::Initializing,
            SessionStatus::Active,
            SessionStatus::Paused,
            SessionStatus::Terminated,
        ];
        assert_eq!(statuses.len(), 4);
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.dimensions.rows, 24);
        assert_eq!(config.dimensions.cols, 80);
        assert!(!config.shell.is_empty());
        assert_eq!(config.working_directory, None);
        assert_eq!(config.env.len(), 0);
    }

    #[test]
    fn test_session_info_creation() {
        let id = SessionId::new();
        let config = SessionConfig::default();
        let info = SessionInfo::new(id, SessionStatus::Active, config.clone());

        assert_eq!(info.id, id);
        assert_eq!(info.status, SessionStatus::Active);
        assert_eq!(info.config, config);
    }

    #[test]
    fn test_session_serialization() {
        let id = SessionId::new();
        let config = SessionConfig::default();
        let info = SessionInfo::new(id, SessionStatus::Active, config);

        // Should be serializable to JSON
        let json = serde_json::to_string(&info).unwrap();
        assert!(!json.is_empty());

        // Should be deserializable from JSON
        let deserialized: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, id);
    }
}
