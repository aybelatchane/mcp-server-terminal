//! Input detector for text input fields with cursor tracking.

use terminal_mcp_core::{Bounds, Element};
use terminal_mcp_emulator::Grid;

use crate::detection::{Confidence, DetectedElement, DetectionContext, ElementDetector};

/// Input detector for text input fields.
pub struct InputDetector {
    /// Minimum width for input field
    min_width: u16,
}

impl InputDetector {
    /// Safely get a substring using byte indices, returning None if indices are invalid.
    fn safe_slice(s: &str, start: usize, end: usize) -> Option<&str> {
        if start <= end && end <= s.len() && s.is_char_boundary(start) && s.is_char_boundary(end) {
            Some(&s[start..end])
        } else {
            None
        }
    }

    /// Create a new input detector.
    pub fn new() -> Self {
        Self { min_width: 3 }
    }

    /// Extract text from a specific row within a region.
    fn extract_row_text(&self, grid: &Grid, row: u16, start_col: u16, width: u16) -> String {
        let mut text = String::new();
        for col_offset in 0..width {
            let col = start_col + col_offset;
            if let Some(cell) = grid.cell(row, col) {
                text.push(cell.character);
            }
        }
        text
    }

    /// Check if a row looks like a labeled input field.
    /// Patterns: "Username: _____", "Enter name: ", "Password: ******"
    fn looks_like_labeled_input(&self, text: &str) -> Option<(usize, usize)> {
        // Look for label followed by colon
        if let Some(colon_pos) = text.find(':') {
            let after_colon = Self::safe_slice(text, colon_pos + 1, text.len())?;

            // Check if there's content after the colon (spaces, underscores, or text)
            let trimmed = after_colon.trim_start();
            if !trimmed.is_empty() || after_colon.len() > 1 {
                // Value starts after colon and any leading spaces
                let value_start = colon_pos + 1 + (after_colon.len() - trimmed.len());
                return Some((value_start, text.len()));
            }
        }
        None
    }

    /// Check if a row looks like a bracketed input field.
    /// Patterns: "[          ]", "(____)", "│   text   │"
    fn looks_like_bracketed_input(&self, text: &str) -> Option<(usize, usize)> {
        let trimmed = text.trim();

        // Check for bracket pairs
        let brackets = [('[', ']'), ('(', ')'), ('{', '}'), ('│', '│')];

        for (open, close) in &brackets {
            if trimmed.starts_with(*open) && trimmed.ends_with(*close) {
                let start = text.find(*open).unwrap() + 1;
                let end = text.rfind(*close).unwrap();
                if end > start && (end - start) >= self.min_width as usize {
                    return Some((start, end));
                }
            }
        }

        None
    }

    /// Detect input field at cursor position.
    fn detect_input_at_cursor(
        &self,
        grid: &Grid,
        cursor_row: u16,
        cursor_col: u16,
    ) -> Option<DetectedElement> {
        let dims = grid.dimensions();

        // Extract the row containing the cursor
        let row_text = self.extract_row_text(grid, cursor_row, 0, dims.cols);

        // Try labeled input detection
        if let Some((value_start, value_end)) = self.looks_like_labeled_input(&row_text) {
            let value_slice = Self::safe_slice(&row_text, value_start, value_end)?;
            let value = value_slice.trim_end().to_string();
            let cursor_pos = if cursor_col as usize >= value_start {
                (cursor_col as usize - value_start).min(value.len())
            } else {
                0
            };

            let ref_id = format!("input_{cursor_row}_{value_start}");

            return Some(DetectedElement {
                element: Element::Input {
                    ref_id,
                    bounds: Bounds::new(
                        cursor_row,
                        value_start as u16,
                        (value_end - value_start) as u16,
                        1,
                    ),
                    value,
                    cursor_pos,
                },
                bounds: Bounds::new(
                    cursor_row,
                    value_start as u16,
                    (value_end - value_start) as u16,
                    1,
                ),
                confidence: Confidence::High,
            });
        }

        // Try bracketed input detection
        if let Some((value_start, value_end)) = self.looks_like_bracketed_input(&row_text) {
            let value_slice = Self::safe_slice(&row_text, value_start, value_end)?;
            let value = value_slice.trim().to_string();
            let cursor_pos =
                if (cursor_col as usize) >= value_start && (cursor_col as usize) < value_end {
                    (cursor_col as usize - value_start).min(value.len())
                } else {
                    0
                };

            let ref_id = format!("input_{cursor_row}_{value_start}");

            return Some(DetectedElement {
                element: Element::Input {
                    ref_id,
                    bounds: Bounds::new(
                        cursor_row,
                        value_start as u16,
                        (value_end - value_start) as u16,
                        1,
                    ),
                    value,
                    cursor_pos,
                },
                bounds: Bounds::new(
                    cursor_row,
                    value_start as u16,
                    (value_end - value_start) as u16,
                    1,
                ),
                confidence: Confidence::Medium,
            });
        }

        None
    }

