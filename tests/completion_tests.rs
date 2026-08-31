//! Tests for completion utilities

use screencapturekit::utils::completion::{
    error_from_cstr, AsyncCompletion, SyncCompletion, UnitCompletion,
};
use std::future::Future;
use std::task::{Context, Poll};

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

#[test]
fn test_async_completion_immediate() {
    let (future, context) = AsyncCompletion::<i32>::create();

    // Complete immediately before polling
    unsafe { AsyncCompletion::complete_ok(context, 42) };

    // Poll should return Ready immediately
    let waker = std::task::Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut pinned = Box::pin(future);

    match pinned.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(v)) => assert_eq!(v, 42),
        _ => panic!("Expected Ready(Ok(42))"),
    }
}

/// Duplicate callbacks must stay harmless even after the consumer has
/// completed and dropped its handle.
#[test]
fn test_sync_completion_double_invocation_is_no_op() {
    let (completion, context) = SyncCompletion::<i32>::new();
    let context_addr = context as usize;

    unsafe { SyncCompletion::complete_ok(context, 7) };
    let result = completion.wait();
    assert_eq!(result, Ok(7), "first invocation's result must win");

    for _ in 0..3 {
        unsafe {
            SyncCompletion::complete_ok(context_addr as *mut std::ffi::c_void, 999);
        }
    }
}

#[test]
fn test_async_completion_double_invocation_is_no_op() {
    let (future, context) = AsyncCompletion::<i32>::create();

    unsafe { AsyncCompletion::complete_ok(context, 1) };
    unsafe { AsyncCompletion::complete_ok(context, 2) };

    let waker = std::task::Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut pinned = Box::pin(future);

    match pinned.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(v)) => assert_eq!(v, 1, "first invocation's result must win"),
        other => panic!("Expected Ready(Ok(1)), got {other:?}"),
    }
    drop(pinned);
    unsafe { AsyncCompletion::complete_ok(context, 3) };
}

/// Null context must be a no-op (defensive programming for Swift bugs).
#[test]
fn test_sync_completion_null_context_is_no_op() {
    // Must not crash.
    unsafe { SyncCompletion::<i32>::complete_ok(std::ptr::null_mut(), 42) };
    unsafe {
        SyncCompletion::<i32>::complete_err(std::ptr::null_mut(), "ignored".to_string());
    }
}

// MARK: - Bounded `wait`

use screencapturekit::utils::completion::{
    default_timeout, is_timeout_error, timed_out_context_count, DEFAULT_TIMEOUT, TIMEOUT_ENV_VAR,
};
use std::time::{Duration, Instant};

/// The timeout counter is process-global.
static TIMEOUT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The whole point of the local completion module: a `ScreenCaptureKit`
/// callback that never fires must not park the caller forever.
#[test]
fn test_wait_timeout_gives_up_instead_of_hanging() {
    let _guard = TIMEOUT_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (completion, _context) = SyncCompletion::<i32>::new();

    let started = Instant::now();
    let result = completion.wait_timeout(Duration::from_millis(50));
    let elapsed = started.elapsed();

    let error = result.expect_err("expected a timeout");
    assert!(is_timeout_error(&error), "unexpected message: {error}");
    assert!(
        elapsed >= Duration::from_millis(50),
        "returned too early: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "waited far past the bound: {elapsed:?}"
    );
}

/// A genuine bridge failure must stay distinguishable from a timeout.
#[test]
fn test_bridge_errors_are_not_reported_as_timeouts() {
    let (completion, context) = SyncCompletion::<i32>::new();
    unsafe { SyncCompletion::<i32>::complete_err(context, "permission denied".to_string()) };

    let error = completion.wait().expect_err("expected an error");
    assert_eq!(error, "permission denied");
    assert!(!is_timeout_error(&error));
}

/// A callback arriving after timeout must be ignored without dereferencing
/// reclaimed memory, including repeated callbacks.
#[test]
fn test_late_callback_after_timeout_is_memory_safe() {
    let _guard = TIMEOUT_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (completion, context) = SyncCompletion::<Vec<u8>>::new();
    let context_addr = context as usize;

    let before = timed_out_context_count();
    let error = completion
        .wait_timeout(Duration::from_millis(20))
        .expect_err("expected a timeout");
    assert!(is_timeout_error(&error));
    assert_eq!(
        timed_out_context_count(),
        before + 1,
        "a timed-out wait must be counted"
    );

    // Swift finally answers, long after nobody is listening.
    unsafe {
        SyncCompletion::<Vec<u8>>::complete_ok(
            context_addr as *mut std::ffi::c_void,
            vec![1, 2, 3, 4],
        );
    }
    for _ in 0..4 {
        unsafe {
            SyncCompletion::<Vec<u8>>::complete_ok(context_addr as *mut std::ffi::c_void, vec![9]);
        }
        unsafe {
            SyncCompletion::<Vec<u8>>::complete_err(
                context_addr as *mut std::ffi::c_void,
                "stray".to_string(),
            );
        }
    }

    assert_eq!(
        timed_out_context_count(),
        before + 1,
        "late callbacks must not alter the timeout count"
    );
}

/// A callback that lands while the waiter is still parked must wake it.
#[test]
fn test_wait_wakes_on_late_callback() {
    let (completion, context) = SyncCompletion::<i32>::new();
    let context_addr = context as usize;

    let handle = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        unsafe { SyncCompletion::<i32>::complete_ok(context_addr as *mut std::ffi::c_void, 5) };
    });

    assert_eq!(completion.wait_timeout(Duration::from_secs(5)), Ok(5));
    handle.join().unwrap();
}

/// `UnitCompletion` is what every `SCStream` control call uses, so its `wait`
/// must be bounded too.
#[test]
fn test_unit_completion_wait_is_bounded() {
    let _guard = TIMEOUT_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (completion, _context) = UnitCompletion::new();

    let error = completion
        .wait_timeout(Duration::from_millis(50))
        .expect_err("expected a timeout");
    assert!(is_timeout_error(&error));
}

#[test]
fn test_default_timeout_is_bounded_and_generous() {
    if std::env::var(TIMEOUT_ENV_VAR).is_ok() {
        // The override is process-wide; honour it rather than asserting the
        // built-in default.
        return;
    }
    assert_eq!(default_timeout(), Some(DEFAULT_TIMEOUT));
    assert!(DEFAULT_TIMEOUT >= Duration::from_secs(10));
}

/// `wait_forever` keeps the historical unbounded behaviour available for
/// callers that know the callback is guaranteed.
#[test]
fn test_wait_forever_still_resolves() {
    let (completion, context) = SyncCompletion::<i32>::new();
    unsafe { SyncCompletion::complete_ok(context, 11) };
    assert_eq!(completion.wait_forever(), Ok(11));
}

/// Dropping an async future cancels its registry entry; late and repeated
/// callbacks must be harmless.
#[test]
fn test_async_completion_survives_dropped_future() {
    let (future, context) = AsyncCompletion::<String>::create();
    drop(future);

    unsafe { AsyncCompletion::<String>::complete_ok(context, "late".to_string()) };
    unsafe { AsyncCompletion::<String>::complete_ok(context, "later".to_string()) };
}
