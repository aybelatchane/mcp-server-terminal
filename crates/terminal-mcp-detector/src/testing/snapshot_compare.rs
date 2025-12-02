//! Snapshot comparison engine for regression testing.
//!
//! Compares Terminal State Tree snapshots to detect changes in element detection.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use terminal_mcp_core::{Bounds, Element};

use crate::detection::DetectedElement;

/// Describes a change to a single element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementChange {
    /// Element was added (not in expected)
    Added,
    /// Element was removed (was in expected)
    Removed,
    /// Element type changed
    TypeChanged {
        expected: String,
        actual: String,
    },
    /// Element bounds changed
    BoundsChanged {
        expected: Bounds,
        actual: Bounds,
    },
    /// Element content changed (label, items, etc.)
    ContentChanged {
        field: String,
        expected: String,
        actual: String,
    },
    /// Element confidence level changed
    ConfidenceChanged {
        expected: String,
        actual: String,
    },
}

/// Difference for a single element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDiff {
    /// Reference ID of the element (if available)
    pub ref_id: Option<String>,
    /// Element type
    pub element_type: String,
    /// Location of the element
    pub bounds: Option<Bounds>,
    /// List of changes
    pub changes: Vec<ElementChange>,
}

/// Result of comparing two snapshots.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SnapshotDiff {
    /// Elements that were added (in actual but not expected)
    pub added: Vec<ElementDiff>,
    /// Elements that were removed (in expected but not actual)
    pub removed: Vec<ElementDiff>,
    /// Elements that were modified
    pub modified: Vec<ElementDiff>,
    /// Summary statistics
    pub stats: DiffStats,
}

/// Summary statistics for a snapshot diff.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    /// Number of elements in expected snapshot
    pub expected_count: usize,
    /// Number of elements in actual snapshot
    pub actual_count: usize,
    /// Number of elements added
    pub added_count: usize,
    /// Number of elements removed
    pub removed_count: usize,
    /// Number of elements modified
    pub modified_count: usize,
    /// Number of elements unchanged
    pub unchanged_count: usize,
}

impl SnapshotDiff {
    /// Check if snapshots are identical.
    pub fn is_match(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }

