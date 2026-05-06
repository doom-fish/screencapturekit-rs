//! Panic-safety helpers for the C ABI boundary.
//!
//! Rust panics that unwind across `extern "C"` into Swift are undefined
//! behaviour. Every extern callback in this crate that invokes user code
//! must wrap that invocation in [`catch_user_panic`], which catches the
//! panic, logs a best-effort diagnostic to stderr, and never returns a
//! panic to its caller.
//!
//! This is intentionally a single shared helper rather than ad-hoc
//! `catch_unwind` calls so the diagnostic format and the
//! "stderr-write itself might panic" defence-in-depth stay consistent.

use std::any::Any;
use std::panic::AssertUnwindSafe;

/// Run `f` and swallow any panic it produces.
///
/// On panic, writes a best-effort diagnostic to stderr identifying the
/// callback site and the panic message (when the payload is a `&str` or
/// `String`). The diagnostic write is itself wrapped in `catch_unwind`
/// so an allocator failure or broken stderr can never propagate out.
///
/// `AssertUnwindSafe` is required because trait objects are not
/// generally `UnwindSafe` and we accept the user's responsibility for
/// their own state consistency on panic.
pub fn catch_user_panic<F: FnOnce()>(site: &str, f: F) {
    if let Err(payload) = std::panic::catch_unwind(AssertUnwindSafe(f)) {
        log_callback_panic(site, payload.as_ref());
    }
}

/// Best-effort logger for panics caught at the C ABI boundary.
///
/// Public to support call sites that already have a panic payload
/// (e.g. those that need to dispatch multiple callbacks individually).
/// Most callers want [`catch_user_panic`] instead.
pub fn log_callback_panic(site: &str, payload: &(dyn Any + Send)) {
    let message = payload.downcast_ref::<&'static str>().map_or_else(
        || {
            payload
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "<non-string panic payload>".to_string())
        },
        |s| (*s).to_string(),
    );
    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        eprintln!("screencapturekit: panic in {site} caught at C ABI boundary: {message}");
    }));
}
