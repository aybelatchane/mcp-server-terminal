//! Terminal grid state buffer and cursor tracking.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use terminal_mcp_core::{Bounds, Cell, CellAttributes, Color, Dimensions, Position};

/// Cursor visual style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CursorStyle {
    /// Block cursor (fills entire cell)
    Block,
    /// Underline cursor (bottom of cell)
    Underline,
    /// Bar cursor (vertical line at left)
    Bar,
}

/// Cursor state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    /// Current position
    pub position: Position,
    /// Visibility
    pub visible: bool,
    /// Cursor style
    pub style: CursorStyle,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            position: Position::origin(),
            visible: true,
            style: CursorStyle::Block,
        }
    }
}

impl Cursor {
    /// Create a new cursor at origin.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create cursor at specific position.
    pub fn at(position: Position) -> Self {
        Self {
            position,
            visible: true,
            style: CursorStyle::Block,
        }
    }
}

/// Terminal grid state buffer.
#[derive(Debug)]
pub struct Grid {
    /// Cell storage (row-major order)
    cells: Vec<Cell>,
    /// Grid dimensions
    dimensions: Dimensions,
    /// Cursor state
    cursor: Cursor,
    /// Saved cursor (for save/restore operations)
    saved_cursor: Option<Cursor>,
    /// Scroll region (top, bottom) - 0-indexed, inclusive
    #[allow(dead_code)] // TODO: Will be used in E2.1 VTE parser implementation
    scroll_region: Option<(u16, u16)>,
    /// Current cell attributes for new characters
    current_attrs: CellAttributes,
    /// Current foreground color
    current_fg: Color,
    /// Current background color
    current_bg: Color,
}

impl Grid {
    /// Create a new grid with the given dimensions.
    ///
    /// All cells are initialized to default (empty space).
    pub fn new(dimensions: Dimensions) -> Self {
        let cell_count = dimensions.cell_count();
        Self {
            cells: vec![Cell::default(); cell_count],
            dimensions,
            cursor: Cursor::default(),
            saved_cursor: None,
            scroll_region: None,
            current_attrs: CellAttributes::default(),
            current_fg: Color::Default,
            current_bg: Color::Default,
        }
    }

    /// Get cell at position (immutable).
    ///
    /// Returns None if position is out of bounds.
    pub fn cell(&self, row: u16, col: u16) -> Option<&Cell> {
        if row < self.dimensions.rows && col < self.dimensions.cols {
            let idx = row as usize * self.dimensions.cols as usize + col as usize;
            self.cells.get(idx)
        } else {
            None
        }
    }

    /// Get mutable cell at position.
    ///
    /// Returns None if position is out of bounds.
    pub fn cell_mut(&mut self, row: u16, col: u16) -> Option<&mut Cell> {
        if row < self.dimensions.rows && col < self.dimensions.cols {
            let idx = row as usize * self.dimensions.cols as usize + col as usize;
            self.cells.get_mut(idx)
        } else {
            None
        }
    }

    /// Get entire row as a slice.
    ///
    /// Returns None if row is out of bounds.
    pub fn row(&self, row: u16) -> Option<&[Cell]> {
        if row < self.dimensions.rows {
            let start = row as usize * self.dimensions.cols as usize;
            let end = start + self.dimensions.cols as usize;
            Some(&self.cells[start..end])
        } else {
            None
        }
    }

    /// Extract text from a specific region.
    ///
    /// Trailing whitespace is trimmed from each line.
    pub fn extract_text(&self, bounds: &Bounds) -> String {
        let mut text = String::new();
        for row_idx in bounds.row..(bounds.row + bounds.height) {
            if row_idx > bounds.row {
                text.push('\n');
            }
            for col_idx in bounds.col..(bounds.col + bounds.width) {
                if let Some(cell) = self.cell(row_idx, col_idx) {
                    text.push(cell.character);
                }
            }
        }
        // Trim trailing whitespace per line
        text.lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Convert entire grid to plain text.
    pub fn to_plain_text(&self) -> String {
        let bounds = Bounds::new(0, 0, self.dimensions.cols, self.dimensions.rows);
        self.extract_text(&bounds)
    }

    /// Get cursor reference.
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Get mutable cursor reference.
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    /// Get dimensions.
    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }

    /// Check if cursor is visible.
    pub fn cursor_visible(&self) -> bool {
        self.cursor.visible
    }

