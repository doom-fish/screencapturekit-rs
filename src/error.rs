//! Error types for `ScreenCaptureKit` operations
//!
//! This module re-exports error types from [`crate::utils::error`].
//!
//! # Examples
//!
//! ```
//! use screencapturekit::error::{SCError, SCResult};
//!
//! fn example() -> SCResult<()> {
//!     // Function that can fail
//!     Ok(())
//! }
//! ```

pub use crate::utils::error::{SCError, SCResult};
