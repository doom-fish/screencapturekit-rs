//! `SCContentSharingPicker` - UI for selecting content to share
//!
//! Available on macOS 14.0+.
//! Provides a system UI for users to select displays, windows, or applications to share.
//!
//! ## When to Use
//!
//! Use the content sharing picker when:
//! - You want users to choose what to capture via a native macOS UI
//! - You need consistent UX with other screen sharing apps
//! - You want to avoid manually listing and presenting content options
//!
//! ## APIs
//!
//! | Method | Returns | Use Case |
//! |--------|---------|----------|
//! | [`SCContentSharingPicker::show()`] | callback with [`SCPickerOutcome`] | Get filter + metadata (dimensions, picked content) |
//! | [`SCContentSharingPicker::show_filter()`] | callback with [`SCPickerFilterOutcome`] | Just get the filter |
//!
//! For async/await, use `AsyncSCContentSharingPicker` from the optional
//! `async_api` module.
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
//!             println!("Selected content: {}x{}", width, height);
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
//!         println!("Selected: {}x{}", width, height);
//!     }
//! }
//! ```
//!
//! ## Configure Picker Modes
//! ```no_run
//! use screencapturekit::content_sharing_picker::*;
//!
//! let mut config = SCContentSharingPickerConfiguration::new();
//! // Only allow single display selection
//! config.set_allowed_picker_modes(&[SCContentSharingPickerMode::SingleDisplay]);
//! // Exclude specific apps from the picker
//! config.set_excluded_bundle_ids(&["com.apple.finder", "com.apple.dock"]);
//! ```

