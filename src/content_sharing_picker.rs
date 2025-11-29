//! `SCContentSharingPicker` - UI for selecting content to share
//!
//! Available on macOS 14.0+
//! Provides a system UI for users to select displays, windows, or applications to share.
//!
//! # APIs
//!
//! | Method | Returns | Use Case |
//! |--------|---------|----------|
//! | `pick()` | `SCPickerOutcome` | Get filter + metadata (dimensions, picked content) |
//! | `pick_filter()` | `SCPickerFilterOutcome` | Just get the filter |
//!
//! # Examples
//!
//! ## Main API: Get filter with metadata and picked content
//! ```no_run
//! use screencapturekit::content_sharing_picker::*;
//! use screencapturekit::prelude::*;
//!
//! let config = SCContentSharingPickerConfiguration::new();
//! match SCContentSharingPicker::pick(&config) {
//!     SCPickerOutcome::Picked(result) => {
//!         // Get dimensions for stream configuration
//!         let (width, height) = result.pixel_size();
//!         
//!         // Access the picked content directly
//!         for window in result.windows() {
//!             println!("Picked window: {:?}", window.title());
//!         }
//!         for display in result.displays() {
//!             println!("Picked display: {}", display.display_id());
//!         }
//!         
//!         // Use the pre-built filter or create your own
//!         let filter = result.filter();
//!         let stream = SCStream::new(&filter, &SCStreamConfiguration::default());
//!     }
//!     SCPickerOutcome::Cancelled => println!("Cancelled"),
//!     SCPickerOutcome::Error(e) => eprintln!("Error: {}", e),
//! }
//! ```
//!
//! ## Simple API: Get filter directly
//! ```no_run
//! use screencapturekit::content_sharing_picker::*;
//! use screencapturekit::prelude::*;
//!
//! let config = SCContentSharingPickerConfiguration::new();
//! match SCContentSharingPicker::pick_filter(&config) {
//!     SCPickerFilterOutcome::Filter(filter) => {
//!         let stream = SCStream::new(&filter, &SCStreamConfiguration::default());
//!     }
//!     SCPickerFilterOutcome::Cancelled => println!("Cancelled"),
//!     SCPickerFilterOutcome::Error(e) => eprintln!("Error: {}", e),
//! }
//! ```
//!
//! ## Custom filter: Build your own filter from picked content
//! ```no_run
//! use screencapturekit::content_sharing_picker::*;
//! use screencapturekit::prelude::*;
//!
//! let config = SCContentSharingPickerConfiguration::new();
//! match SCContentSharingPicker::pick(&config) {
//!     SCPickerOutcome::Picked(result) => {
//!         // Get the picked displays and windows
//!         let displays = result.displays();
//!         let windows = result.windows();
//!         
//!         // Create a custom filter - e.g. capture display but exclude some windows
//!         if let Some(display) = displays.first() {
//!             let custom_filter = SCContentFilter::builder()
//!                 .display(display)
//!                 // .exclude_windows(&windows_to_exclude)
//!                 .build();
//!             
//!             let stream = SCStream::new(&custom_filter, &SCStreamConfiguration::default());
//!         }
//!         
//!         // Or capture a specific window
//!         if let Some(window) = windows.first() {
//!             let window_filter = SCContentFilter::builder()
//!                 .window(window)
//!                 .build();
//!             
//!             let stream = SCStream::new(&window_filter, &SCStreamConfiguration::default());
//!         }
//!     }
//!     SCPickerOutcome::Cancelled => println!("Cancelled"),
//!     SCPickerOutcome::Error(e) => eprintln!("Error: {}", e),
//! }
//! ```

use crate::stream::content_filter::SCContentFilter;
use crate::utils::sync_completion::SyncCompletion;
use std::ffi::c_void;

/// Result from the content sharing picker - returns a filter pointer on success
struct PickerCallbackResult {
    code: i32,
    ptr: *const c_void,
}

extern "C" fn picker_callback(result_code: i32, ptr: *const c_void, user_data: *mut c_void) {
    let result = PickerCallbackResult {
        code: result_code,
        ptr,
    };
    unsafe { SyncCompletion::complete_ok(user_data, result) };
}

