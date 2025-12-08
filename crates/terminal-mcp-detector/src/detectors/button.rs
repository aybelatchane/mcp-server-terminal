//! Button detector for clickable button UI elements.

use terminal_mcp_core::{Bounds, Element};
use terminal_mcp_emulator::Grid;

use crate::detection::{Confidence, DetectedElement, DetectionContext, ElementDetector};

/// Button pattern with open/close delimiters.
#[derive(Debug, Clone)]
struct ButtonPattern {
    open: &'static str,
    close: &'static str,
}

/// Button detector for clickable buttons in bracket patterns.
pub struct ButtonDetector {
    patterns: Vec<ButtonPattern>,
    max_label_length: usize,
    /// Common shell prompt patterns to exclude
    shell_prompt_markers: Vec<&'static str>,
}

impl ButtonDetector {
    /// Create a new button detector.
    pub fn new() -> Self {
        Self {
            patterns: vec![
                ButtonPattern {
                    open: "[ ",
                    close: " ]",
                },
                ButtonPattern {
                    open: "[",
                    close: "]",
                },
                ButtonPattern {
                    open: "< ",
                    close: " >",
                },
                ButtonPattern {
                    open: "<",
                    close: ">",
                },
                // Removed parenthesis patterns - too common in shell prompts
                ButtonPattern {
                    open: "「",
                    close: "」",
                },
            ],
            max_label_length: 30,
            shell_prompt_markers: vec![
                "$", "#", "~", "@",
                ":", // Common prompt symbols (removed ">" to allow angle brackets)
                "git", "main", "master", "dev", // Git branch indicators
            ],
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

    /// Check if a button at the given position overlaps with existing buttons.
    fn overlaps_existing(&self, start: usize, end: usize, existing: &[DetectedElement]) -> bool {
        existing.iter().any(|button| {
            let button_start = button.bounds.col as usize;
            let button_end = (button.bounds.col + button.bounds.width) as usize;
            !(end <= button_start || start >= button_end)
        })
    }

    /// Safely get a substring using byte indices, returning None if indices are invalid.
    fn safe_slice(s: &str, start: usize, end: usize) -> Option<&str> {
        if start <= end && end <= s.len() && s.is_char_boundary(start) && s.is_char_boundary(end) {
            Some(&s[start..end])
        } else {
            None
        }
    }

    /// Check if a row looks like a shell prompt.
    fn is_shell_prompt_row(&self, row_text: &str) -> bool {
        // Check for common shell prompt markers
        for marker in &self.shell_prompt_markers {
            if row_text.contains(marker) {
                return true;
            }
        }
        false
    }

    /// Detect buttons in a specific row.
    fn detect_buttons_in_row(&self, grid: &Grid, row: u16) -> Vec<DetectedElement> {
        let mut buttons = Vec::new();
        let row_text = self.extract_row_text(grid, row);

        // Skip rows that look like shell prompts
        if self.is_shell_prompt_row(&row_text) {
            return buttons;
        }

        for pattern in &self.patterns {
            let mut search_start = 0;
            while let Some(start) = Self::safe_slice(&row_text, search_start, row_text.len())
                .and_then(|s| s.find(pattern.open))
            {
                let abs_start = search_start + start;
                let after_open = abs_start + pattern.open.len();
                let Some(end) = Self::safe_slice(&row_text, after_open, row_text.len())
                    .and_then(|s| s.find(pattern.close))
                else {
                    break;
                };

                let label_start = after_open;
                let label_end = after_open + end;
                let Some(label_str) = Self::safe_slice(&row_text, label_start, label_end) else {
                    search_start = after_open;
                    continue;
                };
                let label = label_str.trim().to_string();

                // Validate label: must be non-empty, reasonable length, and not contain delimiters
                let contains_delimiters = label.contains('[')
                    || label.contains(']')
                    || label.contains('<')
                    || label.contains('>')
                    || label.contains('(')
                    || label.contains(')')
                    || label.contains('「')
                    || label.contains('」');

                if !label.is_empty() && label.len() <= self.max_label_length && !contains_delimiters
                {
                    let button_width = (label_end + pattern.close.len() - abs_start) as u16;
                    let button_end = abs_start + button_width as usize;

                    // Skip if this overlaps with an already-detected button
                    if !self.overlaps_existing(abs_start, button_end, &buttons) {
                        let ref_id = format!("button_{row}_{abs_start}");

                        buttons.push(DetectedElement {
                            element: Element::Button {
                                ref_id,
                                bounds: Bounds::new(row, abs_start as u16, button_width, 1),
                                label,
                            },
                            bounds: Bounds::new(row, abs_start as u16, button_width, 1),
                            confidence: Confidence::High,
                        });
                    }
                }
                search_start = abs_start + pattern.open.len() + end + pattern.close.len();
            }
        }

        buttons
    }
}

impl Default for ButtonDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementDetector for ButtonDetector {
    fn name(&self) -> &'static str {
        "button"
    }

    fn priority(&self) -> u32 {
        60
    }

    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
        let mut results = Vec::new();
        let dims = grid.dimensions();

        for row in 0..dims.rows {
            let buttons = self.detect_buttons_in_row(grid, row);
            for button in buttons {
                // Skip if region already claimed
                if !context.is_region_claimed(&button.bounds) {
                    results.push(button);
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
    fn test_button_detector_bracket_pattern() {
        let text = "[ OK ] [ Cancel ]\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ButtonDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 2);
        if let Element::Button { label, .. } = &detected[0].element {
            assert_eq!(label, "OK");
        } else {
            panic!("Expected Button element");
        }
        if let Element::Button { label, .. } = &detected[1].element {
            assert_eq!(label, "Cancel");
        }
    }

    #[test]
    fn test_button_detector_angle_pattern() {
        let text = "< Submit > < Reset >\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ButtonDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 2);
        if let Element::Button { label, .. } = &detected[0].element {
            assert_eq!(label, "Submit");
        }
        if let Element::Button { label, .. } = &detected[1].element {
            assert_eq!(label, "Reset");
        }
    }

    #[test]
    fn test_button_detector_shell_prompt_excluded() {
        // Shell prompts should not be detected as buttons
        let text = "user@host:/path(main)$ \r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ButtonDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        // Should not detect "(main)" as a button due to shell prompt markers
        assert_eq!(detected.len(), 0);
    }

    #[test]
    fn test_button_detector_tight_bracket() {
        let text = "[OK] [Cancel]\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ButtonDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 2);
        if let Element::Button { label, .. } = &detected[0].element {
            assert_eq!(label, "OK");
        }
        if let Element::Button { label, .. } = &detected[1].element {
            assert_eq!(label, "Cancel");
        }
    }

    #[test]
    fn test_button_detector_priority() {
        let detector = ButtonDetector::new();
        assert_eq!(detector.priority(), 60);
        assert_eq!(detector.name(), "button");
    }

    #[test]
    fn test_button_detector_mixed_patterns() {
        let text = "[OK] < Cancel > 「Help」\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ButtonDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 3);
        assert!(matches!(&detected[0].element, Element::Button { label, .. } if label == "OK"));
        assert!(matches!(&detected[1].element, Element::Button { label, .. } if label == "Cancel"));
        assert!(matches!(&detected[2].element, Element::Button { label, .. } if label == "Help"));
    }

    #[test]
    fn test_button_detector_empty_label_rejected() {
        let text = "[ ] [  ]\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ButtonDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        // Empty labels should not be detected as buttons
        assert_eq!(detected.len(), 0);
    }

    #[test]
    fn test_button_detector_long_label_accepted() {
        // Test that reasonable button labels work
        let text = "[Continue]\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ButtonDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Button { label, .. } = &detected[0].element {
            assert_eq!(label, "Continue");
        }
    }
}
