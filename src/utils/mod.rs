//! Shared utilities.
//!
//! The framework-agnostic helpers (`FourCharCode`, `ffi_string_*`, completion
//! handlers, panic-safe wrappers) now live in `apple_cf::utils` and are
//! re-exported here for backward compatibility.
//!
//! `error.rs` is intentionally NOT migrated — it carries SCStream-specific
//! error variants that don't belong in the framework-agnostic foundation.

pub mod error;
pub(crate) mod retained;

pub use apple_cf::utils::FourCharCode;
pub use apple_cf::utils::{completion, ffi_string, four_char_code, panic_safe};