    /// Detect input fields with reverse video (focused input).
    fn detect_reverse_video_input(&self, grid: &Grid, cursor_row: u16) -> Option<DetectedElement> {
        let dims = grid.dimensions();

        // Find consecutive cells with reverse video on this row
        let mut start_col = None;
        let mut end_col = 0;
        let mut value = String::new();

        for col in 0..dims.cols {
            if let Some(cell) = grid.cell(cursor_row, col) {
                if cell.attrs.reverse && cell.character != ' ' {
                    if start_col.is_none() {
                        start_col = Some(col);
                    }
                    end_col = col + 1;
                    value.push(cell.character);
                }
            }
        }

        if let Some(start) = start_col {
            let width = end_col - start;
            if width >= self.min_width {
                let ref_id = format!("input_{cursor_row}_{start}");
                let cursor_pos = value.len(); // Cursor typically at end for reverse video

                return Some(DetectedElement {
                    element: Element::Input {
                        ref_id,
                        bounds: Bounds::new(cursor_row, start, width, 1),
                        value: value.trim().to_string(),
                        cursor_pos,
                    },
                    bounds: Bounds::new(cursor_row, start, width, 1),
                    confidence: Confidence::High,
                });
            }
        }

        None
    }
}

impl Default for InputDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementDetector for InputDetector {
    fn name(&self) -> &'static str {
        "input"
    }

    fn priority(&self) -> u32 {
        70
    }

    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
        let mut results = Vec::new();

        let cursor_row = context.cursor.row;
        let cursor_col = context.cursor.col;

        // Strategy 1: Try to detect input at cursor position (highest confidence)
        if let Some(input) = self.detect_input_at_cursor(grid, cursor_row, cursor_col) {
            // Check if region is already claimed
            if !context.is_region_claimed(&input.bounds) {
                results.push(input);
                return results;
            }
        }

        // Strategy 2: Try reverse video detection (for focused inputs)
        if let Some(input) = self.detect_reverse_video_input(grid, cursor_row) {
            if !context.is_region_claimed(&input.bounds) {
                results.push(input);
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_mcp_core::{Dimensions, Position};
    use terminal_mcp_emulator::{Grid, Parser};

    fn create_grid_with_text(rows: u16, cols: u16, text: &str) -> Grid {
        let grid = Grid::new(Dimensions::new(rows, cols));
        let mut parser = Parser::new(grid);
        parser.process(text.as_bytes());
        parser.into_grid()
    }

    #[test]
    fn test_input_detector_labeled_field() {
        let text = "Username: john\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = InputDetector::new();
        // Cursor at position 14 (in "john")
        let context = DetectionContext::new(Position::new(0, 14));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Input {
            value, cursor_pos, ..
        } = &detected[0].element
        {
            assert_eq!(value, "john");
            assert_eq!(*cursor_pos, 4); // Cursor at end of "john"
        } else {
            panic!("Expected Input element");
        }
    }

    #[test]
    fn test_input_detector_empty_labeled_field() {
        let text = "Password: \r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = InputDetector::new();
        // Cursor at position 10 (after colon and space)
        let context = DetectionContext::new(Position::new(0, 10));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Input {
            value, cursor_pos, ..
        } = &detected[0].element
        {
            assert_eq!(value, "");
            assert_eq!(*cursor_pos, 0);
        }
    }

    #[test]
    fn test_input_detector_bracketed_field() {
        let text = "[  hello  ]\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = InputDetector::new();
        // Cursor at position 5 (in "hello")
        let context = DetectionContext::new(Position::new(0, 5));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Input { value, .. } = &detected[0].element {
            assert_eq!(value, "hello");
        }
    }

    #[test]
    fn test_input_detector_priority() {
        let detector = InputDetector::new();
        assert_eq!(detector.priority(), 70);
        assert_eq!(detector.name(), "input");
    }

    #[test]
    fn test_input_detector_cursor_position() {
        let text = "Email: user@example.com\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = InputDetector::new();
        // Cursor at position 12 (in middle of email)
        let context = DetectionContext::new(Position::new(0, 12));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Input {
            value, cursor_pos, ..
        } = &detected[0].element
        {
            assert_eq!(value, "user@example.com");
            assert_eq!(*cursor_pos, 5); // Cursor at 'e' in user@example.com
        }
    }
}
