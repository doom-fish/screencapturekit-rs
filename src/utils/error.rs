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
//!     // Configure with mutable configuration
//!     let mut config = SCStreamConfiguration::default();
//!     config.set_width(1920);
//!     config.set_height(1080);
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

    /// Stream operation error
    StreamError(String),

    /// Stream already running
    StreamAlreadyRunning,

    /// Stream not running
    StreamNotRunning,

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

    /// OS error with code
    OSError { code: i32, message: String },
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
            Self::StreamAlreadyRunning => write!(f, "Stream is already running"),
            Self::StreamNotRunning => write!(f, "Stream is not running"),
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
        }
    }
}

impl std::error::Error for SCError {}

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
}

// Legacy compatibility
impl SCError {
    /// Create from a message string (for backward compatibility)
    ///
    /// **Note:** Prefer using specific error constructors like [`SCError::invalid_config`]
    /// or other helper methods for better error categorization.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// // Old style (still works)
    /// let err = SCError::new("Something went wrong");
    /// assert!(err.to_string().contains("Something went wrong"));
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Get the error message (for backward compatibility)
    ///
    /// **Note:** Prefer using [`ToString::to_string`] which provides the same functionality.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::error::SCError;
    ///
    /// let err = SCError::invalid_dimension("width", 0);
    /// let msg = err.message();
    /// assert!(msg.contains("width"));
    /// assert!(msg.contains("0"));
    /// ```
    pub fn message(&self) -> String {
        self.to_string()
    }
}

/// Helper function to create an error (for backward compatibility)
///
/// **Note:** Prefer using [`SCError::new`] or specific constructors.
///
/// # Examples
///
/// ```
/// use screencapturekit::utils::error::create_sc_error;
///
/// let err = create_sc_error("Something failed");
/// assert!(err.to_string().contains("Something failed"));
/// ```
pub fn create_sc_error(message: &str) -> SCError {
    SCError::new(message)
}