    /// Get current cell attributes.
    pub fn current_attrs(&self) -> &CellAttributes {
        &self.current_attrs
    }

    /// Set current cell attributes.
    pub fn set_current_attrs(&mut self, attrs: CellAttributes) {
        self.current_attrs = attrs;
    }

    /// Get current foreground color.
    pub fn current_fg(&self) -> Color {
        self.current_fg
    }

    /// Set current foreground color.
    pub fn set_current_fg(&mut self, color: Color) {
        self.current_fg = color;
    }

    /// Get current background color.
    pub fn current_bg(&self) -> Color {
        self.current_bg
    }

    /// Set current background color.
    pub fn set_current_bg(&mut self, color: Color) {
        self.current_bg = color;
    }

    /// Save current cursor state.
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor.clone());
    }

    /// Restore saved cursor state.
    pub fn restore_cursor(&mut self) {
        if let Some(saved) = self.saved_cursor.take() {
            self.cursor = saved;
        }
    }

    /// Resize grid, preserving content where possible.
    ///
    /// Content from the top-left corner is preserved up to the smaller of
    /// old and new dimensions. Cursor is clamped to new bounds.
    pub fn resize(&mut self, new_dimensions: Dimensions) {
        let mut new_cells = vec![Cell::default(); new_dimensions.cell_count()];

        let copy_rows = self.dimensions.rows.min(new_dimensions.rows);
        let copy_cols = self.dimensions.cols.min(new_dimensions.cols);

        // Copy preserved content
        for row in 0..copy_rows {
            for col in 0..copy_cols {
                let old_idx = row as usize * self.dimensions.cols as usize + col as usize;
                let new_idx = row as usize * new_dimensions.cols as usize + col as usize;
                new_cells[new_idx] = self.cells[old_idx].clone();
            }
        }

        self.cells = new_cells;
        self.dimensions = new_dimensions;

        // Clamp cursor to new dimensions
        if new_dimensions.rows > 0 {
            self.cursor.position.row = self.cursor.position.row.min(new_dimensions.rows - 1);
        }
        if new_dimensions.cols > 0 {
            self.cursor.position.col = self.cursor.position.col.min(new_dimensions.cols - 1);
        }
    }

    /// Clear the entire grid.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    /// Clear a specific region.
    pub fn clear_region(&mut self, bounds: &Bounds) {
        for row in bounds.row..(bounds.row + bounds.height) {
            for col in bounds.col..(bounds.col + bounds.width) {
                if let Some(cell) = self.cell_mut(row, col) {
                    *cell = Cell::default();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_new() {
        let grid = Grid::new(Dimensions::new(24, 80));
        assert_eq!(grid.dimensions().rows, 24);
        assert_eq!(grid.dimensions().cols, 80);
        assert_eq!(grid.cursor().position, Position::origin());
        assert!(grid.cursor_visible());
    }

    #[test]
    fn test_grid_cell_access() {
        let mut grid = Grid::new(Dimensions::new(10, 10));

        // Get default cell
        let cell = grid.cell(0, 0).unwrap();
        assert_eq!(cell.character, ' ');

        // Modify cell
        if let Some(cell) = grid.cell_mut(5, 5) {
            cell.character = 'X';
        }

        // Verify modification
        assert_eq!(grid.cell(5, 5).unwrap().character, 'X');

        // Out of bounds
        assert!(grid.cell(10, 10).is_none());
        assert!(grid.cell_mut(10, 10).is_none());
    }

    #[test]
    fn test_grid_row_access() {
        let mut grid = Grid::new(Dimensions::new(5, 10));

        // Modify a row
        for col in 0..10 {
            if let Some(cell) = grid.cell_mut(2, col) {
                cell.character = (b'0' + col as u8) as char;
            }
        }

        // Get row
        let row = grid.row(2).unwrap();
        assert_eq!(row.len(), 10);
        assert_eq!(row[0].character, '0');
        assert_eq!(row[9].character, '9');

        // Out of bounds
        assert!(grid.row(5).is_none());
    }

    #[test]
    fn test_grid_extract_text() {
        let mut grid = Grid::new(Dimensions::new(5, 10));

        // Write "HELLO" at row 1
        let text = "HELLO";
        for (i, ch) in text.chars().enumerate() {
            if let Some(cell) = grid.cell_mut(1, i as u16) {
                cell.character = ch;
            }
        }

        // Extract the region
        let bounds = Bounds::new(1, 0, 10, 1);
        let extracted = grid.extract_text(&bounds);
        assert_eq!(extracted, "HELLO");
    }

    #[test]
    fn test_grid_to_plain_text() {
        let mut grid = Grid::new(Dimensions::new(3, 5));

        // Write pattern
        for row in 0..3 {
            for col in 0..5 {
                if let Some(cell) = grid.cell_mut(row, col) {
                    cell.character = if (row + col) % 2 == 0 { 'X' } else { 'O' };
                }
            }
        }

        let text = grid.to_plain_text();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "XOXOX");
        assert_eq!(lines[1], "OXOXO");
        assert_eq!(lines[2], "XOXOX");
    }

    #[test]
    fn test_grid_resize_preserve() {
        let mut grid = Grid::new(Dimensions::new(5, 5));

        // Fill with pattern
        for row in 0..5 {
            for col in 0..5 {
                if let Some(cell) = grid.cell_mut(row, col) {
                    cell.character = 'A';
                }
            }
        }

        // Resize to larger
        grid.resize(Dimensions::new(10, 10));
        assert_eq!(grid.dimensions().rows, 10);
        assert_eq!(grid.dimensions().cols, 10);

        // Original content preserved
        assert_eq!(grid.cell(0, 0).unwrap().character, 'A');
        assert_eq!(grid.cell(4, 4).unwrap().character, 'A');

        // New cells are default
        assert_eq!(grid.cell(9, 9).unwrap().character, ' ');
    }

    #[test]
    fn test_grid_resize_shrink() {
        let mut grid = Grid::new(Dimensions::new(10, 10));

        // Set marker cell
        if let Some(cell) = grid.cell_mut(2, 2) {
            cell.character = 'M';
        }

        // Resize to smaller
        grid.resize(Dimensions::new(5, 5));
        assert_eq!(grid.dimensions().rows, 5);

        // Preserved cell still there
        assert_eq!(grid.cell(2, 2).unwrap().character, 'M');
    }

    #[test]
    fn test_cursor_default() {
        let cursor = Cursor::default();
        assert_eq!(cursor.position, Position::origin());
        assert!(cursor.visible);
        assert_eq!(cursor.style, CursorStyle::Block);
    }

    #[test]
    fn test_cursor_at() {
        let cursor = Cursor::at(Position::new(5, 10));
        assert_eq!(cursor.position.row, 5);
        assert_eq!(cursor.position.col, 10);
        assert!(cursor.visible);
    }

    #[test]
    fn test_cursor_save_restore() {
        let mut grid = Grid::new(Dimensions::new(24, 80));

        // Move cursor
        grid.cursor_mut().position = Position::new(10, 20);

        // Save
        grid.save_cursor();

        // Move again
        grid.cursor_mut().position = Position::new(5, 5);
        assert_eq!(grid.cursor().position, Position::new(5, 5));

        // Restore
        grid.restore_cursor();
        assert_eq!(grid.cursor().position, Position::new(10, 20));
    }

    #[test]
    fn test_grid_clear() {
        let mut grid = Grid::new(Dimensions::new(5, 5));

        // Fill grid
        for row in 0..5 {
            for col in 0..5 {
                if let Some(cell) = grid.cell_mut(row, col) {
                    cell.character = 'X';
                }
            }
        }

        // Clear
        grid.clear();

        // All cells should be default
        for row in 0..5 {
            for col in 0..5 {
                assert_eq!(grid.cell(row, col).unwrap().character, ' ');
            }
        }
    }

    #[test]
    fn test_grid_clear_region() {
        let mut grid = Grid::new(Dimensions::new(5, 5));

        // Fill grid
        for row in 0..5 {
            for col in 0..5 {
                if let Some(cell) = grid.cell_mut(row, col) {
                    cell.character = 'X';
                }
            }
        }

        // Clear center region (2x2)
        let bounds = Bounds::new(1, 1, 2, 2);
        grid.clear_region(&bounds);

        // Check cleared region
        assert_eq!(grid.cell(1, 1).unwrap().character, ' ');
        assert_eq!(grid.cell(2, 2).unwrap().character, ' ');

        // Check untouched cells
        assert_eq!(grid.cell(0, 0).unwrap().character, 'X');
        assert_eq!(grid.cell(4, 4).unwrap().character, 'X');
    }
}
