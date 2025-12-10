//! `SCStreamErrorCode` tests
//!
//! Tests for error code handling and conversion

use screencapturekit::error::{SCError, SCStreamErrorCode, SC_STREAM_ERROR_DOMAIN};

#[test]
fn test_error_domain_constant() {
    assert_eq!(SC_STREAM_ERROR_DOMAIN, "com.apple.screencapturekit");
}

// MARK: - Error Code Values

#[test]
fn test_error_code_values() {
    // Verify the raw values match Apple's SCStreamError.Code from SCError.h
    assert_eq!(SCStreamErrorCode::UserDeclined as i32, -3801);
    assert_eq!(SCStreamErrorCode::FailedToStart as i32, -3802);
    assert_eq!(SCStreamErrorCode::MissingEntitlements as i32, -3803);
    assert_eq!(
        SCStreamErrorCode::FailedApplicationConnectionInvalid as i32,
        -3804
    );
    assert_eq!(
        SCStreamErrorCode::FailedApplicationConnectionInterrupted as i32,
        -3805
    );
    assert_eq!(
        SCStreamErrorCode::FailedNoMatchingApplicationContext as i32,
        -3806
    );
    assert_eq!(SCStreamErrorCode::AttemptToStartStreamState as i32, -3807);
    assert_eq!(SCStreamErrorCode::AttemptToStopStreamState as i32, -3808);
    assert_eq!(SCStreamErrorCode::AttemptToUpdateFilterState as i32, -3809);
    assert_eq!(SCStreamErrorCode::AttemptToConfigState as i32, -3810);
    assert_eq!(SCStreamErrorCode::InternalError as i32, -3811);
    assert_eq!(SCStreamErrorCode::InvalidParameter as i32, -3812);
    assert_eq!(SCStreamErrorCode::NoWindowList as i32, -3813);
    assert_eq!(SCStreamErrorCode::NoDisplayList as i32, -3814);
    assert_eq!(SCStreamErrorCode::NoCaptureSource as i32, -3815);
    assert_eq!(SCStreamErrorCode::RemovingStream as i32, -3816);
    assert_eq!(SCStreamErrorCode::UserStopped as i32, -3817);
    assert_eq!(SCStreamErrorCode::FailedToStartAudioCapture as i32, -3818);
    assert_eq!(SCStreamErrorCode::FailedToStopAudioCapture as i32, -3819);
    assert_eq!(
        SCStreamErrorCode::FailedToStartMicrophoneCapture as i32,
        -3820
    );
    assert_eq!(SCStreamErrorCode::SystemStoppedStream as i32, -3821);
}

// MARK: - Error Code Conversion

#[test]
fn test_error_code_from_raw_valid() {
    let codes = [
        (-3801, SCStreamErrorCode::UserDeclined),
        (-3802, SCStreamErrorCode::FailedToStart),
        (-3803, SCStreamErrorCode::MissingEntitlements),
        (-3804, SCStreamErrorCode::FailedApplicationConnectionInvalid),
        (
            -3805,
            SCStreamErrorCode::FailedApplicationConnectionInterrupted,
        ),
        (-3806, SCStreamErrorCode::FailedNoMatchingApplicationContext),
        (-3807, SCStreamErrorCode::AttemptToStartStreamState),
        (-3808, SCStreamErrorCode::AttemptToStopStreamState),
        (-3809, SCStreamErrorCode::AttemptToUpdateFilterState),
        (-3810, SCStreamErrorCode::AttemptToConfigState),
        (-3811, SCStreamErrorCode::InternalError),
        (-3812, SCStreamErrorCode::InvalidParameter),
        (-3813, SCStreamErrorCode::NoWindowList),
        (-3814, SCStreamErrorCode::NoDisplayList),
        (-3815, SCStreamErrorCode::NoCaptureSource),
        (-3816, SCStreamErrorCode::RemovingStream),
        (-3817, SCStreamErrorCode::UserStopped),
        (-3818, SCStreamErrorCode::FailedToStartAudioCapture),
        (-3819, SCStreamErrorCode::FailedToStopAudioCapture),
        (-3820, SCStreamErrorCode::FailedToStartMicrophoneCapture),
        (-3821, SCStreamErrorCode::SystemStoppedStream),
    ];

    for (raw, expected) in codes {
        let result = SCStreamErrorCode::from_raw(raw);
        assert_eq!(result, Some(expected), "Failed for raw value {raw}");
    }
}

#[test]
fn test_error_code_from_raw_invalid() {
    let invalid_codes = [0, 1, -1, -3800, -3822, -9999, i32::MAX, i32::MIN];

    for code in invalid_codes {
        let result = SCStreamErrorCode::from_raw(code);
        assert!(result.is_none(), "Should be None for invalid code {code}");
    }
}

