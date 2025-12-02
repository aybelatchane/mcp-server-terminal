//! Core detection types and traits.

use std::collections::HashMap;
use std::sync::Arc;

use terminal_mcp_core::{Bounds, Element, Position};
use terminal_mcp_emulator::Grid;

/// Detection confidence level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// Low confidence (<60% certain)
    Low,
    /// Medium confidence (60-90% certain)
    Medium,
    /// High confidence (>90% certain)
    High,
}

/// Raw detection result before assembly.
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedElement {
    /// The detected element
    pub element: Element,
    /// Bounding region
    pub bounds: Bounds,
    /// Confidence level
    pub confidence: Confidence,
}

/// Context passed to each detector.
#[derive(Debug, Clone)]
pub struct DetectionContext {
    /// Regions already claimed by higher-priority detectors
    pub claimed_regions: Vec<Bounds>,

    /// Current cursor position
    pub cursor: Position,

    /// Previous frame's elements (for tracking changes)
    pub previous_elements: Option<Vec<Element>>,

    /// Reference ID counter
    pub ref_counter: RefIdGenerator,
}

impl DetectionContext {
    /// Create a new detection context.
    pub fn new(cursor: Position) -> Self {
        Self {
            claimed_regions: Vec::new(),
            cursor,
            previous_elements: None,
            ref_counter: RefIdGenerator::new(),
        }
    }

    /// Check if a region overlaps with any claimed regions.
    pub fn is_region_claimed(&self, bounds: &Bounds) -> bool {
        self.claimed_regions
            .iter()
            .any(|claimed| bounds.intersects(claimed))
    }

    /// Claim a region (prevent other detectors from using it).
    pub fn claim_region(&mut self, bounds: Bounds) {
        self.claimed_regions.push(bounds);
    }
}

/// Reference ID generator for elements.
///
/// Generates unique IDs in the format `type_counter` (e.g., "menu_1", "table_2").
#[derive(Debug, Clone)]
pub struct RefIdGenerator {
    counters: HashMap<String, usize>,
}

impl RefIdGenerator {
    /// Create a new RefIdGenerator.
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    /// Generate the next ID for a given element type.
    ///
    /// # Example
    /// ```
    /// use terminal_mcp_detector::detection::RefIdGenerator;
    ///
    /// let mut gen = RefIdGenerator::new();
    /// assert_eq!(gen.next("menu"), "menu_1");
    /// assert_eq!(gen.next("menu"), "menu_2");
    /// assert_eq!(gen.next("table"), "table_1");
    /// ```
    pub fn next(&mut self, element_type: &str) -> String {
        let counter = self.counters.entry(element_type.to_string()).or_insert(0);
        *counter += 1;
        format!("{element_type}_{counter}")
    }

    /// Reset all counters.
    pub fn reset(&mut self) {
        self.counters.clear();
    }

    /// Reset counter for a specific type.
    pub fn reset_type(&mut self, element_type: &str) {
        self.counters.remove(element_type);
    }
}

impl Default for RefIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for element detectors.
pub trait ElementDetector: Send + Sync {
    /// Detector name for debugging/logging.
    fn name(&self) -> &'static str;

    /// Priority (higher = runs first, can claim regions).
    ///
    /// Typical priorities:
    /// - 100: Structural elements (borders)
    /// - 80-70: Interactive elements (menus, tables, inputs)
    /// - 60-50: Static elements (buttons, progress bars, checkboxes)
    fn priority(&self) -> u32;

    /// Detect elements in the grid.
    fn detect(&self, grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement>;

    /// Whether this detector is enabled.
    fn enabled(&self) -> bool {
        true
    }
}

/// Detection pipeline that runs detectors in priority order.
pub struct DetectionPipeline {
    detectors: Vec<Arc<dyn ElementDetector>>,
}

impl DetectionPipeline {
    /// Create a new detection pipeline.
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    /// Add a detector to the pipeline.
    pub fn add_detector(&mut self, detector: Arc<dyn ElementDetector>) {
        self.detectors.push(detector);
        // Sort by priority (descending)
        self.detectors
            .sort_by_key(|b| std::cmp::Reverse(b.priority()));
    }

    /// Run all detectors on the grid.
    pub fn detect(&self, grid: &Grid, cursor: Position) -> Vec<DetectedElement> {
        let mut context = DetectionContext::new(cursor);
        let mut all_elements = Vec::new();

        for detector in &self.detectors {
            if !detector.enabled() {
                continue;
            }

            let elements = detector.detect(grid, &context);

            // Claim regions for detected elements
            for elem in &elements {
                context.claim_region(elem.bounds);
            }

            all_elements.extend(elements);
        }

        all_elements
    }
}

impl Default for DetectionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_ordering() {
        assert!(Confidence::High > Confidence::Medium);
        assert!(Confidence::Medium > Confidence::Low);
    }