use crate::stream::content_filter::{SCContentFilter, SCShareableContentStyle};
pub use crate::stream::StreamIdentity;
use std::any::Any;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

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
    /// # Panics
    ///
    /// Panics when run on macOS older than 14.0. Use [`Self::try_new`] when
    /// runtime availability is not already known.
    #[must_use]
    pub fn new() -> Self {
        Self::try_new().expect("SCContentSharingPicker requires macOS 14.0 or later")
    }

    /// Create a configuration when the picker is available on this system.
    #[must_use]
    pub fn try_new() -> Option<Self> {
        if !SCContentSharingPicker::is_available() {
            return None;
        }
        let ptr = unsafe { crate::ffi::sc_content_sharing_picker_configuration_create() };
        (!ptr.is_null()).then_some(Self { ptr })
    }

    /// Construct a configuration initialised with the system's default values
    /// (the equivalent of Apple's `SCContentSharingPicker.shared.defaultConfiguration`).
    ///
    /// Use this when you want "system defaults plus my one tweak" — call this
    /// to get the baseline, then mutate the fields you care about. Compared
    /// to [`SCContentSharingPickerConfiguration::new()`], which starts from a
    /// blank-slate `SCContentSharingPickerConfiguration()`, this preserves
    /// any system-wide picker preferences the OS applies to fresh
    /// configurations (e.g. allowed picker modes, default exclusion lists).
    ///
    /// # Panics
    ///
    /// Panics when run on macOS older than 14.0.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// // Start from the system defaults, then override only what you need.
    /// let mut config = SCContentSharingPickerConfiguration::default_from_system();
    /// config.set_excluded_bundle_ids(&["com.apple.dock"]);
    /// ```
    #[must_use]
    pub fn default_from_system() -> Self {
        assert!(
            SCContentSharingPicker::is_available(),
            "SCContentSharingPicker requires macOS 14.0 or later"
        );
        let ptr = unsafe { crate::ffi::sc_content_sharing_picker_create_default_configuration() };
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

    /// Get the currently allowed picker modes.
    pub fn allowed_picker_modes(&self) -> Vec<SCContentSharingPickerMode> {
        let mask = unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_get_allowed_picker_modes_mask(
                self.ptr,
            )
        };
        let mut modes = Vec::new();
        for (raw_value, mode) in [
            (1_u64, SCContentSharingPickerMode::SingleWindow),
            (2_u64, SCContentSharingPickerMode::MultipleWindows),
            (16_u64, SCContentSharingPickerMode::SingleDisplay),
            (4_u64, SCContentSharingPickerMode::SingleApplication),
            (8_u64, SCContentSharingPickerMode::MultipleApplications),
        ] {
            if mask & raw_value != 0 {
                modes.push(mode);
            }
        }
        modes
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
        let c_strings: Vec<std::ffi::CString> = if let Ok(ids) = bundle_ids
            .iter()
            .map(|id| std::ffi::CString::new(*id))
            .collect()
        {
            ids
        } else {
            eprintln!(
                "SCContentSharingPickerConfiguration: excluded bundle ID contains an \
                 interior NUL byte; configuration was not changed"
            );
            return;
        };
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
    ///
    /// A bundle ID that does not fit in the transfer buffer is skipped rather
    /// than returned truncated, so the result can be shorter than
    /// [`excluded_bundle_ids_count`](Self::excluded_bundle_ids_count).
    #[must_use]
    pub fn excluded_bundle_ids(&self) -> Vec<String> {
        let count = self.excluded_bundle_ids_count();
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let id = unsafe {
                crate::utils::ffi_string::ffi_string_from_buffer(
                    crate::utils::ffi_string::DEFAULT_BUFFER_SIZE,
                    |buffer, len| {
                        crate::ffi::sc_content_sharing_picker_configuration_get_excluded_bundle_id_at(
                            self.ptr,
                            i,
                            buffer,
                            usize::try_from(len).unwrap_or(0),
                        )
                    },
                )
            };
            if let Some(id) = id {
                result.push(id);
            }
        }
        result
    }

    /// Number of excluded bundle identifiers configured.
    #[must_use]
    pub fn excluded_bundle_ids_count(&self) -> usize {
        unsafe {
            crate::ffi::sc_content_sharing_picker_configuration_get_excluded_bundle_ids_count(
                self.ptr,
            )
        }
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

crate::utils::retained::sc_retained!(
    SCContentSharingPickerConfiguration,
    field = ptr,
    release = crate::ffi::sc_content_sharing_picker_configuration_release,
);

impl Clone for SCContentSharingPickerConfiguration {
    /// Produce an independent configuration with the same values.
    ///
    /// Deliberately **not** the retain-based clone the other wrappers in this
    /// crate use. Those wrap immutable Objective-C objects, where sharing one
    /// instance between handles is unobservable. This type is different: it
    /// wraps a mutable Swift box and exposes `&mut self` setters, so a
    /// refcount-only clone would let a `&mut` on one handle mutate state that
    /// another handle observes through a shared `&` — and, with `Send + Sync`,
    /// from another thread at the same time.
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { crate::ffi::sc_content_sharing_picker_configuration_copy(self.ptr) },
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
    ///             let filter = SCContentFilter::create()
    ///                 .with_window(window)
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
                unsafe { crate::shareable_content::SCWindow::from_retained_ptr(ptr) }
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
    ///             let filter = SCContentFilter::create()
    ///                 .with_display(display)
    ///                 .with_excluding_windows(&[])
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
                unsafe { crate::shareable_content::SCDisplay::from_retained_ptr(ptr) }
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
                unsafe { crate::shareable_content::SCRunningApplication::from_retained_ptr(ptr) }
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

crate::utils::retained::sc_retained!(
    SCPickerResult,
    field = ptr,
    release = crate::ffi::sc_picker_result_release,
);

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

/// Error returned when applying a picker configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SCPickerConfigurationError {
    /// The picker API is unavailable on this system.
    Unavailable,
    /// Picker configuration properties must be assigned on the process main
    /// thread.
    MainThreadRequired,
}

impl std::fmt::Display for SCPickerConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => f.write_str("content sharing picker is unavailable"),
            Self::MainThreadRequired => {
                f.write_str("content sharing picker configuration requires the main thread")
            }
        }
    }
}

impl std::error::Error for SCPickerConfigurationError {}

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
#[derive(Debug)]
pub struct SCContentSharingPicker;

impl SCContentSharingPicker {
    fn available_or_log(operation: &str) -> bool {
        let available = Self::is_available();
        if !available {
            eprintln!("{operation} requires macOS 14.0 or later");
        }
        available
    }

