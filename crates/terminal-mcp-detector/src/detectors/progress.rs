//! Progress bar detector for progress indicators.

use terminal_mcp_core::{Bounds, Element};
use terminal_mcp_emulator::Grid;

use crate::detection::{Confidence, DetectedElement, DetectionContext, ElementDetector};

/// Progress bar detector for various progress indicator patterns.
pub struct ProgressDetector {
    /// Minimum bar length to consider
    min_length: usize,
    /// Filled characters for block progress
    filled_chars: Vec<char>,
    /// Empty characters for block progress
    empty_chars: Vec<char>,
}

impl ProgressDetector {
    /// Safely get a substring using byte indices, returning None if indices are invalid.
    fn safe_slice(s: &str, start: usize, end: usize) -> Option<&str> {
        if start <= end && end <= s.len() && s.is_char_boundary(start) && s.is_char_boundary(end) {
            Some(&s[start..end])
        } else {
            None
        }
    }

    /// Create a new progress detector.
    pub fn new() -> Self {
        Self {
            min_length: 5,
            filled_chars: vec!['█', '▓', '▒', '#', '=', '*'],
            empty_chars: vec!['░', '·', ' ', '-', '.', '▁'],
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

    /// Detect block progress bars (████░░░░░░).
    fn detect_block_progress(&self, text: &str, row: u16) -> Option<DetectedElement> {
        let trimmed = text.trim_end(); // Remove trailing whitespace
        let chars: Vec<char> = trimmed.chars().collect();

        // Find contiguous sequences of progress characters
        let mut start = None;
        let mut filled_count = 0;
        let mut empty_count = 0;
        let mut has_block_chars = false; // Track if we see actual block characters

        for (i, &ch) in chars.iter().enumerate() {
            let is_filled = self.filled_chars.contains(&ch);
            let is_empty = self.empty_chars.contains(&ch);

            if is_filled || is_empty {
                if start.is_none() {
                    start = Some(i);
                }

                if is_filled {
                    filled_count += 1;
                    // Check if it's an actual block character, not just punctuation
                    if ch == '█' || ch == '▓' || ch == '▒' || ch == '#' {
                        has_block_chars = true;
                    }
                } else {
                    empty_count += 1;
                    // Also check empty block characters
                    if ch == '░' || ch == '▁' {
                        has_block_chars = true;
                    }
                }
            } else if start.is_some() {
                // End of progress bar
                let total = filled_count + empty_count;
                // Only return if we saw actual block characters and length is sufficient
                if total >= self.min_length && has_block_chars {
                    let percent = ((filled_count as f32 / total as f32) * 100.0) as u8;
                    let ref_id = format!("progress_{row}_{}", start.unwrap());

                    return Some(DetectedElement {
                        element: Element::ProgressBar {
                            ref_id,
                            bounds: Bounds::new(row, start.unwrap() as u16, total as u16, 1),
                            percent,
                        },
                        bounds: Bounds::new(row, start.unwrap() as u16, total as u16, 1),
                        confidence: Confidence::High,
                    });
                }

                // Reset for next potential bar
                start = None;
                filled_count = 0;
                empty_count = 0;
                has_block_chars = false;
            }
        }

        // Check if bar extends to end of line
        if let Some(start_pos) = start {
            let total = filled_count + empty_count;
            // Only return if we saw actual block characters and length is sufficient
            if total >= self.min_length && has_block_chars {
                let percent = ((filled_count as f32 / total as f32) * 100.0) as u8;
                let ref_id = format!("progress_{row}_{start_pos}");

                return Some(DetectedElement {
                    element: Element::ProgressBar {
                        ref_id,
                        bounds: Bounds::new(row, start_pos as u16, total as u16, 1),
                        percent,
                    },
                    bounds: Bounds::new(row, start_pos as u16, total as u16, 1),
                    confidence: Confidence::High,
                });
            }
        }

        None
    }

    /// Detect bracket progress bars ([====    ]).
    fn detect_bracket_progress(&self, text: &str, row: u16) -> Vec<DetectedElement> {
        let mut results = Vec::new();
        let chars: Vec<char> = text.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            if ch == '[' {
                // Find matching close bracket
                if let Some(close_pos) = chars[i + 1..].iter().position(|&c| c == ']') {
                    let close_idx = i + 1 + close_pos;
                    let inner = &chars[i + 1..close_idx];

                    if inner.len() >= self.min_length {
                        // Count filled vs empty characters
                        let filled = inner
                            .iter()
                            .filter(|&&c| c == '=' || c == '#' || c == '*')
                            .count();
                        let empty = inner
                            .iter()
                            .filter(|&&c| c == ' ' || c == '-' || c == '.')
                            .count();

                        let total = filled + empty;
                        if total >= self.min_length && total == inner.len() {
                            let percent = ((filled as f32 / total as f32) * 100.0) as u8;
                            let ref_id = format!("progress_{row}_{i}");
                            let width = (close_idx - i + 1) as u16; // Include brackets

                            results.push(DetectedElement {
                                element: Element::ProgressBar {
                                    ref_id,
                                    bounds: Bounds::new(row, i as u16, width, 1),
                                    percent,
                                },
                                bounds: Bounds::new(row, i as u16, width, 1),
                                confidence: Confidence::High,
                            });
                        }
                    }
                }
            }
        }

        results
    }

    /// Detect percentage text (45% or 45.5%) and create a progress bar.
    fn detect_percentage_text(&self, text: &str, row: u16) -> Vec<DetectedElement> {
        let mut results = Vec::new();

        // Look for percentage patterns
        let text_str = text;
        let mut search_start = 0;

        while let Some(search_slice) = Self::safe_slice(text_str, search_start, text_str.len()) {
            let Some(percent_pos) = search_slice.find('%') else {
                break;
            };
            let abs_pos = search_start + percent_pos;

            // Extract digits before %
            let Some(before) = Self::safe_slice(text_str, 0, abs_pos) else {
                search_start = abs_pos + 1;
                continue;
            };
            if let Some(num_start) = before.rfind(|c: char| !c.is_numeric() && c != '.') {
                let Some(num_str) = Self::safe_slice(before, num_start + 1, before.len()) else {
                    search_start = abs_pos + 1;
                    continue;
                };
                if let Ok(percent_val) = num_str.parse::<f32>() {
                    let percent = percent_val.min(100.0) as u8;
                    let ref_id = format!("progress_{row}_{}", num_start + 1);
                    let width = (abs_pos - num_start) as u16; // Number + %

                    results.push(DetectedElement {
                        element: Element::ProgressBar {
                            ref_id,
                            bounds: Bounds::new(row, (num_start + 1) as u16, width, 1),
                            percent,
                        },
                        bounds: Bounds::new(row, (num_start + 1) as u16, width, 1),
                        confidence: Confidence::Medium,
                    });
                }
            } else if !before.is_empty() {
                // Try from start of string
                if let Ok(percent_val) = before.parse::<f32>() {
                    let percent = percent_val.min(100.0) as u8;
                    let ref_id = format!("progress_{row}_0");
                    let width = (abs_pos + 1) as u16;

                    results.push(DetectedElement {
                        element: Element::ProgressBar {
                            ref_id,
                            bounds: Bounds::new(row, 0, width, 1),
                            percent,
                        },
                        bounds: Bounds::new(row, 0, width, 1),
                        confidence: Confidence::Medium,
                    });
                }
            }

            search_start = abs_pos + 1;
        }

        results
    }
}

impl Default for ProgressDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementDetector for ProgressDetector {
    fn name(&self) -> &'static str {
        "progress"
    }

