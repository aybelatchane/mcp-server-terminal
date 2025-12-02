//! Testing utilities for Terminal MCP detection.
//!
//! Provides snapshot comparison, golden file management, and regression testing tools.

pub mod snapshot_compare;

pub use snapshot_compare::{ElementChange, ElementDiff, SnapshotDiff, SnapshotMatcher};
