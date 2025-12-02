//! Terminal State Tree (TST) assembler.

use terminal_mcp_core::{Dimensions, Position, TerminalStateTree};

use crate::detection::DetectedElement;

/// TST assembler for building element hierarchy.
pub struct TSTAssembler;

impl TSTAssembler {
    /// Create a new TST assembler.
    pub fn new() -> Self {
        Self
    }

    /// Assemble detected elements into a Terminal State Tree.
    ///
    /// This converts raw DetectedElement results into a structured TST
    /// with proper hierarchy and containment relationships.
    ///
    /// # Arguments
    /// * `detected` - Detected elements with bounds and confidence
    /// * `session_id` - Session identifier
    /// * `dimensions` - Terminal dimensions
    /// * `cursor` - Current cursor position
    /// * `raw_text` - Raw text content
    pub fn assemble(
        &self,
        detected: Vec<DetectedElement>,
        session_id: String,
        dimensions: Dimensions,
        cursor: Position,
        raw_text: String,
    ) -> TerminalStateTree {
        // Extract elements (ignoring bounds and confidence for now)
        // TODO: Build proper hierarchy based on containment
        let elements = detected.into_iter().map(|d| d.element).collect();

        TerminalStateTree {
            session_id,
            dimensions,
            cursor,
            timestamp: chrono::Utc::now().to_rfc3339(),
            elements,
            raw_text,
            ansi_buffer: None,
        }
    }

    /// Assemble with confidence filtering.
    ///
    /// Only includes elements meeting the minimum confidence threshold.
    pub fn assemble_with_confidence(
        &self,
        detected: Vec<DetectedElement>,
        min_confidence: crate::detection::Confidence,
        session_id: String,
        dimensions: Dimensions,
        cursor: Position,
        raw_text: String,
    ) -> TerminalStateTree {
        let filtered: Vec<DetectedElement> = detected
            .into_iter()
            .filter(|d| d.confidence >= min_confidence)
            .collect();

        self.assemble(filtered, session_id, dimensions, cursor, raw_text)
    }
}

impl Default for TSTAssembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::Confidence;
    use terminal_mcp_core::{Bounds, Element, MenuItem};

    #[test]
    fn test_assembler_basic() {
        let assembler = TSTAssembler::new();

        let detected = vec![
            DetectedElement {
                element: Element::Menu {
                    ref_id: "menu_1".to_string(),
                    bounds: Bounds::new(0, 0, 10, 5),
                    items: vec![MenuItem {
                        text: "Item 1".to_string(),
                        ref_id: "item_1".to_string(),
                        selected: false,
                    }],
                    selected: 0,
                },
                bounds: Bounds::new(0, 0, 10, 5),
                confidence: Confidence::High,
            },
            DetectedElement {
                element: Element::Button {
                    ref_id: "button_1".to_string(),
                    bounds: Bounds::new(0, 10, 5, 1),
                    label: "OK".to_string(),
                },
                bounds: Bounds::new(0, 10, 5, 1),
                confidence: Confidence::Medium,
            },
        ];

        let tst = assembler.assemble(
            detected,
            "test_session".to_string(),
            Dimensions::new(24, 80),
            Position::new(0, 0),
            "test content".to_string(),
        );

        assert_eq!(tst.elements.len(), 2);
        assert_eq!(tst.session_id, "test_session");
    }

    #[test]
    fn test_assembler_confidence_filtering() {
        let assembler = TSTAssembler::new();

        let detected = vec![
            DetectedElement {
                element: Element::Button {
                    ref_id: "button_high".to_string(),
                    bounds: Bounds::new(0, 0, 5, 1),
                    label: "High".to_string(),
                },
                bounds: Bounds::new(0, 0, 5, 1),
                confidence: Confidence::High,
            },
            DetectedElement {
                element: Element::Button {
                    ref_id: "button_medium".to_string(),
                    bounds: Bounds::new(0, 10, 5, 1),
                    label: "Medium".to_string(),
                },
                bounds: Bounds::new(0, 10, 5, 1),
                confidence: Confidence::Medium,
            },
            DetectedElement {
                element: Element::Button {
                    ref_id: "button_low".to_string(),
                    bounds: Bounds::new(0, 20, 5, 1),
                    label: "Low".to_string(),
                },
                bounds: Bounds::new(0, 20, 5, 1),
                confidence: Confidence::Low,
            },
        ];

        // Filter for high confidence only
        let tst = assembler.assemble_with_confidence(
            detected.clone(),
            Confidence::High,
            "test".to_string(),
            Dimensions::new(24, 80),
            Position::new(0, 0),
            "".to_string(),
        );
        assert_eq!(tst.elements.len(), 1);

        // Filter for medium or higher
        let tst = assembler.assemble_with_confidence(
            detected.clone(),
            Confidence::Medium,
            "test".to_string(),
            Dimensions::new(24, 80),
            Position::new(0, 0),
            "".to_string(),
        );
        assert_eq!(tst.elements.len(), 2);

        // Filter for low or higher (all)
        let tst = assembler.assemble_with_confidence(
            detected,
            Confidence::Low,
            "test".to_string(),
            Dimensions::new(24, 80),
            Position::new(0, 0),
            "".to_string(),
        );
        assert_eq!(tst.elements.len(), 3);
    }
}
