//! Session manager for coordinating multiple terminal sessions.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use terminal_mcp_core::{Dimensions, Error, Result, SessionId};

use crate::session::{Session, SessionStatus};

/// Configuration for session manager.
#[derive(Debug, Clone)]
pub struct SessionManagerConfig {
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,

    /// Default terminal rows
    pub default_rows: u16,

    /// Default terminal columns
    pub default_cols: u16,
}

impl Default for SessionManagerConfig {
    fn default() -> Self {
        Self {
            max_sessions: 10,
            default_rows: 24,
            default_cols: 80,
        }
    }
}

/// Session manager for coordinating multiple terminal sessions.
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Session>>>>,
    config: SessionManagerConfig,
}

impl SessionManager {
    /// Create a new session manager with default configuration.
    pub fn new() -> Self {
        Self::with_config(SessionManagerConfig::default())
    }

    /// Create a new session manager with custom configuration.
    pub fn with_config(config: SessionManagerConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a new terminal session.
    pub fn create_session(
        &self,
        command: String,
        args: Vec<String>,
        dimensions: Option<Dimensions>,
    ) -> Result<Arc<Session>> {
        // Check session limit
        let sessions = self.sessions.read().unwrap();
        if sessions.len() >= self.config.max_sessions {
            return Err(Error::SessionLimitReached(self.config.max_sessions));
        }
        drop(sessions);

        // Use provided dimensions or defaults
        let dims = dimensions
            .unwrap_or_else(|| Dimensions::new(self.config.default_rows, self.config.default_cols));

        // Create session
        let session = Session::create(command, args, dims)?;
        let session_id = *session.id();
        let session = Arc::new(session);

        // Store session
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id, Arc::clone(&session));

        Ok(session)
    }

    /// Get a session by ID.
    pub fn get_session(&self, session_id: &SessionId) -> Result<Arc<Session>> {
        let sessions = self.sessions.read().unwrap();
        sessions
            .get(session_id)
            .cloned()
            .ok_or(Error::SessionNotFound(*session_id))
    }

    /// List all active sessions.
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().unwrap();
        sessions
            .values()
            .map(|session| SessionInfo {
                session_id: *session.id(),
                command: session.command().to_string(),
                status: session.status(),
                created_at: session.created_at(),
            })
            .collect()
    }

    /// Close a session by ID.
    pub fn close_session(&self, session_id: &SessionId) -> Result<()> {
        // Get and terminate the session
        let session = self.get_session(session_id)?;
        session.terminate()?;

        // Remove from registry
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(session_id);

        Ok(())
    }

    /// Close all sessions.
    pub fn close_all(&self) -> Result<()> {
        let session_ids: Vec<SessionId> = {
            let sessions = self.sessions.read().unwrap();
            sessions.keys().copied().collect()
        };

        for session_id in session_ids {
            let _ = self.close_session(&session_id);
        }

        Ok(())
    }

    /// Get the number of active sessions.
    pub fn session_count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session ID
    pub session_id: SessionId,

    /// Command
    pub command: String,

    /// Session status
    pub status: SessionStatus,

    /// Creation time
    pub created_at: std::time::SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_create() {
        let manager = SessionManager::new();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_manager_create_session() {
        let manager = SessionManager::new();

        let result = manager.create_session("echo".to_string(), vec!["test".to_string()], None);

        assert!(result.is_ok());
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_session_manager_get_session() {
        let manager = SessionManager::new();

        let session = manager
            .create_session("echo".to_string(), vec![], None)
            .unwrap();

        let session_id = *session.id();

        let retrieved = manager.get_session(&session_id);
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id(), &session_id);
    }

    #[test]
    fn test_session_manager_get_nonexistent_session() {
        let manager = SessionManager::new();
        let fake_id = SessionId::new();

        let result = manager.get_session(&fake_id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SessionNotFound(_)));
    }

    #[test]
    fn test_session_manager_list_sessions() {
        let manager = SessionManager::new();

        manager
            .create_session("echo".to_string(), vec!["1".to_string()], None)
            .unwrap();

        manager
            .create_session("echo".to_string(), vec!["2".to_string()], None)
            .unwrap();

        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_session_manager_close_session() {
        let manager = SessionManager::new();

        let session = manager
            .create_session(
                if cfg!(windows) { "cmd.exe" } else { "sh" }.to_string(),
                vec![],
                None,
            )
            .unwrap();

        let session_id = *session.id();

        assert_eq!(manager.session_count(), 1);

        let result = manager.close_session(&session_id);
        assert!(result.is_ok());
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_manager_session_limit() {
        let config = SessionManagerConfig {
            max_sessions: 2,
            ..Default::default()
        };

        let manager = SessionManager::with_config(config);

        // Create 2 sessions (should succeed)
        manager
            .create_session("echo".to_string(), vec![], None)
            .unwrap();
        manager
            .create_session("echo".to_string(), vec![], None)
            .unwrap();

        // Try to create 3rd session (should fail)
        let result = manager.create_session("echo".to_string(), vec![], None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SessionLimitReached(_)));
    }

    #[test]
    fn test_session_manager_close_all() {
        let manager = SessionManager::new();

        manager
            .create_session(
                if cfg!(windows) { "cmd.exe" } else { "sh" }.to_string(),
                vec![],
                None,
            )
            .unwrap();

        manager
            .create_session(
                if cfg!(windows) { "cmd.exe" } else { "sh" }.to_string(),
                vec![],
                None,
            )
            .unwrap();

        assert_eq!(manager.session_count(), 2);

        let result = manager.close_all();
        assert!(result.is_ok());
        assert_eq!(manager.session_count(), 0);
    }
}
