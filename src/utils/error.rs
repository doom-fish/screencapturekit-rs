//! Error types for ScreenCaptureKit
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
//!     // Errors automatically propagate with ?
//!     let config = SCStreamConfiguration::build()
//!         .set_width(1920)?
//!         .set_height(1080)?;
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

use thiserror::Error;

/// Result type alias for ScreenCaptureKit operations
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

/// Comprehensive error type for ScreenCaptureKit operations
///
/// This enum covers all possible error conditions that can occur when using
/// the ScreenCaptureKit API. Each variant provides specific context about
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
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SCError {
    /// Invalid configuration parameter
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Invalid dimension value (width or height)
    #[error("Invalid dimension: {field} must be greater than 0 (got {value})")]
    InvalidDimension {
        field: String,
        value: usize,
    },

    /// Invalid pixel format
    #[error("Invalid pixel format: {0}")]
    InvalidPixelFormat(String),

    /// No shareable content available
    #[error("No shareable content available: {0}")]
    NoShareableContent(String),

    /// Display not found
    #[error("Display not found: {0}")]
    DisplayNotFound(String),

    /// Window not found
    #[error("Window not found: {0}")]
    WindowNotFound(String),

    /// Application not found
    #[error("Application not found: {0}")]
    ApplicationNotFound(String),

    /// Stream operation error
    #[error("Stream error: {0}")]
    StreamError(String),

    /// Stream already running
    #[error("Stream is already running")]
    StreamAlreadyRunning,

    /// Stream not running
    #[error("Stream is not running")]
    StreamNotRunning,

    /// Failed to start capture
    #[error("Failed to start capture: {0}")]
    CaptureStartFailed(String),

    /// Failed to stop capture
    #[error("Failed to stop capture: {0}")]
    CaptureStopFailed(String),

    /// Buffer lock error
    #[error("Failed to lock pixel buffer: {0}")]
    BufferLockError(String),

    /// Buffer unlock error
    #[error("Failed to unlock pixel buffer: {0}")]
    BufferUnlockError(String),

    /// Invalid buffer
    #[error("Invalid buffer: {0}")]
    InvalidBuffer(String),

    /// Screenshot capture error
    #[error("Screenshot capture failed: {0}")]
    ScreenshotError(String),

    /// Permission denied
    #[error("Permission denied: {0}. Check System Preferences → Security & Privacy → Screen Recording")]
    PermissionDenied(String),

    /// Feature not available on this macOS version
    #[error("Feature not available: {feature} requires macOS {required_version}+")]
    FeatureNotAvailable {
        feature: String,
        required_version: String,
    },

    /// FFI error
    #[error("FFI error: {0}")]
    FFIError(String),

    /// Null pointer encountered
    #[error("Null pointer: {0}")]
    NullPointer(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// OS error with code
    #[error("OS error {code}: {message}")]
    OSError {
        code: i32,
        message: String,
    },
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

