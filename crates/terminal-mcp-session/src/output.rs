//! Terminal output reading and buffering.

use std::sync::{Arc, Mutex};

use terminal_mcp_core::Result;

use crate::session::Session;

/// Output buffer for tracking raw terminal output.
#[derive(Debug)]
pub struct OutputBuffer {
    /// Raw bytes received from PTY (with ANSI codes)
    raw_buffer: Vec<u8>,
    /// Position of last read
    last_read_pos: usize,
}

impl OutputBuffer {
    /// Create a new output buffer.
    pub fn new() -> Self {
        Self {
            raw_buffer: Vec::new(),
            last_read_pos: 0,
        }
    }

    /// Append new output to the buffer.
    pub fn append(&mut self, bytes: &[u8]) {
        self.raw_buffer.extend_from_slice(bytes);
    }

    /// Get all output (with ANSI codes).
    pub fn read_all(&mut self) -> Vec<u8> {
        let output = self.raw_buffer.clone();
        self.last_read_pos = self.raw_buffer.len();
        output
    }

    /// Get output since last read (with ANSI codes).
    pub fn read_since_last(&mut self) -> Vec<u8> {
        let output = self.raw_buffer[self.last_read_pos..].to_vec();
        self.last_read_pos = self.raw_buffer.len();
        output
    }

    /// Get current buffer size.
    pub fn size(&self) -> usize {
        self.raw_buffer.len()
    }

    /// Get unread bytes count.
    pub fn unread_count(&self) -> usize {
        self.raw_buffer.len() - self.last_read_pos
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.raw_buffer.clear();
        self.last_read_pos = 0;
    }
}

impl Default for OutputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Output read result.
#[derive(Debug, Clone)]
pub struct OutputRead {
    /// The output text
    pub output: String,
    /// Number of bytes
    pub bytes: usize,
    /// Whether there's more unread output
    pub has_more: bool,
}

impl Session {
    /// Read raw terminal output.
    ///
    /// # Arguments
    /// * `since_last_read` - If true, only return new output since last read
    /// * `include_ansi` - If true, include ANSI escape codes; otherwise strip them
    pub fn read_output(&self, since_last_read: bool, include_ansi: bool) -> Result<OutputRead> {
        // Process any pending PTY output first
        self.process_output()?;

        // Get the raw bytes
        let output_buf_arc = self.output_buffer();
        let mut output_buffer = output_buf_arc.lock().unwrap();
        let raw_bytes = if since_last_read {
            output_buffer.read_since_last()
        } else {
            output_buffer.read_all()
        };

        let bytes = raw_bytes.len();
        let has_more = output_buffer.unread_count() > 0;
        drop(output_buffer);

        // Convert to string
        let output = if include_ansi {
            // Include ANSI codes
            String::from_utf8_lossy(&raw_bytes).to_string()
        } else {
            // Strip ANSI codes - use the grid's plain text
            let parser_arc = self.parser();
            let parser = parser_arc.lock().unwrap();
            parser.grid().to_plain_text()
        };

        Ok(OutputRead {
            output,
            bytes,
            has_more,
        })
    }

    /// Get the output buffer.
    pub(crate) fn output_buffer(&self) -> Arc<Mutex<OutputBuffer>> {
        // This will be added to Session struct
        Arc::clone(&self.output_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_buffer_new() {
        let buffer = OutputBuffer::new();
        assert_eq!(buffer.size(), 0);
        assert_eq!(buffer.unread_count(), 0);
    }

    #[test]
    fn test_output_buffer_append() {
        let mut buffer = OutputBuffer::new();
        buffer.append(b"Hello");
        buffer.append(b" World");

        assert_eq!(buffer.size(), 11);
        assert_eq!(buffer.unread_count(), 11);
    }

    #[test]
    fn test_output_buffer_read_all() {
        let mut buffer = OutputBuffer::new();
        buffer.append(b"Test output");

        let output = buffer.read_all();
        assert_eq!(output, b"Test output");
        assert_eq!(buffer.unread_count(), 0);

        // Reading again returns empty
        let output2 = buffer.read_since_last();
        assert_eq!(output2, b"");
    }

    #[test]
    fn test_output_buffer_read_since_last() {
        let mut buffer = OutputBuffer::new();
        buffer.append(b"First");

        let output1 = buffer.read_since_last();
        assert_eq!(output1, b"First");

        buffer.append(b" Second");
        let output2 = buffer.read_since_last();
        assert_eq!(output2, b" Second");
    }

    #[test]
    fn test_output_buffer_clear() {
        let mut buffer = OutputBuffer::new();
        buffer.append(b"Some data");
        buffer.clear();

        assert_eq!(buffer.size(), 0);
        assert_eq!(buffer.unread_count(), 0);
    }
}
