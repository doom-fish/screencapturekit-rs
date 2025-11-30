//! `SCStreamDelegateTrait` tests

use screencapturekit::error::SCError;
use screencapturekit::stream::delegate_trait::{ErrorHandler, SCStreamDelegateTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct TestDelegate {
    stopped: Arc<AtomicBool>,
    error_received: Arc<AtomicBool>,
}

impl SCStreamDelegateTrait for TestDelegate {
    fn stream_did_stop(&self, _error: Option<String>) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    fn did_stop_with_error(&self, _error: SCError) {
        self.error_received.store(true, Ordering::SeqCst);
    }
}

#[test]
fn test_delegate_trait_implementation() {
    let stopped = Arc::new(AtomicBool::new(false));
    let error_received = Arc::new(AtomicBool::new(false));

    let delegate = TestDelegate {
        stopped: Arc::clone(&stopped),
        error_received: Arc::clone(&error_received),
    };

    // Call the delegate methods
    delegate.stream_did_stop(None);
    assert!(stopped.load(Ordering::SeqCst));

    delegate.did_stop_with_error(SCError::internal_error("test"));
    assert!(error_received.load(Ordering::SeqCst));
}

#[test]
fn test_delegate_default_implementations() {
    struct MinimalDelegate;
    impl SCStreamDelegateTrait for MinimalDelegate {}

    let delegate = MinimalDelegate;
    // These should not panic - they have default empty implementations
    delegate.output_video_effect_did_start_for_stream();
    delegate.output_video_effect_did_stop_for_stream();
    delegate.did_stop_with_error(SCError::internal_error("test"));
    delegate.stream_did_stop(Some("test".to_string()));
}

#[test]
fn test_error_handler_creation() {
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = Arc::clone(&called);

    let handler = ErrorHandler::new(move |_error| {
        called_clone.store(true, Ordering::SeqCst);
    });

    // Call the delegate method
    handler.did_stop_with_error(SCError::internal_error("test error"));
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_error_handler_receives_error() {
    let error_msg = Arc::new(std::sync::Mutex::new(String::new()));
    let error_msg_clone = Arc::clone(&error_msg);

    let handler = ErrorHandler::new(move |error| {
        *error_msg_clone.lock().unwrap() = format!("{error}");
    });

    handler.did_stop_with_error(SCError::internal_error("specific error"));

    assert!(error_msg.lock().unwrap().contains("specific error"));
}

#[test]
fn test_delegate_trait_send() {
    fn assert_send<T: Send>() {}
    fn check_send<T: Send>(_: &T) {}

    assert_send::<TestDelegate>();

    // ErrorHandler with Send closure should be Send
    let handler = ErrorHandler::new(|_| {});
    check_send(&handler);
}

#[test]
#[cfg(feature = "macos_15_2")]
fn test_stream_did_become_active_inactive() {
    use std::sync::atomic::AtomicU32;

    struct ActivityDelegate {
        active_count: Arc<AtomicU32>,
        inactive_count: Arc<AtomicU32>,
    }

    impl SCStreamDelegateTrait for ActivityDelegate {
        fn stream_did_become_active(&self) {
            self.active_count.fetch_add(1, Ordering::SeqCst);
        }

        fn stream_did_become_inactive(&self) {
            self.inactive_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    let active_count = Arc::new(AtomicU32::new(0));
    let inactive_count = Arc::new(AtomicU32::new(0));

    let delegate = ActivityDelegate {
        active_count: Arc::clone(&active_count),
        inactive_count: Arc::clone(&inactive_count),
    };

    // Simulate activity changes
    delegate.stream_did_become_active();
    delegate.stream_did_become_inactive();
    delegate.stream_did_become_active();

    assert_eq!(active_count.load(Ordering::SeqCst), 2);
    assert_eq!(inactive_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_stream_activity_default_implementations() {
    struct MinimalDelegate;
    impl SCStreamDelegateTrait for MinimalDelegate {}

    let delegate = MinimalDelegate;

    // These should not panic - they have default empty implementations
    delegate.stream_did_become_active();
    delegate.stream_did_become_inactive();
}
