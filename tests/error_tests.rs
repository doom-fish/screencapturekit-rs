//! Error handling tests
//!
//! Tests for error types and error handling

use screencapturekit::error::SCError;

#[test]
fn test_invalid_dimension_error() {
    let err = SCError::invalid_dimension("width", 0);
    let display = format!("{err}");

    assert!(display.contains("width"));
    assert!(display.contains('0'));
}

#[test]
fn test_permission_denied_error() {
    let err = SCError::permission_denied("Screen Recording");
    let display = format!("{err}");

    // Just verify it has content
    assert!(!display.is_empty());
    assert!(display.contains("Screen Recording"));
}

#[test]
fn test_internal_error() {
    let err = SCError::internal_error("Something went wrong");
    let display = format!("{err}");

    assert!(display.contains("Something went wrong"));
}

#[test]
fn test_error_equality() {
    let err1 = SCError::invalid_dimension("width", 0);
    let err2 = SCError::invalid_dimension("width", 0);
    let err3 = SCError::invalid_dimension("height", 0);

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn test_error_debug() {
    let err = SCError::permission_denied("Screen Recording");
    let debug = format!("{err:?}");

    assert!(debug.contains("Screen Recording"));
}

#[test]
fn test_error_display_vs_debug() {
    let err = SCError::internal_error("Test error");

    let display = format!("{err}");
    let debug = format!("{err:?}");

    // Both should contain error information
    assert!(!display.is_empty());
    assert!(!debug.is_empty());
}

#[test]
fn test_error_types_are_different() {
    let err1 = SCError::invalid_dimension("width", 0);
    let err2 = SCError::permission_denied("Screen Recording");
    let err3 = SCError::internal_error("Internal");

    assert_ne!(err1, err2);
    assert_ne!(err1, err3);
    assert_ne!(err2, err3);
}

#[test]
fn test_error_in_result() {
    let result: Result<(), SCError> = Err(SCError::permission_denied("Test"));

    assert!(result.is_err());

    if let Err(err) = result {
        let display = format!("{err}");
        assert!(display.contains("Test"));
    }
}

#[test]
fn test_error_propagation() {
    fn failing_function() -> Result<(), SCError> {
        Err(SCError::internal_error("Failed"))
    }

    fn calling_function() -> Result<(), SCError> {
        failing_function()?;
        Ok(())
    }

    let result = calling_function();
    assert!(result.is_err());
}

#[test]
fn test_multiple_error_messages() {
    let errors = [
        SCError::invalid_dimension("width", 0),
        SCError::invalid_dimension("height", 0),
        SCError::permission_denied("Screen Recording"),
        SCError::permission_denied("Microphone"),
        SCError::internal_error("Error 1"),
        SCError::internal_error("Error 2"),
    ];

    for err in errors {
        let display = format!("{err}");
        assert!(!display.is_empty());
    }
}
