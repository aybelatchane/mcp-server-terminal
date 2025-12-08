//! Integration test to verify issue #141 is fixed
//! Tests that shell prompts and random punctuation are not detected as UI elements

use terminal_mcp_core::{Dimensions, Element, Position};
use terminal_mcp_detector::{
    ButtonDetector, DetectionContext, ElementDetector, ProgressDetector,
};
use terminal_mcp_emulator::{Grid, Parser};

fn create_grid_with_text(rows: u16, cols: u16, text: &str) -> Grid {
    let grid = Grid::new(Dimensions::new(rows, cols));
    let mut parser = Parser::new(grid);
    parser.process(text.as_bytes());
    parser.into_grid()
}

#[test]
fn test_shell_prompt_not_detected_as_button() {
    // Original issue: "(main)" in shell prompt was detected as button
    let text = "user@host:/path(main)$ \r\n";
    let grid = create_grid_with_text(5, 80, text);

    let detector = ButtonDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));
    let detected = detector.detect(&grid, &context);

    // Should NOT detect any buttons in shell prompt
    assert_eq!(
        detected.len(),
        0,
        "Shell prompt should not be detected as button. Found: {:?}",
        detected
    );
}

#[test]
fn test_git_branch_in_prompt_excluded() {
    // Test various git branch patterns
    let test_cases = vec![
        "user@host:/repo(main)$ ",
        "user@host:/repo(dev)$ ",
        "user@host:/repo(master)$ ",
        "~/project(feature-branch)$ ",
    ];

    let detector = ButtonDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    for text in test_cases {
        let grid = create_grid_with_text(5, 80, &format!("{}\r\n", text));
        let detected = detector.detect(&grid, &context);

        assert_eq!(
            detected.len(),
            0,
            "Git branch '{}' should not be detected as button",
            text
        );
    }
}

#[test]
fn test_random_punctuation_not_detected_as_progress() {
    // Original issue: Random dots/dashes detected as progress bars
    let test_cases = vec![
        "Some text with dots.....",
        "And dashes -----",
        "Stars *****",
        "Mixed .-.-.-.--",
    ];

    let detector = ProgressDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    for text in test_cases {
        let grid = create_grid_with_text(5, 80, &format!("{}\r\n", text));
        let detected = detector.detect(&grid, &context);

        assert_eq!(
            detected.len(),
            0,
            "Random punctuation '{}' should not be detected as progress bar. Found: {:?}",
            text,
            detected
        );
    }
}

#[test]
fn test_real_unicode_progress_bar_still_detected() {
    // Ensure we still detect real progress bars
    let text = "Progress: ████░░░░░░\r\n";
    let grid = create_grid_with_text(5, 80, text);

    let detector = ProgressDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));
    let detected = detector.detect(&grid, &context);

    assert_eq!(
        detected.len(),
        1,
        "Real Unicode progress bar should be detected"
    );

    if let Element::ProgressBar { percent, .. } = &detected[0].element {
        // Should be around 36-40% (4 filled out of 10-11 chars depending on parsing)
        assert!(
            *percent >= 36 && *percent <= 40,
            "Progress should be 36-40%, got {}",
            percent
        );
    } else {
        panic!("Expected ProgressBar element");
    }
}

#[test]
fn test_real_buttons_still_detected() {
    // Ensure we still detect real buttons with brackets
    let text = "[ OK ] [ Cancel ]\r\n";
    let grid = create_grid_with_text(5, 80, text);

    let detector = ButtonDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));
    let detected = detector.detect(&grid, &context);

    assert_eq!(detected.len(), 2, "Real buttons should be detected");

    if let Element::Button { label, .. } = &detected[0].element {
        assert_eq!(label, "OK");
    }
    if let Element::Button { label, .. } = &detected[1].element {
        assert_eq!(label, "Cancel");
    }
}

#[test]
fn test_angle_bracket_buttons_still_work() {
    // Angle brackets should still work as buttons
    let text = "< Submit > < Reset >\r\n";
    let grid = create_grid_with_text(5, 80, text);

    let detector = ButtonDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));
    let detected = detector.detect(&grid, &context);

    assert_eq!(detected.len(), 2, "Angle bracket buttons should be detected");
}