    fn priority(&self) -> u32 {
        60
    }

    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
        let mut results = Vec::new();
        let dims = grid.dimensions();

        for row in 0..dims.rows {
            let row_text = self.extract_row_text(grid, row);
            let mut row_has_progress = false;

            // Try block progress detection (highest priority)
            if let Some(progress) = self.detect_block_progress(&row_text, row) {
                if !context.is_region_claimed(&progress.bounds) {
                    results.push(progress);
                    row_has_progress = true;
                }
            }

            // Try bracket progress detection (high priority)
            if !row_has_progress {
                let bracket_progress = self.detect_bracket_progress(&row_text, row);
                for progress in bracket_progress {
                    if !context.is_region_claimed(&progress.bounds) {
                        results.push(progress);
                        row_has_progress = true;
                    }
                }
            }

            // Try percentage text detection (lowest priority, only if nothing else found)
            if !row_has_progress {
                let percentage_progress = self.detect_percentage_text(&row_text, row);
                for progress in percentage_progress {
                    if !context.is_region_claimed(&progress.bounds) {
                        results.push(progress);
                    }
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
    fn test_progress_detector_block() {
        let text = "████░░░░░░\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ProgressDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::ProgressBar { percent, .. } = &detected[0].element {
            assert_eq!(*percent, 40); // 4 filled out of 10
        } else {
            panic!("Expected ProgressBar element");
        }
    }

    #[test]
    fn test_progress_detector_bracket() {
        let text = "[====    ]\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ProgressDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::ProgressBar { percent, .. } = &detected[0].element {
            assert_eq!(*percent, 50); // 4 filled out of 8
        }
    }

    #[test]
    fn test_progress_detector_percentage() {
        let text = "Progress: 75%\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ProgressDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::ProgressBar { percent, .. } = &detected[0].element {
            assert_eq!(*percent, 75);
        }
    }

    #[test]
    fn test_progress_detector_priority() {
        let detector = ProgressDetector::new();
        assert_eq!(detector.priority(), 60);
        assert_eq!(detector.name(), "progress");
    }

    #[test]
    fn test_progress_detector_hash_fill() {
        let text = "[####----]\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ProgressDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::ProgressBar { percent, .. } = &detected[0].element {
            assert_eq!(*percent, 50); // 4 filled out of 8
        }
    }

    #[test]
    fn test_progress_detector_full_bar() {
        let text = "██████████\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ProgressDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::ProgressBar { percent, .. } = &detected[0].element {
            assert_eq!(*percent, 100);
        }
    }

    #[test]
    fn test_progress_detector_empty_bar() {
        let text = "░░░░░░░░░░\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = ProgressDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::ProgressBar { percent, .. } = &detected[0].element {
            assert_eq!(*percent, 0);
        }
    }
}
