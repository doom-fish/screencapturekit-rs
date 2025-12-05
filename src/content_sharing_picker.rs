//! `SCContentSharingPicker` - UI for selecting content to share
//!
//! Available on macOS 14.0+
//! Provides a system UI for users to select displays, windows, or applications to share.
//!
//! # APIs
//!
//! | Method | Returns | Use Case |
//! |--------|---------|----------|
//! | `show()` | callback with `SCPickerOutcome` | Get filter + metadata (dimensions, picked content) |
//! | `show_filter()` | callback with `SCPickerFilterOutcome` | Just get the filter |
//!
//! For async/await, use `AsyncSCContentSharingPicker` from the `async_api` module.
//!
//! # Examples
//!
//! ## Callback API: Get filter with metadata
//! ```no_run
//! use screencapturekit::content_sharing_picker::*;
//! use screencapturekit::prelude::*;
//!
//! let config = SCContentSharingPickerConfiguration::new();
//! SCContentSharingPicker::show(&config, |outcome| {
//!     match outcome {
//!         SCPickerOutcome::Picked(result) => {
//!             let (width, height) = result.pixel_size();
//!             let filter = result.filter();
//!             // Create stream with the filter...
//!         }
//!         SCPickerOutcome::Cancelled => println!("Cancelled"),
//!         SCPickerOutcome::Error(e) => eprintln!("Error: {}", e),
//!     }
//! });
//! ```
//!
//! ## Async API
//! ```no_run
//! use screencapturekit::async_api::AsyncSCContentSharingPicker;
//! use screencapturekit::content_sharing_picker::*;
//!
//! async fn example() {
//!     let config = SCContentSharingPickerConfiguration::new();
//!     if let SCPickerOutcome::Picked(result) = AsyncSCContentSharingPicker::show(&config).await {
//!         let (width, height) = result.pixel_size();
//!         let filter = result.filter();
//!     }
//! }
//! ```

use crate::stream::content_filter::SCContentFilter;
use std::ffi::c_void;

/// Represents the type of content selected in the picker
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SCPickedSource {
    /// A window was selected, with its title
    Window(String),
    /// A display was selected, with its ID
    Display(u32),
    /// An application was selected, with its name
    Application(String),
    /// No specific source identified
    Unknown,
}

