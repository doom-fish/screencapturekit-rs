//! Synchronous completion utilities for async FFI callbacks
//!
//! This module provides a generic mechanism for blocking on async Swift FFI callbacks
//! and propagating results (success or error) back to Rust synchronously.
//!
//! # Example
//!
//! ```no_run
//! use screencapturekit::utils::completion::SyncCompletion;
//!
//! // Create completion for a String result
//! let (completion, _context) = SyncCompletion::<String>::new();
//!
//! // In real use, context would be passed to FFI callback
//! // The callback would signal completion with a result
//!
//! // Block until callback completes (would hang without callback)
//! // let result = completion.wait();
//! ```

use std::ffi::{c_void, CStr};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, Waker};

// ============================================================================
// Synchronous Completion (blocking)
// ============================================================================

/// Internal state for tracking synchronous completion
struct SyncCompletionState<T> {
    completed: bool,
    result: Option<Result<T, String>>,
}

/// Backing storage for `SyncCompletion` — held behind an `Arc` so the
/// callback path can access the `consumed` flag without taking the mutex.
struct SyncCompletionInner<T> {
    /// Atomic guard that ensures `Arc::from_raw` is invoked at most once per
    /// context pointer. Set to `true` on the first completion callback;
    /// subsequent (erroneous) callbacks see `true` and bail out without
    /// touching the `Arc`, preventing the double-`from_raw` UAF/double-free.
    consumed: AtomicBool,
    state: Mutex<SyncCompletionState<T>>,
    cvar: Condvar,
}

/// A synchronous completion handler for async FFI callbacks
///
/// This type provides a way to block until an async callback completes
/// and retrieve the result. It uses `Arc<...>` internally for thread-safe
/// signaling between the callback and the waiting thread, with an
/// `AtomicBool` guard that defends against Swift firing the completion
/// callback more than once (which would otherwise be use-after-free in
/// `Arc::from_raw`).
pub struct SyncCompletion<T> {
    inner: Arc<SyncCompletionInner<T>>,
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
        let inner = Arc::new(SyncCompletionInner {
            consumed: AtomicBool::new(false),
            state: Mutex::new(SyncCompletionState {
                completed: false,
                result: None,
            }),
            cvar: Condvar::new(),
        });
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
        let mut state = self.inner.state.lock().unwrap();
        while !state.completed {
            state = self.inner.cvar.wait(state).unwrap();
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
    /// The `context` pointer must be a valid pointer obtained from
    /// `SyncCompletion::new()`. The intended contract is that the callback
    /// fires exactly once. If Swift erroneously fires it twice, the
    /// `consumed` `AtomicBool` ensures the second invocation is a no-op
    /// rather than triggering a double-`Arc::from_raw` (which would be UB).
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub unsafe fn complete_with_result(context: SyncCompletionPtr, result: Result<T, String>) {
        if context.is_null() {
            return;
        }

        // Atomic guard against double-invocation. We deref the raw pointer
        // *without* taking ownership of the Arc reference; only the call
        // that wins the swap proceeds to `Arc::from_raw`.
        let inner_ref = unsafe { &*context.cast::<SyncCompletionInner<T>>() };
        if inner_ref.consumed.swap(true, Ordering::AcqRel) {
            eprintln!(
                "screencapturekit: SyncCompletion callback fired more than once; \
                 ignoring duplicate to avoid double-free"
            );
            return;
        }

        let inner = unsafe { Arc::from_raw(context.cast::<SyncCompletionInner<T>>()) };
        {
            let mut state = inner.state.lock().unwrap();
            state.completed = true;
            state.result = Some(result);
        }
        inner.cvar.notify_one();
    }
}

impl<T> Default for SyncCompletion<T> {
    fn default() -> Self {
        Self::new().0
    }
}

// ============================================================================
// Asynchronous Completion (Future-based)
// ============================================================================

/// Internal state for tracking async completion
struct AsyncCompletionState<T> {
    result: Option<Result<T, String>>,
    waker: Option<Waker>,
}

/// Backing storage for `AsyncCompletion` — held behind an `Arc`. The
/// `consumed` flag protects against Swift double-firing the completion
/// callback (see `SyncCompletionInner` for the same rationale).
struct AsyncCompletionInner<T> {
    consumed: AtomicBool,
    state: Mutex<AsyncCompletionState<T>>,
}

/// An async completion handler for FFI callbacks
///
/// This type provides a `Future` that resolves when an async callback completes.
/// It uses `Arc<Mutex>` internally for thread-safe signaling and waker management.
pub struct AsyncCompletion<T> {
    _marker: std::marker::PhantomData<T>,
}

/// Future returned by `AsyncCompletion`
pub struct AsyncCompletionFuture<T> {
    inner: Arc<AsyncCompletionInner<T>>,
}

impl<T> AsyncCompletion<T> {
    /// Create a new async completion handler and return the context pointer for FFI
    ///
    /// Returns a tuple of (future, `context_ptr`) where:
    /// - `future` can be awaited to get the result
    /// - `context_ptr` should be passed to the FFI callback
    #[must_use]
    pub fn create() -> (AsyncCompletionFuture<T>, SyncCompletionPtr) {
        let inner = Arc::new(AsyncCompletionInner {
            consumed: AtomicBool::new(false),
            state: Mutex::new(AsyncCompletionState {
                result: None,
                waker: None,
            }),
        });
        let raw = Arc::into_raw(Arc::clone(&inner));
        (AsyncCompletionFuture { inner }, raw as SyncCompletionPtr)
    }

