//! Menu detector for terminal UI menus with selection state.

use terminal_mcp_core::{Bounds, Element, MenuItem, Position};
use terminal_mcp_emulator::Grid;

use crate::detection::{Confidence, DetectedElement, DetectionContext, ElementDetector};

/// Selection indicator patterns.
#[derive(Debug, Clone)]
struct SelectionIndicators {
    /// Prefix markers that indicate selection
    prefix_markers: Vec<char>,
}

impl Default for SelectionIndicators {
    fn default() -> Self {
        Self {
            prefix_markers: vec!['>', '→', '▶', '•', '*', '►'],
        }
    }
}

/// Menu detector for vertical or horizontal menus with selection state.
pub struct MenuDetector {
    /// Minimum items to consider a menu
    min_items: usize,
    /// Selection indicator patterns
    selection_indicators: SelectionIndicators,
}

impl MenuDetector {
    /// Create a new menu detector.
    pub fn new() -> Self {
        Self {
            min_items: 2,
            selection_indicators: SelectionIndicators::default(),
        }
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

    /// Check if a row has a specific attribute (e.g., reverse video).
    fn row_has_attribute<F>(&self, grid: &Grid, row: u16, region: &Bounds, predicate: F) -> bool
    where
        F: Fn(&terminal_mcp_core::CellAttributes) -> bool,
    {
        let mut match_count = 0;
        let mut non_space_count = 0;

        for col_offset in 0..region.width {
            let col = region.col + col_offset;
            if let Some(cell) = grid.cell(row, col) {
                if cell.character != ' ' {
                    non_space_count += 1;
                    if predicate(&cell.attrs) {
                        match_count += 1;
                    }
                }
            }
        }

        // If more than 50% of non-space cells have the attribute, consider the row to have it
        non_space_count > 0 && match_count > non_space_count / 2
    }

    /// Strategy 1: Reverse video detection (highest confidence).
    fn detect_by_reverse_video(
        &self,
        grid: &Grid,
        region: &Bounds,
    ) -> Option<(usize, Vec<MenuItem>)> {
        let mut items = Vec::new();
        let mut selected_idx = None;

        for row_offset in 0..region.height {
            let row = region.row + row_offset;
            let text = self.extract_row_text(grid, row, region.col, region.width);

            if text.trim().is_empty() {
                continue;
            }

            // Check if row has reverse video attribute
            let has_reverse = self.row_has_attribute(grid, row, region, |attrs| attrs.reverse);

            if has_reverse {
                selected_idx = Some(items.len());
            }

            items.push(MenuItem {
                ref_id: String::new(), // Assigned later
                text: text.trim().to_string(),
                selected: has_reverse,
            });
        }

        // Only return if we actually found reverse video
        if items.len() >= self.min_items {
            selected_idx.map(|idx| (idx, items))
        } else {
            None
        }
    }

    /// Strategy 2: Prefix marker detection (high confidence).
    fn detect_by_prefix_marker(
        &self,
        grid: &Grid,
        region: &Bounds,
    ) -> Option<(usize, Vec<MenuItem>)> {
        let mut items = Vec::new();
        let mut selected_idx = None;

        for row_offset in 0..region.height {
            let row = region.row + row_offset;
            let text = self.extract_row_text(grid, row, region.col, region.width);
            let trimmed = text.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for prefix marker
            let first_char = trimmed.chars().next();
            let is_selected = first_char
                .map(|c| self.selection_indicators.prefix_markers.contains(&c))
                .unwrap_or(false);

            let item_text = if is_selected {
                trimmed
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    .trim()
                    .to_string()
            } else {
                trimmed.to_string()
            };

            if is_selected {
                selected_idx = Some(items.len());
            }

            items.push(MenuItem {
                ref_id: String::new(),
                text: item_text,
                selected: is_selected,
            });
        }

        if items.len() >= self.min_items {
            selected_idx.map(|idx| (idx, items))
        } else {
            None
        }
    }

    /// Strategy 3: Cursor position detection (medium confidence).
    fn detect_by_cursor(
        &self,
        grid: &Grid,
        region: &Bounds,
        cursor: Position,
    ) -> Option<(usize, Vec<MenuItem>)> {
        // If cursor is not within region, can't use this strategy
        if !region.contains(&cursor) {
            return None;
        }

        let mut items = Vec::new();
        let cursor_row_offset = cursor.row - region.row;

        for row_offset in 0..region.height {
            let row = region.row + row_offset;
            let text = self.extract_row_text(grid, row, region.col, region.width);

            if text.trim().is_empty() {
                continue;
            }

            let is_selected = row_offset == cursor_row_offset;

            items.push(MenuItem {
                ref_id: String::new(),
                text: text.trim().to_string(),
                selected: is_selected,
            });
        }

        if items.len() >= self.min_items {
            let selected = items.iter().position(|i| i.selected).unwrap_or(0);
            Some((selected, items))
        } else {
            None
        }
    }

    /// Detect menu in a region using multiple strategies.
    fn detect_menu_in_region(
        &self,
        grid: &Grid,
        region: &Bounds,
        cursor: Position,
    ) -> Option<DetectedElement> {
        // Try strategies in order of confidence
        let result = self
            .detect_by_reverse_video(grid, region)
            .or_else(|| self.detect_by_prefix_marker(grid, region))
            .or_else(|| self.detect_by_cursor(grid, region, cursor));

        if let Some((selected_idx, items)) = result {
            let ref_id = format!("menu_{}_{}", region.row, region.col);

            Some(DetectedElement {
                element: Element::Menu {
                    ref_id,
                    bounds: *region,
                    items,
                    selected: selected_idx,
                },
                bounds: *region,
                confidence: Confidence::High,
            })
        } else {
            None
        }
    }

    /// Find potential menu regions in the grid.
    fn find_menu_regions(&self, grid: &Grid) -> Vec<Bounds> {
        let mut regions = Vec::new();
        let dims = grid.dimensions();

        // Look for consecutive rows with similar content patterns
        // This is a simple heuristic - consecutive non-empty rows
        let mut region_start: Option<u16> = None;

        for row in 0..dims.rows {
            let text = self.extract_row_text(grid, row, 0, dims.cols);
            let has_content = !text.trim().is_empty();

            if has_content {
                if region_start.is_none() {
                    region_start = Some(row);
                }
            } else if let Some(start) = region_start {
                // End of region
                let height = row - start;
                if height >= self.min_items as u16 {
                    regions.push(Bounds::new(start, 0, dims.cols, height));
                }
                region_start = None;
            }
        }

        // Handle region that extends to end of grid
        if let Some(start) = region_start {
            let height = dims.rows - start;
            if height >= self.min_items as u16 {
                regions.push(Bounds::new(start, 0, dims.cols, height));
            }
        }

        regions
    }
}

impl Default for MenuDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementDetector for MenuDetector {
    fn name(&self) -> &'static str {
        "menu"
    }

