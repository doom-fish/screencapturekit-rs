//! `SCStreamDelegateTrait` tests

use screencapturekit::error::SCError;
use screencapturekit::stream::delegate_trait::{
    ErrorHandler, SCStreamDelegateTrait, StreamCallbacks,
};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
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

// MARK: - StreamCallbacks Tests

#[test]
fn test_stream_callbacks_new() {
    let callbacks = StreamCallbacks::new();
    // Should not panic
    drop(callbacks);
}

#[test]
fn test_stream_callbacks_default() {
    let callbacks = StreamCallbacks::default();
    // Should not panic
    drop(callbacks);
}

#[test]
fn test_stream_callbacks_on_stop() {
    let called = Arc::new(AtomicBool::new(false));
    let error_msg = Arc::new(std::sync::Mutex::new(Option::<String>::None));

    let called_clone = Arc::clone(&called);
    let error_msg_clone = Arc::clone(&error_msg);

    let callbacks = StreamCallbacks::new().on_stop(move |error| {
        called_clone.store(true, Ordering::SeqCst);
        *error_msg_clone.lock().unwrap() = error;
    });

    // Test with no error
    callbacks.stream_did_stop(None);
    assert!(called.load(Ordering::SeqCst));
    assert!(error_msg.lock().unwrap().is_none());

    // Test with error
    called.store(false, Ordering::SeqCst);
    callbacks.stream_did_stop(Some("test error".to_string()));
    assert!(called.load(Ordering::SeqCst));
    assert_eq!(error_msg.lock().unwrap().as_deref(), Some("test error"));
}

#[test]
fn test_stream_callbacks_on_error() {
    let error_received = Arc::new(std::sync::Mutex::new(String::new()));
    let error_clone = Arc::clone(&error_received);

    let callbacks = StreamCallbacks::new().on_error(move |error| {
        *error_clone.lock().unwrap() = format!("{error}");
    });

    callbacks.did_stop_with_error(SCError::internal_error("callback error"));
    assert!(error_received.lock().unwrap().contains("callback error"));
}

