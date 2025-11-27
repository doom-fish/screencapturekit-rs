//! Dispatch Queue wrapper for custom queue management
//!
//! This module provides a safe Rust wrapper around GCD (Grand Central Dispatch) queues
//! that can be used with `ScreenCaptureKit` streams.

use std::ffi::{c_void, CString};
use std::fmt;

/// Quality of Service levels for dispatch queues
///
/// These `QoS` levels help the system prioritize work appropriately.
///
/// # Examples
///
/// ```
/// use screencapturekit::dispatch_queue::{DispatchQueue, DispatchQoS};
///
/// // High priority for UI-affecting work
/// let queue = DispatchQueue::new("com.myapp.ui", DispatchQoS::UserInteractive);
///
/// // Lower priority for background tasks
/// let bg_queue = DispatchQueue::new("com.myapp.background", DispatchQoS::Background);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DispatchQoS {
    /// Background `QoS` - for maintenance or cleanup tasks
    Background = 0,
    /// Utility `QoS` - for tasks that may take some time
    Utility = 1,
    /// Default `QoS` - standard priority
    #[default]
    Default = 2,
    /// User Initiated `QoS` - for tasks initiated by the user
    UserInitiated = 3,
    /// User Interactive `QoS` - for tasks that affect the UI
    UserInteractive = 4,
}

/// A wrapper around GCD `DispatchQueue`
///
/// This allows you to provide a custom dispatch queue for stream output handling
/// instead of using the default queue.
///
/// # Example
///
/// ```no_run
/// use screencapturekit::dispatch_queue::{DispatchQueue, DispatchQoS};
///
/// let queue = DispatchQueue::new("com.myapp.capture", DispatchQoS::UserInteractive);
/// ```
pub struct DispatchQueue {
    ptr: *const c_void,
}

unsafe impl Send for DispatchQueue {}
unsafe impl Sync for DispatchQueue {}

impl DispatchQueue {
    /// Creates a new dispatch queue with the specified label and `QoS`
    ///
    /// # Arguments
    ///
    /// * `label` - A string label for the queue (e.g., "com.myapp.capture")
    /// * `qos` - The quality of service level for the queue
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::dispatch_queue::{DispatchQueue, DispatchQoS};
    ///
    /// let queue = DispatchQueue::new("com.myapp.capture", DispatchQoS::UserInteractive);
    /// // Use the queue with SCStream's add_output_handler_with_queue
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the label contains null bytes or if queue creation fails
    pub fn new(label: &str, qos: DispatchQoS) -> Self {
        let c_label = CString::new(label).expect("Label contains null byte");
        let ptr = unsafe { crate::ffi::dispatch_queue_create(c_label.as_ptr(), qos as i32) };
        assert!(!ptr.is_null(), "Failed to create dispatch queue");
        Self { ptr }
    }

    /// Returns the raw pointer to the dispatch queue
    ///
    /// This is used internally for FFI calls (and for testing)
    pub fn as_ptr(&self) -> *const c_void {
        self.ptr
    }
}

impl Drop for DispatchQueue {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::dispatch_queue_release(self.ptr);
        }
    }
}

impl fmt::Debug for DispatchQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DispatchQueue")
            .field("ptr", &self.ptr)
            .finish()
    }
}

impl fmt::Display for DispatchQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DispatchQueue")
    }
}