    /// Signal successful completion with a value
    ///
    /// # Safety
    ///
    /// The `context` pointer must be a valid pointer obtained from `AsyncCompletion::new()`.
    /// This function consumes the Arc reference, so it must only be called once per context.
    pub unsafe fn complete_ok(context: SyncCompletionPtr, value: T) {
        Self::complete_with_result(context, Ok(value));
    }

    /// Signal completion with an error
    ///
    /// # Safety
    ///
    /// The `context` pointer must be a valid pointer obtained from `AsyncCompletion::new()`.
    /// This function consumes the Arc reference, so it must only be called once per context.
    pub unsafe fn complete_err(context: SyncCompletionPtr, error: String) {
        Self::complete_with_result(context, Err(error));
    }

    /// Signal completion with a result
    ///
    /// # Safety
    ///
    /// The `context` pointer must be a valid pointer obtained from
    /// `AsyncCompletion::create()`. The intended contract is that the
    /// callback fires exactly once. If Swift erroneously fires it twice,
    /// the `consumed` `AtomicBool` ensures the second invocation is a
    /// no-op rather than triggering a double-`Arc::from_raw`.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub unsafe fn complete_with_result(context: SyncCompletionPtr, result: Result<T, String>) {
        if context.is_null() {
            return;
        }

        let inner_ref = unsafe { &*context.cast::<AsyncCompletionInner<T>>() };
        if inner_ref.consumed.swap(true, Ordering::AcqRel) {
            eprintln!(
                "screencapturekit: AsyncCompletion callback fired more than once; \
                 ignoring duplicate to avoid double-free"
            );
            return;
        }

        let inner = unsafe { Arc::from_raw(context.cast::<AsyncCompletionInner<T>>()) };

        let waker = {
            let mut state = inner.state.lock().unwrap();
            state.result = Some(result);
            state.waker.take()
        };

        if let Some(w) = waker {
            w.wake();
        }

        // Drop the Arc here - the refcount was incremented in create() via Arc::clone(),
        // so the data stays alive via the AsyncCompletionFuture's Arc until it's dropped.
        // Dropping here decrements the refcount from the into_raw() call.
    }
}

impl<T> Future for AsyncCompletionFuture<T> {
    type Output = Result<T, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.inner.state.lock().unwrap();

        state.result.take().map_or_else(
            || {
                // Avoid the lost-wakeup race: when the executor re-polls
                // with a different waker (e.g. tokio::select! moves the
                // future between arms), the previous waker would otherwise
                // remain stored and any pending callback would wake the
                // wrong task. `will_wake` skips the clone if the executor
                // is reusing the same waker.
                let waker = cx.waker();
                match state.waker {
                    Some(ref existing) if existing.will_wake(waker) => {}
                    _ => state.waker = Some(waker.clone()),
                }
                Poll::Pending
            },
            Poll::Ready,
        )
    }
}

// ============================================================================
// Shared Utilities
// ============================================================================

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