    /// Get total number of differences.
    pub fn diff_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }

    /// Generate a human-readable summary.
    pub fn summary(&self) -> String {
        if self.is_match() {
            return "Snapshots match".to_string();
        }

        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("{} added", self.added.len()));
        }
        if !self.removed.is_empty() {
            parts.push(format!("{} removed", self.removed.len()));
        }
        if !self.modified.is_empty() {
            parts.push(format!("{} modified", self.modified.len()));
        }

        format!("Differences: {}", parts.join(", "))
    }

    /// Generate HTML report of the diff.
    pub fn to_html(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Snapshot Diff Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: monospace; margin: 20px; }\n");
        html.push_str(".added { background-color: #d4edda; }\n");
        html.push_str(".removed { background-color: #f8d7da; }\n");
        html.push_str(".modified { background-color: #fff3cd; }\n");
        html.push_str(".section { margin: 20px 0; padding: 10px; border: 1px solid #ddd; }\n");
        html.push_str("h2 { margin-top: 0; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #f5f5f5; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str("<h1>Snapshot Diff Report</h1>\n");
        html.push_str(&format!("<p><strong>Status:</strong> {}</p>\n", self.summary()));

        // Stats
        html.push_str("<div class=\"section\">\n");
        html.push_str("<h2>Statistics</h2>\n");
        html.push_str("<table>\n");
        html.push_str(&format!(
            "<tr><td>Expected elements</td><td>{}</td></tr>\n",
            self.stats.expected_count
        ));
        html.push_str(&format!(
            "<tr><td>Actual elements</td><td>{}</td></tr>\n",
            self.stats.actual_count
        ));
        html.push_str(&format!(
            "<tr><td>Added</td><td>{}</td></tr>\n",
            self.stats.added_count
        ));
        html.push_str(&format!(
            "<tr><td>Removed</td><td>{}</td></tr>\n",
            self.stats.removed_count
        ));
        html.push_str(&format!(
            "<tr><td>Modified</td><td>{}</td></tr>\n",
            self.stats.modified_count
        ));
        html.push_str(&format!(
            "<tr><td>Unchanged</td><td>{}</td></tr>\n",
            self.stats.unchanged_count
        ));
        html.push_str("</table>\n</div>\n");

        // Added elements
        if !self.added.is_empty() {
            html.push_str("<div class=\"section added\">\n");
            html.push_str("<h2>Added Elements</h2>\n");
            html.push_str(&Self::elements_table(&self.added));
            html.push_str("</div>\n");
        }

        // Removed elements
        if !self.removed.is_empty() {
            html.push_str("<div class=\"section removed\">\n");
            html.push_str("<h2>Removed Elements</h2>\n");
            html.push_str(&Self::elements_table(&self.removed));
            html.push_str("</div>\n");
        }

        // Modified elements
        if !self.modified.is_empty() {
            html.push_str("<div class=\"section modified\">\n");
            html.push_str("<h2>Modified Elements</h2>\n");
            for diff in &self.modified {
                html.push_str(&format!(
                    "<h3>{} ({})</h3>\n",
                    diff.element_type,
                    diff.ref_id.as_deref().unwrap_or("unknown")
                ));
                html.push_str("<ul>\n");
                for change in &diff.changes {
                    html.push_str(&format!("<li>{:?}</li>\n", change));
                }
                html.push_str("</ul>\n");
            }
            html.push_str("</div>\n");
        }

        html.push_str("</body>\n</html>\n");
        html
    }

    fn elements_table(elements: &[ElementDiff]) -> String {
        let mut html = String::new();
        html.push_str("<table>\n");
        html.push_str("<tr><th>Type</th><th>Ref ID</th><th>Bounds</th></tr>\n");
        for elem in elements {
            let bounds_str = elem
                .bounds
                .as_ref()
                .map(|b| format!("({},{}) {}x{}", b.row, b.col, b.width, b.height))
                .unwrap_or_else(|| "N/A".to_string());
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                elem.element_type,
                elem.ref_id.as_deref().unwrap_or("N/A"),
                bounds_str
            ));
        }
        html.push_str("</table>\n");
        html
    }
}

/// Snapshot matcher for comparing detection results.
pub struct SnapshotMatcher {
    /// Tolerance for bounds comparison (in cells)
    pub bounds_tolerance: u16,
    /// Whether to compare confidence levels
    pub compare_confidence: bool,
    /// Whether to compare ref_ids (usually false for golden comparison)
    pub compare_ref_ids: bool,
}

impl Default for SnapshotMatcher {
    fn default() -> Self {
        Self {
            bounds_tolerance: 0,
            compare_confidence: false,
            compare_ref_ids: false,
        }
    }
}

impl SnapshotMatcher {
    /// Create a new snapshot matcher with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bounds tolerance for comparison.
    pub fn with_bounds_tolerance(mut self, tolerance: u16) -> Self {
        self.bounds_tolerance = tolerance;
        self
    }

    /// Enable confidence level comparison.
    pub fn with_confidence_comparison(mut self, enabled: bool) -> Self {
        self.compare_confidence = enabled;
        self
    }

