//! PTY (Pseudo-Terminal) handling with portable-pty.

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task;
use tracing::{debug, error, info, warn};

use terminal_mcp_core::{Dimensions, Error, Result};

/// Handle to a spawned PTY process.
pub struct PtyHandle {
    /// The master PTY end (None for tmux mode)
    master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
    /// The child process (None for tmux mode)
    child: Arc<Mutex<Option<Box<dyn Child + Send + Sync>>>>,
    /// Current PTY dimensions
    dimensions: Arc<Mutex<Dimensions>>,
    /// PTY writer (None for tmux mode)
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    /// PTY reader (None for tmux mode) - kept as field to maintain non-blocking FD
    reader: Arc<Mutex<Option<Box<dyn Read + Send>>>>,
    /// Tmux session name (Some for tmux mode)
    tmux_session: Option<String>,
    /// Last tmux capture content (for change detection)
    last_tmux_content: Arc<Mutex<Vec<u8>>>,
}

impl std::fmt::Debug for PtyHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyHandle")
            .field("dimensions", &self.dimensions)
            .finish_non_exhaustive()
    }
}

impl PtyHandle {
    /// Spawn a new PTY with the given command and dimensions.
    ///
    /// # Arguments
    /// * `command` - Command to execute (e.g., "/bin/bash", "powershell.exe")
    /// * `args` - Command arguments
    /// * `dimensions` - Initial terminal dimensions
    ///
    /// # Example
    /// ```no_run
    /// use terminal_mcp_emulator::pty::PtyHandle;
    /// use terminal_mcp_core::Dimensions;
    ///
    /// # async fn example() -> terminal_mcp_core::Result<()> {
    /// let pty = PtyHandle::spawn("/bin/bash", &[], Dimensions::new(24, 80))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn spawn(command: &str, args: &[String], dimensions: Dimensions, cwd: Option<String>) -> Result<Self> {
        info!(
            "Spawning PTY: command='{}' args={:?}, dimensions={}x{}, cwd={:?}",
            command, args, dimensions.rows, dimensions.cols, cwd
        );

        let pty_system = native_pty_system();

