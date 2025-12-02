//! Integration tests for the terminal-mcp system.

use std::sync::Arc;
use terminal_mcp_core::{Dimensions, Element};
use terminal_mcp_detector::{BorderDetector, DetectionPipeline, StatusBarDetector};
use terminal_mcp_emulator::{Grid, Parser};

#[test]
fn test_full_detection_pipeline() {
    // Create a grid with a border and status bar
    let text = concat!(
        "┌─────────────────┐\r\n",
        "│ Main Content    │\r\n",
        "│ More text here  │\r\n",
        "└─────────────────┘\r\n",
        "Press q to quit | F1 Help\r\n"
    );

    // Grid should match the actual content: 5 rows (border rows 0-3, status bar row 4)
    let grid = Grid::new(Dimensions::new(5, 40));
    let mut parser = Parser::new(grid);
    parser.process(text.as_bytes());
    let grid = parser.into_grid();

    // Create detection pipeline with both detectors
    let mut pipeline = DetectionPipeline::new();
    pipeline.add_detector(Arc::new(BorderDetector::new()));
    pipeline.add_detector(Arc::new(StatusBarDetector::new()));

    // Run detection
    let cursor = terminal_mcp_core::Position::new(0, 0);
    let detected = pipeline.detect(&grid, cursor);

    println!("Detected {} elements:", detected.len());
    for (i, elem) in detected.iter().enumerate() {
        match &elem.element {
            Element::Border {
                ref_id,
                bounds,
                title,
                ..
            } => {
                println!(
                    "  {}: Border at ({}, {}) {}x{} - ref_id: {}, title: {:?}",
                    i, bounds.row, bounds.col, bounds.width, bounds.height, ref_id, title
                );
            }
            Element::StatusBar {
                ref_id,
                bounds,
                content,
            } => {
                println!(
                    "  {}: StatusBar at ({}, {}) - ref_id: {}, content: '{}'",
                    i, bounds.row, bounds.col, ref_id, content
                );
            }
            _ => {
                println!("  {}: Other element: {:?}", i, elem.element);
            }
        }
    }

    // Verify we detected both elements
    assert!(
        detected.len() >= 2,
        "Expected at least 2 elements (border + status bar), got {}",
        detected.len()
    );

    // Verify we have a border
    let has_border = detected
        .iter()
        .any(|d| matches!(d.element, Element::Border { .. }));
    assert!(has_border, "Should detect a border");

    // Verify we have a status bar
    let has_status_bar = detected
        .iter()
        .any(|d| matches!(d.element, Element::StatusBar { .. }));
    assert!(has_status_bar, "Should detect a status bar");

    println!("\n✓ Integration test passed!");
}
