//! `SCContentSharingPicker` - UI for selecting content to share
//!
//! Available on macOS 14.0+
//! Provides a system UI for users to select displays, windows, or applications to share.
//!
//! Requires the `macos_14_0` feature flag to be enabled.

use crate::shareable_content::{SCDisplay, SCRunningApplication, SCWindow};
use crate::utils::sync_completion::SyncCompletion;
use std::ffi::c_void;

extern "C" fn picker_callback(result_code: i32, stream_ptr: *const c_void, user_data: *mut c_void) {
    let result = match result_code {
        0 => SCContentSharingPickerResult::Cancelled,
        1 if !stream_ptr.is_null() => {
            // For now, return Cancelled since we need the stream to be properly typed
            // In a real implementation, we'd need to extract the content from the stream
            SCContentSharingPickerResult::Cancelled
        }
        -1 => SCContentSharingPickerResult::Error("Picker failed".to_string()),
        _ => SCContentSharingPickerResult::Cancelled,
    };

    unsafe { SyncCompletion::complete_ok(user_data, result) };
}

/// Picker style determines what content types can be selected
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCContentSharingPickerMode {
    /// Allow selection of both displays and windows
    #[default]
    SingleWindow = 0,
    /// Allow selection of multiple items
    Multiple = 1,
    /// Only allow display selection
    SingleDisplay = 2,
}

/// Configuration for the content sharing picker
pub struct SCContentSharingPickerConfiguration {
    ptr: *const c_void,
}

impl SCContentSharingPickerConfiguration {
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { crate::ffi::sc_content_sharing_picker_configuration_create() };
        Self { ptr }
    }

    /// Set allowed picker modes
    pub fn set_allowed_picker_modes(&mut self, modes: &[SCContentSharingPickerMode]) {
        let mode_values: Vec<i32> = modes.iter().map(|m| *m as i32).collect();
        unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_set_allowed_picker_modes(
                self.ptr,
                mode_values.as_ptr(),
                mode_values.len(),
            );
        }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *const c_void {
        self.ptr
    }
}

impl Default for SCContentSharingPickerConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SCContentSharingPickerConfiguration {
    fn clone(&self) -> Self {
        unsafe {
            Self {
                ptr: crate::ffi::sc_content_sharing_picker_configuration_retain(self.ptr),
            }
        }
    }
}

impl Drop for SCContentSharingPickerConfiguration {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                crate::ffi::sc_content_sharing_picker_configuration_release(self.ptr);
            }
        }
    }
}

impl std::fmt::Debug for SCContentSharingPickerConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCContentSharingPickerConfiguration")
            .field("ptr", &self.ptr)
            .finish()
    }
}

/// Result from the content sharing picker
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SCContentSharingPickerResult {
    /// User selected display content
    Display(SCDisplay),
    /// User selected a window
    Window(SCWindow),
    /// User selected an application
    Application(SCRunningApplication),
    /// User cancelled the picker
    Cancelled,
    /// An error occurred
    Error(String),
}

/// System UI for selecting content to share
///
/// Available on macOS 14.0+
pub struct SCContentSharingPicker;

impl SCContentSharingPicker {
    /// Show the picker UI and wait for user selection
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned.
    pub fn show(config: &SCContentSharingPickerConfiguration) -> SCContentSharingPickerResult {
        let (completion, context) = SyncCompletion::<SCContentSharingPickerResult>::new();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show(config.as_ptr(), picker_callback, context);
        }

        completion
            .wait()
            .unwrap_or_else(|e| SCContentSharingPickerResult::Error(e))
    }
}

// Safety: SCContentSharingPickerConfiguration wraps an Objective-C object that is thread-safe
unsafe impl Send for SCContentSharingPickerConfiguration {}
unsafe impl Sync for SCContentSharingPickerConfiguration {}
