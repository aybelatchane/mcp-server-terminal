//! ANSI/VT escape sequence parser using the VTE crate.

use vte::{Params, Perform};

use terminal_mcp_core::{Cell, CellAttributes, Color, Position};

use crate::grid::Grid;

/// ANSI parser wrapping VTE state machine.
#[derive(Debug)]
pub struct Parser {
    /// Terminal grid state
    grid: Grid,
}

impl Parser {
    /// Create a new parser with the given grid.
    pub fn new(grid: Grid) -> Self {
        Self { grid }
    }

    /// Get a reference to the grid.
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Get a mutable reference to the grid.
    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    /// Consume the parser and return the grid.
    pub fn into_grid(self) -> Grid {
        self.grid
    }

    /// Process bytes through the VTE parser.
    ///
    /// Returns the number of bytes consumed.
    pub fn process(&mut self, bytes: &[u8]) -> usize {
        let mut parser = vte::Parser::new();
        for byte in bytes {
            parser.advance(self, *byte);
        }
        bytes.len()
    }

    /// Move cursor forward by n columns (wrapping if needed).
    fn cursor_forward(&mut self, n: u16) {
        let dims = self.grid.dimensions();
        let cursor = self.grid.cursor_mut();

        cursor.position.col = (cursor.position.col + n).min(dims.cols.saturating_sub(1));
    }

    /// Move cursor backward by n columns.
    fn cursor_backward(&mut self, n: u16) {
        let cursor = self.grid.cursor_mut();
        cursor.position.col = cursor.position.col.saturating_sub(n);
    }

    /// Move cursor down by n rows.
    fn cursor_down(&mut self, n: u16) {
        let dims = self.grid.dimensions();
        let cursor = self.grid.cursor_mut();

        cursor.position.row = (cursor.position.row + n).min(dims.rows.saturating_sub(1));
    }

    /// Move cursor up by n rows.
    fn cursor_up(&mut self, n: u16) {
        let cursor = self.grid.cursor_mut();
        cursor.position.row = cursor.position.row.saturating_sub(n);
    }

    /// Process SGR (Select Graphic Rendition) parameters.
    fn process_sgr(&mut self, params: &Params) {
        let mut iter = params.iter();

        while let Some(param) = iter.next() {
            let code = param[0];

            match code {
                // Reset
                0 => {
                    self.grid.set_current_attrs(CellAttributes::default());
                    self.grid.set_current_fg(Color::Default);
                    self.grid.set_current_bg(Color::Default);
                }

                // Bold
                1 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.bold = true;
                    self.grid.set_current_attrs(attrs);
                }

                // Dim
                2 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.dim = true;
                    self.grid.set_current_attrs(attrs);
                }

                // Italic
                3 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.italic = true;
                    self.grid.set_current_attrs(attrs);
                }

