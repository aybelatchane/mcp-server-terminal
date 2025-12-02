//! Navigation calculator for determining keystrokes to reach target elements.

use terminal_mcp_core::{Element, Error, Key, MenuItem, Result, TerminalStateTree};

/// Navigation calculator for click operations.
///
/// Calculates the sequence of keystrokes needed to navigate to and activate
/// a target element in the terminal.
#[derive(Debug, Clone, Default)]
pub struct NavigationCalculator;

impl NavigationCalculator {
    /// Create a new navigation calculator.
    pub fn new() -> Self {
        Self
    }

    /// Calculate keystrokes to reach and activate target element.
    ///
    /// # Arguments
    /// * `snapshot` - Current terminal state tree
    /// * `target_ref` - Reference ID of the element to click
    ///
    /// # Returns
    /// Vector of keys to press to reach and activate the target.
    ///
    /// # Supported Elements
    /// - **Menu**: Navigate with Up/Down, activate with Enter
    /// - **Button**: Tab navigation, activate with Enter
    /// - **Checkbox**: Tab navigation, toggle with Space
    ///
    /// # Example
    /// ```
    /// # use terminal_mcp_session::NavigationCalculator;
    /// # use terminal_mcp_core::{TerminalStateTree, Element, MenuItem, Bounds, Dimensions, Position};
    /// let calc = NavigationCalculator::new();
    /// // Create a simple TST with a menu
    /// let tst = TerminalStateTree {
    ///     session_id: "test".to_string(),
    ///     dimensions: Dimensions::new(24, 80),
    ///     cursor: Position::new(0, 0),
    ///     timestamp: "2025-11-30T00:00:00Z".to_string(),
    ///     elements: vec![
    ///         Element::Menu {
    ///             ref_id: "menu_0".to_string(),
    ///             bounds: Bounds::new(0, 0, 20, 3),
    ///             items: vec![
    ///                 MenuItem { ref_id: "item_0".to_string(), text: "Item 1".to_string(), selected: true },
    ///                 MenuItem { ref_id: "item_1".to_string(), text: "Item 2".to_string(), selected: false },
    ///             ],
    ///             selected: 0,
    ///         },
    ///     ],
    ///     raw_text: "".to_string(),
    ///     ansi_buffer: None,
    /// };
    /// let keys = calc.calculate(&tst, "item_1").unwrap();
    /// // Should navigate down once and press Enter
    /// ```
    pub fn calculate(&self, snapshot: &TerminalStateTree, target_ref: &str) -> Result<Vec<Key>> {
        // Check if target is a menu item (ref_id starts with "item_")
        if target_ref.starts_with("item_") {
            // Find the menu containing this item
            for element in &snapshot.elements {
                if let Element::Menu {
                    items, selected, ..
                } = element
                {
                    // Check if this menu contains the target item
                    if items.iter().any(|item| item.ref_id == target_ref) {
                        return self.navigate_menu(items, *selected, target_ref);
                    }
                }
            }
            return Err(Error::ElementNotFound(target_ref.to_string()));
        }

        // Find the target element (for non-menu-item elements)
        let target = snapshot
            .find_element(target_ref)
            .ok_or_else(|| Error::ElementNotFound(target_ref.to_string()))?;

        match target {
            Element::Button { .. } => self.navigate_to_button(snapshot, target),
            Element::Checkbox { .. } => self.navigate_to_checkbox(snapshot, target),
            _ => Err(Error::InvalidInput(format!(
                "Element type '{}' is not clickable",
                target.type_name()
            ))),
        }
    }

    /// Navigate within a menu to a target item.
    fn navigate_menu(
        &self,
        items: &[MenuItem],
        current_selected: usize,
        target_ref: &str,
    ) -> Result<Vec<Key>> {
        // Find target item index
        let target_idx = items
            .iter()
            .position(|i| i.ref_id == target_ref)
            .ok_or_else(|| Error::ElementNotFound(target_ref.to_string()))?;

        let mut keys = Vec::new();

        // Calculate direction and count
        let diff = target_idx as i32 - current_selected as i32;

        if diff > 0 {
            // Navigate down
            for _ in 0..diff {
                keys.push(Key::Down);
            }
        } else if diff < 0 {
            // Navigate up
            for _ in 0..diff.abs() {
                keys.push(Key::Up);
            }
        }

        // Add activation key
        keys.push(Key::Enter);

        Ok(keys)
    }

    /// Navigate to a button using Tab and activate with Enter.
    ///
    /// This is a simplified implementation that assumes Tab navigation works.
    /// A more sophisticated approach would analyze the element positions and
    /// determine the optimal navigation strategy.
    fn navigate_to_button(
        &self,
        _snapshot: &TerminalStateTree,
        _target: &Element,
    ) -> Result<Vec<Key>> {
        // For now, just return Enter (assumes button is already focused)
        // In a real implementation, we would:
        // 1. Find all focusable elements
        // 2. Calculate Tab presses needed to reach target
        // 3. Return Tab sequence + Enter
        Ok(vec![Key::Enter])
    }