/// Picker mode determines what content types can be selected
///
/// These modes can be combined to allow users to pick from different source types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCContentSharingPickerMode {
    /// Allow selection of a single window
    #[default]
    SingleWindow = 0,
    /// Allow selection of multiple windows
    MultipleWindows = 1,
    /// Allow selection of a single display/screen
    SingleDisplay = 2,
    /// Allow selection of a single application
    SingleApplication = 3,
    /// Allow selection of multiple applications
    MultipleApplications = 4,
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

    /// Set whether the user can change the selected content while sharing
    ///
    /// When `true`, the user can modify their selection during an active session.
    pub fn set_allows_changing_selected_content(&mut self, allows: bool) {
        unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_set_allows_changing_selected_content(
                self.ptr,
                allows,
            );
        }
    }

    /// Get whether changing selected content is allowed
    pub fn allows_changing_selected_content(&self) -> bool {
        unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_get_allows_changing_selected_content(
                self.ptr,
            )
        }
    }

    /// Set bundle identifiers to exclude from the picker
    ///
    /// Applications with these bundle IDs will not appear in the picker.
    pub fn set_excluded_bundle_ids(&mut self, bundle_ids: &[&str]) {
        let c_strings: Vec<std::ffi::CString> = bundle_ids
            .iter()
            .filter_map(|s| std::ffi::CString::new(*s).ok())
            .collect();
        let ptrs: Vec<*const i8> = c_strings.iter().map(|s| s.as_ptr()).collect();
        unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_set_excluded_bundle_ids(
                self.ptr,
                ptrs.as_ptr(),
                ptrs.len(),
            );
        }
    }

    /// Get the list of excluded bundle identifiers
    pub fn excluded_bundle_ids(&self) -> Vec<String> {
        let count = unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_get_excluded_bundle_ids_count(
                self.ptr,
            )
        };
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let mut buffer = vec![0i8; 256];
            let success = unsafe {
                crate::ffi::sc_content_sharing_picker_configuration_get_excluded_bundle_id_at(
                    self.ptr,
                    i,
                    buffer.as_mut_ptr(),
                    buffer.len(),
                )
            };
            if success {
                let c_str = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) };
                if let Ok(s) = c_str.to_str() {
                    result.push(s.to_string());
                }
            }
        }
        result
    }

    /// Set window IDs to exclude from the picker
    ///
    /// Windows with these IDs will not appear in the picker.
    pub fn set_excluded_window_ids(&mut self, window_ids: &[u32]) {
        unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_set_excluded_window_ids(
                self.ptr,
                window_ids.as_ptr(),
                window_ids.len(),
            );
        }
    }

    /// Get the list of excluded window IDs
    pub fn excluded_window_ids(&self) -> Vec<u32> {
        let count = unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_get_excluded_window_ids_count(
                self.ptr,
            )
        };
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let id = unsafe {
                crate::ffi::sc_content_sharing_picker_configuration_get_excluded_window_id_at(
                    self.ptr, i,
                )
            };
            result.push(id);
        }
        result
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
    #[cfg(feature = "async")]
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
                self.ptr,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
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
                self.ptr,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
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
    /// SCContentSharingPicker::show(&config, |outcome| {
    ///     if let SCPickerOutcome::Picked(result) = outcome {
    ///         let windows = result.windows();
    ///         if let Some(window) = windows.first() {
    ///             // Create custom filter with a picked window
    ///             let filter = SCContentFilter::builder()
    ///                 .window(window)
    ///                 .build();
    ///         }
    ///     }
    /// });
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
    /// SCContentSharingPicker::show(&config, |outcome| {
    ///     if let SCPickerOutcome::Picked(result) = outcome {
    ///         let displays = result.displays();
    ///         if let Some(display) = displays.first() {
    ///             // Create custom filter with the picked display
    ///             let filter = SCContentFilter::builder()
    ///                 .display(display)
    ///                 .build();
    ///         }
    ///     }
    /// });
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

    /// Get the source type that was picked
    ///
    /// Returns information about what the user selected: window, display, or application.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// fn example() {
    ///     let config = SCContentSharingPickerConfiguration::new();
    ///     SCContentSharingPicker::show(&config, |outcome| {
    ///         if let SCPickerOutcome::Picked(result) = outcome {
    ///             match result.source() {
    ///                 SCPickedSource::Window(title) => println!("[W] {}", title),
    ///                 SCPickedSource::Display(id) => println!("[D] Display {}", id),
    ///                 SCPickedSource::Application(name) => println!("[A] {}", name),
    ///                 SCPickedSource::Unknown => println!("Unknown source"),
    ///             }
    ///         }
    ///     });
    /// }
    /// ```
    #[must_use]
    #[allow(clippy::option_if_let_else)]
    pub fn source(&self) -> SCPickedSource {
        if let Some(window) = self.windows().first() {
            SCPickedSource::Window(window.title().unwrap_or_else(|| "Untitled".to_string()))
        } else if let Some(display) = self.displays().first() {
            SCPickedSource::Display(display.display_id())
        } else if let Some(app) = self.applications().first() {
            SCPickedSource::Application(app.application_name())
        } else {
            SCPickedSource::Unknown
        }
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
///
/// The picker requires user interaction and cannot block the calling thread.
/// Use one of these approaches:
///
/// - **Callback-based**: `show()` / `show_filter()` - pass a callback closure
/// - **Async/await**: `AsyncSCContentSharingPicker` from the `async_api` module
///
/// # Example (callback)
/// ```no_run
/// use screencapturekit::content_sharing_picker::*;
///
/// let config = SCContentSharingPickerConfiguration::new();
/// SCContentSharingPicker::show(&config, |outcome| {
///     if let SCPickerOutcome::Picked(result) = outcome {
///         let (width, height) = result.pixel_size();
///         let filter = result.filter();
///         // ... create stream
///     }
/// });
/// ```
///
/// # Example (async)
/// ```no_run
/// use screencapturekit::async_api::AsyncSCContentSharingPicker;
/// use screencapturekit::content_sharing_picker::*;
///
/// async fn example() {
///     let config = SCContentSharingPickerConfiguration::new();
///     if let SCPickerOutcome::Picked(result) = AsyncSCContentSharingPicker::show(&config).await {
///         let (width, height) = result.pixel_size();
///         let filter = result.filter();
///         // ... create stream
///     }
/// }
/// ```
pub struct SCContentSharingPicker;

impl SCContentSharingPicker {
    /// Show the picker UI with a callback for the result
    ///
    /// This is non-blocking - the callback is invoked when the user makes a selection
    /// or cancels the picker.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// let config = SCContentSharingPickerConfiguration::new();
    /// SCContentSharingPicker::show(&config, |outcome| {
    ///     match outcome {
    ///         SCPickerOutcome::Picked(result) => {
    ///             let (width, height) = result.pixel_size();
    ///             let filter = result.filter();
    ///             println!("Selected {}x{}", width, height);
    ///         }
    ///         SCPickerOutcome::Cancelled => println!("Cancelled"),
    ///         SCPickerOutcome::Error(e) => eprintln!("Error: {}", e),
    ///     }
    /// });
    /// ```
    pub fn show<F>(config: &SCContentSharingPickerConfiguration, callback: F)
    where
        F: FnOnce(SCPickerOutcome) + Send + 'static,
    {
        let callback = Box::new(callback);
        let context = Box::into_raw(callback).cast::<std::ffi::c_void>();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_with_result(
                config.as_ptr(),
                picker_callback_boxed::<F>,
                context,
            );
        }
    }

    /// Show the picker UI for an existing stream (to change source while capturing)
    ///
    /// Use this when you have an active `SCStream` and want to let the user
    /// select a new content source. The callback receives the new filter
    /// which can be used with `stream.update_content_filter()`.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    /// use screencapturekit::stream::SCStream;
    /// use screencapturekit::stream::configuration::SCStreamConfiguration;
    /// use screencapturekit::stream::content_filter::SCContentFilter;
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// fn example() -> Option<()> {
    ///     let content = SCShareableContent::get().ok()?;
    ///     let displays = content.displays();
    ///     let display = displays.first()?;
    ///     let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
    ///     let stream_config = SCStreamConfiguration::new();
    ///     let stream = SCStream::new(&filter, &stream_config);
    ///
    ///     // When stream is active and user wants to change source
    ///     let config = SCContentSharingPickerConfiguration::new();
    ///     SCContentSharingPicker::show_for_stream(&config, &stream, |outcome| {
    ///         if let SCPickerOutcome::Picked(result) = outcome {
    ///             // Use result.filter() with stream.update_content_filter()
    ///             let _ = result.filter();
    ///         }
    ///     });
    ///     Some(())
    /// }
    /// ```
    pub fn show_for_stream<F>(
        config: &SCContentSharingPickerConfiguration,
        stream: &crate::stream::SCStream,
        callback: F,
    ) where
        F: FnOnce(SCPickerOutcome) + Send + 'static,
    {
        let callback = Box::new(callback);
        let context = Box::into_raw(callback).cast::<std::ffi::c_void>();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_for_stream(
                config.as_ptr(),
                stream.as_ptr(),
                picker_callback_boxed::<F>,
                context,
            );
        }
    }

    /// Show the picker UI with a callback that receives just the filter
    ///
    /// This is the simple API - use when you just need the filter without metadata.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// let config = SCContentSharingPickerConfiguration::new();
    /// SCContentSharingPicker::show_filter(&config, |outcome| {
    ///     if let SCPickerFilterOutcome::Filter(filter) = outcome {
    ///         // Use filter with SCStream
    ///     }
    /// });
    /// ```
    pub fn show_filter<F>(config: &SCContentSharingPickerConfiguration, callback: F)
    where
        F: FnOnce(SCPickerFilterOutcome) + Send + 'static,
    {
        let callback = Box::new(callback);
        let context = Box::into_raw(callback).cast::<std::ffi::c_void>();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show(
                config.as_ptr(),
                picker_filter_callback_boxed::<F>,
                context,
            );
        }
    }

    /// Show the picker UI with a specific content style
    ///
    /// Presents the picker pre-filtered to a specific content type.
    ///
    /// # Arguments
    /// * `config` - The picker configuration
    /// * `style` - The content style to show (Window, Display, Application)
    /// * `callback` - Called with the picker result
    pub fn show_using_style<F>(
        config: &SCContentSharingPickerConfiguration,
        style: crate::stream::content_filter::SCShareableContentStyle,
        callback: F,
    ) where
        F: FnOnce(SCPickerOutcome) + Send + 'static,
    {
        let callback = Box::new(callback);
        let context = Box::into_raw(callback).cast::<std::ffi::c_void>();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_using_style(
                config.as_ptr(),
                style as i32,
                picker_callback_boxed::<F>,
                context,
            );
        }
    }

    /// Show the picker for an existing stream with a specific content style
    ///
    /// # Arguments
    /// * `config` - The picker configuration
    /// * `stream` - The stream to update
    /// * `style` - The content style to show (Window, Display, Application)
    /// * `callback` - Called with the picker result
    pub fn show_for_stream_using_style<F>(
        config: &SCContentSharingPickerConfiguration,
        stream: &crate::stream::SCStream,
        style: crate::stream::content_filter::SCShareableContentStyle,
        callback: F,
    ) where
        F: FnOnce(SCPickerOutcome) + Send + 'static,
    {
        let callback = Box::new(callback);
        let context = Box::into_raw(callback).cast::<std::ffi::c_void>();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_for_stream_using_style(
                config.as_ptr(),
                stream.as_ptr(),
                style as i32,
                picker_callback_boxed::<F>,
                context,
            );
        }
    }

    /// Set the maximum number of streams that can be created from the picker
    ///
    /// Pass 0 to allow unlimited streams.
    pub fn set_maximum_stream_count(count: usize) {
        unsafe {
            crate::ffi::sc_content_sharing_picker_set_maximum_stream_count(count);
        }
    }

    /// Get the maximum number of streams allowed
    ///
    /// Returns 0 if unlimited streams are allowed.
    pub fn maximum_stream_count() -> usize {
        unsafe { crate::ffi::sc_content_sharing_picker_get_maximum_stream_count() }
    }
}

