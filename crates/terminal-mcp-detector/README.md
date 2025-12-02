# terminal-mcp-detector

Element detection engine for inferring semantic structure from terminal grid.

## Overview

This crate implements pattern-based detection of UI elements from terminal content:

- **Borders**: Box-drawing characters forming panels and windows
- **Menus**: Vertical/horizontal lists with selection indicators
- **Tables**: Structured data with headers and columns
- **Buttons**: Bracketed or highlighted interactive elements
- **Inputs**: Text fields with labels and editable regions
- **Progress Bars**: Visual indicators of completion status
- **Status Bars**: Information displays at screen edges

## Detection Pipeline

Elements are detected in priority order to prevent overlapping claims:

1. Borders (priority 100) - structural containers
2. Menus/Tables (priority 80) - data structures
3. Inputs (priority 70) - interactive fields
4. Buttons/Progress (priority 60) - inline elements
5. Status bars (priority 50) - edge displays

## Usage

```rust
use terminal_mcp_detector::{DetectionEngine, DetectorConfig};
use terminal_mcp_emulator::Grid;

let grid = /* ... */;
let engine = DetectionEngine::new(DetectorConfig::default());
let elements = engine.detect(&grid);

for element in elements {
    println!("{}: {:?}", element.ref_id, element.element_type);
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
