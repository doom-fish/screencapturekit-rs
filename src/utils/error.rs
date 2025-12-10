//! Error types for `ScreenCaptureKit`
//!
//! This module provides comprehensive error types for all operations in the library.
//! All operations return [`SCResult<T>`] which is an alias for `Result<T, SCError>`.
//!
//! # Examples
//!
//! ```
//! use screencapturekit::prelude::*;
//!
//! fn setup_capture() -> SCResult<()> {
//!     // Configure with builder pattern
//!     let config = SCStreamConfiguration::new()
//!         .with_width(1920)
//!         .with_height(1080);
//!     Ok(())
//! }
//!
//! // Pattern matching on errors
//! match setup_capture() {
//!     Ok(_) => println!("Success!"),
//!     Err(SCError::InvalidDimension { field, value }) => {
//!         eprintln!("Invalid {}: {}", field, value);
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

use std::fmt;

/// Result type alias for `ScreenCaptureKit` operations
///
/// This is a convenience alias for `Result<T, SCError>` used throughout the library.
///
/// # Examples
///
/// ```
/// use screencapturekit::prelude::*;
///
/// fn validate_dimensions(width: u32, height: u32) -> SCResult<()> {
///     if width == 0 {
///         return Err(SCError::invalid_dimension("width", 0));
///     }
///     if height == 0 {
///         return Err(SCError::invalid_dimension("height", 0));
///     }
///     Ok(())
/// }
///
/// assert!(validate_dimensions(0, 1080).is_err());
/// assert!(validate_dimensions(1920, 1080).is_ok());
/// ```
pub type SCResult<T> = Result<T, SCError>;

/// Comprehensive error type for `ScreenCaptureKit` operations
///
/// This enum covers all possible error conditions that can occur when using
/// the `ScreenCaptureKit` API. Each variant provides specific context about
/// what went wrong.
///
/// # Examples
///
/// ## Creating Errors
///
/// ```
/// use screencapturekit::error::SCError;
///
/// // Using helper methods (recommended)
/// let err = SCError::invalid_dimension("width", 0);
/// assert_eq!(err.to_string(), "Invalid dimension: width must be greater than 0 (got 0)");
///
/// let err = SCError::permission_denied("Screen Recording");
/// assert!(err.to_string().contains("Screen Recording"));
/// ```
///
/// ## Pattern Matching
///
/// ```
/// use screencapturekit::error::SCError;
///
/// fn handle_error(err: SCError) {
///     match err {
///         SCError::InvalidDimension { field, value } => {
///             println!("Invalid {}: {}", field, value);
///         }
///         SCError::PermissionDenied(msg) => {
///             println!("Permission needed: {}", msg);
///         }
///         _ => println!("Other error: {}", err),
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SCError {
    /// Invalid configuration parameter
    InvalidConfiguration(String),

    /// Invalid dimension value (width or height)
    InvalidDimension { field: String, value: usize },

    /// Invalid pixel format
    InvalidPixelFormat(String),

    /// No shareable content available
    NoShareableContent(String),

    /// Display not found
    DisplayNotFound(String),

    /// Window not found
    WindowNotFound(String),

    /// Application not found
    ApplicationNotFound(String),

    /// Stream operation error (generic)
    StreamError(String),

    /// Failed to start capture
    CaptureStartFailed(String),

    /// Failed to stop capture
    CaptureStopFailed(String),

    /// Buffer lock error
    BufferLockError(String),

    /// Buffer unlock error
    BufferUnlockError(String),

    /// Invalid buffer
    InvalidBuffer(String),

    /// Screenshot capture error
    ScreenshotError(String),

    /// Permission denied
    PermissionDenied(String),

    /// Feature not available on this macOS version
    FeatureNotAvailable {
        feature: String,
        required_version: String,
    },

    /// FFI error
    FFIError(String),

    /// Null pointer encountered
    NullPointer(String),

    /// Timeout error
    Timeout(String),

    /// Generic internal error
    InternalError(String),

    /// OS error with code (for non-SCStream errors)
    OSError { code: i32, message: String },

    /// `ScreenCaptureKit` stream error with specific error code
    ///
    /// This variant wraps Apple's `SCStreamError.Code` for precise error handling.
    /// Use [`SCStreamErrorCode`] to match specific error conditions.
    SCStreamError {
        code: SCStreamErrorCode,
        message: Option<String>,
    },
}