/// Picker style determines what content types can be selected
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCContentSharingPickerMode {
    /// Allow selection of a single window
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

// ============================================================================
// Simple API: Returns SCContentFilter directly
// ============================================================================

/// Result from the simple `show_filter()` API
#[derive(Debug)]
pub enum SCPickerFilterOutcome {
    /// User selected content - contains the filter to use with `SCStream`
    Filter(SCContentFilter),
    /// User cancelled the picker
    Cancelled,
    /// An error occurred
    Error(String),
}

// ============================================================================
// Main API: Returns SCPickerResult with metadata
// ============================================================================

/// Result from the main `show()` API - contains filter and content metadata
///
/// Provides access to:
/// - The `SCContentFilter` for use with `SCStream`
/// - Content dimensions and scale factor
/// - The picked windows, displays, and applications for custom filter creation
pub struct SCPickerResult {
    ptr: *const c_void,
}

impl SCPickerResult {
    /// Create from raw pointer (used by async API)
    #[must_use]
    pub(crate) fn from_ptr(ptr: *const c_void) -> Self {
        Self { ptr }
    }

    /// Get the content filter for use with `SCStream::new()`
    #[must_use]
    pub fn filter(&self) -> SCContentFilter {
        let filter_ptr = unsafe { crate::ffi::sc_picker_result_get_filter(self.ptr) };
        SCContentFilter::from_picker_ptr(filter_ptr)
    }

    /// Get the content size in points (width, height)
    #[must_use]
    pub fn size(&self) -> (f64, f64) {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;
        unsafe {
            crate::ffi::sc_picker_result_get_content_rect(
                self.ptr, &mut x, &mut y, &mut width, &mut height,
            );
        }
        (width, height)
    }

    /// Get the content rect (x, y, width, height) in points
    #[must_use]
    pub fn rect(&self) -> (f64, f64, f64, f64) {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;
        unsafe {
            crate::ffi::sc_picker_result_get_content_rect(
                self.ptr, &mut x, &mut y, &mut width, &mut height,
            );
        }
        (x, y, width, height)
    }

    /// Get the point-to-pixel scale factor (typically 2.0 for Retina displays)
    #[must_use]
    pub fn scale(&self) -> f64 {
        unsafe { crate::ffi::sc_picker_result_get_scale(self.ptr) }
    }

    /// Get the pixel dimensions (size * scale)
    #[must_use]
    pub fn pixel_size(&self) -> (u32, u32) {
        let (w, h) = self.size();
        let scale = self.scale();
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let width = (w * scale) as u32;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let height = (h * scale) as u32;
        (width, height)
    }

    /// Get the windows selected by the user
    ///
    /// Returns the picked windows that can be used to create a custom `SCContentFilter`.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCContentSharingPickerConfiguration::new();
    /// if let SCPickerOutcome::Picked(result) = SCContentSharingPicker::pick(&config) {
    ///     let windows = result.windows();
    ///     if let Some(window) = windows.first() {
    ///         // Create custom filter with a picked window
    ///         let filter = SCContentFilter::builder()
    ///             .window(window)
    ///             .build();
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn windows(&self) -> Vec<crate::shareable_content::SCWindow> {
        let count = unsafe { crate::ffi::sc_picker_result_get_windows_count(self.ptr) };
        (0..count)
            .filter_map(|i| {
                let ptr = unsafe { crate::ffi::sc_picker_result_get_window_at(self.ptr, i) };
                if ptr.is_null() {
                    None
                } else {
                    Some(crate::shareable_content::SCWindow::from_ffi_owned(ptr))
                }
            })
            .collect()
    }

    /// Get the displays selected by the user
    ///
    /// Returns the picked displays that can be used to create a custom `SCContentFilter`.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCContentSharingPickerConfiguration::new();
    /// if let SCPickerOutcome::Picked(result) = SCContentSharingPicker::pick(&config) {
    ///     let displays = result.displays();
    ///     if let Some(display) = displays.first() {
    ///         // Create custom filter with the picked display
    ///         let filter = SCContentFilter::builder()
    ///             .display(display)
    ///             .build();
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn displays(&self) -> Vec<crate::shareable_content::SCDisplay> {
        let count = unsafe { crate::ffi::sc_picker_result_get_displays_count(self.ptr) };
        (0..count)
            .filter_map(|i| {
                let ptr = unsafe { crate::ffi::sc_picker_result_get_display_at(self.ptr, i) };
                if ptr.is_null() {
                    None
                } else {
                    Some(crate::shareable_content::SCDisplay::from_ffi_owned(ptr))
                }
            })
            .collect()
    }

    /// Get the applications selected by the user
    ///
    /// Returns the picked applications that can be used to create a custom `SCContentFilter`.
    #[must_use]
    pub fn applications(&self) -> Vec<crate::shareable_content::SCRunningApplication> {
        let count = unsafe { crate::ffi::sc_picker_result_get_applications_count(self.ptr) };
        (0..count)
            .filter_map(|i| {
                let ptr = unsafe { crate::ffi::sc_picker_result_get_application_at(self.ptr, i) };
                if ptr.is_null() {
                    None
                } else {
                    Some(crate::shareable_content::SCRunningApplication::from_ffi_owned(ptr))
                }
            })
            .collect()
    }
}

impl Drop for SCPickerResult {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                crate::ffi::sc_picker_result_release(self.ptr);
            }
        }
    }
}

