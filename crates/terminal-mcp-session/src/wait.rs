//! Wait conditions and mechanisms for terminal state changes.

use regex::Regex;
use std::time::{Duration, Instant};

use terminal_mcp_core::{Error, Result, TerminalStateTree};
use terminal_mcp_detector::DetectionPipeline;

use crate::session::Session;
use crate::snapshot::SnapshotConfig;

/// Condition to wait for in terminal state.
#[derive(Debug, Clone)]
pub struct WaitCondition {
    /// Text pattern to match (regex)
    pub text: Option<String>,

    /// Element type to wait for
    pub element_type: Option<String>,

    /// Wait for condition to disappear instead of appear
    pub gone: bool,

    /// Wait for terminal to be idle
    pub idle: bool,

    /// Maximum time to wait
    pub timeout: Duration,

    /// Polling interval between checks
    pub poll_interval: Duration,
}

impl Default for WaitCondition {
    fn default() -> Self {
        Self {
            text: None,
            element_type: None,
            gone: false,
            idle: false,
            timeout: Duration::from_secs(30),
            poll_interval: Duration::from_millis(100),
        }
    }
}

impl WaitCondition {
    /// Create a new wait condition with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Wait for text to appear.
    pub fn for_text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Self::default()
        }
    }

    /// Wait for text to disappear.
    pub fn for_text_gone(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            gone: true,
            ..Self::default()
        }
    }

    /// Wait for element type to appear.
    pub fn for_element(element_type: impl Into<String>) -> Self {
        Self {
            element_type: Some(element_type.into()),
            ..Self::default()
        }
    }

    /// Wait for element type to disappear.
    pub fn for_element_gone(element_type: impl Into<String>) -> Self {
        Self {
            element_type: Some(element_type.into()),
            gone: true,
            ..Self::default()
        }
    }

    /// Wait for terminal to become idle.
    pub fn for_idle() -> Self {
        Self {
            idle: true,
            ..Self::default()
        }
    }

    /// Set timeout duration.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set polling interval.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }
}

/// Result of a wait operation.
#[derive(Debug, Clone)]
pub struct WaitResult {
    /// Whether the condition was met
    pub condition_met: bool,

    /// Time waited in milliseconds
    pub waited_ms: u64,

    /// Terminal state tree when condition was met (or at timeout)
    pub snapshot: TerminalStateTree,
}

impl Session {
    /// Wait for a condition to be met.
    ///
    /// Repeatedly takes snapshots and checks if the condition is satisfied.
    /// Returns when the condition is met or timeout is reached.
    ///
    /// # Arguments
    /// * `condition` - The condition to wait for
    /// * `pipeline` - Detection pipeline for taking snapshots
    /// * `snapshot_config` - Configuration for snapshot operations
    ///
    /// # Returns
    /// `WaitResult` containing the snapshot and whether condition was met
    ///
    /// # Example
    /// ```no_run
    /// # use terminal_mcp_session::{Session, WaitCondition};
    /// # use terminal_mcp_detector::DetectionPipeline;
    /// # use terminal_mcp_session::SnapshotConfig;
    /// # use terminal_mcp_core::Dimensions;
    /// # use std::time::Duration;
    /// # let session = Session::create("bash".to_string(), vec![], Dimensions::new(24, 80)).unwrap();
    /// let pipeline = DetectionPipeline::new();
    /// let config = SnapshotConfig::default();
    ///
    /// // Wait for "Success" to appear
    /// let condition = WaitCondition::for_text("Success")
    ///     .with_timeout(Duration::from_secs(10));
    ///
    /// let result = session.wait_for(&condition, &pipeline, &config).unwrap();
    /// if result.condition_met {
    ///     println!("Condition met after {}ms", result.waited_ms);
    /// }
    /// ```
    pub fn wait_for(
        &self,
        condition: &WaitCondition,
        pipeline: &DetectionPipeline,
        snapshot_config: &SnapshotConfig,
    ) -> Result<WaitResult> {
        let start = Instant::now();

        // For idle condition, we need to wait for terminal to become idle
        // Track the last activity time to detect when terminal stabilizes
        let mut last_activity_check = if condition.idle {
            Some(Instant::now())
        } else {
            None
        };

        loop {
            // Check timeout
            let elapsed = start.elapsed();
            if elapsed >= condition.timeout {
                // Timeout reached - take final snapshot
                let snapshot = self.snapshot(pipeline, snapshot_config)?;
                return Ok(WaitResult {
                    condition_met: false,
                    waited_ms: elapsed.as_millis() as u64,
                    snapshot,
                });
            }

            // For idle condition, check if terminal has new output
            if let Some(last_check) = last_activity_check {
                // Process output without blocking
                let bytes_read = self.process_output()?;

                if bytes_read > 0 {
                    // Terminal is still active, reset timer
                    last_activity_check = Some(Instant::now());
                } else if last_check.elapsed() >= snapshot_config.idle_threshold {
                    // Terminal has been idle long enough
                    let snapshot = self.snapshot(pipeline, snapshot_config)?;
                    return Ok(WaitResult {
                        condition_met: true,
                        waited_ms: elapsed.as_millis() as u64,
                        snapshot,
                    });
                }

                // Small sleep to avoid busy-waiting
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }

            // Take snapshot for non-idle conditions
            let snapshot = self.snapshot(pipeline, snapshot_config)?;

            // Check if condition is met
            if Self::check_condition(&snapshot, condition)? {
                return Ok(WaitResult {
                    condition_met: true,
                    waited_ms: elapsed.as_millis() as u64,
                    snapshot,
                });
            }

            // Wait before next poll
            std::thread::sleep(condition.poll_interval);
        }
    }

