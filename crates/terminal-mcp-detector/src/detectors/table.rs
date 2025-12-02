//! Table detector for data tables with headers and aligned columns.

use terminal_mcp_core::{Bounds, Element};
use terminal_mcp_emulator::Grid;

use crate::detection::{Confidence, DetectedElement, DetectionContext, ElementDetector};

/// Table detector for data tables with headers and aligned columns.
pub struct TableDetector {
    /// Minimum columns to consider a table
    min_columns: usize,
    /// Minimum rows (including header)
    min_rows: usize,
}

impl TableDetector {
    /// Create a new table detector.
    pub fn new() -> Self {
        Self {
            min_columns: 2,
            min_rows: 2,
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

    /// Check if a row is mostly bold.
    fn row_is_bold(&self, grid: &Grid, row: u16, region: &Bounds) -> bool {
        let mut bold_count = 0;
        let mut non_space_count = 0;

        for col_offset in 0..region.width {
            let col = region.col + col_offset;
            if let Some(cell) = grid.cell(row, col) {
                if cell.character != ' ' {
                    non_space_count += 1;
                    if cell.attrs.bold {
                        bold_count += 1;
                    }
                }
            }
        }

        non_space_count > 0 && bold_count > non_space_count / 2
    }

    /// Check if a row has a different background color than the rest.
    fn row_has_different_bg(&self, grid: &Grid, row: u16, region: &Bounds) -> bool {
        // Simple heuristic: if first row has non-default background
        for col_offset in 0..region.width {
            let col = region.col + col_offset;
            if let Some(cell) = grid.cell(row, col) {
                if cell.character != ' ' && cell.bg != terminal_mcp_core::Color::Default {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a row is a separator line (mostly ─, ━, -, =, ═).
    fn is_separator_line(&self, grid: &Grid, row: u16, region: &Bounds) -> bool {
        let separator_chars = ['─', '━', '-', '=', '═', '|', '│'];
        let mut separator_count = 0;
        let mut total_non_space = 0;

        for col in region.col..(region.col + region.width) {
            if let Some(cell) = grid.cell(row, col) {
                if cell.character != ' ' {
                    total_non_space += 1;
                    if separator_chars.contains(&cell.character) {
                        separator_count += 1;
                    }
                }
            }
        }

        total_non_space > 0 && (separator_count as f32 / total_non_space as f32) > 0.8
    }

    /// Detect header row (often bold, or followed by separator line).
    fn detect_header_row(&self, grid: &Grid, region: &Bounds) -> Option<usize> {
        // Strategy 1: Bold first row
        if self.row_is_bold(grid, region.row, region) {
            return Some(0);
        }

        // Strategy 2: Separator line after first row
        if region.height > 1 {
            let second_row = region.row + 1;
            if self.is_separator_line(grid, second_row, region) {
                return Some(0);
            }
        }

        // Strategy 3: First row has different background
        if self.row_has_different_bg(grid, region.row, region) {
            return Some(0);
        }

        None
    }

    /// Convert separators to column boundaries.
    fn separators_to_columns(&self, separators: &[usize], width: usize) -> Option<Vec<usize>> {
        if separators.is_empty() {
            return None;
        }

        let mut columns = vec![0]; // Start of first column

        // Group consecutive separators and use the midpoint
        let mut i = 0;
        while i < separators.len() {
            let start = separators[i];
            let mut end = start;

            // Find consecutive separators
            while i + 1 < separators.len() && separators[i + 1] == end + 1 {
                i += 1;
                end = separators[i];
            }

            // Skip if this separator extends very close to the end (trailing empty space)
            // Allow 20% margin to account for minor trailing spaces
            if end >= width * 8 / 10 {
                break;
            }

            // Use midpoint of separator region as column boundary
            columns.push((start + end) / 2 + 1);
            i += 1;
        }

        if columns.len() >= self.min_columns {
            Some(columns)
        } else {
            None
        }
    }

    /// Detect column boundaries by finding consistent vertical alignment.
    fn detect_columns(&self, grid: &Grid, region: &Bounds) -> Option<Vec<usize>> {
        // Build column occupancy map
        let mut col_occupancy: Vec<usize> = vec![0; region.width as usize];

        for row_offset in 0..region.height {
            let row = region.row + row_offset;
            for col_offset in 0..region.width {
                let col = region.col + col_offset;
                if let Some(cell) = grid.cell(row, col) {
                    if cell.character != ' ' {
                        col_occupancy[col_offset as usize] += 1;
                    }
                }
            }
        }

        // Find column separators (consistently empty columns)
        let threshold = region.height as usize / 2;
        let mut separators = Vec::new();

        for (idx, &count) in col_occupancy.iter().enumerate() {
            if count < threshold {
                separators.push(idx);
            }
        }

        // Convert separators to column boundaries
        self.separators_to_columns(&separators, region.width as usize)
    }

    /// Extract cell text from a row between column boundaries.
    fn extract_cell(
        &self,
        grid: &Grid,
        row: u16,
        region: &Bounds,
        col_start: usize,
        col_end: usize,
    ) -> String {
        let start = region.col + col_start as u16;
        let width = (col_end - col_start) as u16;
        self.extract_row_text(grid, row, start, width)
            .trim()
            .to_string()
    }

    /// Parse table rows using detected column boundaries.
    /// Returns (headers, data_rows) tuple.
    fn parse_table_rows(
        &self,
        grid: &Grid,
        region: &Bounds,
        columns: &[usize],
        header_row_idx: Option<usize>,
    ) -> (Vec<String>, Vec<Vec<String>>) {
        let mut all_rows = Vec::new();

        for row_offset in 0..region.height {
            let row = region.row + row_offset;

            // Skip separator lines
            if self.is_separator_line(grid, row, region) {
                continue;
            }

            // Extract cells for this row
            let mut cells = Vec::new();

            // Extract cells using column boundaries
            for i in 0..columns.len() {
                let col_start = columns[i];
                let col_end = if i + 1 < columns.len() {
                    columns[i + 1]
                } else {
                    region.width as usize
                };

                let content = self.extract_cell(grid, row, region, col_start, col_end);

                // Don't add empty trailing columns
                if i == columns.len() - 1
                    && content.is_empty()
                    && cells.iter().any(|c: &String| !c.is_empty())
                {
                    continue;
                }

                cells.push(content);
            }

            all_rows.push((row_offset as usize, cells));
        }

        // Separate headers from data rows
        let header_idx = header_row_idx.unwrap_or(0);
        let mut headers = Vec::new();
        let mut data_rows = Vec::new();

        for (row_idx, cells) in all_rows {
            if row_idx == header_idx {
                headers = cells;
            } else {
                data_rows.push(cells);
            }
        }

        (headers, data_rows)
    }

    /// Find potential table regions in the grid.
    fn find_table_regions(&self, grid: &Grid) -> Vec<Bounds> {
        let mut regions = Vec::new();
        let dims = grid.dimensions();

        // Look for consecutive rows with content
        let mut region_start: Option<u16> = None;

        for row in 0..dims.rows {
            let text = self.extract_row_text(grid, row, 0, dims.cols);
            let has_content = !text.trim().is_empty();

            if has_content {
                if region_start.is_none() {
                    region_start = Some(row);
                }
            } else if let Some(start) = region_start {
                let height = row - start;
                if height >= self.min_rows as u16 {
                    regions.push(Bounds::new(start, 0, dims.cols, height));
                }
                region_start = None;
            }
        }

        // Handle region that extends to end
        if let Some(start) = region_start {
            let height = dims.rows - start;
            if height >= self.min_rows as u16 {
                regions.push(Bounds::new(start, 0, dims.cols, height));
            }
        }

        regions
    }

    /// Detect table in a region.
    fn detect_table_in_region(&self, grid: &Grid, region: &Bounds) -> Option<DetectedElement> {
        // Detect columns
        let columns = self.detect_columns(grid, region)?;

        if columns.len() < self.min_columns {
            return None;
        }

        // Detect header row
        let header_row_idx = self.detect_header_row(grid, region);

        // Parse table rows
        let (headers, rows) = self.parse_table_rows(grid, region, &columns, header_row_idx);

        // Total rows including header
        let total_rows = rows.len() + if !headers.is_empty() { 1 } else { 0 };

        if total_rows < self.min_rows {
            return None;
        }

        let ref_id = format!("table_{}_{}", region.row, region.col);

        Some(DetectedElement {
            element: Element::Table {
                ref_id,
                bounds: *region,
                headers,
                rows,
            },
            bounds: *region,
            confidence: Confidence::Medium,
        })
    }
}

impl Default for TableDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementDetector for TableDetector {
    fn name(&self) -> &'static str {
        "table"
    }

    fn priority(&self) -> u32 {
        80
    }

    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
        let mut results = Vec::new();

        // Find potential table regions
        let regions = self.find_table_regions(grid);

        for region in regions {
            // Skip if region already claimed
            if context.is_region_claimed(&region) {
                continue;
            }

            // Try to detect table in this region
            if let Some(table) = self.detect_table_in_region(grid, &region) {
                results.push(table);
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
    fn test_table_detector_basic() {
        let text = concat!(
            "Name      Age  City\r\n",
            "Alice     25   NYC\r\n",
            "Bob       30   LA\r\n",
        );
        let grid = create_grid_with_text(10, 40, text);

        let detector = TableDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Table { headers, rows, .. } = &detected[0].element {
            assert_eq!(headers.len(), 3);
            assert!(rows.len() >= 2);
        } else {
            panic!("Expected Table element");
        }
    }

    #[test]
    fn test_table_detector_with_separator() {
        let text = concat!(
            "Name      Age\r\n",
            "----------\r\n",
            "Alice     25\r\n",
            "Bob       30\r\n",
        );
        let grid = create_grid_with_text(10, 40, text);

        let detector = TableDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        assert_eq!(detected.len(), 1);
        if let Element::Table { headers, rows, .. } = &detected[0].element {
            // Should have headers and 2 data rows (separator line excluded)
            assert!(!headers.is_empty(), "Should have headers");
            assert_eq!(rows.len(), 2, "Should have 2 data rows");
        }
    }

    #[test]
    fn test_table_detector_min_columns() {
        // Only one column, should not detect as table
        let text = "Item1\r\nItem2\r\nItem3\r\n";
        let grid = create_grid_with_text(10, 40, text);

        let detector = TableDetector::new();
        let context = DetectionContext::new(Position::new(0, 0));
        let detected = detector.detect(&grid, &context);

        // Should not detect single column as table
        assert_eq!(detected.len(), 0);
    }

    #[test]
    fn test_table_detector_priority() {
        let detector = TableDetector::new();
        assert_eq!(detector.priority(), 80);
        assert_eq!(detector.name(), "table");
    }
}