// MARK: - Error Code Display

#[test]
fn test_error_code_display() {
    let code = SCStreamErrorCode::UserDeclined;
    let display = format!("{code}");
    assert!(!display.is_empty());
}

#[test]
fn test_all_error_codes_display() {
    let codes = [
        SCStreamErrorCode::UserDeclined,
        SCStreamErrorCode::FailedToStart,
        SCStreamErrorCode::MissingEntitlements,
        SCStreamErrorCode::FailedApplicationConnectionInvalid,
        SCStreamErrorCode::FailedApplicationConnectionInterrupted,
        SCStreamErrorCode::FailedNoMatchingApplicationContext,
        SCStreamErrorCode::AttemptToStartStreamState,
        SCStreamErrorCode::AttemptToStopStreamState,
        SCStreamErrorCode::AttemptToUpdateFilterState,
        SCStreamErrorCode::AttemptToConfigState,
        SCStreamErrorCode::InternalError,
        SCStreamErrorCode::InvalidParameter,
        SCStreamErrorCode::NoWindowList,
        SCStreamErrorCode::NoDisplayList,
        SCStreamErrorCode::NoCaptureSource,
        SCStreamErrorCode::RemovingStream,
        SCStreamErrorCode::UserStopped,
        SCStreamErrorCode::FailedToStartAudioCapture,
        SCStreamErrorCode::FailedToStopAudioCapture,
        SCStreamErrorCode::FailedToStartMicrophoneCapture,
        SCStreamErrorCode::SystemStoppedStream,
    ];

    for code in codes {
        let display = format!("{code}");
        assert!(
            !display.is_empty(),
            "Display should not be empty for {code:?}"
        );
    }
}

// MARK: - Error Code Debug

#[test]
fn test_error_code_debug() {
    let code = SCStreamErrorCode::UserDeclined;
    let debug = format!("{code:?}");
    assert!(debug.contains("UserDeclined"));
}

// MARK: - SCError Integration

#[test]
fn test_scerror_from_stream_error_code() {
    let code = SCStreamErrorCode::UserDeclined;
    let error = SCError::from_stream_error_code(code);

    match error {
        SCError::SCStreamError {
            code: err_code,
            message,
        } => {
            assert_eq!(err_code, code);
            assert!(message.is_none());
        }
        _ => panic!("Expected SCStreamError variant"),
    }
}

#[test]
fn test_scerror_from_stream_error_code_with_message() {
    let code = SCStreamErrorCode::InvalidParameter;
    let error = SCError::from_stream_error_code_with_message(code, "width must be positive");

    match error {
        SCError::SCStreamError {
            code: err_code,
            message,
        } => {
            assert_eq!(err_code, code);
            assert_eq!(message, Some("width must be positive".to_string()));
        }
        _ => panic!("Expected SCStreamError variant"),
    }
}

#[test]
fn test_scerror_from_error_code() {
    // Valid error code
    let error = SCError::from_error_code(-3801);
    match error {
        SCError::SCStreamError { code, .. } => {
            assert_eq!(code, SCStreamErrorCode::UserDeclined);
        }
        _ => panic!("Expected SCStreamError variant"),
    }

    // Invalid error code
    let error = SCError::from_error_code(-9999);
    match error {
        SCError::OSError { code, .. } => {
            assert_eq!(code, -9999);
        }
        _ => panic!("Expected OSError variant"),
    }
}

#[test]
fn test_scerror_stream_error_code_getter() {
    let code = SCStreamErrorCode::InternalError;
    let error = SCError::from_stream_error_code(code);

    let retrieved = error.stream_error_code();
    assert_eq!(retrieved, Some(code));

    // Non-SCStreamError should return None
    let other_error = SCError::InternalError("test".to_string());
    assert!(other_error.stream_error_code().is_none());
}

// MARK: - Error Code Equality and Hashing

#[test]
fn test_error_code_equality() {
    assert_eq!(
        SCStreamErrorCode::UserDeclined,
        SCStreamErrorCode::UserDeclined
    );
    assert_ne!(
        SCStreamErrorCode::UserDeclined,
        SCStreamErrorCode::InternalError
    );
}

#[test]
fn test_error_code_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(SCStreamErrorCode::UserDeclined);
    set.insert(SCStreamErrorCode::InternalError);
    set.insert(SCStreamErrorCode::UserDeclined); // Duplicate

    assert_eq!(set.len(), 2);
}

#[test]
fn test_error_code_clone() {
    let code = SCStreamErrorCode::UserDeclined;
    let cloned = code;
    assert_eq!(code, cloned);
}

#[test]
fn test_error_code_copy() {
    let code = SCStreamErrorCode::UserDeclined;
    let copied: SCStreamErrorCode = code;
    assert_eq!(code, copied);
}

