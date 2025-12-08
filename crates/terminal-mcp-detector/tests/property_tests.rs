//! Property-based tests for element detection.
//!
//! Uses proptest to generate random inputs and verify detector invariants.

use proptest::prelude::*;
use std::sync::Arc;

use terminal_mcp_core::{Dimensions, Position};
use terminal_mcp_detector::{
    BorderDetector, ButtonDetector, CheckboxDetector, DetectionContext, DetectionPipeline,
    ElementDetector, InputDetector, MenuDetector, ProgressDetector, StatusBarDetector,
    TableDetector,
};
use terminal_mcp_emulator::{Grid, Parser};

/// Generate a random grid size within reasonable bounds.
fn grid_dimensions() -> impl Strategy<Value = (u16, u16)> {
    (10u16..100, 40u16..200)
}

/// Generate random alphanumeric text for button labels.
fn button_label() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{1,15}".prop_map(|s| s.trim().to_string())
}

/// Generate a random button pattern.
/// Note: Parenthesis patterns removed to avoid false positives with shell prompts (issue #141)
fn button_text() -> impl Strategy<Value = String> {
    prop_oneof![
        button_label().prop_map(|l| format!("[ {} ]", l)),
        button_label().prop_map(|l| format!("[{}]", l)),
        button_label().prop_map(|l| format!("< {} >", l)),
        button_label().prop_map(|l| format!("「{}」", l)), // Japanese corner brackets as alternative
    ]
}

/// Create a grid from text content.
fn create_grid_from_text(rows: u16, cols: u16, text: &str) -> Grid {
    let grid = Grid::new(Dimensions::new(rows, cols));
    let mut parser = Parser::new(grid);
    parser.process(text.as_bytes());
    parser.into_grid()
}

