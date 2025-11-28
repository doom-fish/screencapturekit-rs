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

// MARK: - New Error Types

#[test]
fn test_user_declined_error() {
    let err = SCError::UserDeclined;
    let display = format!("{err}");

    assert!(display.contains("declined"));
    assert!(display.contains("permission"));
}

#[test]
fn test_microphone_capture_failed_error() {
    let err = SCError::MicrophoneCaptureFailed("Audio device not available".to_string());
    let display = format!("{err}");

    assert!(display.contains("microphone"));
    assert!(display.contains("Audio device not available"));
}

#[test]
fn test_system_stopped_stream_error() {
    let err = SCError::SystemStoppedStream;
    let display = format!("{err}");

    assert!(display.contains("System"));
    assert!(display.contains("stopped"));
}

#[test]
fn test_new_error_types_equality() {
    assert_eq!(SCError::UserDeclined, SCError::UserDeclined);
    assert_eq!(SCError::SystemStoppedStream, SCError::SystemStoppedStream);
    assert_eq!(
        SCError::MicrophoneCaptureFailed("test".to_string()),
        SCError::MicrophoneCaptureFailed("test".to_string())
    );

    assert_ne!(SCError::UserDeclined, SCError::SystemStoppedStream);
    assert_ne!(
        SCError::MicrophoneCaptureFailed("a".to_string()),
        SCError::MicrophoneCaptureFailed("b".to_string())
    );
}

#[test]
fn test_new_error_types_debug() {
    let errors = [
        SCError::UserDeclined,
        SCError::SystemStoppedStream,
        SCError::MicrophoneCaptureFailed("Test error".to_string()),
    ];

    for err in errors {
        let debug = format!("{err:?}");
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_error_is_std_error() {
    fn assert_error<T: std::error::Error>() {}
    assert_error::<SCError>();
}

#[test]
fn test_all_error_variants_display() {
    // Test that all error variants have non-empty display strings
    let errors: Vec<SCError> = vec![
        SCError::InvalidConfiguration("test".to_string()),
        SCError::InvalidDimension { field: "width".to_string(), value: 0 },
        SCError::InvalidPixelFormat("test".to_string()),
        SCError::NoShareableContent("test".to_string()),
        SCError::DisplayNotFound("test".to_string()),
        SCError::WindowNotFound("test".to_string()),
        SCError::ApplicationNotFound("test".to_string()),
        SCError::StreamError("test".to_string()),
        SCError::StreamAlreadyRunning,
        SCError::StreamNotRunning,
        SCError::CaptureStartFailed("test".to_string()),
        SCError::CaptureStopFailed("test".to_string()),
        SCError::BufferLockError("test".to_string()),
        SCError::BufferUnlockError("test".to_string()),
        SCError::InvalidBuffer("test".to_string()),
        SCError::ScreenshotError("test".to_string()),
        SCError::PermissionDenied("test".to_string()),
        SCError::FeatureNotAvailable { feature: "test".to_string(), required_version: "14.0".to_string() },
        SCError::FFIError("test".to_string()),
        SCError::NullPointer("test".to_string()),
        SCError::Timeout("test".to_string()),
        SCError::InternalError("test".to_string()),
        SCError::OSError { code: 1, message: "test".to_string() },
        SCError::UserDeclined,
        SCError::MicrophoneCaptureFailed("test".to_string()),
        SCError::SystemStoppedStream,
    ];

    for err in errors {
        let display = format!("{err}");
        assert!(!display.is_empty(), "Display should not be empty for {:?}", err);
    }
}