impl fmt::Display for SCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {msg}"),
            Self::InvalidDimension { field, value } => {
                write!(
                    f,
                    "Invalid dimension: {field} must be greater than 0 (got {value})"
                )
            }
            Self::InvalidPixelFormat(msg) => write!(f, "Invalid pixel format: {msg}"),
            Self::NoShareableContent(msg) => write!(f, "No shareable content available: {msg}"),
            Self::DisplayNotFound(msg) => write!(f, "Display not found: {msg}"),
            Self::WindowNotFound(msg) => write!(f, "Window not found: {msg}"),
            Self::ApplicationNotFound(msg) => write!(f, "Application not found: {msg}"),
            Self::StreamError(msg) => write!(f, "Stream error: {msg}"),
            Self::CaptureStartFailed(msg) => write!(f, "Failed to start capture: {msg}"),
            Self::CaptureStopFailed(msg) => write!(f, "Failed to stop capture: {msg}"),
            Self::BufferLockError(msg) => write!(f, "Failed to lock pixel buffer: {msg}"),
            Self::BufferUnlockError(msg) => write!(f, "Failed to unlock pixel buffer: {msg}"),
            Self::InvalidBuffer(msg) => write!(f, "Invalid buffer: {msg}"),
            Self::ScreenshotError(msg) => write!(f, "Screenshot capture failed: {msg}"),
            Self::PermissionDenied(msg) => {
                write!(f, "Permission denied: {msg}. Check System Preferences → Security & Privacy → Screen Recording")
            }
            Self::FeatureNotAvailable {
                feature,
                required_version,
            } => {
                write!(
                    f,
                    "Feature not available: {feature} requires macOS {required_version}+"
                )
            }
            Self::FFIError(msg) => write!(f, "FFI error: {msg}"),
            Self::NullPointer(msg) => write!(f, "Null pointer: {msg}"),
            Self::Timeout(msg) => write!(f, "Operation timed out: {msg}"),
            Self::InternalError(msg) => write!(f, "Internal error: {msg}"),
            Self::OSError { code, message } => write!(f, "OS error {code}: {message}"),
            Self::SCStreamError { code, message } => {
                if let Some(msg) = message {
                    write!(f, "SCStream error ({code}): {msg}")
                } else {
                    write!(f, "SCStream error: {code}")
                }
            }
        }
    }
}

impl std::error::Error for SCError {}

impl From<SCStreamErrorCode> for SCError {
    fn from(code: SCStreamErrorCode) -> Self {
        Self::from_stream_error_code(code)
    }
}