proptest! {
    /// Detectors should never panic on any grid size.
    #[test]
    fn detectors_never_panic_on_any_grid_size((rows, cols) in grid_dimensions()) {
        let grid = Grid::new(Dimensions::new(rows, cols));
        let context = DetectionContext::new(Position::origin());

        // Test all detectors
        let detectors: Vec<Box<dyn ElementDetector>> = vec![
            Box::new(BorderDetector::new()),
            Box::new(MenuDetector::new()),
            Box::new(TableDetector::new()),
            Box::new(InputDetector::new()),
            Box::new(ButtonDetector::new()),
            Box::new(ProgressDetector::new()),
            Box::new(CheckboxDetector::new()),
            Box::new(StatusBarDetector::new()),
        ];

        for detector in &detectors {
            let _ = detector.detect(&grid, &context);
        }
    }

    /// Detection pipeline should never panic on any grid size.
    #[test]
    fn pipeline_never_panics((rows, cols) in grid_dimensions()) {
        let grid = Grid::new(Dimensions::new(rows, cols));

        let mut pipeline = DetectionPipeline::new();
        pipeline.add_detector(Arc::new(BorderDetector::new()));
        pipeline.add_detector(Arc::new(MenuDetector::new()));
        pipeline.add_detector(Arc::new(TableDetector::new()));
        pipeline.add_detector(Arc::new(InputDetector::new()));
        pipeline.add_detector(Arc::new(ButtonDetector::new()));
        pipeline.add_detector(Arc::new(ProgressDetector::new()));
        pipeline.add_detector(Arc::new(CheckboxDetector::new()));
        pipeline.add_detector(Arc::new(StatusBarDetector::new()));

        let cursor = Position::origin();
        let _ = pipeline.detect(&grid, cursor);
    }

    /// Detectors should never panic with any cursor position.
    #[test]
    fn detectors_handle_any_cursor_position(
        (rows, cols) in grid_dimensions(),
        cursor_row in 0u16..200,
        cursor_col in 0u16..400
    ) {
        let grid = Grid::new(Dimensions::new(rows, cols));

        // Cursor might be out of bounds - detectors should handle this
        let context = DetectionContext::new(Position::new(cursor_row, cursor_col));

        let detector = ButtonDetector::new();
        let _ = detector.detect(&grid, &context);
    }

    /// Button detector should find valid button patterns.
    #[test]
    fn button_detector_finds_valid_buttons(button_text in button_text()) {
        let grid = create_grid_from_text(10, 80, &format!("{}\r\n", button_text));
        let context = DetectionContext::new(Position::origin());

        let detector = ButtonDetector::new();
        let detected = detector.detect(&grid, &context);

        // Should find at least one button (unless label was empty after trimming)
        // Empty labels are intentionally rejected
        let label_text = button_text
            .trim_start_matches(&['[', '<', '「', ' '][..])
            .trim_end_matches(&[']', '>', '」', ' '][..])
            .trim();

        if !label_text.is_empty() {
            prop_assert!(!detected.is_empty(), "Button should be detected for: {}", button_text);
        }
    }

    /// Detected elements should have valid bounds within grid.
    #[test]
    fn detected_elements_have_valid_bounds((rows, cols) in grid_dimensions()) {
        let text = "[ OK ] [ Cancel ]\r\n[X] Checkbox\r\nProgress: ████████░░░░ 66%\r\n";
        let grid = create_grid_from_text(rows, cols, text);

        let mut pipeline = DetectionPipeline::new();
        pipeline.add_detector(Arc::new(ButtonDetector::new()));
        pipeline.add_detector(Arc::new(CheckboxDetector::new()));
        pipeline.add_detector(Arc::new(ProgressDetector::new()));

        let elements = pipeline.detect(&grid, Position::origin());

        for elem in &elements {
            let bounds = &elem.bounds;
            // Bounds should be within grid
            prop_assert!(bounds.row < rows, "Row {} >= grid rows {}", bounds.row, rows);
            prop_assert!(bounds.col < cols, "Col {} >= grid cols {}", bounds.col, cols);
            prop_assert!(bounds.row + bounds.height <= rows,
                "Row {} + height {} > rows {}", bounds.row, bounds.height, rows);
            prop_assert!(bounds.col + bounds.width <= cols,
                "Col {} + width {} > cols {}", bounds.col, bounds.width, cols);
        }
    }

    /// Checkbox detector should handle random checkbox patterns.
    #[test]
    fn checkbox_detector_finds_checkboxes(
        checked in prop::bool::ANY,
        label in "[A-Za-z ]{1,20}"
    ) {
        let marker = if checked { "X" } else { " " };
        let text = format!("[{}] {}\r\n", marker, label);
        let grid = create_grid_from_text(10, 80, &text);
        let context = DetectionContext::new(Position::origin());

        let detector = CheckboxDetector::new();
        let detected = detector.detect(&grid, &context);

        // Should find the checkbox (label must not be empty)
        let trimmed_label = label.trim();
        if !trimmed_label.is_empty() {
            prop_assert!(!detected.is_empty(), "Checkbox should be detected");
        }
    }

    /// Progress detector should handle various percentage values.
    #[test]
    fn progress_detector_handles_percentages(percent in 0u8..=100) {
        let filled = (percent as usize * 20) / 100;
        let empty = 20 - filled;
        let bar = format!(
            "[{}{}] {}%\r\n",
            "█".repeat(filled),
            "░".repeat(empty),
            percent
        );
        let grid = create_grid_from_text(10, 80, &bar);
        let context = DetectionContext::new(Position::origin());

        let detector = ProgressDetector::new();
        let _ = detector.detect(&grid, &context);
        // Just ensure no panic - detection accuracy is tested elsewhere
    }

    /// Safe_slice should never panic on any input.
    #[test]
    fn safe_slice_helper_never_panics(
        s in ".*",
        start in 0usize..1000,
        end in 0usize..1000
    ) {
        // Simulate the safe_slice function
        fn safe_slice(s: &str, start: usize, end: usize) -> Option<&str> {
            if start <= end && end <= s.len()
                && s.is_char_boundary(start)
                && s.is_char_boundary(end)
            {
                Some(&s[start..end])
            } else {
                None
            }
        }

        // Should never panic
        let _ = safe_slice(&s, start, end);
    }

    /// Unicode strings should be handled safely.
    #[test]
    fn detectors_handle_unicode_safely(text in "[^\x00-\x1f]{0,100}") {
        let grid = create_grid_from_text(24, 80, &format!("{}\r\n", text));
        let context = DetectionContext::new(Position::origin());

        let detector = ButtonDetector::new();
        let _ = detector.detect(&grid, &context);

        let detector = CheckboxDetector::new();
        let _ = detector.detect(&grid, &context);

        let detector = ProgressDetector::new();
        let _ = detector.detect(&grid, &context);
    }

    /// Grid text extraction should never panic.
    #[test]
    fn grid_text_extraction_never_panics((rows, cols) in grid_dimensions()) {
        let grid = Grid::new(Dimensions::new(rows, cols));

        // Full grid extraction
        let bounds = terminal_mcp_core::Bounds::new(0, 0, cols, rows);
        let _ = grid.extract_text(&bounds);

        // Random region extraction (may be out of bounds)
        let bounds = terminal_mcp_core::Bounds::new(
            rows / 2,
            cols / 2,
            cols,  // Intentionally larger than remaining
            rows,  // Intentionally larger than remaining
        );
        let _ = grid.extract_text(&bounds);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        let grid = Grid::new(Dimensions::new(1, 1));
        let context = DetectionContext::new(Position::origin());

        let detector = ButtonDetector::new();
        let detected = detector.detect(&grid, &context);
        assert!(detected.is_empty());
    }

    #[test]
    fn test_minimal_grid() {
        let grid = Grid::new(Dimensions::new(5, 10));

        let mut pipeline = DetectionPipeline::new();
        pipeline.add_detector(Arc::new(BorderDetector::new()));
        pipeline.add_detector(Arc::new(ButtonDetector::new()));

        let elements = pipeline.detect(&grid, Position::origin());
        // Empty grid should have no elements
        assert!(elements.is_empty());
    }

    #[test]
    fn test_box_drawing_characters() {
        // Test with various box-drawing Unicode characters
        let text = "┌────────────────────┐\r\n│ Test Content       │\r\n└────────────────────┘\r\n";
        let grid = create_grid_from_text(24, 80, text);

        let mut pipeline = DetectionPipeline::new();
        pipeline.add_detector(Arc::new(BorderDetector::new()));
        pipeline.add_detector(Arc::new(ButtonDetector::new()));

        // Should not panic
        let _ = pipeline.detect(&grid, Position::origin());
    }
}