// MARK: - SCStreamError Display

#[test]
fn test_scstream_error_display_without_message() {
    let error = SCError::from_stream_error_code(SCStreamErrorCode::UserDeclined);
    let display = format!("{error}");

    assert!(display.contains("SCStream"));
}

#[test]
fn test_scstream_error_display_with_message() {
    let error = SCError::from_stream_error_code_with_message(
        SCStreamErrorCode::InvalidParameter,
        "width must be > 0",
    );
    let display = format!("{error}");

    assert!(display.contains("SCStream"));
    assert!(display.contains("width must be > 0"));
}

// MARK: - Error Categories

#[test]
fn test_audio_related_errors() {
    let audio_errors = [
        SCStreamErrorCode::FailedToStartAudioCapture,
        SCStreamErrorCode::FailedToStopAudioCapture,
        SCStreamErrorCode::FailedToStartMicrophoneCapture,
    ];

    for code in audio_errors {
        let error = SCError::from_stream_error_code(code);
        let display = format!("{error}");
        assert!(!display.is_empty());
    }
}

#[test]
fn test_stream_lifecycle_errors() {
    let lifecycle_errors = [
        SCStreamErrorCode::FailedToStart,
        SCStreamErrorCode::AttemptToStartStreamState,
        SCStreamErrorCode::AttemptToStopStreamState,
        SCStreamErrorCode::SystemStoppedStream,
        SCStreamErrorCode::UserStopped,
    ];

    for code in lifecycle_errors {
        let error = SCError::from_stream_error_code(code);
        let display = format!("{error}");
        assert!(!display.is_empty());
    }
}

#[test]
fn test_configuration_errors() {
    let config_errors = [
        SCStreamErrorCode::InvalidParameter,
        SCStreamErrorCode::NoWindowList,
        SCStreamErrorCode::NoDisplayList,
        SCStreamErrorCode::NoCaptureSource,
        SCStreamErrorCode::AttemptToConfigState,
        SCStreamErrorCode::AttemptToUpdateFilterState,
    ];

    for code in config_errors {
        let error = SCError::from_stream_error_code(code);
        let display = format!("{error}");
        assert!(!display.is_empty());
    }
}

// MARK: - New Error Codes (macOS 13.0+, 15.0+)

#[test]
fn test_macos_13_audio_errors() {
    // FailedToStartAudioCapture and FailedToStopAudioCapture are macOS 13.0+
    let audio_start = SCStreamErrorCode::FailedToStartAudioCapture;
    let audio_stop = SCStreamErrorCode::FailedToStopAudioCapture;

    assert_eq!(audio_start as i32, -3818);
    assert_eq!(audio_stop as i32, -3819);

    // Verify they can be created from raw values
    assert_eq!(
        SCStreamErrorCode::from_raw(-3818),
        Some(SCStreamErrorCode::FailedToStartAudioCapture)
    );
    assert_eq!(
        SCStreamErrorCode::from_raw(-3819),
        Some(SCStreamErrorCode::FailedToStopAudioCapture)
    );
}

#[test]
fn test_macos_15_errors() {
    // FailedToStartMicrophoneCapture and SystemStoppedStream are macOS 15.0+
    let mic = SCStreamErrorCode::FailedToStartMicrophoneCapture;
    let system_stopped = SCStreamErrorCode::SystemStoppedStream;

    assert_eq!(mic as i32, -3820);
    assert_eq!(system_stopped as i32, -3821);

    // Verify they can be created from raw values
    assert_eq!(
        SCStreamErrorCode::from_raw(-3820),
        Some(SCStreamErrorCode::FailedToStartMicrophoneCapture)
    );
    assert_eq!(
        SCStreamErrorCode::from_raw(-3821),
        Some(SCStreamErrorCode::SystemStoppedStream)
    );
}

#[test]
fn test_entitlements_error() {
    let code = SCStreamErrorCode::MissingEntitlements;
    assert_eq!(code as i32, -3803);

    let error = SCError::from_stream_error_code(code);
    let display = format!("{error}");
    assert!(display.to_lowercase().contains("entitlement"));
}

#[test]
fn test_application_connection_errors() {
    let codes = [
        (
            SCStreamErrorCode::FailedApplicationConnectionInvalid,
            -3804,
        ),
        (
            SCStreamErrorCode::FailedApplicationConnectionInterrupted,
            -3805,
        ),
        (
            SCStreamErrorCode::FailedNoMatchingApplicationContext,
            -3806,
        ),
    ];

    for (code, expected_value) in codes {
        assert_eq!(code as i32, expected_value);
        assert_eq!(SCStreamErrorCode::from_raw(expected_value), Some(code));
    }
}
