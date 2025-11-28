//! Synchronous completion utilities for async FFI callbacks
//!
//! This module provides a generic mechanism for blocking on async Swift FFI callbacks
//! and propagating results (success or error) back to Rust synchronously.
//!
//! # Example
//!
//! ```ignore
//! use screencapturekit::utils::sync_completion::{SyncCompletion, SyncCompletionPtr};
//!
//! // Create completion for a String result
//! let (completion, context) = SyncCompletion::<String>::new();
//!
//! // Pass context to FFI, which will call back with result
//! unsafe { some_ffi_call(context, callback) };
//!
//! // Block until callback completes
//! let result = completion.wait();
//! ```

use std::ffi::{c_void, CStr};
use std::sync::{Arc, Condvar, Mutex};

/// Internal state for tracking completion
struct CompletionState<T> {
    completed: bool,
    result: Option<Result<T, String>>,
}

/// A synchronous completion handler for async FFI callbacks
///
/// This type provides a way to block until an async callback completes
/// and retrieve the result. It uses `Arc<(Mutex, Condvar)>` internally
/// for thread-safe signaling between the callback and the waiting thread.
pub struct SyncCompletion<T> {
    inner: Arc<(Mutex<CompletionState<T>>, Condvar)>,
}

/// Raw pointer type for passing to FFI callbacks
pub type SyncCompletionPtr = *mut c_void;

impl<T> SyncCompletion<T> {
    /// Create a new completion handler and return the context pointer for FFI
    ///
    /// Returns a tuple of (completion, `context_ptr`) where:
    /// - `completion` is used to wait for and retrieve the result
    /// - `context_ptr` should be passed to the FFI callback
    #[must_use]
    pub fn new() -> (Self, SyncCompletionPtr) {
        let inner = Arc::new((
            Mutex::new(CompletionState {
                completed: false,
                result: None,
            }),
            Condvar::new(),
        ));
        let raw = Arc::into_raw(Arc::clone(&inner));
        (Self { inner }, raw as SyncCompletionPtr)
    }

    /// Wait for the completion callback and return the result
    ///
    /// This method blocks until the callback signals completion.
    ///
    /// # Errors
    ///
    /// Returns an error string if the callback signaled an error.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn wait(self) -> Result<T, String> {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        while !state.completed {
            state = cvar.wait(state).unwrap();
        }
        state
            .result
            .take()
            .unwrap_or_else(|| Err("Completion signaled without result".to_string()))
    }

    /// Signal successful completion with a value
    ///
    /// # Safety
    ///
    /// The `context` pointer must be a valid pointer obtained from `SyncCompletion::new()`.
    /// This function consumes the Arc reference, so it must only be called once per context.
    pub unsafe fn complete_ok(context: SyncCompletionPtr, value: T) {
        Self::complete_with_result(context, Ok(value));
    }

    /// Signal completion with an error
    ///
    /// # Safety
    ///
    /// The `context` pointer must be a valid pointer obtained from `SyncCompletion::new()`.
    /// This function consumes the Arc reference, so it must only be called once per context.
    pub unsafe fn complete_err(context: SyncCompletionPtr, error: String) {
        Self::complete_with_result(context, Err(error));
    }

    /// Signal completion with a result
    ///
    /// # Safety
    ///
    /// The `context` pointer must be a valid pointer obtained from `SyncCompletion::new()`.
    /// This function consumes the Arc reference, so it must only be called once per context.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub unsafe fn complete_with_result(context: SyncCompletionPtr, result: Result<T, String>) {
        if context.is_null() {
            return;
        }
        let inner = Arc::from_raw(context.cast::<(Mutex<CompletionState<T>>, Condvar)>());
        let (lock, cvar) = &*inner;
        {
            let mut state = lock.lock().unwrap();
            state.completed = true;
            state.result = Some(result);
        }
        cvar.notify_one();
    }
}

impl<T> Default for SyncCompletion<T> {
    fn default() -> Self {
        Self::new().0
    }
}

/// Helper to extract error message from a C string pointer
///
/// # Safety
///
/// The `msg` pointer must be either null or point to a valid null-terminated C string.
#[must_use]
pub unsafe fn error_from_cstr(msg: *const i8) -> String {
    if msg.is_null() {
        "Unknown error".to_string()
    } else {
        CStr::from_ptr(msg)
            .to_str()
            .map_or_else(|_| "Unknown error".to_string(), String::from)
    }
}

/// Unit completion - for operations that return success/error without a value
pub type UnitCompletion = SyncCompletion<()>;

impl UnitCompletion {
    /// C callback for operations that return (context, success, `error_msg`)
    ///
    /// This can be used directly as an FFI callback function.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub extern "C" fn callback(context: *mut c_void, success: bool, msg: *const i8) {
        if success {
            unsafe { Self::complete_ok(context, ()) };
        } else {
            let error = unsafe { error_from_cstr(msg) };
            unsafe { Self::complete_err(context, error) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_completion_success() {
        let (completion, context) = SyncCompletion::<i32>::new();

        // Simulate callback being called (normally from FFI)
        unsafe { SyncCompletion::complete_ok(context, 42) };

        let result = completion.wait();
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_sync_completion_error() {
        let (completion, context) = SyncCompletion::<i32>::new();

        // Simulate callback being called with error
        unsafe { SyncCompletion::<i32>::complete_err(context, "test error".to_string()) };

        let result = completion.wait();
        assert_eq!(result, Err("test error".to_string()));
    }

    #[test]
    fn test_unit_completion_callback_success() {
        let (completion, context) = UnitCompletion::new();

        // Simulate successful callback
        UnitCompletion::callback(context, true, std::ptr::null());

        let result = completion.wait();
        assert!(result.is_ok());
    }

    #[test]
    fn test_unit_completion_callback_error() {
        let (completion, context) = UnitCompletion::new();
        let error_msg = std::ffi::CString::new("test error").unwrap();

        // Simulate error callback
        UnitCompletion::callback(context, false, error_msg.as_ptr());

        let result = completion.wait();
        assert_eq!(result, Err("test error".to_string()));
    }

    #[test]
    fn test_error_from_cstr_null() {
        let result = unsafe { error_from_cstr(std::ptr::null()) };
        assert_eq!(result, "Unknown error");
    }

    #[test]
    fn test_error_from_cstr_valid() {
        let msg = std::ffi::CString::new("hello").unwrap();
        let result = unsafe { error_from_cstr(msg.as_ptr()) };
        assert_eq!(result, "hello");
    }
}
