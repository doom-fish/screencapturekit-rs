//! Tests for sync completion utilities

use screencapturekit::utils::sync_completion::{
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
    let mut cx = Context::from_waker(&waker);
    let mut pinned = std::pin::pin!(future);

    match pinned.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(v)) => assert_eq!(v, 42),
        _ => panic!("Expected Ready(Ok(42))"),
    }
}
