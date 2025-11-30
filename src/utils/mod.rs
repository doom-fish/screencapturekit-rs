//! Utility modules
//!
//! This module contains helper types and functions used throughout the library.
//!
//! ## Modules
//!
//! - [`error`] - Error types and result aliases
//! - [`ffi_string`] - FFI string retrieval utilities
//! - [`four_char_code`] - Four-character code handling (used for pixel formats, codecs)
//! - [`sync_completion`] - Completion utilities for async FFI callbacks

pub mod error;
pub mod ffi_string;
pub mod four_char_code;
pub mod sync_completion;
