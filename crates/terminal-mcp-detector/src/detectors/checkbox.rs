//! Checkbox detector for checkbox and radio button UI elements.

use terminal_mcp_core::{Bounds, Element};
use terminal_mcp_emulator::Grid;

use crate::detection::{Confidence, DetectedElement, DetectionContext, ElementDetector};

/// Checkbox pattern with marker and brackets.
#[derive(Debug, Clone)]
struct CheckboxPattern {
    open: char,
    close: char,
    checked_markers: Vec<char>,
    unchecked_marker: char,
}

/// Checkbox detector for checkboxes and radio buttons.
pub struct CheckboxDetector {
    patterns: Vec<CheckboxPattern>,
    max_label_length: usize,
}

impl CheckboxDetector {
    /// Safely get a substring using byte indices, returning None if indices are invalid.
    fn safe_slice(s: &str, start: usize, end: usize) -> Option<&str> {
        if start <= end && end <= s.len() && s.is_char_boundary(start) && s.is_char_boundary(end) {
            Some(&s[start..end])
        } else {
            None
        }
    }

    /// Create a new checkbox detector.
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // Checkbox patterns: [x], [ ], [X], [*]
                CheckboxPattern {
                    open: '[',
                    close: ']',
                    checked_markers: vec!['x', 'X', '*', '✓', '✔'],
                    unchecked_marker: ' ',
                },
                // Radio button patterns: (*), ( ), (o), (O)
                CheckboxPattern {
                    open: '(',
                    close: ')',
                    checked_markers: vec!['*', 'o', 'O', '●', '◉'],
                    unchecked_marker: ' ',
                },
            ],
            max_label_length: 60,
        }
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

    /// Extract label following a checkbox/radio button.
    fn extract_label(&self, text: &str, start_pos: usize) -> String {
        // Skip any leading whitespace after the checkbox
        let Some(after_checkbox) = Self::safe_slice(text, start_pos, text.len()) else {
            return String::new();
        };
        let trimmed = after_checkbox.trim_start();
        let label_start = start_pos + (after_checkbox.len() - trimmed.len());

        // Extract text until end of line or max length
        let label_end = (label_start + self.max_label_length).min(text.len());
        let Some(label_slice) = Self::safe_slice(text, label_start, label_end) else {
            return String::new();
        };
        let mut label = label_slice.trim_end().to_string();

        // Truncate at common delimiters that might indicate another checkbox or element
        if let Some(pos) = label.find(['[', '(', '\n', '\r']) {
            label.truncate(pos);
            label = label.trim_end().to_string();
        }

        label
    }

    /// Detect checkboxes in a specific row.
    fn detect_checkboxes_in_row(&self, grid: &Grid, row: u16) -> Vec<DetectedElement> {
        let mut checkboxes = Vec::new();
        let row_text = self.extract_row_text(grid, row);

        for pattern in &self.patterns {
            let chars: Vec<char> = row_text.chars().collect();

            for (i, &ch) in chars.iter().enumerate() {
                if ch == pattern.open && i + 2 < chars.len() && chars[i + 2] == pattern.close {
                    let marker = chars[i + 1];
                    let checked = pattern.checked_markers.contains(&marker);
                    let is_valid_checkbox = checked || marker == pattern.unchecked_marker;

                    if is_valid_checkbox {
                        // Extract label
                        let label_start = i + 3; // After closing bracket
                        let label = self.extract_label(&row_text, label_start);

                        let ref_id = format!("checkbox_{row}_{i}");

                        // Calculate bounds (checkbox + label)
                        let checkbox_width = 3; // e.g., "[x]" or "(*)"
                        let total_width = if label.is_empty() {
                            checkbox_width
                        } else {
                            checkbox_width + 1 + label.len() as u16 // checkbox + space + label
                        };

                        checkboxes.push(DetectedElement {
                            element: Element::Checkbox {
                                ref_id,
                                bounds: Bounds::new(row, i as u16, total_width, 1),
                                label,
                                checked,
                            },
                            bounds: Bounds::new(row, i as u16, total_width, 1),
                            confidence: Confidence::High,
                        });
                    }
                }
            }
        }

        checkboxes
    }
}

impl Default for CheckboxDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementDetector for CheckboxDetector {
    fn name(&self) -> &'static str {
        "checkbox"
    }

    fn priority(&self) -> u32 {
        60
    }

    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
        let mut results = Vec::new();
        let dims = grid.dimensions();

        for row in 0..dims.rows {
            let checkboxes = self.detect_checkboxes_in_row(grid, row);
            for checkbox in checkboxes {
                // Skip if region already claimed
                if !context.is_region_claimed(&checkbox.bounds) {
                    results.push(checkbox);
                }
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
    fn test_checkbox_detector_checked() {
        let text = "[x] Enable feature\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = CheckboxDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Checkbox { label, checked, .. } = &detected[0].element {
            assert_eq!(label, "Enable feature");
            assert!(checked);
        } else {
            panic!("Expected Checkbox element");
        }
    }

    #[test]
    fn test_checkbox_detector_unchecked() {
        let text = "[ ] Disable option\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = CheckboxDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Checkbox { label, checked, .. } = &detected[0].element {
            assert_eq!(label, "Disable option");
            assert!(!checked);
        }
    }

    #[test]
    fn test_checkbox_detector_capital_x() {
        let text = "[X] Accept terms\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = CheckboxDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Checkbox { checked, .. } = &detected[0].element {
            assert!(checked);
        }
    }

    #[test]
    fn test_checkbox_detector_radio_checked() {
        let text = "(*) Option A\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = CheckboxDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Checkbox { label, checked, .. } = &detected[0].element {
            assert_eq!(label, "Option A");
            assert!(checked);
        }
    }

    #[test]
    fn test_checkbox_detector_radio_unchecked() {
        let text = "( ) Option B\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = CheckboxDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Checkbox { label, checked, .. } = &detected[0].element {
            assert_eq!(label, "Option B");
            assert!(!checked);
        }
    }

    #[test]
    fn test_checkbox_detector_multiple() {
        let text = concat!(
            "[x] First option\r\n",
            "[ ] Second option\r\n",
            "[*] Third option\r\n",
        );
        let grid = create_grid_with_text(10, 40, text);

        let detector = CheckboxDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 3);

        // First checkbox - checked
        if let Element::Checkbox { label, checked, .. } = &detected[0].element {
            assert_eq!(label, "First option");
            assert!(checked);
        }

        // Second checkbox - unchecked
        if let Element::Checkbox { label, checked, .. } = &detected[1].element {
            assert_eq!(label, "Second option");
            assert!(!checked);
        }

        // Third checkbox - checked with asterisk
        if let Element::Checkbox { label, checked, .. } = &detected[2].element {
            assert_eq!(label, "Third option");
            assert!(checked);
        }
    }

    #[test]
    fn test_checkbox_detector_priority() {
        let detector = CheckboxDetector::new();
        assert_eq!(detector.priority(), 60);
        assert_eq!(detector.name(), "checkbox");
    }

    #[test]
    fn test_checkbox_detector_no_label() {
        let text = "[x]\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = CheckboxDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Checkbox { label, .. } = &detected[0].element {
            assert_eq!(label, "");
        }
    }

    #[test]
    fn test_checkbox_detector_mixed_types() {
        let text = concat!("[x] Checkbox item\r\n", "(*) Radio item\r\n",);
        let grid = create_grid_with_text(10, 40, text);

        let detector = CheckboxDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 2);
    }
}
