//! Integration tests for visual terminal support.

use terminal_mcp_core::Dimensions;
use terminal_mcp_session::{Session, SessionMode};

#[test]
fn test_create_headless_session() {
    // Create a headless session (default behavior)
    let session = Session::create(
        "echo".to_string(),
        vec!["Hello".to_string()],
        Dimensions::new(24, 80),
    );

    assert!(session.is_ok(), "Failed to create headless session");
    let session = session.unwrap();
    assert_eq!(session.mode(), SessionMode::Headless);
    assert!(session.visual_handle().is_none());
}

#[test]
#[ignore = "Requires display and terminal emulator (run locally with --ignored)"]
fn test_create_visual_session_auto() {
    // Try to create a visual session with auto-detect terminal
    let result = Session::create_with_mode(
        "echo".to_string(),
        vec!["Hello".to_string()],
        Dimensions::new(24, 80),
        SessionMode::Visual,
        None, // Auto-detect
        None, // cwd
    );

    // May fail if no terminal emulator is available on the test system
    // We just check that it either succeeds or fails gracefully
    match result {
        Ok(session) => {
            assert_eq!(session.mode(), SessionMode::Visual);
            assert!(session.visual_handle().is_some());

            let handle = session.visual_handle().unwrap();
            assert!(!handle.terminal_name.is_empty());
            println!("Visual session created with: {}", handle.terminal_name);
        }
        Err(e) => {
            // Expected to fail on headless CI environments
            println!("Visual session creation failed (expected on CI): {e}");
        }
    }
}

#[test]
#[ignore = "Requires display and terminal emulator (run locally with --ignored)"]
fn test_create_visual_session_specific_terminal() {
    // Try to create with a specific terminal that likely doesn't exist
    let result = Session::create_with_mode(
        "echo".to_string(),
        vec!["Hello".to_string()],
        Dimensions::new(24, 80),
        SessionMode::Visual,
        Some("nonexistent-terminal".to_string()),
        None, // cwd
    );

    // Should fail because terminal doesn't exist
    assert!(result.is_err(), "Should fail for nonexistent terminal");
}

#[test]
fn test_session_mode_backward_compatibility() {
    // Ensure old code using Session::create still works
    let session1 = Session::create("bash".to_string(), vec![], Dimensions::new(24, 80));
    assert!(session1.is_ok());

    // New code using create_with_mode for headless should work the same
    let session2 = Session::create_with_mode(
        "bash".to_string(),
        vec![],
        Dimensions::new(24, 80),
        SessionMode::Headless,
        None,
        None, // cwd
    );
    assert!(session2.is_ok());

    // Both should be in headless mode
    assert_eq!(session1.unwrap().mode(), SessionMode::Headless);
    assert_eq!(session2.unwrap().mode(), SessionMode::Headless);
}