                // Underline
                4 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.underline = true;
                    self.grid.set_current_attrs(attrs);
                }

                // Blink
                5 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.blink = true;
                    self.grid.set_current_attrs(attrs);
                }

                // Reverse
                7 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.reverse = true;
                    self.grid.set_current_attrs(attrs);
                }

                // Hidden
                8 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.hidden = true;
                    self.grid.set_current_attrs(attrs);
                }

                // Strikethrough
                9 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.strikethrough = true;
                    self.grid.set_current_attrs(attrs);
                }

                // Normal intensity (not bold/dim)
                22 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.bold = false;
                    attrs.dim = false;
                    self.grid.set_current_attrs(attrs);
                }

                // Not italic
                23 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.italic = false;
                    self.grid.set_current_attrs(attrs);
                }

                // Not underlined
                24 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.underline = false;
                    self.grid.set_current_attrs(attrs);
                }

                // Not blinking
                25 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.blink = false;
                    self.grid.set_current_attrs(attrs);
                }

                // Not reversed
                27 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.reverse = false;
                    self.grid.set_current_attrs(attrs);
                }

                // Not hidden
                28 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.hidden = false;
                    self.grid.set_current_attrs(attrs);
                }

                // Not strikethrough
                29 => {
                    let mut attrs = *self.grid.current_attrs();
                    attrs.strikethrough = false;
                    self.grid.set_current_attrs(attrs);
                }

                // Foreground colors (30-37)
                30 => self.grid.set_current_fg(Color::Black),
                31 => self.grid.set_current_fg(Color::Red),
                32 => self.grid.set_current_fg(Color::Green),
                33 => self.grid.set_current_fg(Color::Yellow),
                34 => self.grid.set_current_fg(Color::Blue),
                35 => self.grid.set_current_fg(Color::Magenta),
                36 => self.grid.set_current_fg(Color::Cyan),
                37 => self.grid.set_current_fg(Color::White),

                // Default foreground
                39 => self.grid.set_current_fg(Color::Default),

                // Background colors (40-47)
                40 => self.grid.set_current_bg(Color::Black),
                41 => self.grid.set_current_bg(Color::Red),
                42 => self.grid.set_current_bg(Color::Green),
                43 => self.grid.set_current_bg(Color::Yellow),
                44 => self.grid.set_current_bg(Color::Blue),
                45 => self.grid.set_current_bg(Color::Magenta),
                46 => self.grid.set_current_bg(Color::Cyan),
                47 => self.grid.set_current_bg(Color::White),

                // Default background
                49 => self.grid.set_current_bg(Color::Default),

                // Bright foreground colors (90-97)
                90 => self.grid.set_current_fg(Color::BrightBlack),
                91 => self.grid.set_current_fg(Color::BrightRed),
                92 => self.grid.set_current_fg(Color::BrightGreen),
                93 => self.grid.set_current_fg(Color::BrightYellow),
                94 => self.grid.set_current_fg(Color::BrightBlue),
                95 => self.grid.set_current_fg(Color::BrightMagenta),
                96 => self.grid.set_current_fg(Color::BrightCyan),
                97 => self.grid.set_current_fg(Color::BrightWhite),

                // Bright background colors (100-107)
                100 => self.grid.set_current_bg(Color::BrightBlack),
                101 => self.grid.set_current_bg(Color::BrightRed),
                102 => self.grid.set_current_bg(Color::BrightGreen),
                103 => self.grid.set_current_bg(Color::BrightYellow),
                104 => self.grid.set_current_bg(Color::BrightBlue),
                105 => self.grid.set_current_bg(Color::BrightMagenta),
                106 => self.grid.set_current_bg(Color::BrightCyan),
                107 => self.grid.set_current_bg(Color::BrightWhite),

                // 256-color foreground (38;5;n)
                38 => {
                    if let Some(next) = iter.next() {
                        if next[0] == 5 {
                            if let Some(color_param) = iter.next() {
                                let color_idx = color_param[0] as u8;
                                self.grid.set_current_fg(Color::Indexed(color_idx));
                            }
                        } else if next[0] == 2 {
                            // RGB foreground (38;2;r;g;b)
                            if let (Some(r), Some(g), Some(b)) =
                                (iter.next(), iter.next(), iter.next())
                            {
                                self.grid.set_current_fg(Color::Rgb {
                                    r: r[0] as u8,
                                    g: g[0] as u8,
                                    b: b[0] as u8,
                                });
                            }
                        }
                    }
                }

                // 256-color background (48;5;n)
                48 => {
                    if let Some(next) = iter.next() {
                        if next[0] == 5 {
                            if let Some(color_param) = iter.next() {
                                let color_idx = color_param[0] as u8;
                                self.grid.set_current_bg(Color::Indexed(color_idx));
                            }
                        } else if next[0] == 2 {
                            // RGB background (48;2;r;g;b)
                            if let (Some(r), Some(g), Some(b)) =
                                (iter.next(), iter.next(), iter.next())
                            {
                                self.grid.set_current_bg(Color::Rgb {
                                    r: r[0] as u8,
                                    g: g[0] as u8,
                                    b: b[0] as u8,
                                });
                            }
                        }
                    }
                }

                _ => {} // Ignore unknown SGR codes
            }
        }
    }
}