    /// Compare two lists of detected elements.
    pub fn compare(
        &self,
        expected: &[DetectedElement],
        actual: &[DetectedElement],
    ) -> SnapshotDiff {
        let mut diff = SnapshotDiff::default();
        diff.stats.expected_count = expected.len();
        diff.stats.actual_count = actual.len();

        // Build maps by element type and approximate position
        let expected_map = self.build_element_map(expected);
        let actual_map = self.build_element_map(actual);

        // Track which elements have been matched
        let mut matched_actual: Vec<bool> = vec![false; actual.len()];

        // Compare expected elements
        for (i, exp) in expected.iter().enumerate() {
            let exp_key = self.element_key(exp);

            if let Some(actual_indices) = actual_map.get(&exp_key) {
                // Try to find a matching element
                let mut found_match = false;
                for &act_idx in actual_indices {
                    if matched_actual[act_idx] {
                        continue;
                    }

                    let act = &actual[act_idx];
                    let changes = self.compare_elements(exp, act);

                    if changes.is_empty() {
                        // Perfect match
                        matched_actual[act_idx] = true;
                        found_match = true;
                        diff.stats.unchanged_count += 1;
                        break;
                    } else if self.is_same_element(exp, act) {
                        // Same element but with changes
                        matched_actual[act_idx] = true;
                        found_match = true;
                        diff.modified.push(ElementDiff {
                            ref_id: self.get_ref_id(&exp.element),
                            element_type: self.element_type_name(&exp.element),
                            bounds: Some(exp.bounds),
                            changes,
                        });
                        diff.stats.modified_count += 1;
                        break;
                    }
                }

                if !found_match {
                    // Element was removed
                    diff.removed.push(ElementDiff {
                        ref_id: self.get_ref_id(&exp.element),
                        element_type: self.element_type_name(&exp.element),
                        bounds: Some(exp.bounds),
                        changes: vec![ElementChange::Removed],
                    });
                    diff.stats.removed_count += 1;
                }
            } else {
                // Element type not found at all - removed
                diff.removed.push(ElementDiff {
                    ref_id: self.get_ref_id(&exp.element),
                    element_type: self.element_type_name(&exp.element),
                    bounds: Some(exp.bounds),
                    changes: vec![ElementChange::Removed],
                });
                diff.stats.removed_count += 1;
            }
        }

        // Find added elements (in actual but not matched)
        for (i, act) in actual.iter().enumerate() {
            if !matched_actual[i] {
                diff.added.push(ElementDiff {
                    ref_id: self.get_ref_id(&act.element),
                    element_type: self.element_type_name(&act.element),
                    bounds: Some(act.bounds),
                    changes: vec![ElementChange::Added],
                });
                diff.stats.added_count += 1;
            }
        }

        diff
    }

