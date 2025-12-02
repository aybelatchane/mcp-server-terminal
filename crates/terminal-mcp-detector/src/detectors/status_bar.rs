//! Status bar detector for terminal UI status/help bars.

use terminal_mcp_core::{Bounds, Color, Element};
use terminal_mcp_emulator::Grid;

use crate::detection::{Confidence, DetectedElement, DetectionContext, ElementDetector};

/// Status bar detector for status/help bars (typically at bottom).
pub struct StatusBarDetector;

impl StatusBarDetector {
    /// Create a new status bar detector.
    pub fn new() -> Self {
        Self
    }

    /// Extract text from a specific row.
    fn extract_row_text(&self, grid: &Grid, row: u16) -> String {
        let dims = grid.dimensions();
        let mut text = String::new();

        for col in 0..dims.cols {
            if let Some(cell) = grid.cell(row, col) {
                text.push(cell.character);
            }
        }

        text
    }

    /// Check if the text looks like a status bar based on common patterns.
    fn looks_like_status_bar(&self, text: &str, grid: &Grid, row: u16) -> bool {
        let text = text.trim();

        if text.is_empty() {
            return false;
        }

        // Check for common status bar patterns
        let patterns = [
            "Press",
            "press",
            "ESC",
            "Esc",
            "q to quit",
            "Q to quit",
            "Help:",
            "Status:",
            "│",
            "|", // Separators
            "Ctrl+",
            "Alt+",
            "F1",
            "F2",
            "F3",
            "F4",
            "F5",
            "F6",
            "F7",
            "F8",
            "F9",
            "F10",
        ];

        let has_pattern = patterns.iter().any(|p| text.contains(p));

        // Check for reverse video or distinct background
        let has_distinct_bg = self.row_has_distinct_background(grid, row);

        has_pattern || has_distinct_bg
    }

    /// Check if the entire row has a distinct background color.
    fn row_has_distinct_background(&self, grid: &Grid, row: u16) -> bool {
        let dims = grid.dimensions();

        // Collect background colors from the row
        let mut bg_colors = Vec::new();
        for col in 0..dims.cols {
            if let Some(cell) = grid.cell(row, col) {
                // Skip empty cells
                if cell.character != ' ' {
                    bg_colors.push(cell.bg);
                }
            }
        }

        if bg_colors.is_empty() {
            return false;
        }

        // Check if most cells have non-default background
        let non_default_count = bg_colors.iter().filter(|&&bg| bg != Color::Default).count();

        // If more than 50% of non-empty cells have a non-default background, consider it distinct
        non_default_count > bg_colors.len() / 2
    }
}

impl Default for StatusBarDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementDetector for StatusBarDetector {
    fn name(&self) -> &'static str {
        "status_bar"
    }

    fn priority(&self) -> u32 {
        50
    }

    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
        let mut results = Vec::new();
        let dims = grid.dimensions();

        // Check last row
        let last_row = dims.rows.saturating_sub(1);

        // Skip if region already claimed
        let bounds = Bounds::new(last_row, 0, dims.cols, 1);
        if context.is_region_claimed(&bounds) {
            return results;
        }

        // Extract text from last row
        let text = self.extract_row_text(grid, last_row);

        // Check if it looks like a status bar
        if self.looks_like_status_bar(&text, grid, last_row) {
            let ref_id = format!("status_bar_{last_row}");

            results.push(DetectedElement {
                element: Element::StatusBar {
                    ref_id,
                    bounds,
                    content: text.trim().to_string(),
                },
                bounds,
                confidence: Confidence::Medium,
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_mcp_core::Dimensions;
    use terminal_mcp_emulator::{Grid, Parser};

    fn create_grid_with_text(rows: u16, cols: u16, text: &str) -> Grid {
        let grid = Grid::new(Dimensions::new(rows, cols));
        let mut parser = Parser::new(grid);
        parser.process(text.as_bytes());
        parser.into_grid()
    }

    #[test]
    fn test_status_bar_detector_with_pattern() {
        // Create a grid with status bar at bottom
        let text = "Line 1\r\nLine 2\r\nPress q to quit | F1 Help";
        let grid = create_grid_with_text(3, 40, text);

        let detector = StatusBarDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].bounds.row, 2);
        assert_eq!(detected[0].bounds.height, 1);
        assert_eq!(detected[0].confidence, Confidence::Medium);

        if let Element::StatusBar { content, .. } = &detected[0].element {
            assert!(content.contains("Press q to quit"));
        } else {
            panic!("Expected StatusBar element");
        }
    }

    #[test]
    fn test_status_bar_detector_with_esc() {
        let text = "Content here\r\nESC: Exit | Ctrl+S: Save";
        let grid = create_grid_with_text(2, 40, text);

        let detector = StatusBarDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::StatusBar { content, .. } = &detected[0].element {
            assert!(content.contains("ESC"));
            assert!(content.contains("Ctrl+S"));
        }
    }

    #[test]
    fn test_status_bar_detector_empty_last_row() {
        let text = "Line 1\r\nLine 2\r\n";
        let grid = create_grid_with_text(3, 40, text);

        let detector = StatusBarDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        // Should not detect empty row as status bar
        assert_eq!(detected.len(), 0);
    }

    #[test]
    fn test_status_bar_detector_no_pattern() {
        let text = "Line 1\r\nLine 2\r\nJust some regular text here";
        let grid = create_grid_with_text(3, 40, text);

        let detector = StatusBarDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        // Should not detect without pattern match or distinct background
        assert_eq!(detected.len(), 0);
    }

    #[test]
    fn test_status_bar_detector_with_distinct_background() {
        // Create grid and manually set background colors
        let mut grid = Grid::new(Dimensions::new(3, 40));

        // Fill last row with cells that have non-default background
        let last_row = 2;
        let status_text = "Status";
        for col in 0..20 {
            if let Some(cell) = grid.cell_mut(last_row, col) {
                cell.character = if col < 6 {
                    status_text.chars().nth(col as usize % 6).unwrap_or(' ')
                } else {
                    ' '
                };
                cell.bg = Color::Blue; // Non-default background
            }
        }

        let detector = StatusBarDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        // Should detect based on distinct background
        assert_eq!(detected.len(), 1);
    }

    #[test]
    fn test_status_bar_detector_priority() {
        let detector = StatusBarDetector::new();
        assert_eq!(detector.priority(), 50);
        assert_eq!(detector.name(), "status_bar");
    }

    #[test]
    fn test_status_bar_detector_region_claimed() {
        let text = "Line 1\r\nPress q to quit";
        let grid = create_grid_with_text(2, 40, text);

        // Create context with claimed region
        let mut context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let last_row = grid.dimensions().rows - 1;
        context.claim_region(Bounds::new(last_row, 0, grid.dimensions().cols, 1));

        let detector = StatusBarDetector::new();
        let detected = detector.detect(&grid, &context);

        // Should not detect because region is claimed
        assert_eq!(detected.len(), 0);
    }
}
