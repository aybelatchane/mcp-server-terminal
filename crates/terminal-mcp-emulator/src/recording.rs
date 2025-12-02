//! Session recording in asciinema v2 format.
//!
//! This module provides functionality to record terminal sessions in a format
//! compatible with asciinema (https://asciinema.org/), allowing playback of
//! sessions using standard asciinema tools.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use terminal_mcp_core::{Dimensions, Result};

/// Asciinema v2 format header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciinemaHeader {
    /// Format version (always 2)
    pub version: u8,
    /// Terminal width
    pub width: u16,
    /// Terminal height
    pub height: u16,
    /// Unix timestamp of recording start
    pub timestamp: Option<i64>,
    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// A single recording event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordEvent {
    /// Time offset from start in seconds (as float)
    #[serde(rename = "time")]
    pub time: f64,
    /// Event type: "o" for output, "i" for input
    #[serde(rename = "event_type")]
    pub event_type: String,
    /// Event data (terminal output or input)
    #[serde(rename = "data")]
    pub data: String,
}

/// Records terminal session events in asciinema v2 format.
///
/// The asciinema format consists of:
/// 1. A header line (JSON object with metadata)
/// 2. Event lines (JSON arrays with [time, event_type, data])
///
/// # Example
///
/// ```
/// use terminal_mcp_core::Dimensions;
/// use terminal_mcp_emulator::SessionRecorder;
///
/// let mut recorder = SessionRecorder::new(Dimensions::new(24, 80));
/// recorder.record_output(b"Hello, world!\r\n");
/// recorder.record_input(b"ls -la\r\n");
///
/// // Save to file
/// recorder.save_to_file("recording.cast").unwrap();
/// ```
#[derive(Debug)]
pub struct SessionRecorder {
    /// Recorded events
    events: Vec<RecordEvent>,
    /// Recording start time
    start_time: Instant,
    /// Terminal dimensions
    dimensions: Dimensions,
    /// Optional environment variables
    env: Option<HashMap<String, String>>,
}

impl SessionRecorder {
    /// Create a new session recorder.
    pub fn new(dimensions: Dimensions) -> Self {
        Self {
            events: Vec::new(),
            start_time: Instant::now(),
            dimensions,
            env: None,
        }
    }

    /// Create a new session recorder with environment variables.
    pub fn with_env(dimensions: Dimensions, env: HashMap<String, String>) -> Self {
        Self {
            events: Vec::new(),
            start_time: Instant::now(),
            dimensions,
            env: Some(env),
        }
    }

    /// Record terminal output.
    ///
    /// Records raw bytes received from the PTY.
    pub fn record_output(&mut self, data: &[u8]) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.events.push(RecordEvent {
            time: elapsed,
            event_type: "o".to_string(),
            data: String::from_utf8_lossy(data).to_string(),
        });
    }

    /// Record terminal input.
    ///
    /// Records raw bytes sent to the PTY.
    pub fn record_input(&mut self, data: &[u8]) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.events.push(RecordEvent {
            time: elapsed,
            event_type: "i".to_string(),
            data: String::from_utf8_lossy(data).to_string(),
        });
    }

    /// Get the number of recorded events.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get the duration of the recording in seconds.
    pub fn duration(&self) -> f64 {
        self.events.last().map(|e| e.time).unwrap_or(0.0)
    }

    /// Generate the asciinema header.
    fn header(&self) -> AsciinemaHeader {
        AsciinemaHeader {
            version: 2,
            width: self.dimensions.cols,
            height: self.dimensions.rows,
            timestamp: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
            env: self.env.clone(),
        }
    }

    /// Save the recording to a file in asciinema v2 format.
    ///
    /// The file format is:
    /// - Line 1: JSON header
    /// - Line 2+: JSON event arrays [time, event_type, data]
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path)?;

        // Write header
        let header = self.header();
        serde_json::to_writer(&mut file, &header)?;
        writeln!(file)?;

        // Write events
        for event in &self.events {
            let event_array = serde_json::json!([event.time, event.event_type, event.data]);
            serde_json::to_writer(&mut file, &event_array)?;
            writeln!(file)?;
        }

        file.flush()?;
        Ok(())
    }

    /// Save the recording to a writer in asciinema v2 format.
    pub fn save_to_writer<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write header
        let header = self.header();
        serde_json::to_writer(&mut *writer, &header)?;
        writeln!(writer)?;

        // Write events
        for event in &self.events {
            let event_array = serde_json::json!([event.time, event.event_type, event.data]);
            serde_json::to_writer(&mut *writer, &event_array)?;
            writeln!(writer)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Convert the recording to a string in asciinema v2 format.
    pub fn to_string(&self) -> Result<String> {
        let mut buffer = Vec::new();
        self.save_to_writer(&mut buffer)?;
        String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e).into())
    }

    /// Load a recording from a file.
    ///
    /// Parses an asciinema v2 format file and reconstructs the recording.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        content.parse().map_err(Into::into)
    }
}

impl FromStr for SessionRecorder {
    type Err = io::Error;

