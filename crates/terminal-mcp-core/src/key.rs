//! Keyboard input types for terminal interaction.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Keyboard key for terminal input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    // Character keys
    /// Regular character
    Char(char),

    // Navigation
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up
    PageUp,
    /// Page Down
    PageDown,

    // Actions
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Escape key
    Escape,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Space key
    Space,
    /// Insert key
    Insert,

    // Function keys
    /// F1
    F1,
    /// F2
    F2,
    /// F3
    F3,
    /// F4
    F4,
    /// F5
    F5,
    /// F6
    F6,
    /// F7
    F7,
    /// F8
    F8,
    /// F9
    F9,
    /// F10
    F10,
    /// F11
    F11,
    /// F12
    F12,

    // Modified keys
    /// Ctrl + character
    Ctrl(char),
    /// Alt + character
    Alt(char),
    /// Shift + key
    Shift(Box<Key>),
    /// Ctrl + Alt + character
    CtrlAlt(char),
}

impl Key {
    /// Parse key from string representation.
    ///
    /// Examples:
    /// - "a" -> Key::Char('a')
    /// - "Ctrl+c" -> Key::Ctrl('c')
    /// - "Alt+f" -> Key::Alt('f')
    /// - "Enter" -> Key::Enter
    /// - "Up" -> Key::Up
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        // Handle modifiers
        if let Some(rest) = s.strip_prefix("Ctrl+") {
            let ch = rest
                .chars()
                .next()
                .ok_or_else(|| Error::InvalidInput(format!("Invalid Ctrl+ key: {s}")))?;
            return Ok(Key::Ctrl(ch.to_ascii_lowercase()));
        }

        if let Some(rest) = s.strip_prefix("Alt+") {
            let ch = rest
                .chars()
                .next()
                .ok_or_else(|| Error::InvalidInput(format!("Invalid Alt+ key: {s}")))?;
            return Ok(Key::Alt(ch));
        }

        if let Some(rest) = s.strip_prefix("Shift+") {
            let inner = Key::parse(rest)?;
            return Ok(Key::Shift(Box::new(inner)));
        }