impl Perform for Parser {
    /// Print a character to the terminal.
    fn print(&mut self, c: char) {
        let cursor_pos = self.grid.cursor().position;
        let dims = self.grid.dimensions();

        // Get current attributes and colors before borrowing cell mutably
        let attrs = *self.grid.current_attrs();
        let fg = self.grid.current_fg();
        let bg = self.grid.current_bg();

        // Get or create cell at cursor position
        if let Some(cell) = self.grid.cell_mut(cursor_pos.row, cursor_pos.col) {
            cell.character = c;
            cell.attrs = attrs;
            cell.fg = fg;
            cell.bg = bg;
        }

        // Move cursor forward (with wrapping)
        let cursor = self.grid.cursor_mut();
        cursor.position.col += 1;

        if cursor.position.col >= dims.cols {
            cursor.position.col = 0;
            cursor.position.row = (cursor.position.row + 1).min(dims.rows.saturating_sub(1));
        }
    }

    /// Execute a control character.
    fn execute(&mut self, byte: u8) {
        match byte {
            // Backspace (BS)
            0x08 => {
                self.cursor_backward(1);
            }

            // Horizontal Tab (HT)
            0x09 => {
                let dims = self.grid.dimensions();
                let cursor = self.grid.cursor_mut();
                // Move to next tab stop (every 8 columns)
                let next_tab = ((cursor.position.col / 8) + 1) * 8;
                cursor.position.col = next_tab.min(dims.cols.saturating_sub(1));
            }

            // Line Feed (LF)
            0x0A => {
                self.cursor_down(1);
            }

            // Carriage Return (CR)
            0x0D => {
                self.grid.cursor_mut().position.col = 0;
            }

            _ => {} // Ignore other control codes for now
        }
    }

    /// Hook into DCS (Device Control String) - not implemented yet.
    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {
        // Not implemented
    }

    /// Put data into current DCS - not implemented yet.
    fn put(&mut self, _byte: u8) {
        // Not implemented
    }

    /// Unhook from DCS - not implemented yet.
    fn unhook(&mut self) {
        // Not implemented
    }