    /// Build a map of elements by type for faster lookup.
    fn build_element_map(&self, elements: &[DetectedElement]) -> HashMap<String, Vec<usize>> {
        let mut map: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, elem) in elements.iter().enumerate() {
            let key = self.element_key(elem);
            map.entry(key).or_default().push(i);
        }
        map
    }

    /// Generate a key for element matching (type name).
    fn element_key(&self, elem: &DetectedElement) -> String {
        self.element_type_name(&elem.element)
    }

    /// Get element type name as string.
    fn element_type_name(&self, elem: &Element) -> String {
        match elem {
            Element::Button { .. } => "button".to_string(),
            Element::Checkbox { .. } => "checkbox".to_string(),
            Element::Input { .. } => "input".to_string(),
            Element::Menu { .. } => "menu".to_string(),
            Element::ProgressBar { .. } => "progress_bar".to_string(),
            Element::StatusBar { .. } => "status_bar".to_string(),
            Element::Table { .. } => "table".to_string(),
            Element::Border { .. } => "border".to_string(),
            Element::Text { .. } => "text".to_string(),
        }
    }

    /// Get ref_id from element if available.
    fn get_ref_id(&self, elem: &Element) -> Option<String> {
        match elem {
            Element::Button { ref_id, .. } => Some(ref_id.clone()),
            Element::Checkbox { ref_id, .. } => Some(ref_id.clone()),
            Element::Input { ref_id, .. } => Some(ref_id.clone()),
            Element::Menu { ref_id, .. } => Some(ref_id.clone()),
            Element::ProgressBar { ref_id, .. } => Some(ref_id.clone()),
            Element::StatusBar { ref_id, .. } => Some(ref_id.clone()),
            Element::Table { ref_id, .. } => Some(ref_id.clone()),
            Element::Border { ref_id, .. } => Some(ref_id.clone()),
            Element::Text { ref_id, .. } => Some(ref_id.clone()),
        }
    }

    /// Check if two elements are the same (based on position and type).
    fn is_same_element(&self, a: &DetectedElement, b: &DetectedElement) -> bool {
        if self.element_type_name(&a.element) != self.element_type_name(&b.element) {
            return false;
        }

        // Check bounds with tolerance
        self.bounds_match(&a.bounds, &b.bounds)
    }

    /// Check if bounds match within tolerance.
    fn bounds_match(&self, a: &Bounds, b: &Bounds) -> bool {
        let row_diff = (a.row as i32 - b.row as i32).unsigned_abs() as u16;
        let col_diff = (a.col as i32 - b.col as i32).unsigned_abs() as u16;

        row_diff <= self.bounds_tolerance && col_diff <= self.bounds_tolerance
    }

    /// Compare two elements and return list of changes.
    fn compare_elements(&self, expected: &DetectedElement, actual: &DetectedElement) -> Vec<ElementChange> {
        let mut changes = Vec::new();

        // Compare bounds
        if expected.bounds != actual.bounds && self.bounds_tolerance == 0 {
            changes.push(ElementChange::BoundsChanged {
                expected: expected.bounds,
                actual: actual.bounds,
            });
        }

        // Compare element-specific content
        self.compare_element_content(&expected.element, &actual.element, &mut changes);

        // Compare confidence if enabled
        if self.compare_confidence && expected.confidence != actual.confidence {
            changes.push(ElementChange::ConfidenceChanged {
                expected: format!("{:?}", expected.confidence),
                actual: format!("{:?}", actual.confidence),
            });
        }

        changes
    }

    /// Compare element-specific content.
    fn compare_element_content(&self, expected: &Element, actual: &Element, changes: &mut Vec<ElementChange>) {
        match (expected, actual) {
            (Element::Button { label: exp_label, .. }, Element::Button { label: act_label, .. }) => {
                if exp_label != act_label {
                    changes.push(ElementChange::ContentChanged {
                        field: "label".to_string(),
                        expected: exp_label.clone(),
                        actual: act_label.clone(),
                    });
                }
            }
            (Element::Checkbox { label: exp_label, checked: exp_checked, .. },
             Element::Checkbox { label: act_label, checked: act_checked, .. }) => {
                if exp_label != act_label {
                    changes.push(ElementChange::ContentChanged {
                        field: "label".to_string(),
                        expected: exp_label.clone(),
                        actual: act_label.clone(),
                    });
                }
                if exp_checked != act_checked {
                    changes.push(ElementChange::ContentChanged {
                        field: "checked".to_string(),
                        expected: exp_checked.to_string(),
                        actual: act_checked.to_string(),
                    });
                }
            }
            (Element::Input { value: exp_value, cursor_pos: exp_cursor, .. },
             Element::Input { value: act_value, cursor_pos: act_cursor, .. }) => {
                if exp_value != act_value {
                    changes.push(ElementChange::ContentChanged {
                        field: "value".to_string(),
                        expected: exp_value.clone(),
                        actual: act_value.clone(),
                    });
                }
                if exp_cursor != act_cursor {
                    changes.push(ElementChange::ContentChanged {
                        field: "cursor_pos".to_string(),
                        expected: exp_cursor.to_string(),
                        actual: act_cursor.to_string(),
                    });
                }
            }
            (Element::Menu { items: exp_items, selected: exp_selected, .. },
             Element::Menu { items: act_items, selected: act_selected, .. }) => {
                if exp_items != act_items {
                    changes.push(ElementChange::ContentChanged {
                        field: "items".to_string(),
                        expected: format!("{:?}", exp_items),
                        actual: format!("{:?}", act_items),
                    });
                }
                if exp_selected != act_selected {
                    changes.push(ElementChange::ContentChanged {
                        field: "selected".to_string(),
                        expected: exp_selected.to_string(),
                        actual: act_selected.to_string(),
                    });
                }
            }
            (Element::ProgressBar { percent: exp_pct, .. },
             Element::ProgressBar { percent: act_pct, .. }) => {
                if exp_pct != act_pct {
                    changes.push(ElementChange::ContentChanged {
                        field: "percent".to_string(),
                        expected: exp_pct.to_string(),
                        actual: act_pct.to_string(),
                    });
                }
            }
            (Element::Table { headers: exp_headers, rows: exp_rows, .. },
             Element::Table { headers: act_headers, rows: act_rows, .. }) => {
                if exp_headers != act_headers {
                    changes.push(ElementChange::ContentChanged {
                        field: "headers".to_string(),
                        expected: format!("{:?}", exp_headers),
                        actual: format!("{:?}", act_headers),
                    });
                }
                if exp_rows.len() != act_rows.len() {
                    changes.push(ElementChange::ContentChanged {
                        field: "row_count".to_string(),
                        expected: exp_rows.len().to_string(),
                        actual: act_rows.len().to_string(),
                    });
                }
            }
            _ => {
                // Different element types - this shouldn't happen if is_same_element works correctly
                if std::mem::discriminant(expected) != std::mem::discriminant(actual) {
                    changes.push(ElementChange::TypeChanged {
                        expected: self.element_type_name(expected),
                        actual: self.element_type_name(actual),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::Confidence;

    fn create_button(label: &str, row: u16, col: u16) -> DetectedElement {
        DetectedElement {
            element: Element::Button {
                ref_id: format!("button_{}", label),
                bounds: Bounds::new(col, row, label.len() as u16 + 4, 1),
                label: label.to_string(),
            },
            bounds: Bounds::new(col, row, label.len() as u16 + 4, 1),
            confidence: Confidence::High,
        }
    }

    fn create_checkbox(label: &str, checked: bool, row: u16, col: u16) -> DetectedElement {
        DetectedElement {
            element: Element::Checkbox {
                ref_id: format!("checkbox_{}", label),
                bounds: Bounds::new(col, row, label.len() as u16 + 4, 1),
                label: label.to_string(),
                checked,
            },
            bounds: Bounds::new(col, row, label.len() as u16 + 4, 1),
            confidence: Confidence::High,
        }
    }

    #[test]
    fn test_identical_snapshots() {
        let expected = vec![
            create_button("OK", 5, 10),
            create_button("Cancel", 5, 20),
        ];
        let actual = expected.clone();

        let matcher = SnapshotMatcher::new();
        let diff = matcher.compare(&expected, &actual);

        assert!(diff.is_match());
        assert_eq!(diff.stats.unchanged_count, 2);
    }

    #[test]
    fn test_added_element() {
        let expected = vec![create_button("OK", 5, 10)];
        let actual = vec![
            create_button("OK", 5, 10),
            create_button("Cancel", 5, 20),
        ];

        let matcher = SnapshotMatcher::new();
        let diff = matcher.compare(&expected, &actual);

        assert!(!diff.is_match());
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].element_type, "button");
    }

    #[test]
    fn test_removed_element() {
        let expected = vec![
            create_button("OK", 5, 10),
            create_button("Cancel", 5, 20),
        ];
        let actual = vec![create_button("OK", 5, 10)];

        let matcher = SnapshotMatcher::new();
        let diff = matcher.compare(&expected, &actual);

        assert!(!diff.is_match());
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].element_type, "button");
    }

    #[test]
    fn test_modified_element() {
        let expected = vec![create_checkbox("Option", false, 5, 10)];
        let actual = vec![create_checkbox("Option", true, 5, 10)];

        let matcher = SnapshotMatcher::new();
        let diff = matcher.compare(&expected, &actual);

        assert!(!diff.is_match());
        assert_eq!(diff.modified.len(), 1);
        assert!(diff.modified[0].changes.iter().any(|c| matches!(c, ElementChange::ContentChanged { field, .. } if field == "checked")));
    }

    #[test]
    fn test_bounds_tolerance() {
        let expected = vec![create_button("OK", 5, 10)];
        let actual = vec![create_button("OK", 6, 11)]; // Slightly different position

        // Without tolerance - should detect as modified
        let matcher = SnapshotMatcher::new();
        let diff = matcher.compare(&expected, &actual);
        assert!(!diff.is_match());

        // With tolerance - should match
        let matcher = SnapshotMatcher::new().with_bounds_tolerance(2);
        let diff = matcher.compare(&expected, &actual);
        assert!(diff.is_match());
    }

    #[test]
    fn test_html_report() {
        let expected = vec![create_button("OK", 5, 10)];
        let actual = vec![
            create_button("OK", 5, 10),
            create_button("Cancel", 5, 20),
        ];

        let matcher = SnapshotMatcher::new();
        let diff = matcher.compare(&expected, &actual);
        let html = diff.to_html();

        assert!(html.contains("Snapshot Diff Report"));
        assert!(html.contains("Added Elements"));
        assert!(html.contains("button"));
    }
}