impl std::fmt::Debug for SCPickerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (w, h) = self.size();
        let scale = self.scale();
        f.debug_struct("SCPickerResult")
            .field("size", &(w, h))
            .field("scale", &scale)
            .field("pixel_size", &self.pixel_size())
            .finish()
    }
}

/// Outcome from the main `show()` API
#[derive(Debug)]
pub enum SCPickerOutcome {
    /// User selected content - contains result with filter and metadata
    Picked(SCPickerResult),
    /// User cancelled the picker
    Cancelled,
    /// An error occurred
    Error(String),
}

// ============================================================================
// SCContentSharingPicker
// ============================================================================

/// System UI for selecting content to share
///
/// Available on macOS 14.0+
pub struct SCContentSharingPicker;

impl SCContentSharingPicker {
    /// Show the picker UI and return `SCPickerResult` with filter and metadata
    ///
    /// This is the main API - use when you need content dimensions or want to build custom filters.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCContentSharingPickerConfiguration::new();
    /// if let SCPickerOutcome::Picked(result) = SCContentSharingPicker::pick(&config) {
    ///     let (width, height) = result.pixel_size();
    ///     let mut stream_config = SCStreamConfiguration::default();
    ///     stream_config.set_width(width);
    ///     stream_config.set_height(height);
    ///     
    ///     let filter = result.filter();
    ///     let stream = SCStream::new(&filter, &stream_config);
    /// }
    /// ```
    pub fn pick(config: &SCContentSharingPickerConfiguration) -> SCPickerOutcome {
        let (completion, context) = SyncCompletion::<PickerCallbackResult>::new();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_with_result(
                config.as_ptr(),
                picker_callback,
                context,
            );
        }

        match completion.wait() {
            Ok(result) => match result.code {
                1 if !result.ptr.is_null() => {
                    SCPickerOutcome::Picked(SCPickerResult { ptr: result.ptr })
                }
                0 => SCPickerOutcome::Cancelled,
                _ => SCPickerOutcome::Error("Picker failed".to_string()),
            },
            Err(e) => SCPickerOutcome::Error(e),
        }
    }

    /// Show the picker UI and return an `SCContentFilter` directly
    ///
    /// This is the simple API - use when you just need the filter without metadata.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// let config = SCContentSharingPickerConfiguration::new();
    /// if let SCPickerFilterOutcome::Filter(filter) = SCContentSharingPicker::pick_filter(&config) {
    ///     // Use filter with SCStream
    /// }
    /// ```
    pub fn pick_filter(config: &SCContentSharingPickerConfiguration) -> SCPickerFilterOutcome {
        let (completion, context) = SyncCompletion::<PickerCallbackResult>::new();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show(config.as_ptr(), picker_callback, context);
        }

        match completion.wait() {
            Ok(result) => match result.code {
                1 if !result.ptr.is_null() => {
                    SCPickerFilterOutcome::Filter(SCContentFilter::from_picker_ptr(result.ptr))
                }
                0 => SCPickerFilterOutcome::Cancelled,
                _ => SCPickerFilterOutcome::Error("Picker failed".to_string()),
            },
            Err(e) => SCPickerFilterOutcome::Error(e),
        }
    }
}

// Safety: Configuration wraps an Objective-C object that is thread-safe
unsafe impl Send for SCContentSharingPickerConfiguration {}
unsafe impl Sync for SCContentSharingPickerConfiguration {}
unsafe impl Send for SCPickerResult {}
unsafe impl Sync for SCPickerResult {}