    /// OSC (Operating System Command) dispatch.
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // Not implemented yet - will handle title changes, etc.
    }

    /// CSI (Control Sequence Introducer) dispatch.
    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            // Cursor Up (CUU)
            'A' => {
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1);
                self.cursor_up(n);
            }

            // Cursor Down (CUD)
            'B' => {
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1);
                self.cursor_down(n);
            }

            // Cursor Forward (CUF)
            'C' => {
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1);
                self.cursor_forward(n);
            }

            // Cursor Backward (CUB)
            'D' => {
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1);
                self.cursor_backward(n);
            }

            // Cursor Position (CUP)
            'H' => {
                let mut iter = params.iter();
                let row = iter.next().map(|p| p[0]).unwrap_or(1).saturating_sub(1);
                let col = iter.next().map(|p| p[0]).unwrap_or(1).saturating_sub(1);

                let dims = self.grid.dimensions();
                self.grid.cursor_mut().position = Position::new(
                    row.min(dims.rows.saturating_sub(1)),
                    col.min(dims.cols.saturating_sub(1)),
                );
            }

            // Erase in Display (ED)
            'J' => {
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                let cursor_pos = self.grid.cursor().position;
                let dims = self.grid.dimensions();

                match mode {
                    // Clear from cursor to end of screen
                    0 => {
                        // Clear rest of current row
                        for col in cursor_pos.col..dims.cols {
                            if let Some(cell) = self.grid.cell_mut(cursor_pos.row, col) {
                                *cell = Cell::default();
                            }
                        }

                        // Clear all rows below
                        for row in (cursor_pos.row + 1)..dims.rows {
                            for col in 0..dims.cols {
                                if let Some(cell) = self.grid.cell_mut(row, col) {
                                    *cell = Cell::default();
                                }
                            }
                        }
                    }

                    // Clear from cursor to beginning of screen
                    1 => {
                        // Clear all rows above
                        for row in 0..cursor_pos.row {
                            for col in 0..dims.cols {
                                if let Some(cell) = self.grid.cell_mut(row, col) {
                                    *cell = Cell::default();
                                }
                            }
                        }

                        // Clear from start of current row to cursor
                        for col in 0..=cursor_pos.col {
                            if let Some(cell) = self.grid.cell_mut(cursor_pos.row, col) {
                                *cell = Cell::default();
                            }
                        }
                    }

                    // Clear entire screen
                    2 | 3 => {
                        self.grid.clear();
                    }

                    _ => {}
                }
            }

            // Erase in Line (EL)
            'K' => {
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                let cursor_pos = self.grid.cursor().position;
                let dims = self.grid.dimensions();

                match mode {
                    // Clear from cursor to end of line
                    0 => {
                        for col in cursor_pos.col..dims.cols {
                            if let Some(cell) = self.grid.cell_mut(cursor_pos.row, col) {
                                *cell = Cell::default();
                            }
                        }
                    }

                    // Clear from start of line to cursor
                    1 => {
                        for col in 0..=cursor_pos.col {
                            if let Some(cell) = self.grid.cell_mut(cursor_pos.row, col) {
                                *cell = Cell::default();
                            }
                        }
                    }

                    // Clear entire line
                    2 => {
                        for col in 0..dims.cols {
                            if let Some(cell) = self.grid.cell_mut(cursor_pos.row, col) {
                                *cell = Cell::default();
                            }
                        }
                    }

                    _ => {}
                }
            }

            // SGR (Select Graphic Rendition)
            'm' => {
                self.process_sgr(params);
            }

            // Save Cursor Position (SCP)
            's' => {
                self.grid.save_cursor();
            }

            // Restore Cursor Position (RCP)
            'u' => {
                self.grid.restore_cursor();
            }

            _ => {} // Ignore unknown CSI sequences
        }
    }

    /// ESC (Escape) dispatch.
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // Not implemented yet - will handle things like DECSC, DECRC
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_mcp_core::Dimensions;

    #[test]
    fn test_parser_print() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        // Print some characters
        parser.print('H');
        parser.print('e');
        parser.print('l');
        parser.print('l');
        parser.print('o');

        // Check cells
        assert_eq!(parser.grid().cell(0, 0).unwrap().character, 'H');
        assert_eq!(parser.grid().cell(0, 1).unwrap().character, 'e');
        assert_eq!(parser.grid().cell(0, 2).unwrap().character, 'l');
        assert_eq!(parser.grid().cell(0, 3).unwrap().character, 'l');
        assert_eq!(parser.grid().cell(0, 4).unwrap().character, 'o');

        // Cursor should be at column 5
        assert_eq!(parser.grid().cursor().position.col, 5);
    }

    #[test]
    fn test_parser_process_basic() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        let bytes = b"Hello, World!";
        let consumed = parser.process(bytes);

        assert_eq!(consumed, bytes.len());

        // Check text
        let text = parser
            .grid()
            .row(0)
            .unwrap()
            .iter()
            .take(13)
            .map(|c| c.character)
            .collect::<String>();

        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_parser_execute_lf() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        assert_eq!(parser.grid().cursor().position.row, 0);

        parser.execute(0x0A); // LF
        assert_eq!(parser.grid().cursor().position.row, 1);

        parser.execute(0x0A); // LF
        assert_eq!(parser.grid().cursor().position.row, 2);
    }

    #[test]
    fn test_parser_execute_cr() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        // Move cursor
        parser.cursor_forward(10);
        assert_eq!(parser.grid().cursor().position.col, 10);

        parser.execute(0x0D); // CR
        assert_eq!(parser.grid().cursor().position.col, 0);
    }

    #[test]
    fn test_parser_execute_bs() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        parser.cursor_forward(5);
        assert_eq!(parser.grid().cursor().position.col, 5);

        parser.execute(0x08); // BS
        assert_eq!(parser.grid().cursor().position.col, 4);
    }

    #[test]
    fn test_parser_execute_tab() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        assert_eq!(parser.grid().cursor().position.col, 0);

        parser.execute(0x09); // HT
        assert_eq!(parser.grid().cursor().position.col, 8);

        parser.execute(0x09); // HT
        assert_eq!(parser.grid().cursor().position.col, 16);
    }

    #[test]
    fn test_parser_csi_cursor_up() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        // Move cursor down first
        parser.cursor_down(10);
        assert_eq!(parser.grid().cursor().position.row, 10);

        // Test CSI A (cursor up)
        parser.process(b"\x1b[5A");
        assert_eq!(parser.grid().cursor().position.row, 5);
    }

    #[test]
    fn test_parser_csi_cursor_position() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        // Move to row 10, col 20 (1-indexed in escape sequence)
        parser.process(b"\x1b[11;21H");

        assert_eq!(parser.grid().cursor().position.row, 10);
        assert_eq!(parser.grid().cursor().position.col, 20);
    }

    #[test]
    fn test_parser_sgr_colors() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        // Set red foreground
        parser.process(b"\x1b[31mX");

        let cell = parser.grid().cell(0, 0).unwrap();
        assert_eq!(cell.character, 'X');
        assert_eq!(cell.fg, Color::Red);
    }

    #[test]
    fn test_parser_sgr_attributes() {
        let grid = Grid::new(Dimensions::new(24, 80));
        let mut parser = Parser::new(grid);

        // Set bold and underline
        parser.process(b"\x1b[1;4mX");

        let cell = parser.grid().cell(0, 0).unwrap();
        assert_eq!(cell.character, 'X');
        assert!(cell.attrs.bold);
        assert!(cell.attrs.underline);
    }

    #[test]
    fn test_parser_erase_in_display() {
        let grid = Grid::new(Dimensions::new(5, 10));
        let mut parser = Parser::new(grid);

        // Fill grid with 'X'
        for _ in 0..50 {
            parser.print('X');
        }

        // Move cursor to middle
        parser.grid_mut().cursor_mut().position = Position::new(2, 5);

        // Erase from cursor to end
        parser.process(b"\x1b[J");

        // Check that cells before cursor still have 'X'
        assert_eq!(parser.grid().cell(0, 0).unwrap().character, 'X');
        assert_eq!(parser.grid().cell(2, 4).unwrap().character, 'X');

        // Check that cells from cursor onward are cleared
        assert_eq!(parser.grid().cell(2, 5).unwrap().character, ' ');
        assert_eq!(parser.grid().cell(4, 9).unwrap().character, ' ');
    }

    #[test]
    fn test_parser_erase_in_line() {
        let grid = Grid::new(Dimensions::new(5, 10));
        let mut parser = Parser::new(grid);

        // Fill first row
        for _ in 0..10 {
            parser.print('X');
        }

        // Move cursor to middle of first row
        parser.grid_mut().cursor_mut().position = Position::new(0, 5);

        // Erase from cursor to end of line
        parser.process(b"\x1b[K");

        // Check before cursor
        assert_eq!(parser.grid().cell(0, 4).unwrap().character, 'X');

        // Check from cursor onward
        assert_eq!(parser.grid().cell(0, 5).unwrap().character, ' ');
        assert_eq!(parser.grid().cell(0, 9).unwrap().character, ' ');
    }
}