    /// Check if a condition is satisfied by the given snapshot.
    fn check_condition(snapshot: &TerminalStateTree, condition: &WaitCondition) -> Result<bool> {
        // Check text condition
        if let Some(pattern) = &condition.text {
            let regex = Regex::new(pattern)
                .map_err(|e| Error::InvalidInput(format!("Invalid regex: {e}")))?;

            let found = regex.is_match(&snapshot.raw_text);
            return Ok(if condition.gone { !found } else { found });
        }

        // Check element type condition
        if let Some(elem_type) = &condition.element_type {
            let found = snapshot.elements.iter().any(|e| e.type_name() == elem_type);

            return Ok(if condition.gone { !found } else { found });
        }

        // No specific condition to check (idle is handled separately in wait_for)
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_mcp_core::Dimensions;

    #[test]
    fn test_wait_condition_default() {
        let condition = WaitCondition::default();
        assert_eq!(condition.timeout, Duration::from_secs(30));
        assert_eq!(condition.poll_interval, Duration::from_millis(100));
        assert!(!condition.gone);
        assert!(!condition.idle);
    }

    #[test]
    fn test_wait_condition_for_text() {
        let condition = WaitCondition::for_text("hello");
        assert_eq!(condition.text, Some("hello".to_string()));
        assert!(!condition.gone);
    }

    #[test]
    fn test_wait_condition_for_text_gone() {
        let condition = WaitCondition::for_text_gone("loading");
        assert_eq!(condition.text, Some("loading".to_string()));
        assert!(condition.gone);
    }

    #[test]
    fn test_wait_condition_for_element() {
        let condition = WaitCondition::for_element("menu");
        assert_eq!(condition.element_type, Some("menu".to_string()));
        assert!(!condition.gone);
    }

    #[test]
    fn test_wait_condition_for_idle() {
        let condition = WaitCondition::for_idle();
        assert!(condition.idle);
    }

    #[test]
    fn test_wait_condition_with_timeout() {
        let condition = WaitCondition::for_text("test").with_timeout(Duration::from_secs(5));
        assert_eq!(condition.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_wait_for_idle_always_succeeds() {
        let session = Session::create(
            "echo".to_string(),
            vec!["test".to_string()],
            Dimensions::new(24, 80),
        )
        .unwrap();

        let pipeline = DetectionPipeline::new();
        let snapshot_config = SnapshotConfig::default();

        // Wait for idle with short timeout
        let condition = WaitCondition::for_idle().with_timeout(Duration::from_secs(2));

        std::thread::sleep(Duration::from_millis(200));

        let result = session.wait_for(&condition, &pipeline, &snapshot_config);
        assert!(result.is_ok());

        let wait_result = result.unwrap();
        assert!(wait_result.condition_met);
    }

    #[test]
    fn test_wait_for_text_appears() {
        let session = Session::create(
            "echo".to_string(),
            vec!["hello world".to_string()],
            Dimensions::new(24, 80),
        )
        .unwrap();

        let pipeline = DetectionPipeline::new();
        let snapshot_config = SnapshotConfig {
            idle_threshold: Duration::from_millis(50),
            ..Default::default()
        };

        // Wait for "hello" to appear
        let condition = WaitCondition::for_text("hello")
            .with_timeout(Duration::from_secs(2))
            .with_poll_interval(Duration::from_millis(50));

        std::thread::sleep(Duration::from_millis(200));

        let result = session.wait_for(&condition, &pipeline, &snapshot_config);
        assert!(result.is_ok());

        let wait_result = result.unwrap();
        assert!(wait_result.condition_met);
        assert!(wait_result.snapshot.raw_text.contains("hello"));
    }

    #[test]
    fn test_wait_for_text_timeout() {
        let session = Session::create(
            "echo".to_string(),
            vec!["test".to_string()],
            Dimensions::new(24, 80),
        )
        .unwrap();

        let pipeline = DetectionPipeline::new();
        let snapshot_config = SnapshotConfig {
            idle_threshold: Duration::from_millis(50),
            idle_timeout: Duration::from_millis(500),
            ..Default::default()
        };

        // Wait for text that won't appear
        let condition = WaitCondition::for_text("nonexistent")
            .with_timeout(Duration::from_millis(300))
            .with_poll_interval(Duration::from_millis(50));

        std::thread::sleep(Duration::from_millis(200));

        let result = session.wait_for(&condition, &pipeline, &snapshot_config);
        assert!(result.is_ok());

        let wait_result = result.unwrap();
        assert!(!wait_result.condition_met); // Timeout reached
        assert!(wait_result.waited_ms >= 300);
    }

    #[test]
    fn test_check_condition_text_regex() {
        use terminal_mcp_core::Position;

        let snapshot = TerminalStateTree {
            session_id: "test".to_string(),
            dimensions: Dimensions::new(24, 80),
            cursor: Position::new(0, 0),
            timestamp: "2025-11-30T00:00:00Z".to_string(),
            elements: vec![],
            raw_text: "Server started successfully on port 8080".to_string(),
            ansi_buffer: None,
        };

        // Test regex pattern matching
        let condition = WaitCondition::for_text("port \\d+");
        assert!(Session::check_condition(&snapshot, &condition).unwrap());

        // Test non-matching pattern
        let condition = WaitCondition::for_text("failed");
        assert!(!Session::check_condition(&snapshot, &condition).unwrap());

        // Test gone condition
        let condition = WaitCondition::for_text_gone("failed");
        assert!(Session::check_condition(&snapshot, &condition).unwrap());
    }
}
