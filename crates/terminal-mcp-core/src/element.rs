//! Element types for Terminal State Tree (TST).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Bounds, Dimensions, Position};

/// Menu item within a menu element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MenuItem {
    /// Reference ID for this item
    pub ref_id: String,
    /// Display text
    pub text: String,
    /// Whether this item is currently selected
    pub selected: bool,
}

impl MenuItem {
    /// Create a new menu item.
    pub fn new(ref_id: impl Into<String>, text: impl Into<String>, selected: bool) -> Self {
        Self {
            ref_id: ref_id.into(),
            text: text.into(),
            selected,
        }
    }
}

/// Detected UI element within the terminal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Element {
    /// Vertical or horizontal menu with selectable items
    Menu {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Menu items
        items: Vec<MenuItem>,
        /// Index of selected item
        selected: usize,
    },

    /// Data table with headers and rows
    Table {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Column headers
        headers: Vec<String>,
        /// Table rows (each row is a vec of cell values)
        rows: Vec<Vec<String>>,
    },

    /// Text input field
    Input {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Current input value
        value: String,
        /// Cursor position within value
        cursor_pos: usize,
    },

    /// Clickable button
    Button {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Button label
        label: String,
    },

    /// Progress indicator
    ProgressBar {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Progress percentage (0-100)
        percent: u8,
    },

    /// Checkbox or toggle
    Checkbox {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Label text
        label: String,
        /// Checked state
        checked: bool,
    },

    /// Status bar (typically at bottom)
    StatusBar {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Status content
        content: String,
    },

    /// Bordered region containing other elements
    Border {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Optional title
        title: Option<String>,
        /// Reference IDs of contained elements
        children: Vec<String>,
    },

    /// Generic text region
    Text {
        /// Unique reference ID
        ref_id: String,
        /// Bounding box
        bounds: Bounds,
        /// Text content
        content: String,
    },
}

impl Element {
    /// Get the reference ID of this element.
    pub fn ref_id(&self) -> &str {
        match self {
            Element::Menu { ref_id, .. } => ref_id,
            Element::Table { ref_id, .. } => ref_id,
            Element::Input { ref_id, .. } => ref_id,
            Element::Button { ref_id, .. } => ref_id,
            Element::ProgressBar { ref_id, .. } => ref_id,
            Element::Checkbox { ref_id, .. } => ref_id,
            Element::StatusBar { ref_id, .. } => ref_id,
            Element::Border { ref_id, .. } => ref_id,
            Element::Text { ref_id, .. } => ref_id,
        }
    }

    /// Get the element type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Element::Menu { .. } => "menu",
            Element::Table { .. } => "table",
            Element::Input { .. } => "input",
            Element::Button { .. } => "button",
            Element::ProgressBar { .. } => "progress_bar",
            Element::Checkbox { .. } => "checkbox",
            Element::StatusBar { .. } => "status_bar",
            Element::Border { .. } => "border",
            Element::Text { .. } => "text",
        }
    }

    /// Get the bounds of this element.
    pub fn bounds(&self) -> &Bounds {
        match self {
            Element::Menu { bounds, .. } => bounds,
            Element::Table { bounds, .. } => bounds,
            Element::Input { bounds, .. } => bounds,
            Element::Button { bounds, .. } => bounds,
            Element::ProgressBar { bounds, .. } => bounds,
            Element::Checkbox { bounds, .. } => bounds,
            Element::StatusBar { bounds, .. } => bounds,
            Element::Border { bounds, .. } => bounds,
            Element::Text { bounds, .. } => bounds,
        }
    }
}

/// Terminal State Tree - structured snapshot of terminal content.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TerminalStateTree {
    /// Session identifier
    pub session_id: String,
    /// Terminal dimensions
    pub dimensions: Dimensions,
    /// Current cursor position
    pub cursor: Position,
    /// Snapshot timestamp (ISO 8601)
    pub timestamp: String,
    /// Detected UI elements
    pub elements: Vec<Element>,
    /// Raw text content (stripped of formatting)
    pub raw_text: String,
    /// Raw ANSI buffer (optional, for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ansi_buffer: Option<String>,
}

