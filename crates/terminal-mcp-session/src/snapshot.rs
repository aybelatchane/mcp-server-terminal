//! Terminal snapshot functionality.

use std::time::{Duration, Instant};

use terminal_mcp_core::{Result, TerminalStateTree};
use terminal_mcp_detector::{DetectionPipeline, TSTAssembler};

use crate::session::Session;

/// Configuration for snapshot operations.
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Maximum time to wait for idle before taking snapshot
    pub idle_timeout: Duration,

    /// Time to consider terminal "idle" (no output received)
    pub idle_threshold: Duration,

    /// Maximum number of bytes to process per iteration
    pub max_bytes_per_iteration: usize,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(5),
            idle_threshold: Duration::from_millis(100),
            max_bytes_per_iteration: 4096,
        }
    }
}

impl Session {
    /// Capture a snapshot of the terminal state.
    ///
    /// This waits for the terminal to become idle (no output for idle_threshold),
    /// then captures the current grid state and runs detection to build a
    /// Terminal State Tree.
    pub fn snapshot(
        &self,
        pipeline: &DetectionPipeline,
        config: &SnapshotConfig,
    ) -> Result<TerminalStateTree> {
        // Wait for idle
        self.wait_for_idle(config)?;

        // For tmux mode (visual mode), clear the grid and take one final snapshot
        // This ensures we get a clean state without accumulated content
        let pty_arc = self.pty();
        let pty = pty_arc.lock().unwrap();
        let is_tmux = pty.is_tmux_mode();
        drop(pty);

        if is_tmux {
            // Clear the grid to start fresh
            let parser_arc = self.parser();
            let mut parser = parser_arc.lock().unwrap();
            parser.grid_mut().clear();
            drop(parser);

            // Invalidate tmux cache to force fresh read
            let pty_arc = self.pty();
            let pty = pty_arc.lock().unwrap();
            pty.invalidate_tmux_cache()?;
            drop(pty);

            // Process one final read to get the clean tmux snapshot
            self.process_output()?;
        }

        // Get grid state
        let parser_arc = self.parser();
        let parser = parser_arc.lock().unwrap();
        let grid = parser.grid();
        let cursor = grid.cursor().position;
        let dimensions = grid.dimensions();
        let raw_text = grid.to_plain_text();

        // Run detection pipeline
        let detected = pipeline.detect(grid, cursor);

        // Build TST
        let assembler = TSTAssembler::new();
        let tst = assembler.assemble(
            detected,
            self.id().to_string(),
            dimensions,
            cursor,
            raw_text,
        );

        Ok(tst)
    }

    /// Wait for terminal to become idle.
    ///
    /// Continuously processes PTY output until no new output is received
    /// for the configured idle_threshold duration, or until idle_timeout is reached.
    fn wait_for_idle(&self, config: &SnapshotConfig) -> Result<()> {
        let start = Instant::now();
        let mut last_output = Instant::now();

        loop {
            // Check timeout
            if start.elapsed() > config.idle_timeout {
                break;
            }

            // Process available output
            let bytes_read = self.process_output()?;

            if bytes_read > 0 {
                // Reset idle timer
                last_output = Instant::now();
            } else {
                // Check if idle long enough
                if last_output.elapsed() >= config.idle_threshold {
                    break;
                }
            }

            // Small sleep to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_mcp_core::Dimensions;

    #[test]
    fn test_snapshot_config_default() {
        let config = SnapshotConfig::default();
        assert_eq!(config.idle_timeout, Duration::from_secs(5));
        assert_eq!(config.idle_threshold, Duration::from_millis(100));
        assert_eq!(config.max_bytes_per_iteration, 4096);
    }

    #[test]
    fn test_snapshot() {
        let session = Session::create(
            "echo".to_string(),
            vec!["test".to_string()],
            Dimensions::new(24, 80),
        )
        .unwrap();

        let pipeline = DetectionPipeline::new();
        let config = SnapshotConfig::default();

        // Allow some time for command to execute
        std::thread::sleep(Duration::from_millis(200));

        let result = session.snapshot(&pipeline, &config);
        assert!(result.is_ok());

        let tst = result.unwrap();
        assert_eq!(tst.session_id, session.id().to_string());
        assert_eq!(tst.dimensions, Dimensions::new(24, 80));
    }

    #[test]
    fn test_wait_for_idle() {
        let session = Session::create(
            "echo".to_string(),
            vec!["test".to_string()],
            Dimensions::new(24, 80),
        )
        .unwrap();

        let config = SnapshotConfig {
            idle_timeout: Duration::from_secs(2),
            idle_threshold: Duration::from_millis(100),
            max_bytes_per_iteration: 4096,
        };

        let start = Instant::now();
        let result = session.wait_for_idle(&config);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Should complete within idle_timeout
        assert!(elapsed < config.idle_timeout + Duration::from_millis(500));
    }

    #[test]
    fn test_snapshot_with_custom_config() {
        let session = Session::create(
            "echo".to_string(),
            vec!["hello".to_string()],
            Dimensions::new(24, 80),
        )
        .unwrap();

        let pipeline = DetectionPipeline::new();
        let config = SnapshotConfig {
            idle_timeout: Duration::from_secs(2),
            idle_threshold: Duration::from_millis(50),
            max_bytes_per_iteration: 2048,
        };

        std::thread::sleep(Duration::from_millis(200));

        let result = session.snapshot(&pipeline, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_with_idle_bash_no_hang() {
        // Regression test for #99: terminal_snapshot should not hang on idle shells
        let session = Session::create("bash".to_string(), vec![], Dimensions::new(24, 80)).unwrap();

        let pipeline = DetectionPipeline::new();
        let config = SnapshotConfig {
            idle_threshold: Duration::from_millis(500),
            idle_timeout: Duration::from_secs(3),
            ..Default::default()
        };

        // Let bash start and reach idle prompt
        std::thread::sleep(Duration::from_millis(500));

        // This should complete within idle_timeout, not hang indefinitely
        let start = Instant::now();
        let result = session.snapshot(&pipeline, &config);
        let elapsed = start.elapsed();

        assert!(result.is_ok(), "Snapshot should succeed on idle bash");
        assert!(
            elapsed < Duration::from_secs(3),
            "Snapshot should complete within timeout, not hang (elapsed: {:?})",
            elapsed
        );
    }
}
