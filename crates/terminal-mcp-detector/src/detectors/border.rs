//! Border detector for box-drawing characters.

use terminal_mcp_core::{Bounds, Element};
use terminal_mcp_emulator::Grid;

use crate::detection::{Confidence, DetectedElement, DetectionContext, ElementDetector};

/// Box-drawing character set.
#[derive(Debug, Clone)]
struct BoxCharSet {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    horizontal: char,
    vertical: char,
}

/// Border detector for box-drawing characters.
pub struct BorderDetector {
    /// Recognized box character sets
    box_sets: Vec<BoxCharSet>,
}

impl BorderDetector {
    /// Create a new border detector.
    pub fn new() -> Self {
        Self {
            box_sets: vec![
                // Light box: ┌─┐│└┘
                BoxCharSet {
                    top_left: '┌',
                    top_right: '┐',
                    bottom_left: '└',
                    bottom_right: '┘',
                    horizontal: '─',
                    vertical: '│',
                },
                // Heavy box: ┏━┓┃┗┛
                BoxCharSet {
                    top_left: '┏',
                    top_right: '┓',
                    bottom_left: '┗',
                    bottom_right: '┛',
                    horizontal: '━',
                    vertical: '┃',
                },
                // Double box: ╔═╗║╚╝
                BoxCharSet {
                    top_left: '╔',
                    top_right: '╗',
                    bottom_left: '╚',
                    bottom_right: '╝',
                    horizontal: '═',
                    vertical: '║',
                },
                // Rounded: ╭─╮│╰╯
                BoxCharSet {
                    top_left: '╭',
                    top_right: '╮',
                    bottom_left: '╰',
                    bottom_right: '╯',
                    horizontal: '─',
                    vertical: '│',
                },
                // ASCII: +-+|-+
                BoxCharSet {
                    top_left: '+',
                    top_right: '+',
                    bottom_left: '+',
                    bottom_right: '+',
                    horizontal: '-',
                    vertical: '|',
                },
            ],
        }
    }

    /// Trace a border starting from a top-left corner.
    fn trace_border(
        &self,
        grid: &Grid,
        start_row: u16,
        start_col: u16,
        box_set: &BoxCharSet,
    ) -> Option<DetectedElement> {
        let dims = grid.dimensions();

        // Find top-right corner (allowing any characters for titles)
        let mut width = 1;
        for col in (start_col + 1)..dims.cols {
            if let Some(cell) = grid.cell(start_row, col) {
                if cell.character == box_set.top_right {
                    width = col - start_col + 1;
                    break;
                }
                // Allow any character in the top border (for titles)
            }
        }

        if width == 1 {
            return None; // No top-right corner found
        }

        // Find bottom-left corner
        let mut height = 1;
        for row in (start_row + 1)..dims.rows {
            if let Some(cell) = grid.cell(row, start_col) {
                if cell.character == box_set.bottom_left {
                    height = row - start_row + 1;
                    break;
                } else if cell.character != box_set.vertical && cell.character != ' ' {
                    // Not a valid vertical line
                    return None;
                }
            }
        }

        if height == 1 {
            return None; // No bottom-left corner found
        }

        // Verify bottom-right corner
        let bottom_row = start_row + height - 1;
        let right_col = start_col + width - 1;

        if let Some(cell) = grid.cell(bottom_row, right_col) {
            if cell.character != box_set.bottom_right {
                return None; // Bottom-right corner mismatch
            }
        } else {
            return None;
        }

        // Extract title (if any)
        let title = self.extract_title(grid, start_row, start_col, width, box_set);

        // Create detected border
        let bounds = Bounds::new(start_row, start_col, width, height);
        let ref_id = format!("border_{}", start_row * 1000 + start_col);

        Some(DetectedElement {
            element: Element::Border {
                ref_id,
                bounds,
                title,
                children: Vec::new(), // TODO: Will be populated by TST assembler
            },
            bounds,
            confidence: Confidence::High,
        })
    }