    /// Whether content-sharing picker APIs are available on this system.
    #[must_use]
    pub fn is_available() -> bool {
        unsafe { crate::ffi::sc_content_sharing_picker_is_available() }
    }

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
        let context = into_callback_context::<SCPickerOutcome, F>(callback);

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_with_result(
                config.as_ptr(),
                picker_trampoline::<ResultDecoder>,
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
    ///     let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
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
        let context = into_callback_context::<SCPickerOutcome, F>(callback);

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_for_stream(
                config.as_ptr(),
                stream.as_ptr(),
                picker_trampoline::<ResultDecoder>,
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
        let context = into_callback_context::<SCPickerFilterOutcome, F>(callback);

        unsafe {
            crate::ffi::sc_content_sharing_picker_show(
                config.as_ptr(),
                picker_trampoline::<FilterDecoder>,
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
        let context = into_callback_context::<SCPickerOutcome, F>(callback);

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_using_style(
                config.as_ptr(),
                style as i32,
                picker_trampoline::<ResultDecoder>,
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
        let context = into_callback_context::<SCPickerOutcome, F>(callback);

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_for_stream_using_style(
                config.as_ptr(),
                stream.as_ptr(),
                style as i32,
                picker_trampoline::<ResultDecoder>,
                context,
            );
        }
    }

    /// Set the maximum number of streams that can be created from the picker
    ///
    /// Pass 0 to allow unlimited streams.
    pub fn set_maximum_stream_count(count: usize) {
        if !Self::available_or_log("SCContentSharingPicker::set_maximum_stream_count") {
            return;
        }
        unsafe {
            crate::ffi::sc_content_sharing_picker_set_maximum_stream_count(count);
        }
    }

    /// Get the maximum number of streams allowed
    ///
    /// Returns 0 if unlimited streams are allowed.
    pub fn maximum_stream_count() -> usize {
        if !Self::is_available() {
            return 0;
        }
        unsafe { crate::ffi::sc_content_sharing_picker_get_maximum_stream_count() }
    }

    /// Returns whether the shared content-sharing picker is currently
    /// marked active.
    ///
    /// Apple requires `picker.isActive = true` before its UI can appear.
    /// The various `show*()` trampolines on this type set it implicitly
    /// before presenting, but this getter is useful for callers that
    /// want to:
    ///
    /// * avoid double-presenting (skip a second `show()` while the first
    ///   picker session is still up),
    /// * render UI affordances based on whether the picker is currently
    ///   visible to the user.
    #[must_use]
    pub fn is_active() -> bool {
        if !Self::is_available() {
            return false;
        }
        unsafe { crate::ffi::sc_content_sharing_picker_get_active() }
    }

    /// Mark the shared content-sharing picker active or inactive.
    ///
    /// Setting this to `false` hides the picker UI between sessions
    /// (the recommended hygiene step after a long-running app finishes
    /// using the picker — leaving it active leaves the system-level
    /// Control Center entry in a "ready to share" state).
    ///
    /// Setting to `true` is required before `present*()` can surface
    /// the picker; the `show*()` trampolines do this for you. Set it
    /// manually only if you want to opt into the picker UI without
    /// immediately presenting it.
    pub fn set_active(active: bool) {
        if !Self::available_or_log("SCContentSharingPicker::set_active") {
            return;
        }
        unsafe { crate::ffi::sc_content_sharing_picker_set_active(active) }
    }

    /// Deactivate the picker and undo any activation-policy promotion the
    /// bridge performed on behalf of a non-UI (`.prohibited`) host process.
    ///
    /// Presenting `SCContentSharingPicker` requires the process to be a
    /// regular, Dock-visible app. Pure-Rust hosts usually are not, so the
    /// bridge temporarily promotes them; the promotion is reference counted
    /// and unwound automatically when each one-shot `show*()` resolves. Call
    /// this after you are done with a *long-lived* observer session to drop
    /// the promotion immediately and clear the Control Center "ready to
    /// share" indicator.
    ///
    /// Registered observers are **not** removed — drop their
    /// [`SCPickerSubscription`] for that.
    pub fn deactivate() {
        if !Self::is_available() {
            return;
        }
        unsafe { crate::ffi::sc_content_sharing_picker_deactivate() }
    }

    // ------------------------------------------------------------------
    // Standalone configuration operations
    // ------------------------------------------------------------------

    /// Read the picker's process-wide default configuration.
    ///
    /// Equivalent to Apple's `SCContentSharingPicker.shared.defaultConfiguration`.
    /// This is the same value returned by
    /// [`SCContentSharingPickerConfiguration::default_from_system`].
    #[must_use]
    pub fn default_configuration() -> SCContentSharingPickerConfiguration {
        SCContentSharingPickerConfiguration::default_from_system()
    }

    /// Assign the picker's process-wide default configuration.
    ///
    /// Apple's `SCContentSharingPicker.defaultConfiguration` is read-write;
    /// previously this crate could only set it as a side effect of calling a
    /// `show*()` helper. Setting it explicitly is what you want when driving
    /// the picker with a persistent observer plus [`Self::present`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// let mut config = SCContentSharingPickerConfiguration::new();
    /// config.set_allows_changing_selected_content(true);
    /// SCContentSharingPicker::set_default_configuration(&config)
    ///     .expect("call from the process main thread");
    /// ```
    ///
    /// The assignment is complete when this method returns, so an immediate
    /// call to [`Self::default_configuration`] observes the new value.
    ///
    /// # Errors
    ///
    /// Returns [`SCPickerConfigurationError::Unavailable`] when the API is
    /// unavailable, or [`SCPickerConfigurationError::MainThreadRequired`]
    /// when called from any other thread. Failed calls do not enqueue a later
    /// mutation.
    pub fn set_default_configuration(
        config: &SCContentSharingPickerConfiguration,
    ) -> Result<(), SCPickerConfigurationError> {
        if !Self::is_available() {
            return Err(SCPickerConfigurationError::Unavailable);
        }
        if unsafe {
            crate::ffi::sc_content_sharing_picker_set_default_configuration(config.as_ptr())
        } {
            Ok(())
        } else {
            Err(SCPickerConfigurationError::MainThreadRequired)
        }
    }

    /// Assign a picker configuration scoped to a single stream, mirroring
    /// Apple's `setConfiguration(_:for:)`.
    ///
    /// Pass `None` to clear the stream-specific configuration and fall back to
    /// the process-wide default.
    ///
    /// The assignment is complete when this method returns.
    ///
    /// # Errors
    ///
    /// Returns [`SCPickerConfigurationError::Unavailable`] when the API is
    /// unavailable, or [`SCPickerConfigurationError::MainThreadRequired`]
    /// when called from any other thread. Failed calls do not enqueue a later
    /// mutation.
    pub fn set_configuration_for_stream(
        config: Option<&SCContentSharingPickerConfiguration>,
        stream: &crate::stream::SCStream,
    ) -> Result<(), SCPickerConfigurationError> {
        if !Self::is_available() {
            return Err(SCPickerConfigurationError::Unavailable);
        }
        let config_ptr = config.map_or(
            std::ptr::null(),
            SCContentSharingPickerConfiguration::as_ptr,
        );
        if unsafe {
            crate::ffi::sc_content_sharing_picker_set_configuration_for_stream(
                config_ptr,
                stream.as_ptr(),
            )
        } {
            Ok(())
        } else {
            Err(SCPickerConfigurationError::MainThreadRequired)
        }
    }

    // ------------------------------------------------------------------
    // Persistent observers
    // ------------------------------------------------------------------

    /// Register a **repeating** observer that receives every picker event for
    /// as long as the returned subscription is alive.
    ///
    /// This is the API to use with
    /// [`SCContentSharingPickerConfiguration::set_allows_changing_selected_content`]:
    /// Apple re-invokes `contentSharingPicker(_:didUpdateWith:for:)` each time
    /// the user re-picks during an active share, and the one-shot
    /// [`Self::show`] family deliberately latches after the first event.
    ///
    /// The subscription unregisters on drop, so bind it to a variable that
    /// lives as long as you want events (use [`SCPickerSubscription::detach`]
    /// to keep it for the remainder of the process).
    ///
    /// Pair this with [`Self::present`] / [`Self::present_for_stream`] to
    /// surface the UI.
    ///
    /// Apple marks `SCContentSharingPicker` as `@MainActor`. Call this on the
    /// process main thread, or while an `AppKit` main run loop is active so the
    /// bridge can synchronously hop to it. Otherwise registration fails and
    /// the returned subscription is inactive.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// let mut config = SCContentSharingPickerConfiguration::new();
    /// config.set_allows_changing_selected_content(true);
    /// SCContentSharingPicker::set_default_configuration(&config)
    ///     .expect("call from the process main thread");
    ///
    /// let subscription = SCContentSharingPicker::add_observer(|event| match event {
    ///     SCPickerEvent::Updated { result, stream } => {
    ///         // Fires again every time the user changes their selection.
    ///         let _filter = result.filter();
    ///         let _existing_stream = stream;
    ///     }
    ///     SCPickerEvent::Cancelled { .. } => println!("cancelled"),
    ///     SCPickerEvent::Failed(err) => eprintln!("picker failed: {err}"),
    /// });
    ///
    /// SCContentSharingPicker::present();
    /// // ... keep `subscription` alive for as long as you want updates ...
    /// drop(subscription);
    /// ```
    #[must_use = "the observer is removed as soon as the subscription is dropped"]
    pub fn add_observer<F>(handler: F) -> SCPickerSubscription
    where
        F: Fn(SCPickerEvent) + Send + Sync + 'static,
    {
        if !Self::available_or_log("SCContentSharingPicker::add_observer") {
            return SCPickerSubscription::inactive();
        }
        let (context, active) = SCPickerObserverContext::into_raw(handler);
        let token = unsafe {
            crate::ffi::sc_content_sharing_picker_add_observer(
                observer_trampoline,
                observer_context_release,
                context,
            )
        };

        if token == 0 {
            eprintln!(
                "SCContentSharingPicker::add_observer must run on the main thread or while an \
                 AppKit main run loop is active"
            );
            observer_context_release(context);
        }

        SCPickerSubscription { token, active }
    }

    /// Remove every repeating observer registered through
    /// [`Self::add_observer`], regardless of which subscriptions are still
    /// alive. Returns how many were removed.
    ///
    /// Dropping the corresponding [`SCPickerSubscription`] afterwards is
    /// harmless — removal is idempotent.
    pub fn remove_all_observers() -> usize {
        if !Self::is_available() {
            return 0;
        }
        unsafe { crate::ffi::sc_content_sharing_picker_remove_all_observers() }
    }

    // ------------------------------------------------------------------
    // Standalone presentation
    // ------------------------------------------------------------------

    /// Present the picker without a content-style hint.
    ///
    /// Use with [`Self::add_observer`]; the one-shot [`Self::show`] family
    /// presents for you.
    pub fn present() {
        if !Self::available_or_log("SCContentSharingPicker::present") {
            return;
        }
        unsafe { crate::ffi::sc_content_sharing_picker_present(-1) }
    }

    /// Present the picker preselecting a content style.
    pub fn present_using_style(style: SCShareableContentStyle) {
        if !Self::available_or_log("SCContentSharingPicker::present_using_style") {
            return;
        }
        unsafe { crate::ffi::sc_content_sharing_picker_present(style as i32) }
    }

    /// Present the picker targeting an existing stream, so the user can swap
    /// the shared source mid-capture.
    pub fn present_for_stream(stream: &crate::stream::SCStream) {
        if !Self::available_or_log("SCContentSharingPicker::present_for_stream") {
            return;
        }
        unsafe { crate::ffi::sc_content_sharing_picker_present_for_stream(stream.as_ptr(), -1) }
    }

    /// Present the picker targeting an existing stream, preselecting a style.
    pub fn present_for_stream_using_style(
        stream: &crate::stream::SCStream,
        style: SCShareableContentStyle,
    ) {
        if !Self::available_or_log("SCContentSharingPicker::present_for_stream_using_style") {
            return;
        }
        unsafe {
            crate::ffi::sc_content_sharing_picker_present_for_stream(stream.as_ptr(), style as i32);
        }
    }
}

// ============================================================================
// Persistent observer: events, subscription handle, context, trampoline
// ============================================================================

/// A single event delivered to a repeating observer registered with
/// [`SCContentSharingPicker::add_observer`].
///
/// Unlike [`SCPickerOutcome`], which resolves a one-shot `show*()` call,
/// these arrive as many times as the user interacts with the picker.
#[derive(Debug)]
pub enum SCPickerEvent {
    /// The user selected (or re-selected) content. Mirrors Apple's
    /// `contentSharingPicker(_:didUpdateWith:for:)`. `stream` identifies an
    /// existing stream being updated; `None` means the user made a new
    /// selection rather than replacing a stream's source.
    Updated {
        /// Selected filter and metadata.
        result: SCPickerResult,
        /// Non-owning identity of the stream being updated.
        stream: Option<StreamIdentity>,
    },
    /// The user dismissed the picker. Mirrors
    /// `contentSharingPicker(_:didCancelFor:)`.
    Cancelled {
        /// Non-owning identity of the stream whose update was cancelled.
        stream: Option<StreamIdentity>,
    },
    /// The picker failed to start. Mirrors
    /// `contentSharingPickerStartDidFailWithError(_:)`.
    Failed(String),
}

/// Handle representing a live repeating-observer registration.
///
/// The observer is removed when this value is dropped. Call
/// [`Self::detach`] to keep the observer alive for the rest of the process.
#[derive(Debug)]
#[must_use = "the observer is removed as soon as the subscription is dropped"]
pub struct SCPickerSubscription {
    token: i64,
    active: std::sync::Arc<AtomicBool>,
}

impl SCPickerSubscription {
    fn inactive() -> Self {
        Self {
            token: 0,
            active: std::sync::Arc::new(AtomicBool::new(false)),
        }
    }

    /// Opaque identifier for this registration. Non-zero when registration
    /// succeeded.
    #[must_use]
    pub const fn token(&self) -> i64 {
        self.token
    }

    /// Whether this subscription refers to a live registration.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Remove the observer now instead of waiting for the drop.
    ///
    /// Returns `true` if a live observer was removed.
    pub fn unsubscribe(mut self) -> bool {
        self.remove()
    }

    /// Give up ownership without removing the observer, keeping it registered
    /// for the remainder of the process.
    ///
    /// Useful for "install once at startup" wiring where there is no natural
    /// owner for the handle. The registration can still be torn down with
    /// [`SCContentSharingPicker::remove_all_observers`].
    pub fn detach(mut self) {
        self.token = 0;
    }

    fn remove(&mut self) -> bool {
        if self.token == 0 || !self.is_active() {
            self.token = 0;
            return false;
        }
        let removed = unsafe { crate::ffi::sc_content_sharing_picker_remove_observer(self.token) };
        self.token = 0;
        if !removed {
            self.active.store(false, Ordering::Release);
        }
        removed
    }
}

impl Drop for SCPickerSubscription {
    fn drop(&mut self) {
        self.remove();
    }
}

struct SCPickerObserverContext {
    /// Cleared on unsubscribe so a callback already in flight is dropped
    /// rather than delivered after the user asked to stop listening.
    active: std::sync::Arc<AtomicBool>,
    handler: Box<dyn Fn(SCPickerEvent) + Send + Sync>,
}

impl SCPickerObserverContext {
    fn into_raw<F>(handler: F) -> (*mut c_void, std::sync::Arc<AtomicBool>)
    where
        F: Fn(SCPickerEvent) + Send + Sync + 'static,
    {
        let active = std::sync::Arc::new(AtomicBool::new(true));
        let context = std::sync::Arc::new(Self {
            active: std::sync::Arc::clone(&active),
            handler: Box::new(handler),
        });
        let mut registry = PICKER_OBSERVER_CONTEXTS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let contexts = registry.get_or_insert_with(HashMap::new);
        let id = loop {
            let id = NEXT_PICKER_OBSERVER_CONTEXT_ID.fetch_add(1, Ordering::Relaxed);
            if id != 0 && !contexts.contains_key(&id) {
                break id;
            }
        };
        contexts.insert(id, context);
        drop(registry);
        (id as *mut c_void, active)
    }
}

static NEXT_PICKER_OBSERVER_CONTEXT_ID: AtomicUsize = AtomicUsize::new(1);
static PICKER_OBSERVER_CONTEXTS: Mutex<
    Option<HashMap<usize, std::sync::Arc<SCPickerObserverContext>>>,
> = Mutex::new(None);

fn picker_observer_context(
    context: *mut c_void,
) -> Option<std::sync::Arc<SCPickerObserverContext>> {
    let id = context as usize;
    if id == 0 {
        return None;
    }
    PICKER_OBSERVER_CONTEXTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()?
        .get(&id)
        .cloned()
}

extern "C" fn observer_context_release(context: *mut c_void) {
    crate::utils::panic_safe::catch_user_panic("picker observer context release", || {
        let id = context as usize;
        if id == 0 {
            return;
        }
        let removed = {
            let mut contexts = PICKER_OBSERVER_CONTEXTS
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            contexts.as_mut().and_then(|contexts| contexts.remove(&id))
        };
        if let Some(context) = removed {
            context.active.store(false, Ordering::Release);
        }
    });
}

/// Trampoline for every repeating-observer event.
///
/// `event` follows the Swift bridge contract: 1 = updated (with a result
/// pointer), 0 = cancelled, anything else = start failure (with a message).
extern "C" fn observer_trampoline(
    event: i32,
    result_ptr: *const c_void,
    message: *const i8,
    stream_ptr: *const c_void,
    context: *mut c_void,
) {
    // The whole body sits inside the barrier: the registry lookup and the
    // message decoding both allocate, and an unwind out of an `extern "C"`
    // function is undefined behaviour on this crate's MSRV.
    crate::utils::panic_safe::catch_user_panic("picker observer callback", move || {
        let Some(context) = picker_observer_context(context) else {
            if !result_ptr.is_null() {
                unsafe { crate::ffi::sc_picker_result_release(result_ptr) };
            }
            return;
        };

        if !context.active.load(Ordering::Acquire) {
            // Unsubscribed while this callback was in flight; drop the result
            // rather than delivering it. Release the retained result first.
            if !result_ptr.is_null() {
                unsafe { crate::ffi::sc_picker_result_release(result_ptr) };
            }
            return;
        }

        let stream = StreamIdentity::from_ptr(stream_ptr);
        let decoded = match event {
            1 if !result_ptr.is_null() => SCPickerEvent::Updated {
                result: SCPickerResult { ptr: result_ptr },
                stream,
            },
            1 => SCPickerEvent::Failed("picker delivered an update without a result".to_string()),
            0 => SCPickerEvent::Cancelled { stream },
            _ => {
                let text = if message.is_null() {
                    "Content sharing picker failed to start".to_string()
                } else {
                    // SAFETY: Swift passes a NUL-terminated UTF-8 buffer that is
                    // valid for the duration of this call.
                    unsafe { std::ffi::CStr::from_ptr(message) }
                        .to_string_lossy()
                        .into_owned()
                };
                SCPickerEvent::Failed(text)
            }
        };

        (context.handler)(decoded);
    });
}

// ============================================================================
// One-shot callback context + trampoline (shared by all `show*()` methods)
// ============================================================================

/// Context owned by the one-shot callback registry.
struct PickerCallbackContext<O> {
    closure: Box<dyn FnOnce(O) + Send>,
}

static NEXT_PICKER_CALLBACK_ID: AtomicUsize = AtomicUsize::new(1);
static PICKER_CALLBACKS: Mutex<Option<HashMap<usize, Box<dyn Any + Send>>>> = Mutex::new(None);

/// Store a user closure and hand Swift a token that is never dereferenced.
#[allow(clippy::significant_drop_tightening)]
fn into_callback_context<O, F>(callback: F) -> *mut c_void
where
    O: 'static,
    F: FnOnce(O) + Send + 'static,
{
    let context = Box::new(PickerCallbackContext {
        closure: Box::new(callback),
    });
    let mut callbacks = PICKER_CALLBACKS
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let callbacks = callbacks.get_or_insert_with(HashMap::new);
    loop {
        let id = NEXT_PICKER_CALLBACK_ID.fetch_add(1, Ordering::Relaxed);
        if id != 0 && !callbacks.contains_key(&id) {
            callbacks.insert(id, context);
            return id as *mut c_void;
        }
    }
}

fn take_callback_context<O: 'static>(
    context: *mut c_void,
) -> Option<Box<PickerCallbackContext<O>>> {
    let id = context as usize;
    if id == 0 {
        return None;
    }
    let entry = PICKER_CALLBACKS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()?
        .remove(&id)?;
    entry.downcast::<PickerCallbackContext<O>>().ok()
}

/// Decodes the `(code, ptr)` pair from the Swift bridge into a typed outcome.
///
/// Implemented by zero-sized marker types so a single generic trampoline can
/// serve both the result-bearing and filter-only APIs while keeping the FFI
/// signature identical.
trait PickerDecode {
    type Outcome: 'static;
    fn decode(code: i32, ptr: *const c_void) -> Self::Outcome;
    unsafe fn release(ptr: *const c_void);
}

struct ResultDecoder;
impl PickerDecode for ResultDecoder {
    type Outcome = SCPickerOutcome;
    fn decode(code: i32, ptr: *const c_void) -> SCPickerOutcome {
        match code {
            1 if !ptr.is_null() => SCPickerOutcome::Picked(SCPickerResult { ptr }),
            0 => SCPickerOutcome::Cancelled,
            _ => SCPickerOutcome::Error("Picker failed".to_string()),
        }
    }

    unsafe fn release(ptr: *const c_void) {
        if !ptr.is_null() {
            unsafe { crate::ffi::sc_picker_result_release(ptr) };
        }
    }
}

struct FilterDecoder;
impl PickerDecode for FilterDecoder {
    type Outcome = SCPickerFilterOutcome;
    fn decode(code: i32, ptr: *const c_void) -> SCPickerFilterOutcome {
        match code {
            1 if !ptr.is_null() => {
                SCPickerFilterOutcome::Filter(SCContentFilter::from_picker_ptr(ptr))
            }
            0 => SCPickerFilterOutcome::Cancelled,
            _ => SCPickerFilterOutcome::Error("Picker failed".to_string()),
        }
    }

    unsafe fn release(ptr: *const c_void) {
        if !ptr.is_null() {
            unsafe { crate::ffi::sc_content_filter_release(ptr) };
        }
    }
}

/// Single trampoline for every picker `show*()` callback.
///
/// `code` follows the Swift bridge contract (1 = picked, 0 = cancelled,
/// anything else = error). A `code` of 0 is also produced by the Swift
/// replacement path when a pending observer is superseded by a newer
/// `show*()`, so a replaced picker resolves as `Cancelled` rather than
/// leaking its context.
///
/// The registry entry is removed exactly once. Duplicate or late callbacks
/// find no entry, so they cannot dereference freed memory.
extern "C" fn picker_trampoline<D: PickerDecode>(
    code: i32,
    ptr: *const c_void,
    context: *mut c_void,
) {
    crate::utils::panic_safe::catch_user_panic("picker callback", move || {
        let Some(context) = take_callback_context::<D::Outcome>(context) else {
            unsafe { D::release(ptr) };
            return;
        };
        let outcome = D::decode(code, ptr);
        (context.closure)(outcome);
    });
}

// SAFETY: the wrapper owns its Swift box exclusively — `Clone` copies the box
// rather than retaining it, and no constructor hands out a second handle to the
// same allocation. Mutation therefore only happens through `&mut self`, which
// Rust already makes exclusive, and the `&self` methods are pure getters. The
// box itself is a Swift class, so the refcount traffic that `Drop` performs is
// atomic.
unsafe impl Send for SCContentSharingPickerConfiguration {}
unsafe impl Sync for SCContentSharingPickerConfiguration {}
// SAFETY: `SCPickerResult` holds retained Objective-C objects whose reference
// counting is atomic; it is safe to send between and share across threads.
unsafe impl Send for SCPickerResult {}
unsafe impl Sync for SCPickerResult {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn duplicate_one_shot_callback_is_ignored_after_context_drop() {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let context = into_callback_context::<SCPickerFilterOutcome, _>(move |_| {
            observed.fetch_add(1, Ordering::SeqCst);
        });

        picker_trampoline::<FilterDecoder>(0, std::ptr::null(), context);
        picker_trampoline::<FilterDecoder>(0, std::ptr::null(), context);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn released_repeating_observer_ignores_late_callback() {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let (context, active) = SCPickerObserverContext::into_raw(move |_| {
            observed.fetch_add(1, Ordering::SeqCst);
        });

        observer_context_release(context);
        observer_trampoline(
            0,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            context,
        );

        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(!active.load(Ordering::Acquire));
    }

    #[test]
    fn repeating_observer_preserves_stream_identity() {
        let observed = Arc::new(Mutex::new(None));
        let output = Arc::clone(&observed);
        let (context, _) = SCPickerObserverContext::into_raw(move |event| {
            if let SCPickerEvent::Cancelled { stream } = event {
                *output.lock().unwrap_or_else(PoisonError::into_inner) = stream;
            }
        });
        let stream_ptr = std::ptr::NonNull::<c_void>::dangling()
            .as_ptr()
            .cast_const();

        observer_trampoline(0, std::ptr::null(), std::ptr::null(), stream_ptr, context);
        observer_context_release(context);

        assert_eq!(
            *observed.lock().unwrap_or_else(PoisonError::into_inner),
            StreamIdentity::from_ptr(stream_ptr)
        );
    }
}