        // Handle named keys
        match s {
            "Enter" | "Return" => Ok(Key::Enter),
            "Tab" => Ok(Key::Tab),
            "Escape" | "Esc" => Ok(Key::Escape),
            "Backspace" => Ok(Key::Backspace),
            "Delete" | "Del" => Ok(Key::Delete),
            "Space" => Ok(Key::Space),
            "Insert" | "Ins" => Ok(Key::Insert),
            "Up" => Ok(Key::Up),
            "Down" => Ok(Key::Down),
            "Left" => Ok(Key::Left),
            "Right" => Ok(Key::Right),
            "Home" => Ok(Key::Home),
            "End" => Ok(Key::End),
            "PageUp" | "PgUp" => Ok(Key::PageUp),
            "PageDown" | "PgDn" => Ok(Key::PageDown),
            "F1" => Ok(Key::F1),
            "F2" => Ok(Key::F2),
            "F3" => Ok(Key::F3),
            "F4" => Ok(Key::F4),
            "F5" => Ok(Key::F5),
            "F6" => Ok(Key::F6),
            "F7" => Ok(Key::F7),
            "F8" => Ok(Key::F8),
            "F9" => Ok(Key::F9),
            "F10" => Ok(Key::F10),
            "F11" => Ok(Key::F11),
            "F12" => Ok(Key::F12),
            _ => {
                // Single character
                if s.len() == 1 {
                    Ok(Key::Char(s.chars().next().unwrap()))
                } else {
                    Err(Error::InvalidInput(format!("Unknown key: {s}")))
                }
            }
        }
    }

    /// Convert key to terminal escape sequence bytes.
    pub fn to_escape_sequence(&self) -> Vec<u8> {
        match self {
            Key::Char(c) => c.to_string().into_bytes(),
            Key::Enter => vec![0x0D], // CR
            Key::Tab => vec![0x09],
            Key::Escape => vec![0x1B],
            Key::Backspace => vec![0x7F],
            Key::Delete => b"\x1b[3~".to_vec(),
            Key::Space => vec![0x20],
            Key::Insert => b"\x1b[2~".to_vec(),
            Key::Up => b"\x1b[A".to_vec(),
            Key::Down => b"\x1b[B".to_vec(),
            Key::Right => b"\x1b[C".to_vec(),
            Key::Left => b"\x1b[D".to_vec(),
            Key::Home => b"\x1b[H".to_vec(),
            Key::End => b"\x1b[F".to_vec(),
            Key::PageUp => b"\x1b[5~".to_vec(),
            Key::PageDown => b"\x1b[6~".to_vec(),
            Key::F1 => b"\x1bOP".to_vec(),
            Key::F2 => b"\x1bOQ".to_vec(),
            Key::F3 => b"\x1bOR".to_vec(),
            Key::F4 => b"\x1bOS".to_vec(),
            Key::F5 => b"\x1b[15~".to_vec(),
            Key::F6 => b"\x1b[17~".to_vec(),
            Key::F7 => b"\x1b[18~".to_vec(),
            Key::F8 => b"\x1b[19~".to_vec(),
            Key::F9 => b"\x1b[20~".to_vec(),
            Key::F10 => b"\x1b[21~".to_vec(),
            Key::F11 => b"\x1b[23~".to_vec(),
            Key::F12 => b"\x1b[24~".to_vec(),
            Key::Ctrl(c) => {
                // Ctrl+A = 0x01, Ctrl+Z = 0x1A
                let code = (*c as u8).to_ascii_lowercase() - b'a' + 1;
                vec![code]
            }
            Key::Alt(c) => {
                // Alt sends ESC prefix
                let mut seq = vec![0x1B];
                seq.extend(c.to_string().bytes());
                seq
            }
            Key::Shift(inner) => {
                // Shift modifies escape sequences
                match inner.as_ref() {
                    Key::Tab => b"\x1b[Z".to_vec(), // Shift+Tab
                    Key::Up => b"\x1b[1;2A".to_vec(),
                    Key::Down => b"\x1b[1;2B".to_vec(),
                    Key::Right => b"\x1b[1;2C".to_vec(),
                    Key::Left => b"\x1b[1;2D".to_vec(),
                    _ => inner.to_escape_sequence(),
                }
            }
            Key::CtrlAlt(c) => {
                // Ctrl+Alt sends ESC + Ctrl code
                let code = (*c as u8).to_ascii_lowercase() - b'a' + 1;
                vec![0x1B, code]
            }
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Char(c) => write!(f, "{c}"),
            Key::Up => write!(f, "Up"),
            Key::Down => write!(f, "Down"),
            Key::Left => write!(f, "Left"),
            Key::Right => write!(f, "Right"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
            Key::Enter => write!(f, "Enter"),
            Key::Tab => write!(f, "Tab"),
            Key::Escape => write!(f, "Escape"),
            Key::Backspace => write!(f, "Backspace"),
            Key::Delete => write!(f, "Delete"),
            Key::Space => write!(f, "Space"),
            Key::Insert => write!(f, "Insert"),
            Key::F1 => write!(f, "F1"),
            Key::F2 => write!(f, "F2"),
            Key::F3 => write!(f, "F3"),
            Key::F4 => write!(f, "F4"),
            Key::F5 => write!(f, "F5"),
            Key::F6 => write!(f, "F6"),
            Key::F7 => write!(f, "F7"),
            Key::F8 => write!(f, "F8"),
            Key::F9 => write!(f, "F9"),
            Key::F10 => write!(f, "F10"),
            Key::F11 => write!(f, "F11"),
            Key::F12 => write!(f, "F12"),
            Key::Ctrl(c) => write!(f, "Ctrl+{c}"),
            Key::Alt(c) => write!(f, "Alt+{c}"),
            Key::Shift(k) => write!(f, "Shift+{k}"),
            Key::CtrlAlt(c) => write!(f, "Ctrl+Alt+{c}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_parse_char() {
        assert_eq!(Key::parse("a").unwrap(), Key::Char('a'));
        assert_eq!(Key::parse("Z").unwrap(), Key::Char('Z'));
        assert_eq!(Key::parse("5").unwrap(), Key::Char('5'));
    }

    #[test]
    fn test_key_parse_named() {
        assert_eq!(Key::parse("Enter").unwrap(), Key::Enter);
        assert_eq!(Key::parse("Return").unwrap(), Key::Enter);
        assert_eq!(Key::parse("Tab").unwrap(), Key::Tab);
        assert_eq!(Key::parse("Escape").unwrap(), Key::Escape);
        assert_eq!(Key::parse("Esc").unwrap(), Key::Escape);
        assert_eq!(Key::parse("Up").unwrap(), Key::Up);
        assert_eq!(Key::parse("Down").unwrap(), Key::Down);
        assert_eq!(Key::parse("F1").unwrap(), Key::F1);
        assert_eq!(Key::parse("F12").unwrap(), Key::F12);
    }

    #[test]
    fn test_key_parse_ctrl() {
        assert_eq!(Key::parse("Ctrl+c").unwrap(), Key::Ctrl('c'));
        assert_eq!(Key::parse("Ctrl+C").unwrap(), Key::Ctrl('c')); // Normalized to lowercase
        assert_eq!(Key::parse("Ctrl+a").unwrap(), Key::Ctrl('a'));
    }

    #[test]
    fn test_key_parse_alt() {
        assert_eq!(Key::parse("Alt+f").unwrap(), Key::Alt('f'));
        assert_eq!(Key::parse("Alt+x").unwrap(), Key::Alt('x'));
    }

    #[test]
    fn test_key_parse_shift() {
        assert_eq!(
            Key::parse("Shift+Tab").unwrap(),
            Key::Shift(Box::new(Key::Tab))
        );
        assert_eq!(
            Key::parse("Shift+Up").unwrap(),
            Key::Shift(Box::new(Key::Up))
        );
    }

    #[test]
    fn test_key_parse_invalid() {
        assert!(Key::parse("InvalidKey").is_err());
        assert!(Key::parse("Ctrl+").is_err());
        assert!(Key::parse("Alt+").is_err());
    }

    #[test]
    fn test_key_to_escape_sequence_char() {
        assert_eq!(Key::Char('a').to_escape_sequence(), b"a");
        assert_eq!(Key::Char('Z').to_escape_sequence(), b"Z");
    }

    #[test]
    fn test_key_to_escape_sequence_special() {
        assert_eq!(Key::Enter.to_escape_sequence(), vec![0x0D]);
        assert_eq!(Key::Tab.to_escape_sequence(), vec![0x09]);
        assert_eq!(Key::Escape.to_escape_sequence(), vec![0x1B]);
        assert_eq!(Key::Backspace.to_escape_sequence(), vec![0x7F]);
        assert_eq!(Key::Space.to_escape_sequence(), vec![0x20]);
    }

    #[test]
    fn test_key_to_escape_sequence_arrows() {
        assert_eq!(Key::Up.to_escape_sequence(), b"\x1b[A");
        assert_eq!(Key::Down.to_escape_sequence(), b"\x1b[B");
        assert_eq!(Key::Right.to_escape_sequence(), b"\x1b[C");
        assert_eq!(Key::Left.to_escape_sequence(), b"\x1b[D");
    }

    #[test]
    fn test_key_to_escape_sequence_function() {
        assert_eq!(Key::F1.to_escape_sequence(), b"\x1bOP");
        assert_eq!(Key::F2.to_escape_sequence(), b"\x1bOQ");
        assert_eq!(Key::F5.to_escape_sequence(), b"\x1b[15~");
        assert_eq!(Key::F12.to_escape_sequence(), b"\x1b[24~");
    }

    #[test]
    fn test_key_to_escape_sequence_ctrl() {
        // Ctrl+A = 0x01
        assert_eq!(Key::Ctrl('a').to_escape_sequence(), vec![0x01]);
        // Ctrl+C = 0x03
        assert_eq!(Key::Ctrl('c').to_escape_sequence(), vec![0x03]);
        // Ctrl+Z = 0x1A
        assert_eq!(Key::Ctrl('z').to_escape_sequence(), vec![0x1A]);
    }

    #[test]
    fn test_key_to_escape_sequence_alt() {
        // Alt sends ESC + character
        assert_eq!(Key::Alt('f').to_escape_sequence(), b"\x1bf");
        assert_eq!(Key::Alt('x').to_escape_sequence(), b"\x1bx");
    }

    #[test]
    fn test_key_to_escape_sequence_shift() {
        // Shift+Tab
        assert_eq!(
            Key::Shift(Box::new(Key::Tab)).to_escape_sequence(),
            b"\x1b[Z"
        );
        // Shift+Up
        assert_eq!(
            Key::Shift(Box::new(Key::Up)).to_escape_sequence(),
            b"\x1b[1;2A"
        );
    }

    #[test]
    fn test_key_serialization() {
        let key = Key::Ctrl('c');
        let json = serde_json::to_string(&key).unwrap();
        let deserialized: Key = serde_json::from_str(&json).unwrap();
        assert_eq!(key, deserialized);
    }

    #[test]
    fn test_key_parse_round_trip() {
        let test_cases = vec![
            "a",
            "Enter",
            "Tab",
            "Up",
            "F1",
            "Ctrl+c",
            "Alt+f",
            "Shift+Tab",
        ];

        for case in test_cases {
            let key = Key::parse(case).unwrap();
            // Verify it doesn't panic
            let _ = key.to_escape_sequence();
        }
    }
}