impl SCError {
    /// Create an invalid configuration error
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::invalid_config("Queue depth must be positive");
    /// assert!(err.to_string().contains("Queue depth"));
    /// ```
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfiguration(message.into())
    }

    /// Create an invalid dimension error
    ///
    /// Use this when width or height validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::invalid_dimension("width", 0);
    /// assert_eq!(
    ///     err.to_string(),
    ///     "Invalid dimension: width must be greater than 0 (got 0)"
    /// );
    ///
    /// let err = SCError::invalid_dimension("height", 0);
    /// assert!(err.to_string().contains("height"));
    /// ```
    pub fn invalid_dimension(field: impl Into<String>, value: usize) -> Self {
        Self::InvalidDimension {
            field: field.into(),
            value,
        }
    }

    /// Create a stream error
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::stream_error("Failed to start");
    /// assert!(err.to_string().contains("Stream error"));
    /// ```
    pub fn stream_error(message: impl Into<String>) -> Self {
        Self::StreamError(message.into())
    }

    /// Create a permission denied error
    ///
    /// The error message automatically includes instructions to check System Preferences.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::permission_denied("Screen Recording");
    /// let msg = err.to_string();
    /// assert!(msg.contains("Screen Recording"));
    /// assert!(msg.contains("System Preferences"));
    /// ```
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied(message.into())
    }

    /// Create an FFI error
    ///
    /// Use for errors crossing the Rust/Swift boundary.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::ffi_error("Swift bridge call failed");
    /// assert!(err.to_string().contains("FFI error"));
    /// ```
    pub fn ffi_error(message: impl Into<String>) -> Self {
        Self::FFIError(message.into())
    }

    /// Create an internal error
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::internal_error("Unexpected state");
    /// assert!(err.to_string().contains("Internal error"));
    /// ```
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Create a null pointer error
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::null_pointer("Display pointer");
    /// assert!(err.to_string().contains("Null pointer"));
    /// assert!(err.to_string().contains("Display pointer"));
    /// ```
    pub fn null_pointer(context: impl Into<String>) -> Self {
        Self::NullPointer(context.into())
    }

    /// Create a feature not available error
    ///
    /// Use when a feature requires a newer macOS version.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::feature_not_available("Screenshot Manager", "14.0");
    /// let msg = err.to_string();
    /// assert!(msg.contains("Screenshot Manager"));
    /// assert!(msg.contains("14.0"));
    /// ```
    pub fn feature_not_available(feature: impl Into<String>, version: impl Into<String>) -> Self {
        Self::FeatureNotAvailable {
            feature: feature.into(),
            required_version: version.into(),
        }
    }

    /// Create a buffer lock error
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::buffer_lock_error("Already locked");
    /// assert!(err.to_string().contains("lock pixel buffer"));
    /// ```
    pub fn buffer_lock_error(message: impl Into<String>) -> Self {
        Self::BufferLockError(message.into())
    }

    /// Create an OS error with error code
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::os_error(-1, "System call failed");
    /// let msg = err.to_string();
    /// assert!(msg.contains("-1"));
    /// assert!(msg.contains("System call failed"));
    /// ```
    pub fn os_error(code: i32, message: impl Into<String>) -> Self {
        Self::OSError {
            code,
            message: message.into(),
        }
    }

    /// Create an error from an `SCStreamErrorCode`
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::{SCError, SCStreamErrorCode};
    ///
    /// let err = SCError::from_stream_error_code(SCStreamErrorCode::UserDeclined);
    /// assert!(err.to_string().contains("User declined"));
    /// ```
    pub fn from_stream_error_code(code: SCStreamErrorCode) -> Self {
        Self::SCStreamError {
            code,
            message: None,
        }
    }

    /// Create an error from an `SCStreamErrorCode` with additional message
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::{SCError, SCStreamErrorCode};
    ///
    /// let err = SCError::from_stream_error_code_with_message(
    ///     SCStreamErrorCode::FailedToStart,
    ///     "No available displays"
    /// );
    /// assert!(err.to_string().contains("Failed to start"));
    /// ```
    pub fn from_stream_error_code_with_message(
        code: SCStreamErrorCode,
        message: impl Into<String>,
    ) -> Self {
        Self::SCStreamError {
            code,
            message: Some(message.into()),
        }
    }

    /// Create an error from a raw error code
    ///
    /// If the code matches a known `SCStreamErrorCode`, creates an `SCStreamError`.
    /// Otherwise, creates an `OSError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// // Known SCStreamError code
    /// let err = SCError::from_error_code(-3801); // UserDeclined
    /// assert!(matches!(err, SCError::SCStreamError { .. }));
    ///
    /// // Unknown code falls back to OSError
    /// let err = SCError::from_error_code(-999);
    /// assert!(matches!(err, SCError::OSError { .. }));
    /// ```
    pub fn from_error_code(code: i32) -> Self {
        SCStreamErrorCode::from_raw(code).map_or_else(
            || Self::OSError {
                code,
                message: "Unknown error".to_string(),
            },
            Self::from_stream_error_code,
        )
    }

    /// Get the `SCStreamErrorCode` if this is an `SCStreamError`
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::{SCError, SCStreamErrorCode};
    ///
    /// let err = SCError::from_stream_error_code(SCStreamErrorCode::UserDeclined);
    /// assert_eq!(err.stream_error_code(), Some(SCStreamErrorCode::UserDeclined));
    ///
    /// let err = SCError::StreamError("test".to_string());
    /// assert_eq!(err.stream_error_code(), None);
    /// ```
    pub fn stream_error_code(&self) -> Option<SCStreamErrorCode> {
        match self {
            Self::SCStreamError { code, .. } => Some(*code),
            _ => None,
        }
    }
}

/// Error domain for `ScreenCaptureKit` errors
pub const SC_STREAM_ERROR_DOMAIN: &str = "com.apple.screencapturekit";