impl TerminalStateTree {
    /// Find element by reference ID.
    pub fn find_element(&self, ref_id: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.ref_id() == ref_id)
    }

    /// Get all elements of a specific type.
    pub fn elements_of_type(&self, element_type: &str) -> Vec<&Element> {
        self.elements
            .iter()
            .filter(|e| e.type_name() == element_type)
            .collect()
    }

    /// Get all menu elements.
    pub fn menus(&self) -> Vec<&Element> {
        self.elements_of_type("menu")
    }

    /// Get all table elements.
    pub fn tables(&self) -> Vec<&Element> {
        self.elements_of_type("table")
    }

    /// Get all input elements.
    pub fn inputs(&self) -> Vec<&Element> {
        self.elements_of_type("input")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::new("item1", "Option 1", true);
        assert_eq!(item.ref_id, "item1");
        assert_eq!(item.text, "Option 1");
        assert!(item.selected);
    }

    #[test]
    fn test_element_ref_id() {
        let button = Element::Button {
            ref_id: "btn1".to_string(),
            bounds: Bounds::new(0, 0, 10, 1),
            label: "Click me".to_string(),
        };
        assert_eq!(button.ref_id(), "btn1");
    }

    #[test]
    fn test_element_type_name() {
        let input = Element::Input {
            ref_id: "input1".to_string(),
            bounds: Bounds::new(0, 0, 20, 1),
            value: "test".to_string(),
            cursor_pos: 4,
        };
        assert_eq!(input.type_name(), "input");
    }

    #[test]
    fn test_element_bounds() {
        let table = Element::Table {
            ref_id: "table1".to_string(),
            bounds: Bounds::new(5, 10, 30, 15),
            headers: vec!["Col1".to_string(), "Col2".to_string()],
            rows: vec![],
        };
        assert_eq!(table.bounds(), &Bounds::new(5, 10, 30, 15));
    }

    #[test]
    fn test_tst_find_element() {
        let button = Element::Button {
            ref_id: "btn1".to_string(),
            bounds: Bounds::new(0, 0, 10, 1),
            label: "Click".to_string(),
        };

        let input = Element::Input {
            ref_id: "input1".to_string(),
            bounds: Bounds::new(0, 2, 20, 1),
            value: "".to_string(),
            cursor_pos: 0,
        };

        let tst = TerminalStateTree {
            session_id: "sess1".to_string(),
            dimensions: Dimensions::new(24, 80),
            cursor: Position::new(0, 0),
            timestamp: "2025-11-29T00:00:00Z".to_string(),
            elements: vec![button, input],
            raw_text: "".to_string(),
            ansi_buffer: None,
        };

        assert!(tst.find_element("btn1").is_some());
        assert!(tst.find_element("input1").is_some());
        assert!(tst.find_element("nonexistent").is_none());
    }

    #[test]
    fn test_tst_elements_of_type() {
        let button1 = Element::Button {
            ref_id: "btn1".to_string(),
            bounds: Bounds::new(0, 0, 10, 1),
            label: "Button 1".to_string(),
        };

        let button2 = Element::Button {
            ref_id: "btn2".to_string(),
            bounds: Bounds::new(0, 2, 10, 1),
            label: "Button 2".to_string(),
        };

        let input = Element::Input {
            ref_id: "input1".to_string(),
            bounds: Bounds::new(0, 4, 20, 1),
            value: "".to_string(),
            cursor_pos: 0,
        };

        let tst = TerminalStateTree {
            session_id: "sess1".to_string(),
            dimensions: Dimensions::new(24, 80),
            cursor: Position::new(0, 0),
            timestamp: "2025-11-29T00:00:00Z".to_string(),
            elements: vec![button1, button2, input],
            raw_text: "".to_string(),
            ansi_buffer: None,
        };

        let buttons = tst.elements_of_type("button");
        assert_eq!(buttons.len(), 2);

        let inputs = tst.elements_of_type("input");
        assert_eq!(inputs.len(), 1);
    }

    #[test]
    fn test_element_serialization() {
        let menu = Element::Menu {
            ref_id: "menu1".to_string(),
            bounds: Bounds::new(0, 0, 20, 5),
            items: vec![
                MenuItem::new("item1", "Option 1", true),
                MenuItem::new("item2", "Option 2", false),
            ],
            selected: 0,
        };

        let json = serde_json::to_string(&menu).unwrap();
        let deserialized: Element = serde_json::from_str(&json).unwrap();
        assert_eq!(menu, deserialized);
    }
}
