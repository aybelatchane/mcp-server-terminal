//! # terminal-mcp-detector
//!
//! Element detection engine for the Terminal MCP Server.
//!
//! This crate provides:
//! - Detection pipeline for inferring semantic structure
//! - Element detectors (borders, menus, tables, inputs, buttons, etc.)
//! - Terminal State Tree (TST) construction
//! - Priority-based detection ordering
//!
//! ## Architecture
//!
//! This is Layer 4 in the architecture - it depends on terminal-mcp-core
//! and terminal-mcp-emulator to analyze terminal grid and detect elements.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod assembler;
pub mod detection;
pub mod detectors;
pub mod testing;

// Re-export commonly used types
pub use assembler::TSTAssembler;
pub use detection::{
    Confidence, DetectedElement, DetectionContext, DetectionPipeline, ElementDetector,
    RefIdGenerator,
};
pub use detectors::{
    BorderDetector, ButtonDetector, CheckboxDetector, InputDetector, MenuDetector,
    ProgressDetector, StatusBarDetector, TableDetector,
};