    #[test]
    fn test_ref_id_generator() {
        let mut gen = RefIdGenerator::new();

        assert_eq!(gen.next("menu"), "menu_1");
        assert_eq!(gen.next("menu"), "menu_2");
        assert_eq!(gen.next("table"), "table_1");
        assert_eq!(gen.next("menu"), "menu_3");
    }

    #[test]
    fn test_ref_id_generator_reset() {
        let mut gen = RefIdGenerator::new();

        assert_eq!(gen.next("menu"), "menu_1");
        assert_eq!(gen.next("menu"), "menu_2");

        gen.reset();

        assert_eq!(gen.next("menu"), "menu_1");
    }

    #[test]
    fn test_ref_id_generator_reset_type() {
        let mut gen = RefIdGenerator::new();

        assert_eq!(gen.next("menu"), "menu_1");
        assert_eq!(gen.next("table"), "table_1");
        assert_eq!(gen.next("menu"), "menu_2");

        gen.reset_type("menu");

        assert_eq!(gen.next("menu"), "menu_1");
        assert_eq!(gen.next("table"), "table_2");
    }

    #[test]
    fn test_detection_context_region_claiming() {
        let mut context = DetectionContext::new(Position::new(0, 0));

        let bounds1 = Bounds::new(0, 0, 10, 10); // rows 0-9, cols 0-9
        let bounds2 = Bounds::new(5, 5, 10, 10); // rows 5-14, cols 5-14 (overlaps)
        let bounds3 = Bounds::new(20, 20, 5, 5); // rows 20-24, cols 20-24 (no overlap)

        assert!(!context.is_region_claimed(&bounds1));

        context.claim_region(bounds1);

        assert!(context.is_region_claimed(&bounds1));
        assert!(context.is_region_claimed(&bounds2)); // Overlaps
        assert!(!context.is_region_claimed(&bounds3)); // No overlap
    }

    #[test]
    fn test_detection_pipeline_priority_ordering() {
        use terminal_mcp_emulator::Grid;

        struct TestDetector {
            name: &'static str,
            priority: u32,
        }

        impl ElementDetector for TestDetector {
            fn name(&self) -> &'static str {
                self.name
            }

            fn priority(&self) -> u32 {
                self.priority
            }

            fn detect(&self, _grid: &Grid, _context: &DetectionContext) -> Vec<DetectedElement> {
                Vec::new()
            }
        }

        let mut pipeline = DetectionPipeline::new();

        pipeline.add_detector(Arc::new(TestDetector {
            name: "low",
            priority: 10,
        }));
        pipeline.add_detector(Arc::new(TestDetector {
            name: "high",
            priority: 100,
        }));
        pipeline.add_detector(Arc::new(TestDetector {
            name: "medium",
            priority: 50,
        }));

        // Verify sorting
        assert_eq!(pipeline.detectors[0].name(), "high");
        assert_eq!(pipeline.detectors[1].name(), "medium");
        assert_eq!(pipeline.detectors[2].name(), "low");
    }

    #[test]
    fn test_detection_pipeline_region_claiming() {
        use terminal_mcp_core::{Dimensions, MenuItem};
        use terminal_mcp_emulator::Grid;

        struct ClaimingDetector {
            bounds: Bounds,
        }

        impl ElementDetector for ClaimingDetector {
            fn name(&self) -> &'static str {
                "claiming"
            }

            fn priority(&self) -> u32 {
                100
            }

            fn detect(&self, _grid: &Grid, context: &DetectionContext) -> Vec<DetectedElement> {
                if context.is_region_claimed(&self.bounds) {
                    Vec::new()
                } else {
                    vec![DetectedElement {
                        element: Element::Menu {
                            ref_id: "test_menu".to_string(),
                            bounds: self.bounds,
                            items: vec![MenuItem {
                                text: "Item".to_string(),
                                ref_id: "item_1".to_string(),
                                selected: false,
                            }],
                            selected: 0,
                        },
                        bounds: self.bounds,
                        confidence: Confidence::High,
                    }]
                }
            }
        }

        let mut pipeline = DetectionPipeline::new();

        let bounds = Bounds::new(0, 0, 10, 5);
        pipeline.add_detector(Arc::new(ClaimingDetector { bounds }));
        pipeline.add_detector(Arc::new(ClaimingDetector { bounds })); // Same region

        let grid = Grid::new(Dimensions::new(24, 80));
        let results = pipeline.detect(&grid, Position::new(0, 0));

        // Only first detector should return elements (second sees region as claimed)
        assert_eq!(results.len(), 1);
    }
}
