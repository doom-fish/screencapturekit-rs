//! Shared utilities.
//!
//! The framework-agnostic helpers (`FourCharCode`, `ffi_string_*`, panic-safe
//! wrappers) now live in `apple_cf::utils` and are re-exported here for
//! backward compatibility.
//!
//! `completion.rs` is intentionally NOT re-exported from `apple_cf`: it is an
//! API-compatible local replacement whose `SyncCompletion::wait` is bounded, so
//! a `ScreenCaptureKit` callback that never fires can't hang the caller. See
//! the module docs for the timeout policy.
//!
//! `error.rs` is intentionally NOT migrated — it carries SCStream-specific
//! error variants that don't belong in the framework-agnostic foundation.

pub mod completion;
pub mod error;
pub(crate) mod retained;

pub use apple_cf::utils::FourCharCode;
pub use apple_cf::utils::{ffi_string, four_char_code, panic_safe};
