//! Geometry types for terminal coordinates and regions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Position in the terminal grid (row, column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Position {
    /// Row index (0-based)
    pub row: u16,
    /// Column index (0-based)
    pub col: u16,
}

impl Position {
    /// Create a new position.
    pub fn new(row: u16, col: u16) -> Self {
        Self { row, col }
    }

    /// Origin position (0, 0).
    pub fn origin() -> Self {
        Self { row: 0, col: 0 }
    }
}

/// Dimensions of a terminal or region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Dimensions {
    /// Number of rows
    pub rows: u16,
    /// Number of columns
    pub cols: u16,
}

impl Dimensions {
    /// Create new dimensions.
    pub fn new(rows: u16, cols: u16) -> Self {
        Self { rows, cols }
    }

    /// Total cell count (rows * cols).
    pub fn cell_count(&self) -> usize {
        self.rows as usize * self.cols as usize
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self::new(24, 80)
    }
}

/// Bounding box for a terminal region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Bounds {
    /// Starting row
    pub row: u16,
    /// Starting column
    pub col: u16,
    /// Width in columns
    pub width: u16,
    /// Height in rows
    pub height: u16,
}

impl Bounds {
    /// Create new bounds.
    pub fn new(row: u16, col: u16, width: u16, height: u16) -> Self {
        Self {
            row,
            col,
            width,
            height,
        }
    }

    /// Check if a position is contained within these bounds.
    pub fn contains(&self, pos: &Position) -> bool {
        pos.row >= self.row
            && pos.row < self.row + self.height
            && pos.col >= self.col
            && pos.col < self.col + self.width
    }

    /// Check if these bounds intersect with another bounds.
    pub fn intersects(&self, other: &Bounds) -> bool {
        !(self.row + self.height <= other.row
            || other.row + other.height <= self.row
            || self.col + self.width <= other.col
            || other.col + other.width <= self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.row, 5);
        assert_eq!(pos.col, 10);
    }

    #[test]
    fn test_dimensions_default() {
        let dims = Dimensions::default();
        assert_eq!(dims.rows, 24);
        assert_eq!(dims.cols, 80);
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(5, 10, 20, 10);

        assert!(bounds.contains(&Position::new(5, 10))); // top-left corner
        assert!(bounds.contains(&Position::new(10, 15))); // inside
        assert!(bounds.contains(&Position::new(14, 29))); // bottom-right corner (inclusive)

        assert!(!bounds.contains(&Position::new(4, 10))); // above
        assert!(!bounds.contains(&Position::new(15, 10))); // below
        assert!(!bounds.contains(&Position::new(10, 9))); // left
        assert!(!bounds.contains(&Position::new(10, 30))); // right
    }

    #[test]
    fn test_bounds_intersects() {
        let bounds1 = Bounds::new(5, 10, 20, 10);
        let bounds2 = Bounds::new(10, 15, 10, 10); // overlaps
        let bounds3 = Bounds::new(20, 10, 10, 10); // no overlap

        assert!(bounds1.intersects(&bounds2));
        assert!(bounds2.intersects(&bounds1));
        assert!(!bounds1.intersects(&bounds3));
        assert!(!bounds3.intersects(&bounds1));
    }
}
