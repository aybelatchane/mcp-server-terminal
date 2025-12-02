//! Cell and color types for terminal grid rendering.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Terminal color supporting ANSI, 256-color palette, and true RGB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    /// Default terminal color
    Default,

    /// Standard ANSI colors (0-7)
    Black,
    /// ANSI Red
    Red,
    /// ANSI Green
    Green,
    /// ANSI Yellow
    Yellow,
    /// ANSI Blue
    Blue,
    /// ANSI Magenta
    Magenta,
    /// ANSI Cyan
    Cyan,
    /// ANSI White
    White,

    /// Bright ANSI colors (8-15)
    BrightBlack,
    /// Bright Red
    BrightRed,
    /// Bright Green
    BrightGreen,
    /// Bright Yellow
    BrightYellow,
    /// Bright Blue
    BrightBlue,
    /// Bright Magenta
    BrightMagenta,
    /// Bright Cyan
    BrightCyan,
    /// Bright White
    BrightWhite,

    /// 256-color palette index (0-255)
    Indexed(u8),

    /// True color RGB (24-bit)
    Rgb {
        /// Red component
        r: u8,
        /// Green component
        g: u8,
        /// Blue component
        b: u8,
    },
}

/// Text attributes for a terminal cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellAttributes {
    /// Bold/bright text
    pub bold: bool,
    /// Dimmed text
    pub dim: bool,
    /// Italic text
    pub italic: bool,
    /// Underlined text
    pub underline: bool,
    /// Blinking text
    pub blink: bool,
    /// Reverse video (swap fg/bg)
    pub reverse: bool,
    /// Hidden text
    pub hidden: bool,
    /// Strikethrough text
    pub strikethrough: bool,
}

impl CellAttributes {
    /// Check if attributes are all default (no formatting).
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Create attributes with bold enabled.
    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Create attributes with reverse video enabled.
    pub fn with_reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    /// Create attributes with underline enabled.
    pub fn with_underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Create attributes with italic enabled.
    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }
}

/// Single character cell in the terminal grid.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Unicode character (space if empty)
    pub character: char,
    /// Foreground color
    pub fg: Color,
    /// Background color
    pub bg: Color,
    /// Text attributes
    pub attrs: CellAttributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            fg: Color::Default,
            bg: Color::Default,
            attrs: CellAttributes::default(),
        }
    }
}

impl Cell {
    /// Create a new cell with a character and default styling.
    pub fn new(character: char) -> Self {
        Self {
            character,
            ..Default::default()
        }
    }

    /// Create a cell with character and foreground color.
    pub fn with_fg(character: char, fg: Color) -> Self {
        Self {
            character,
            fg,
            ..Default::default()
        }
    }

    /// Check if cell is empty (space with default attributes).
    pub fn is_empty(&self) -> bool {
        self.character == ' ' && self.attrs.is_default()
    }

    /// Check if cell is whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.character.is_whitespace()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_variants() {
        assert_eq!(Color::Default, Color::Default);
        assert_eq!(Color::Red, Color::Red);
        assert_eq!(Color::Indexed(42), Color::Indexed(42));
        assert_eq!(
            Color::Rgb {
                r: 255,
                g: 128,
                b: 64
            },
            Color::Rgb {
                r: 255,
                g: 128,
                b: 64
            }
        );
    }

    #[test]
    fn test_color_serialization() {
        let color = Color::Rgb {
            r: 255,
            g: 128,
            b: 0,
        };
        let json = serde_json::to_string(&color).unwrap();
        let deserialized: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(color, deserialized);
    }

    #[test]
    fn test_cell_attributes_default() {
        let attrs = CellAttributes::default();
        assert!(attrs.is_default());
        assert!(!attrs.bold);
        assert!(!attrs.italic);
        assert!(!attrs.underline);
    }

    #[test]
    fn test_cell_attributes_with_methods() {
        let attrs = CellAttributes::default().with_bold().with_underline();

        assert!(attrs.bold);
        assert!(attrs.underline);
        assert!(!attrs.italic);
        assert!(!attrs.is_default());
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.character, ' ');
        assert_eq!(cell.fg, Color::Default);
        assert_eq!(cell.bg, Color::Default);
        assert!(cell.attrs.is_default());
        assert!(cell.is_empty());
    }

    #[test]
    fn test_cell_new() {
        let cell = Cell::new('A');
        assert_eq!(cell.character, 'A');
        assert_eq!(cell.fg, Color::Default);
        assert!(!cell.is_empty());
    }

    #[test]
    fn test_cell_with_fg() {
        let cell = Cell::with_fg('B', Color::Red);
        assert_eq!(cell.character, 'B');
        assert_eq!(cell.fg, Color::Red);
        assert_eq!(cell.bg, Color::Default);
    }

    #[test]
    fn test_cell_is_empty() {
        let empty = Cell::default();
        assert!(empty.is_empty());

        let not_empty = Cell::new('X');
        assert!(!not_empty.is_empty());

        let space_with_attrs = Cell {
            character: ' ',
            attrs: CellAttributes::default().with_bold(),
            ..Default::default()
        };
        assert!(!space_with_attrs.is_empty());
    }

    #[test]
    fn test_cell_is_whitespace() {
        assert!(Cell::new(' ').is_whitespace());
        assert!(Cell::new('\t').is_whitespace());
        assert!(Cell::new('\n').is_whitespace());
        assert!(!Cell::new('A').is_whitespace());
    }
}