    fn priority(&self) -> u32 {
        80
    }

    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
        let mut results = Vec::new();

        // Find potential menu regions
        let regions = self.find_menu_regions(grid);

        for region in regions {
            // Skip if region already claimed
            if context.is_region_claimed(&region) {
                continue;
            }

            // Try to detect menu in this region
            if let Some(menu) = self.detect_menu_in_region(grid, &region, context.cursor) {
                results.push(menu);
            }
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
    fn test_menu_detector_with_prefix_marker() {
        // Test without title line - just menu items
        let text = "> File\r\n  Edit\r\n  View\r\n  Help\r\n";
        let grid = create_grid_with_text(10, 40, text);

        let detector = MenuDetector::new();
        // Use cursor outside the menu region to force prefix marker strategy
        let context = DetectionContext::new(Position::new(10, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Menu {
            items, selected, ..
        } = &detected[0].element
        {
            assert_eq!(items.len(), 4);
            assert_eq!(selected, &0);
            assert_eq!(items[0].text, "File");
            assert!(items[0].selected);
            assert_eq!(items[1].text, "Edit");
            assert!(!items[1].selected);
        } else {
            panic!("Expected Menu element");
        }
    }

    #[test]
    fn test_menu_detector_priority() {
        let detector = MenuDetector::new();
        assert_eq!(detector.priority(), 80);
        assert_eq!(detector.name(), "menu");
    }

    #[test]
    fn test_menu_detector_min_items() {
        // Only one item, should not detect as menu
        let text = "> Single Item\r\n";
        let grid = create_grid_with_text(5, 40, text);

        let detector = MenuDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 0);
    }

    #[test]
    fn test_menu_detector_with_arrow_marker() {
        // Test without title line
        let text = "→ Option 1\r\n  Option 2\r\n  Option 3\r\n";
        let grid = create_grid_with_text(10, 40, text);

        let detector = MenuDetector::new();
        // Use cursor outside the menu region to force prefix marker strategy
        let context = DetectionContext::new(Position::new(10, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Menu {
            items, selected, ..
        } = &detected[0].element
        {
            assert_eq!(items.len(), 3);
            assert_eq!(selected, &0);
            assert_eq!(items[0].text, "Option 1");
        }
    }

    #[test]
    fn test_menu_detector_no_selection() {
        // No selection marker, should detect with cursor position strategy
        let text = "Item 1\r\nItem 2\r\nItem 3\r\n";
        let grid = create_grid_with_text(10, 40, text);

        let detector = MenuDetector::new();
        // Cursor at row 1 (Item 2) within the menu region
        let context = DetectionContext::new(Position::new(1, 0));
        let detected = detector.detect(&grid, &context);

        // Should detect with cursor position strategy
        assert_eq!(detected.len(), 1);
        if let Element::Menu {
            items, selected, ..
        } = &detected[0].element
        {
            assert_eq!(items.len(), 3);
            assert_eq!(selected, &1); // Second item selected (where cursor is)
        }
    }
}