/// Error codes from Apple's `SCStreamError.Code`
///
/// These correspond to the error codes returned by `ScreenCaptureKit` operations.
///
/// Based on Apple's `SCStreamErrorCode` from `SCError.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SCStreamErrorCode {
    /// The user chose not to authorize capture
    UserDeclined = -3801,
    /// The stream failed to start
    FailedToStart = -3802,
    /// The stream failed due to missing entitlements
    MissingEntitlements = -3803,
    /// Failed during recording - application connection invalid
    FailedApplicationConnectionInvalid = -3804,
    /// Failed during recording - application connection interrupted
    FailedApplicationConnectionInterrupted = -3805,
    /// Failed during recording - context id does not match application
    FailedNoMatchingApplicationContext = -3806,
    /// Failed due to attempting to start a stream that's already in a recording state
    AttemptToStartStreamState = -3807,
    /// Failed due to attempting to stop a stream that's already in a recording state
    AttemptToStopStreamState = -3808,
    /// Failed due to attempting to update the filter on a stream
    AttemptToUpdateFilterState = -3809,
    /// Failed due to attempting to update stream config on a stream
    AttemptToConfigState = -3810,
    /// Failed to start due to video/audio capture failure
    InternalError = -3811,
    /// Failed due to invalid parameter
    InvalidParameter = -3812,
    /// Failed due to no window list
    NoWindowList = -3813,
    /// Failed due to no display list
    NoDisplayList = -3814,
    /// Failed due to no display or window list to capture
    NoCaptureSource = -3815,
    /// Failed to remove stream
    RemovingStream = -3816,
    /// The stream was stopped by the user
    UserStopped = -3817,
    /// The stream failed to start audio (macOS 13.0+)
    FailedToStartAudioCapture = -3818,
    /// The stream failed to stop audio (macOS 13.0+)
    FailedToStopAudioCapture = -3819,
    /// The stream failed to start microphone (macOS 15.0+)
    FailedToStartMicrophoneCapture = -3820,
    /// The stream was stopped by the system (macOS 15.0+)
    SystemStoppedStream = -3821,
}

impl SCStreamErrorCode {
    /// Create from raw error code value
    pub fn from_raw(code: i32) -> Option<Self> {
        match code {
            -3801 => Some(Self::UserDeclined),
            -3802 => Some(Self::FailedToStart),
            -3803 => Some(Self::MissingEntitlements),
            -3804 => Some(Self::FailedApplicationConnectionInvalid),
            -3805 => Some(Self::FailedApplicationConnectionInterrupted),
            -3806 => Some(Self::FailedNoMatchingApplicationContext),
            -3807 => Some(Self::AttemptToStartStreamState),
            -3808 => Some(Self::AttemptToStopStreamState),
            -3809 => Some(Self::AttemptToUpdateFilterState),
            -3810 => Some(Self::AttemptToConfigState),
            -3811 => Some(Self::InternalError),
            -3812 => Some(Self::InvalidParameter),
            -3813 => Some(Self::NoWindowList),
            -3814 => Some(Self::NoDisplayList),
            -3815 => Some(Self::NoCaptureSource),
            -3816 => Some(Self::RemovingStream),
            -3817 => Some(Self::UserStopped),
            -3818 => Some(Self::FailedToStartAudioCapture),
            -3819 => Some(Self::FailedToStopAudioCapture),
            -3820 => Some(Self::FailedToStartMicrophoneCapture),
            -3821 => Some(Self::SystemStoppedStream),
            _ => None,
        }
    }

    /// Get the raw error code value
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

impl std::fmt::Display for SCStreamErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserDeclined => write!(f, "User declined screen recording"),
            Self::FailedToStart => write!(f, "Failed to start stream"),
            Self::MissingEntitlements => write!(f, "Missing entitlements"),
            Self::FailedApplicationConnectionInvalid => {
                write!(f, "Application connection invalid")
            }
            Self::FailedApplicationConnectionInterrupted => {
                write!(f, "Application connection interrupted")
            }
            Self::FailedNoMatchingApplicationContext => {
                write!(f, "No matching application context")
            }
            Self::AttemptToStartStreamState => write!(f, "Stream is already running"),
            Self::AttemptToStopStreamState => write!(f, "Stream is not running"),
            Self::AttemptToUpdateFilterState => write!(f, "Cannot update filter while streaming"),
            Self::AttemptToConfigState => write!(f, "Cannot configure while streaming"),
            Self::InternalError => write!(f, "Internal error"),
            Self::InvalidParameter => write!(f, "Invalid parameter"),
            Self::NoWindowList => write!(f, "No window list provided"),
            Self::NoDisplayList => write!(f, "No display list provided"),
            Self::NoCaptureSource => write!(f, "No capture source provided"),
            Self::RemovingStream => write!(f, "Failed to remove stream"),
            Self::UserStopped => write!(f, "User stopped the stream"),
            Self::FailedToStartAudioCapture => write!(f, "Failed to start audio capture"),
            Self::FailedToStopAudioCapture => write!(f, "Failed to stop audio capture"),
            Self::FailedToStartMicrophoneCapture => write!(f, "Failed to start microphone capture"),
            Self::SystemStoppedStream => write!(f, "System stopped the stream"),
        }
    }
}