    /// Navigate to a checkbox using Tab and toggle with Space.
    fn navigate_to_checkbox(
        &self,
        _snapshot: &TerminalStateTree,
        _target: &Element,
    ) -> Result<Vec<Key>> {
        // For now, just return Space (assumes checkbox is already focused)
        // In a real implementation, we would:
        // 1. Find all focusable elements
        // 2. Calculate Tab presses needed to reach target
        // 3. Return Tab sequence + Space
        Ok(vec![Key::Space])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal_mcp_core::{Bounds, Dimensions, Position};

    fn create_test_snapshot() -> TerminalStateTree {
        TerminalStateTree {
            session_id: "test_session".to_string(),
            dimensions: Dimensions::new(24, 80),
            cursor: Position::new(0, 0),
            timestamp: "2025-11-30T00:00:00Z".to_string(),
            elements: vec![Element::Menu {
                ref_id: "menu_0".to_string(),
                bounds: Bounds::new(2, 5, 30, 5),
                items: vec![
                    MenuItem {
                        ref_id: "item_0".to_string(),
                        text: "View Status".to_string(),
                        selected: true,
                    },
                    MenuItem {
                        ref_id: "item_1".to_string(),
                        text: "Start Service".to_string(),
                        selected: false,
                    },
                    MenuItem {
                        ref_id: "item_2".to_string(),
                        text: "Stop Service".to_string(),
                        selected: false,
                    },
                    MenuItem {
                        ref_id: "item_3".to_string(),
                        text: "Quit".to_string(),
                        selected: false,
                    },
                ],
                selected: 0,
            }],
            raw_text: "".to_string(),
            ansi_buffer: None,
        }
    }

    #[test]
    fn test_navigate_menu_down() {
        let calc = NavigationCalculator::new();
        let snapshot = create_test_snapshot();

        // Navigate from item 0 to item 2 (2 down)
        let keys = calc.calculate(&snapshot, "item_2").unwrap();

        assert_eq!(keys.len(), 3); // 2x Down + 1x Enter
        assert_eq!(keys[0], Key::Down);
        assert_eq!(keys[1], Key::Down);
        assert_eq!(keys[2], Key::Enter);
    }

    #[test]
    fn test_navigate_menu_up() {
        let calc = NavigationCalculator::new();
        let mut snapshot = create_test_snapshot();

        // Change selected to item 3
        if let Some(Element::Menu { selected, .. }) = snapshot.elements.get_mut(0) {
            *selected = 3;
        }

        // Navigate from item 3 to item 1 (2 up)
        let keys = calc.calculate(&snapshot, "item_1").unwrap();

        assert_eq!(keys.len(), 3); // 2x Up + 1x Enter
        assert_eq!(keys[0], Key::Up);
        assert_eq!(keys[1], Key::Up);
        assert_eq!(keys[2], Key::Enter);
    }

    #[test]
    fn test_navigate_menu_same_item() {
        let calc = NavigationCalculator::new();
        let snapshot = create_test_snapshot();

        // Navigate to currently selected item (item 0)
        let keys = calc.calculate(&snapshot, "item_0").unwrap();

        assert_eq!(keys.len(), 1); // Just Enter
        assert_eq!(keys[0], Key::Enter);
    }

    #[test]
    fn test_navigate_menu_not_found() {
        let calc = NavigationCalculator::new();
        let snapshot = create_test_snapshot();

        // Try to navigate to non-existent item
        let result = calc.calculate(&snapshot, "item_99");

        assert!(result.is_err());
        match result {
            Err(Error::ElementNotFound(ref_id)) => {
                assert!(ref_id.contains("item_99"));
            }
            _ => panic!("Expected ElementNotFound error"),
        }
    }

    #[test]
    fn test_navigate_button() {
        let calc = NavigationCalculator::new();
        let snapshot = TerminalStateTree {
            session_id: "test".to_string(),
            dimensions: Dimensions::new(24, 80),
            cursor: Position::new(0, 0),
            timestamp: "2025-11-30T00:00:00Z".to_string(),
            elements: vec![Element::Button {
                ref_id: "button_0".to_string(),
                bounds: Bounds::new(10, 10, 8, 1),
                label: "OK".to_string(),
            }],
            raw_text: "".to_string(),
            ansi_buffer: None,
        };

        let keys = calc.calculate(&snapshot, "button_0").unwrap();

        // For now, should just return Enter
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], Key::Enter);
    }

    #[test]
    fn test_navigate_checkbox() {
        let calc = NavigationCalculator::new();
        let snapshot = TerminalStateTree {
            session_id: "test".to_string(),
            dimensions: Dimensions::new(24, 80),
            cursor: Position::new(0, 0),
            timestamp: "2025-11-30T00:00:00Z".to_string(),
            elements: vec![Element::Checkbox {
                ref_id: "checkbox_0".to_string(),
                bounds: Bounds::new(5, 5, 20, 1),
                label: "Enable feature".to_string(),
                checked: false,
            }],
            raw_text: "".to_string(),
            ansi_buffer: None,
        };

        let keys = calc.calculate(&snapshot, "checkbox_0").unwrap();

        // For now, should just return Space
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], Key::Space);
    }

    #[test]
    fn test_navigate_non_clickable_element() {
        let calc = NavigationCalculator::new();
        let snapshot = TerminalStateTree {
            session_id: "test".to_string(),
            dimensions: Dimensions::new(24, 80),
            cursor: Position::new(0, 0),
            timestamp: "2025-11-30T00:00:00Z".to_string(),
            elements: vec![Element::ProgressBar {
                ref_id: "progress_0".to_string(),
                bounds: Bounds::new(10, 10, 20, 1),
                percent: 50,
            }],
            raw_text: "".to_string(),
            ansi_buffer: None,
        };

        let result = calc.calculate(&snapshot, "progress_0");

        assert!(result.is_err());
    }
}