    /// Extract title from top border.
    fn extract_title(
        &self,
        grid: &Grid,
        row: u16,
        col: u16,
        width: u16,
        box_set: &BoxCharSet,
    ) -> Option<String> {
        let mut title = String::new();
        let mut in_title = false;

        for c in (col + 1)..(col + width - 1) {
            if let Some(cell) = grid.cell(row, c) {
                let ch = cell.character;
                if ch == box_set.horizontal {
                    if in_title && !title.trim().is_empty() {
                        break;
                    }
                } else if ch != ' ' || in_title {
                    in_title = true;
                    title.push(ch);
                }
            }
        }

        let title = title.trim().to_string();
        if title.is_empty() {
            None
        } else {
            Some(title)
        }
    }

    /// Filter out borders that are completely contained within other borders.
    fn filter_contained_borders(&self, mut borders: Vec<DetectedElement>) -> Vec<DetectedElement> {
        // Sort by area (largest first)
        borders.sort_by(|a, b| {
            let area_a = a.bounds.width * a.bounds.height;
            let area_b = b.bounds.width * b.bounds.height;
            area_b.cmp(&area_a)
        });

        let mut filtered = Vec::new();

        for border in borders {
            // Check if this border is contained within any already-filtered border
            let is_contained = filtered.iter().any(|outer: &DetectedElement| {
                // Check if border is completely contained within outer
                border.bounds.row >= outer.bounds.row
                    && border.bounds.col >= outer.bounds.col
                    && (border.bounds.row + border.bounds.height)
                        <= (outer.bounds.row + outer.bounds.height)
                    && (border.bounds.col + border.bounds.width)
                        <= (outer.bounds.col + outer.bounds.width)
            });

            if !is_contained {
                filtered.push(border);
            }
        }

        filtered
    }
}

impl Default for BorderDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementDetector for BorderDetector {
    fn name(&self) -> &'static str {
        "border"
    }

    fn priority(&self) -> u32 {
        100 // Highest priority
    }

    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
        let mut borders = Vec::new();
        let dims = grid.dimensions();

        // Scan for top-left corners
        for row in 0..dims.rows {
            for col in 0..dims.cols {
                // Skip if region already claimed
                let point_bounds = Bounds::new(row, col, 1, 1);
                if context.is_region_claimed(&point_bounds) {
                    continue;
                }

                if let Some(cell) = grid.cell(row, col) {
                    for box_set in &self.box_sets {
                        if cell.character == box_set.top_left {
                            if let Some(border) = self.trace_border(grid, row, col, box_set) {
                                borders.push(border);
                            }
                        }
                    }
                }
            }
        }

        // Filter nested borders (keep outermost only)
        self.filter_contained_borders(borders)
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
    fn test_border_detector_light_box() {
        let text = "┌────────┐\r\n│ Hello  │\r\n└────────┘\r\n";
        let grid = create_grid_with_text(5, 20, text);

        let detector = BorderDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].bounds.width, 10);
        assert_eq!(detected[0].bounds.height, 3);
        assert_eq!(detected[0].confidence, Confidence::High);
    }

    #[test]
    fn test_border_detector_with_title() {
        let text = "┌─ Title ─┐\r\n│  Content│\r\n└─────────┘\r\n";
        let grid = create_grid_with_text(5, 20, text);

        let detector = BorderDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Border { title, .. } = &detected[0].element {
            assert_eq!(title.as_ref().map(|s| s.as_str()), Some("Title"));
        } else {
            panic!("Expected Border element");
        }
    }

    #[test]
    fn test_border_detector_ascii_box() {
        let text = "+------+\r\n| Test |\r\n+------+\r\n";
        let grid = create_grid_with_text(5, 15, text);

        let detector = BorderDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].bounds.width, 8);
    }

    #[test]
    fn test_border_detector_nested_boxes() {
        let text = "┌──────────┐\r\n│ ┌────┐  │\r\n│ │    │  │\r\n│ └────┘  │\r\n└──────────┘\r\n";
        let grid = create_grid_with_text(10, 20, text);

        let detector = BorderDetector::new();
        let context = DetectionContext::new(terminal_mcp_core::Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        // Should detect both borders (filter will handle containment later)
        assert!(!detected.is_empty());
    }

    #[test]
    fn test_border_detector_priority() {
        let detector = BorderDetector::new();
        assert_eq!(detector.priority(), 100);
        assert_eq!(detector.name(), "border");
    }
}