    /// Parse a recording from a string.
    fn from_str(content: &str) -> std::result::Result<Self, Self::Err> {
        let mut lines = content.lines();

        // Parse header
        let header_line = lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Empty recording file"))?;
        let header: AsciinemaHeader = serde_json::from_str(header_line)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Parse events
        let mut events = Vec::new();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let event_array: serde_json::Value = serde_json::from_str(line)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            if let Some(arr) = event_array.as_array() {
                if arr.len() >= 3 {
                    events.push(RecordEvent {
                        time: arr[0].as_f64().unwrap_or(0.0),
                        event_type: arr[1].as_str().unwrap_or("o").to_string(),
                        data: arr[2].as_str().unwrap_or("").to_string(),
                    });
                }
            }
        }

        Ok(Self {
            events,
            start_time: Instant::now(), // Reset to now
            dimensions: Dimensions::new(header.height, header.width),
            env: header.env,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorder_creation() {
        let recorder = SessionRecorder::new(Dimensions::new(24, 80));
        assert_eq!(recorder.event_count(), 0);
        assert_eq!(recorder.duration(), 0.0);
    }

    #[test]
    fn test_record_output() {
        let mut recorder = SessionRecorder::new(Dimensions::new(24, 80));
        recorder.record_output(b"Hello, world!");

        assert_eq!(recorder.event_count(), 1);
        assert!(recorder.duration() >= 0.0);
        assert_eq!(recorder.events[0].event_type, "o");
        assert_eq!(recorder.events[0].data, "Hello, world!");
    }

    #[test]
    fn test_record_input() {
        let mut recorder = SessionRecorder::new(Dimensions::new(24, 80));
        recorder.record_input(b"ls -la\r\n");

        assert_eq!(recorder.event_count(), 1);
        assert_eq!(recorder.events[0].event_type, "i");
        assert_eq!(recorder.events[0].data, "ls -la\r\n");
    }

    #[test]
    fn test_multiple_events() {
        let mut recorder = SessionRecorder::new(Dimensions::new(24, 80));
        recorder.record_output(b"$ ");
        std::thread::sleep(std::time::Duration::from_millis(10));
        recorder.record_input(b"echo hello\r\n");
        std::thread::sleep(std::time::Duration::from_millis(10));
        recorder.record_output(b"hello\r\n");

        assert_eq!(recorder.event_count(), 3);
        assert!(recorder.duration() >= 0.02); // At least 20ms
        assert_eq!(recorder.events[0].event_type, "o");
        assert_eq!(recorder.events[1].event_type, "i");
        assert_eq!(recorder.events[2].event_type, "o");
    }

    #[test]
    fn test_save_to_writer() {
        let mut recorder = SessionRecorder::new(Dimensions::new(24, 80));
        recorder.record_output(b"Hello, world!\r\n");

        let mut buffer = Vec::new();
        recorder.save_to_writer(&mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"version\":2"));
        assert!(output.contains("\"width\":80"));
        assert!(output.contains("\"height\":24"));
        assert!(output.contains("Hello, world!"));
    }

    #[test]
    fn test_to_string() {
        let mut recorder = SessionRecorder::new(Dimensions::new(24, 80));
        recorder.record_output(b"test");

        let output = recorder.to_string().unwrap();
        assert!(output.contains("\"version\":2"));
        assert!(output.contains("test"));
    }

    #[test]
    fn test_header_generation() {
        let recorder = SessionRecorder::new(Dimensions::new(30, 100));
        let header = recorder.header();

        assert_eq!(header.version, 2);
        assert_eq!(header.width, 100);
        assert_eq!(header.height, 30);
        assert!(header.timestamp.is_some());
    }

    #[test]
    fn test_with_env() {
        let mut env = HashMap::new();
        env.insert("SHELL".to_string(), "/bin/bash".to_string());
        env.insert("TERM".to_string(), "xterm-256color".to_string());

        let recorder = SessionRecorder::with_env(Dimensions::new(24, 80), env);
        let header = recorder.header();

        assert!(header.env.is_some());
        assert_eq!(header.env.unwrap().get("SHELL").unwrap(), "/bin/bash");
    }

    #[test]
    fn test_roundtrip_string() {
        let mut recorder = SessionRecorder::new(Dimensions::new(24, 80));
        recorder.record_output(b"Hello\r\n");
        recorder.record_input(b"test\r\n");
        recorder.record_output(b"World\r\n");

        let serialized = recorder.to_string().unwrap();
        let loaded = SessionRecorder::from_str(&serialized).unwrap();

        assert_eq!(loaded.event_count(), 3);
        assert_eq!(loaded.dimensions.rows, 24);
        assert_eq!(loaded.dimensions.cols, 80);
        assert_eq!(loaded.events[0].event_type, "o");
        assert_eq!(loaded.events[0].data, "Hello\r\n");
        assert_eq!(loaded.events[1].event_type, "i");
        assert_eq!(loaded.events[2].event_type, "o");
    }

    #[test]
    fn test_load_empty_file() {
        let result = SessionRecorder::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_json() {
        let result = SessionRecorder::from_str("invalid json");
        assert!(result.is_err());
    }
}