        // Create PTY with specified dimensions
        let pty_size = PtySize {
            rows: dimensions.rows,
            cols: dimensions.cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        debug!("Opening PTY with native system");
        let pair = pty_system.openpty(pty_size).map_err(|e| {
            error!("Failed to open PTY: {}", e);
            Error::PtyError(format!("Failed to open PTY: {e}"))
        })?;

        // Build command
        let mut cmd = CommandBuilder::new(command);
        for arg in args {
            cmd.arg(arg);
        }

        // Set working directory if specified
        if let Some(dir) = cwd {
            debug!("Setting working directory to: {}", dir);
            cmd.cwd(dir);
        }

        debug!("Spawning child process: {}", command);
        // Spawn child process
        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            error!("Failed to spawn command '{}': {}", command, e);
            Error::PtyError(format!("Failed to spawn command: {e}"))
        })?;

        // Take the writer once and store it
        let writer = pair.master.take_writer().map_err(|e| {
            error!("Failed to take PTY writer: {}", e);
            Error::PtyError(format!("Failed to take writer: {e}"))
        })?;

        // Take the reader once and store it
        let reader = pair.master.try_clone_reader().map_err(|e| {
            error!("Failed to clone PTY reader: {}", e);
            Error::PtyError(format!("Failed to clone reader: {e}"))
        })?;

        // Set reader to non-blocking mode (critical for wait_for_idle to work)
        #[cfg(unix)]
        {
            // We need to access the raw FD through the master since reader is a trait object
            if let Some(master_fd) = pair.master.as_raw_fd() {
                unsafe {
                    // Get current flags
                    let flags = libc::fcntl(master_fd, libc::F_GETFL, 0);
                    if flags != -1 {
                        // Set O_NONBLOCK on the master FD
                        // This affects all reads from this FD
                        let result =
                            libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                        if result == -1 {
                            error!("Failed to set master PTY to non-blocking mode");
                        } else {
                            debug!("Set master PTY FD {} to non-blocking mode", master_fd);
                        }
                    }
                }
            }
        }

        info!("PTY spawned successfully: command='{}'", command);

        Ok(Self {
            master: Arc::new(Mutex::new(Some(pair.master))),
            child: Arc::new(Mutex::new(Some(child))),
            dimensions: Arc::new(Mutex::new(dimensions)),
            writer: Arc::new(Mutex::new(Some(writer))),
            reader: Arc::new(Mutex::new(Some(reader))),
            tmux_session: None,
            last_tmux_content: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Spawn a PTY wrapper for controlling an existing tmux session.
    ///
    /// This creates a "virtual" PTY that uses tmux commands for I/O,
    /// allowing MCP tools to control a visual terminal session.
    pub fn spawn_tmux(session_name: &str, dimensions: Dimensions) -> Result<Self> {
        // Verify tmux session exists
        use std::process::Command;
        let check = Command::new("tmux")
            .arg("has-session")
            .arg("-t")
            .arg(session_name)
            .status()
            .map_err(|e| Error::PtyError(format!("Failed to check tmux session: {e}")))?;

        if !check.success() {
            return Err(Error::PtyError(format!(
                "Tmux session '{session_name}' does not exist"
            )));
        }

        Ok(Self {
            master: Arc::new(Mutex::new(None)),
            child: Arc::new(Mutex::new(None)),
            dimensions: Arc::new(Mutex::new(dimensions)),
            writer: Arc::new(Mutex::new(None)),
            reader: Arc::new(Mutex::new(None)),
            tmux_session: Some(session_name.to_string()),
            last_tmux_content: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Read available output from the PTY (non-blocking).
    ///
    /// Returns bytes read from the PTY. May return empty vec if no data available.
    /// For tmux mode, only returns data if content has changed since last read.
    pub fn read(&self) -> Result<Vec<u8>> {
        // Tmux mode: capture pane content and detect changes
        if let Some(session) = &self.tmux_session {
            use std::process::Command;

            // First check if tmux session still exists (with timeout)
            let check = Command::new("tmux")
                .arg("has-session")
                .arg("-t")
                .arg(session)
                .output();

            match check {
                Ok(output) if !output.status.success() => {
                    return Err(Error::PtyError(format!(
                        "Tmux session '{session}' no longer exists"
                    )));
                }
                Err(e) => {
                    return Err(Error::PtyError(format!(
                        "Failed to check tmux session '{session}': {e}"
                    )));
                }
                _ => {}
            }

            // Try to capture alternate screen first (for TUI apps like vim, htop, bubbletea)
            // If no alt-screen exists, fall back to normal screen
            // NOTE: We don't use -e flag to avoid escape sequences that cause cursor positioning issues
            // when parsed through VTE. Plain text gives us correct, unfragmented output.
            // We use -J to join wrapped lines so long lines appear correctly without artificial breaks.
            let mut output = Command::new("tmux")
                .arg("capture-pane")
                .arg("-p") // Print to stdout
                .arg("-J") // Join wrapped lines
                .arg("-a") // Capture alternate screen (for TUI apps)
                .arg("-q") // Quiet (don't error if no alt-screen)
                .arg("-t")
                .arg(session)
                .output()
                .map_err(|e| Error::PtyError(format!("Failed to capture tmux pane: {e}")))?;

            // If alt-screen capture returned empty or only whitespace (no alt-screen active),
            // fall back to normal screen capture
            let alt_output_is_empty = output.stdout.is_empty()
                || output.stdout.iter().all(|&b| b == b'\n' || b == b'\r' || b == b' ' || b == b'\t');

            if alt_output_is_empty && output.status.success() {
                output = Command::new("tmux")
                    .arg("capture-pane")
                    .arg("-p") // Print to stdout
                    .arg("-J") // Join wrapped lines
                    .arg("-t")
                    .arg(session)
                    .output()
                    .map_err(|e| Error::PtyError(format!("Failed to capture tmux pane: {e}")))?;
            }

            // Check if tmux command failed
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::PtyError(format!(
                    "Tmux capture-pane failed for session '{session}': {stderr}"
                )));
            }

            let new_content = output.stdout;

            // Check if content has changed since last read
            let mut last_content = self.last_tmux_content.lock().unwrap();

            // If cache was invalidated (empty), always return content even if unchanged
            // Otherwise, only return content if it changed
            let cache_was_invalidated = last_content.is_empty();
            if !cache_was_invalidated && new_content == *last_content {
                // No change and cache is valid - return empty to signal "idle"
                return Ok(Vec::new());
            }

            // Content changed or cache was invalidated - update cache
            if cache_was_invalidated {
                debug!("Tmux cache was invalidated, forcing fresh read: {} bytes", new_content.len());
            } else {
                debug!("Tmux pane content changed: {} bytes", new_content.len());
            }
            *last_content = new_content.clone();

            // For tmux snapshots, prepend a cursor home sequence to ensure VTE parser
            // writes from the top-left corner. This prevents fragmentation issues.
            // ESC[H = Cursor Home (move to row 1, col 1)
            let mut content_with_home = vec![0x1b, b'[', b'H']; // ESC[H
            content_with_home.extend_from_slice(&new_content);

            // Return the snapshot content with cursor home prepended
            return Ok(content_with_home);
        }

        // Regular PTY mode - use the stored reader (already set to non-blocking)
        let mut reader_lock = self
            .reader
            .lock()
            .map_err(|e| Error::PtyError(format!("Reader lock error: {e}")))?;

        let reader = reader_lock
            .as_mut()
            .ok_or_else(|| Error::PtyError("PTY reader not initialized".to_string()))?;

        // Read with non-blocking mode (set during spawn)
        let mut buffer = vec![0u8; 4096];

        match reader.read(&mut buffer) {
            Ok(n) => {
                buffer.truncate(n);
                if n > 0 {
                    debug!("Read {} bytes from PTY", n);
                }
                Ok(buffer)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available - this is expected in non-blocking mode
                Ok(Vec::new())
            }
            Err(e) => Err(Error::Io(e)),
        }
    }

    /// Read output from PTY asynchronously.
    ///
    /// Returns a receiver that yields chunks of output as they become available.
    pub fn read_async(&self) -> mpsc::UnboundedReceiver<Vec<u8>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let master = Arc::clone(&self.master);

        task::spawn(async move {
            loop {
                let result = task::spawn_blocking({
                    let master = Arc::clone(&master);
                    move || {
                        let master_lock = master.lock().ok()?;
                        let master_ref = master_lock.as_ref()?;
                        let mut reader = master_ref.try_clone_reader().ok()?;
                        let mut buffer = vec![0u8; 4096];

                        match reader.read(&mut buffer) {
                            Ok(0) => None, // EOF
                            Ok(n) => {
                                buffer.truncate(n);
                                Some(buffer)
                            }
                            Err(_) => None,
                        }
                    }
                })
                .await;

                match result {
                    Ok(Some(data)) => {
                        if tx.send(data).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Ok(None) | Err(_) => break, // EOF or error
                }

                // Small delay to avoid busy-waiting
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });

        rx
    }

    /// Write data to tmux session using hex mode for reliable escape sequence delivery.
    ///
    /// Uses the `-H` flag to send bytes as hexadecimal, which ensures escape sequences
    /// are delivered correctly without interpretation issues.
    fn write_to_tmux(&self, session: &str, data: &[u8]) -> Result<usize> {
        use std::process::Command;

        // Convert bytes to hex strings for -H mode
        // Each byte becomes a separate argument in hex format
        let hex_args: Vec<String> = data.iter().map(|b| format!("{:02x}", b)).collect();

        debug!(
            "Sending {} bytes to tmux session '{}' via hex mode",
            data.len(),
            session
        );

        let mut cmd = Command::new("tmux");
        cmd.arg("send-keys").arg("-t").arg(session).arg("-H");

        for hex in &hex_args {
            cmd.arg(hex);
        }

        let status = cmd
            .status()
            .map_err(|e| Error::PtyError(format!("Failed to send keys to tmux: {e}")))?;

        if !status.success() {
            return Err(Error::PtyError(format!(
                "Tmux send-keys failed with status: {status}"
            )));
        }

        Ok(data.len())
    }

    /// Write data to the PTY.
    ///
    /// # Arguments
    /// * `data` - Bytes to write to the PTY
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        debug!("Writing {} bytes to PTY", data.len());
        // Tmux mode: use send-keys with hex mode for reliable delivery
        if let Some(session) = &self.tmux_session {
            return self.write_to_tmux(session, data);
        }

        // Regular PTY mode
        let mut writer_lock = self
            .writer
            .lock()
            .map_err(|e| Error::PtyError(format!("Lock error: {e}")))?;

        let writer = writer_lock
            .as_mut()
            .ok_or_else(|| Error::PtyError("PTY writer not initialized".to_string()))?;

        writer.write_all(data).map_err(Error::Io)?;
        writer.flush().map_err(Error::Io)?;

        Ok(data.len())
    }

    /// Resize the PTY to new dimensions.
    ///
    /// Sends SIGWINCH to the child process to notify of the size change.
    pub fn resize(&self, new_dimensions: Dimensions) -> Result<()> {
        info!(
            "Resizing PTY to {}x{}",
            new_dimensions.rows, new_dimensions.cols
        );
        // Tmux mode: resize window
        if let Some(session) = &self.tmux_session {
            use std::process::Command;
            let status = Command::new("tmux")
                .arg("resize-window")
                .arg("-t")
                .arg(session)
                .arg("-x")
                .arg(new_dimensions.cols.to_string())
                .arg("-y")
                .arg(new_dimensions.rows.to_string())
                .status()
                .map_err(|e| Error::PtyError(format!("Failed to resize tmux window: {e}")))?;

            if !status.success() {
                return Err(Error::PtyError("Tmux resize failed".to_string()));
            }

            // Update stored dimensions
            let mut dims = self
                .dimensions
                .lock()
                .map_err(|e| Error::PtyError(format!("Lock error: {e}")))?;
            *dims = new_dimensions;

            return Ok(());
        }

        // Regular PTY mode
        let master_lock = self
            .master
            .lock()
            .map_err(|e| Error::PtyError(format!("Lock error: {e}")))?;

        let master = master_lock
            .as_ref()
            .ok_or_else(|| Error::PtyError("PTY not initialized".to_string()))?;

        let new_size = PtySize {
            rows: new_dimensions.rows,
            cols: new_dimensions.cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        master
            .resize(new_size)
            .map_err(|e| Error::PtyError(format!("Resize failed: {e}")))?;

        // Update stored dimensions
        let mut dims = self
            .dimensions
            .lock()
            .map_err(|e| Error::PtyError(format!("Lock error: {e}")))?;
        *dims = new_dimensions;

        Ok(())
    }

    /// Get current PTY dimensions.
    pub fn dimensions(&self) -> Result<Dimensions> {
        let dims = self
            .dimensions
            .lock()
            .map_err(|e| Error::PtyError(format!("Lock error: {e}")))?;
        Ok(*dims)
    }

    /// Check if this PTY is in tmux mode.
    pub fn is_tmux_mode(&self) -> bool {
        self.tmux_session.is_some()
    }

    /// Force refresh of tmux content cache on next read.
    /// This ensures the next read will fetch fresh content from tmux.
    pub fn invalidate_tmux_cache(&self) -> Result<()> {
        if self.tmux_session.is_some() {
            let mut last_content = self.last_tmux_content.lock().unwrap();
            last_content.clear();
        }
        Ok(())
    }

    /// Check if the child process is still running.
    pub fn is_alive(&self) -> bool {
        // Tmux mode: check if session exists
        if let Some(session) = &self.tmux_session {
            use std::process::Command;
            return Command::new("tmux")
                .arg("has-session")
                .arg("-t")
                .arg(session)
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
        }

        // Regular PTY mode
        let mut child_lock = match self.child.lock() {
            Ok(c) => c,
            Err(_) => return false,
        };

        let child = match child_lock.as_mut() {
            Some(c) => c,
            None => return false,
        };

        child.try_wait().ok().flatten().is_none()
    }

    /// Wait for the child process to exit.
    ///
    /// Returns the exit status if available.
    pub fn wait(&self) -> Result<()> {
        // Tmux mode: can't wait for session (it may be detached)
        if self.tmux_session.is_some() {
            return Ok(());
        }

        // Regular PTY mode
        let mut child_lock = self
            .child
            .lock()
            .map_err(|e| Error::PtyError(format!("Lock error: {e}")))?;

        let child = child_lock
            .as_mut()
            .ok_or_else(|| Error::PtyError("Child not initialized".to_string()))?;

        child
            .wait()
            .map(|_| ()) // Ignore the portable-pty ExitStatus
            .map_err(|e| Error::PtyError(format!("Wait failed: {e}")))
    }

    /// Kill the child process.
    ///
    /// Attempts graceful termination first (SIGTERM), then forced (SIGKILL) if needed.
    pub fn kill(&self) -> Result<()> {
        info!("Killing PTY process");
        // Tmux mode: kill session
        if let Some(session) = &self.tmux_session {
            use std::process::Command;
            use std::thread;
            use std::time::Duration;

            info!("Killing tmux session: {}", session);

            let status = Command::new("tmux")
                .arg("kill-session")
                .arg("-t")
                .arg(session)
                .status()
                .map_err(|e| Error::PtyError(format!("Failed to kill tmux session: {e}")))?;

            if !status.success() {
                warn!("Tmux kill-session returned non-zero status");
            }

            // Give tmux time to clean up
            thread::sleep(Duration::from_millis(100));

            // Verify session is actually gone
            let verify = Command::new("tmux")
                .arg("has-session")
                .arg("-t")
                .arg(session)
                .status();

            match verify {
                Ok(status) if !status.success() => {
                    info!("Tmux session {} successfully terminated", session);
                }
                Ok(_) => {
                    warn!("Tmux session {} still exists after kill attempt", session);
                }
                Err(e) => {
                    debug!("Error verifying tmux session cleanup: {}", e);
                }
            }

            return Ok(());
        }

        // Regular PTY mode
        let mut child_lock = self
            .child
            .lock()
            .map_err(|e| Error::PtyError(format!("Lock error: {e}")))?;

        let child = child_lock
            .as_mut()
            .ok_or_else(|| Error::PtyError("Child not initialized".to_string()))?;

        child
            .kill()
            .map_err(|e| Error::PtyError(format!("Kill failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_pty_spawn() {
        let shell = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };

        let pty = PtyHandle::spawn(shell, &[], Dimensions::new(24, 80));
        assert!(pty.is_ok());

        let pty = pty.unwrap();
        assert!(pty.is_alive());
    }

    #[test]
    fn test_pty_dimensions() {
        let shell = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };

        let dims = Dimensions::new(30, 100);
        let pty = PtyHandle::spawn(shell, &[], dims).unwrap();

        let current_dims = pty.dimensions().unwrap();
        assert_eq!(current_dims.rows, 30);
        assert_eq!(current_dims.cols, 100);
    }

    #[test]
    fn test_pty_write_and_read() {
        let shell = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };

        let pty = PtyHandle::spawn(shell, &[], Dimensions::new(24, 80)).unwrap();

        // Write a command
        let command: &[u8] = if cfg!(windows) {
            b"echo hello\r\n"
        } else {
            b"echo hello\n"
        };

        pty.write(command).unwrap();

        // Give it time to process
        std::thread::sleep(Duration::from_millis(100));

        // Read output
        let output = pty.read().unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_pty_resize() {
        let shell = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };

        let pty = PtyHandle::spawn(shell, &[], Dimensions::new(24, 80)).unwrap();

        // Resize
        let new_dims = Dimensions::new(40, 120);
        let result = pty.resize(new_dims);
        assert!(result.is_ok());

        // Verify new dimensions
        let dims = pty.dimensions().unwrap();
        assert_eq!(dims.rows, 40);
        assert_eq!(dims.cols, 120);
    }

    #[test]
    fn test_pty_kill() {
        let shell = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };

        let pty = PtyHandle::spawn(shell, &[], Dimensions::new(24, 80)).unwrap();
        assert!(pty.is_alive());

        // Kill the process
        pty.kill().unwrap();

        // Give it time to die
        std::thread::sleep(Duration::from_millis(100));

        // Should no longer be alive
        assert!(!pty.is_alive());
    }

    #[tokio::test]
    async fn test_pty_read_async() {
        let shell = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };

        let pty = PtyHandle::spawn(shell, &[], Dimensions::new(24, 80)).unwrap();
        let mut rx = pty.read_async();

        // Write a command
        let command: &[u8] = if cfg!(windows) {
            b"echo test\r\n"
        } else {
            b"echo test\n"
        };

        pty.write(command).unwrap();

        // Wait for output with timeout
        let output = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;

        assert!(output.is_ok());
        assert!(output.unwrap().is_some());

        // Cleanup
        pty.kill().unwrap();
    }
}