#[test]
fn test_stream_callbacks_on_active_inactive() {
    let active_count = Arc::new(AtomicU32::new(0));
    let inactive_count = Arc::new(AtomicU32::new(0));

    let active_clone = Arc::clone(&active_count);
    let inactive_clone = Arc::clone(&inactive_count);

    let callbacks = StreamCallbacks::new()
        .on_active(move || {
            active_clone.fetch_add(1, Ordering::SeqCst);
        })
        .on_inactive(move || {
            inactive_clone.fetch_add(1, Ordering::SeqCst);
        });

    callbacks.stream_did_become_active();
    callbacks.stream_did_become_inactive();
    callbacks.stream_did_become_active();

    assert_eq!(active_count.load(Ordering::SeqCst), 2);
    assert_eq!(inactive_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_stream_callbacks_on_video_effects() {
    let start_count = Arc::new(AtomicU32::new(0));
    let stop_count = Arc::new(AtomicU32::new(0));

    let start_clone = Arc::clone(&start_count);
    let stop_clone = Arc::clone(&stop_count);

    let callbacks = StreamCallbacks::new()
        .on_video_effect_start(move || {
            start_clone.fetch_add(1, Ordering::SeqCst);
        })
        .on_video_effect_stop(move || {
            stop_clone.fetch_add(1, Ordering::SeqCst);
        });

    callbacks.output_video_effect_did_start_for_stream();
    callbacks.output_video_effect_did_stop_for_stream();

    assert_eq!(start_count.load(Ordering::SeqCst), 1);
    assert_eq!(stop_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_stream_callbacks_all_callbacks() {
    let stop_called = Arc::new(AtomicBool::new(false));
    let error_called = Arc::new(AtomicBool::new(false));
    let active_called = Arc::new(AtomicBool::new(false));
    let inactive_called = Arc::new(AtomicBool::new(false));
    let video_start_called = Arc::new(AtomicBool::new(false));
    let video_stop_called = Arc::new(AtomicBool::new(false));

    let stop_clone = Arc::clone(&stop_called);
    let error_clone = Arc::clone(&error_called);
    let active_clone = Arc::clone(&active_called);
    let inactive_clone = Arc::clone(&inactive_called);
    let video_start_clone = Arc::clone(&video_start_called);
    let video_stop_clone = Arc::clone(&video_stop_called);

    let callbacks = StreamCallbacks::new()
        .on_stop(move |_| stop_clone.store(true, Ordering::SeqCst))
        .on_error(move |_| error_clone.store(true, Ordering::SeqCst))
        .on_active(move || active_clone.store(true, Ordering::SeqCst))
        .on_inactive(move || inactive_clone.store(true, Ordering::SeqCst))
        .on_video_effect_start(move || video_start_clone.store(true, Ordering::SeqCst))
        .on_video_effect_stop(move || video_stop_clone.store(true, Ordering::SeqCst));

    // Trigger all callbacks
    callbacks.stream_did_stop(None);
    callbacks.did_stop_with_error(SCError::internal_error("test"));
    callbacks.stream_did_become_active();
    callbacks.stream_did_become_inactive();
    callbacks.output_video_effect_did_start_for_stream();
    callbacks.output_video_effect_did_stop_for_stream();

    // Verify all were called
    assert!(stop_called.load(Ordering::SeqCst));
    assert!(error_called.load(Ordering::SeqCst));
    assert!(active_called.load(Ordering::SeqCst));
    assert!(inactive_called.load(Ordering::SeqCst));
    assert!(video_start_called.load(Ordering::SeqCst));
    assert!(video_stop_called.load(Ordering::SeqCst));
}

#[test]
fn test_stream_callbacks_without_handlers() {
    // Test that callbacks without handlers don't panic
    let callbacks = StreamCallbacks::new();

    callbacks.stream_did_stop(None);
    callbacks.stream_did_stop(Some("error".to_string()));
    callbacks.did_stop_with_error(SCError::internal_error("test"));
    callbacks.stream_did_become_active();
    callbacks.stream_did_become_inactive();
    callbacks.output_video_effect_did_start_for_stream();
    callbacks.output_video_effect_did_stop_for_stream();
}

#[test]
fn test_stream_callbacks_partial_handlers() {
    let active_called = Arc::new(AtomicBool::new(false));
    let active_clone = Arc::clone(&active_called);

    // Only set one callback
    let callbacks =
        StreamCallbacks::new().on_active(move || active_clone.store(true, Ordering::SeqCst));

    // Call all methods - only the one with handler should do anything
    callbacks.stream_did_stop(None);
    callbacks.did_stop_with_error(SCError::internal_error("test"));
    callbacks.stream_did_become_active();
    callbacks.stream_did_become_inactive();

    assert!(active_called.load(Ordering::SeqCst));
}

#[test]
fn test_stream_callbacks_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<StreamCallbacks>();
}

// MARK: - Full Delegate Implementation Tests

#[test]
fn test_full_delegate_with_all_callbacks() {
    struct FullDelegate {
        stop_count: Arc<AtomicU32>,
        error_count: Arc<AtomicU32>,
        active_count: Arc<AtomicU32>,
        inactive_count: Arc<AtomicU32>,
        video_start_count: Arc<AtomicU32>,
        video_stop_count: Arc<AtomicU32>,
    }

    impl SCStreamDelegateTrait for FullDelegate {
        fn stream_did_stop(&self, _error: Option<String>) {
            self.stop_count.fetch_add(1, Ordering::SeqCst);
        }

        fn did_stop_with_error(&self, _error: SCError) {
            self.error_count.fetch_add(1, Ordering::SeqCst);
        }

        fn stream_did_become_active(&self) {
            self.active_count.fetch_add(1, Ordering::SeqCst);
        }

        fn stream_did_become_inactive(&self) {
            self.inactive_count.fetch_add(1, Ordering::SeqCst);
        }

        fn output_video_effect_did_start_for_stream(&self) {
            self.video_start_count.fetch_add(1, Ordering::SeqCst);
        }

        fn output_video_effect_did_stop_for_stream(&self) {
            self.video_stop_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    let delegate = FullDelegate {
        stop_count: Arc::new(AtomicU32::new(0)),
        error_count: Arc::new(AtomicU32::new(0)),
        active_count: Arc::new(AtomicU32::new(0)),
        inactive_count: Arc::new(AtomicU32::new(0)),
        video_start_count: Arc::new(AtomicU32::new(0)),
        video_stop_count: Arc::new(AtomicU32::new(0)),
    };

    // Call each method multiple times
    delegate.stream_did_stop(None);
    delegate.stream_did_stop(Some("error".to_string()));
    delegate.did_stop_with_error(SCError::internal_error("test"));
    delegate.stream_did_become_active();
    delegate.stream_did_become_active();
    delegate.stream_did_become_inactive();
    delegate.output_video_effect_did_start_for_stream();
    delegate.output_video_effect_did_stop_for_stream();

    assert_eq!(delegate.stop_count.load(Ordering::SeqCst), 2);
    assert_eq!(delegate.error_count.load(Ordering::SeqCst), 1);
    assert_eq!(delegate.active_count.load(Ordering::SeqCst), 2);
    assert_eq!(delegate.inactive_count.load(Ordering::SeqCst), 1);
    assert_eq!(delegate.video_start_count.load(Ordering::SeqCst), 1);
    assert_eq!(delegate.video_stop_count.load(Ordering::SeqCst), 1);
}