/// Callback trampoline for boxed closures (picker with result)
extern "C" fn picker_callback_boxed<F>(
    code: i32,
    ptr: *const std::ffi::c_void,
    context: *mut std::ffi::c_void,
) where
    F: FnOnce(SCPickerOutcome) + Send + 'static,
{
    let callback = unsafe { Box::from_raw(context.cast::<F>()) };
    let outcome = match code {
        1 if !ptr.is_null() => SCPickerOutcome::Picked(SCPickerResult { ptr }),
        0 => SCPickerOutcome::Cancelled,
        _ => SCPickerOutcome::Error("Picker failed".to_string()),
    };
    callback(outcome);
}

/// Callback trampoline for boxed closures (picker filter only)
extern "C" fn picker_filter_callback_boxed<F>(
    code: i32,
    ptr: *const std::ffi::c_void,
    context: *mut std::ffi::c_void,
) where
    F: FnOnce(SCPickerFilterOutcome) + Send + 'static,
{
    let callback = unsafe { Box::from_raw(context.cast::<F>()) };
    let outcome = match code {
        1 if !ptr.is_null() => SCPickerFilterOutcome::Filter(SCContentFilter::from_picker_ptr(ptr)),
        0 => SCPickerFilterOutcome::Cancelled,
        _ => SCPickerFilterOutcome::Error("Picker failed".to_string()),
    };
    callback(outcome);
}

// Safety: Configuration wraps an Objective-C object that is thread-safe
unsafe impl Send for SCContentSharingPickerConfiguration {}
unsafe impl Sync for SCContentSharingPickerConfiguration {}
unsafe impl Send for SCPickerResult {}
unsafe impl Sync for SCPickerResult {}
