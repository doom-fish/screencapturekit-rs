//! Error types for `ScreenCaptureKit` operations
//!
//! This module provides error types and result aliases for handling
//! failures in screen capture operations.
//!
//! ## Main Types
//!
//! - [`SCError`] - The main error type for all `ScreenCaptureKit` operations
//! - [`SCResult<T>`] - Type alias for `Result<T, SCError>`
//! - [`SCStreamErrorCode`] - Specific error codes from `ScreenCaptureKit` framework
//!
//! ## Error Handling Example
//!
//! ```rust,no_run
//! use screencapturekit::error::{SCError, SCResult};
//! use screencapturekit::prelude::*;
//!
//! fn capture_screen() -> SCResult<()> {
//!     let content = SCShareableContent::get()?;
//!     
//!     if content.displays().is_empty() {
//!         return Err(SCError::internal_error("No displays available"));
//!     }
//!     
//!     // ... capture logic ...
//!     Ok(())
//! }
//!
//! fn main() {
//!     match capture_screen() {
//!         Ok(()) => println!("Capture successful"),
//!         Err(SCError::NoShareableContent(msg)) => {
//!             eprintln!("Permission denied: {}", msg);
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```

pub use crate::utils::error::{SCError, SCResult, SCStreamErrorCode, SC_STREAM_ERROR_DOMAIN};
