#![allow(clippy::pedantic, clippy::nursery)]
//! Tests for dispatch queue functionality

use screencapturekit::dispatch_queue::{DispatchQueue, DispatchQoS};

#[test]
fn test_dispatch_queue_creation() {
    let queue = DispatchQueue::new("com.test.queue", DispatchQoS::Default);
    assert!(!queue.as_ptr().is_null());
}

#[test]
fn test_dispatch_queue_all_qos_levels() {
    let qos_levels = vec![
        DispatchQoS::Background,
        DispatchQoS::Utility,
        DispatchQoS::Default,
        DispatchQoS::UserInitiated,
        DispatchQoS::UserInteractive,
    ];

    for qos in qos_levels {
        let queue = DispatchQueue::new("com.test.queue", qos);
        assert!(!queue.as_ptr().is_null());
    }
}

#[test]
fn test_dispatch_queue_with_different_labels() {
    let queue1 = DispatchQueue::new("com.test.queue1", DispatchQoS::Default);
    let queue2 = DispatchQueue::new("com.test.queue2", DispatchQoS::Default);
    
    assert!(!queue1.as_ptr().is_null());
    assert!(!queue2.as_ptr().is_null());
    assert_ne!(queue1.as_ptr(), queue2.as_ptr());
}

#[test]
#[should_panic(expected = "Label contains null byte")]
fn test_dispatch_queue_null_byte_in_label() {
    let _queue = DispatchQueue::new("com.test\0.queue", DispatchQoS::Default);
}

#[test]
fn test_dispatch_queue_drop() {
    // This test verifies that dropping a queue doesn't panic
    {
        let _queue = DispatchQueue::new("com.test.queue", DispatchQoS::Default);
    } // Queue should be dropped here without issues
}
