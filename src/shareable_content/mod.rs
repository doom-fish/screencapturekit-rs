//! Shareable content types - displays, windows, and applications
//!
//! This module provides access to the system's displays, windows, and running
//! applications that can be captured by `ScreenCaptureKit`.
//!
//! ## Main Types
//!
//! - [`SCShareableContent`] - Container for all available content (displays, windows, apps)
//! - [`SCDisplay`] - A physical or virtual display that can be captured
//! - [`SCWindow`] - A window that can be captured
//! - [`SCRunningApplication`] - A running application whose windows can be captured
//!
//! ## Workflow
//!
//! 1. Call [`SCShareableContent::get()`] to retrieve available content
//! 2. Select displays/windows/apps to capture
//! 3. Create an [`SCContentFilter`](crate::stream::content_filter::SCContentFilter) from the selection
//!
//! # Examples
//!
//! ## List All Content
//!
//! ```no_run
//! use screencapturekit::shareable_content::SCShareableContent;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Get all shareable content
//! let content = SCShareableContent::get()?;
//!
//! // List displays
//! for display in content.displays() {
//!     println!("Display {}: {}x{}",
//!         display.display_id(),
//!         display.width(),
//!         display.height()
//!     );
//! }
//!
//! // List windows
//! for window in content.windows() {
//!     if let Some(title) = window.title() {
//!         println!("Window: {}", title);
//!     }
//! }
//!
//! // List applications
//! for app in content.applications() {
//!     println!("App: {} ({})", app.application_name(), app.bundle_identifier());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Filter On-Screen Windows Only
//!
//! ```no_run
//! use screencapturekit::shareable_content::SCShareableContent;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::with_options()
//!     .on_screen_windows_only(true)
//!     .exclude_desktop_windows(true)
//!     .get()?;
//!
//! println!("Found {} on-screen windows", content.windows().len());
//! # Ok(())
//! # }
//! ```

pub mod display;
pub mod running_application;
pub mod window;
pub use display::SCDisplay;
pub use running_application::SCRunningApplication;
pub use window::SCWindow;

use crate::error::SCError;
use crate::utils::completion::{error_from_cstr, SyncCompletion};
use core::fmt;
use std::ffi::c_void;

#[repr(transparent)]
pub struct SCShareableContent(*const c_void);

unsafe impl Send for SCShareableContent {}
unsafe impl Sync for SCShareableContent {}

/// Callback for shareable content retrieval
extern "C" fn shareable_content_callback(
    content_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    if !error_ptr.is_null() {
        let error = unsafe { error_from_cstr(error_ptr) };
        unsafe { SyncCompletion::<SCShareableContent>::complete_err(user_data, error) };
    } else if !content_ptr.is_null() {
        let content = unsafe { SCShareableContent::from_ptr(content_ptr) };
        unsafe { SyncCompletion::complete_ok(user_data, content) };
    } else {
        unsafe {
            SyncCompletion::<SCShareableContent>::complete_err(
                user_data,
                "Unknown error".to_string(),
            );
        };
    }
}

impl PartialEq for SCShareableContent {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SCShareableContent {}

impl std::hash::Hash for SCShareableContent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Clone for SCShareableContent {
    fn clone(&self) -> Self {
        unsafe { Self(crate::ffi::sc_shareable_content_retain(self.0)) }
    }
}

impl SCShareableContent {
    /// Create from raw pointer (used internally)
    ///
    /// # Safety
    /// The pointer must be a valid retained `SCShareableContent` pointer from Swift FFI.
    pub(crate) unsafe fn from_ptr(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get shareable content (displays, windows, and applications)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// println!("Found {} displays", content.displays().len());
    /// println!("Found {} windows", content.windows().len());
    /// println!("Found {} apps", content.applications().len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if screen recording permission is not granted.
    pub fn get() -> Result<Self, SCError> {
        Self::with_options().get()
    }

    /// Create options builder for customizing shareable content retrieval
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::with_options()
    ///     .on_screen_windows_only(true)
    ///     .exclude_desktop_windows(true)
    ///     .get()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_options() -> SCShareableContentOptions {
        SCShareableContentOptions::default()
    }

    /// Get all available displays
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// for display in content.displays() {
    ///     println!("Display: {}x{}", display.width(), display.height());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn displays(&self) -> Vec<SCDisplay> {
        unsafe {
            let count = crate::ffi::sc_shareable_content_get_displays_count(self.0);
            // FFI returns isize but count is always positive
            #[allow(clippy::cast_sign_loss)]
            let mut displays = Vec::with_capacity(count as usize);

            for i in 0..count {
                let display_ptr = crate::ffi::sc_shareable_content_get_display_at(self.0, i);
                if !display_ptr.is_null() {
                    displays.push(SCDisplay::from_ptr(display_ptr));
                }
            }

            displays
        }
    }

    /// Get all available windows
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// for window in content.windows() {
    ///     if let Some(title) = window.title() {
    ///         println!("Window: {}", title);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn windows(&self) -> Vec<SCWindow> {
        unsafe {
            let count = crate::ffi::sc_shareable_content_get_windows_count(self.0);
            // FFI returns isize but count is always positive
            #[allow(clippy::cast_sign_loss)]
            let mut windows = Vec::with_capacity(count as usize);

            for i in 0..count {
                let window_ptr = crate::ffi::sc_shareable_content_get_window_at(self.0, i);
                if !window_ptr.is_null() {
                    windows.push(SCWindow::from_ptr(window_ptr));
                }
            }

            windows
        }
    }

    /// Get all available running applications
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// for app in content.applications() {
    ///     println!("App: {} (PID: {})", app.application_name(), app.process_id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn applications(&self) -> Vec<SCRunningApplication> {
        unsafe {
            let count = crate::ffi::sc_shareable_content_get_applications_count(self.0);
            // FFI returns isize but count is always positive
            #[allow(clippy::cast_sign_loss)]
            let mut apps = Vec::with_capacity(count as usize);

            for i in 0..count {
                let app_ptr = crate::ffi::sc_shareable_content_get_application_at(self.0, i);
                if !app_ptr.is_null() {
                    apps.push(SCRunningApplication::from_ptr(app_ptr));
                }
            }

            apps
        }
    }

    #[allow(dead_code)]
    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.0
    }
}

impl Drop for SCShareableContent {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                crate::ffi::sc_shareable_content_release(self.0);
            }
        }
    }
}

impl fmt::Debug for SCShareableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCShareableContent")
            .field("displays", &self.displays().len())
            .field("windows", &self.windows().len())
            .field("applications", &self.applications().len())
            .finish()
    }
}

impl fmt::Display for SCShareableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SCShareableContent ({} displays, {} windows, {} applications)",
            self.displays().len(),
            self.windows().len(),
            self.applications().len()
        )
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SCShareableContentOptions {
    exclude_desktop_windows: bool,
    on_screen_windows_only: bool,
}

impl SCShareableContentOptions {
    /// Exclude desktop windows from the shareable content.
    ///
    /// When set to `true`, desktop-level windows (like the desktop background)
    /// are excluded from the returned window list.
    #[must_use]
    pub fn exclude_desktop_windows(mut self, exclude: bool) -> Self {
        self.exclude_desktop_windows = exclude;
        self
    }

    /// Include only on-screen windows in the shareable content.
    ///
    /// When set to `true`, only windows that are currently visible on screen
    /// are included. Minimized or off-screen windows are excluded.
    #[must_use]
    pub fn on_screen_windows_only(mut self, on_screen_only: bool) -> Self {
        self.on_screen_windows_only = on_screen_only;
        self
    }

    /// Get shareable content synchronously
    ///
    /// This blocks until the content is retrieved.
    ///
    /// # Errors
    ///
    /// Returns an error if screen recording permission is not granted or retrieval fails.
    pub fn get(self) -> Result<SCShareableContent, SCError> {
        let (completion, context) = SyncCompletion::<SCShareableContent>::new();

        unsafe {
            crate::ffi::sc_shareable_content_get_with_options(
                self.exclude_desktop_windows,
                self.on_screen_windows_only,
                shareable_content_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::NoShareableContent)
    }

    /// Get shareable content with only windows below a reference window
    ///
    /// This returns windows that are stacked below the specified reference window
    /// in the window layering order.
    ///
    /// # Arguments
    ///
    /// * `reference_window` - The window to use as the reference point
    ///
    /// # Errors
    ///
    /// Returns an error if screen recording permission is not granted or retrieval fails.
    pub fn below_window(self, reference_window: &SCWindow) -> Result<SCShareableContent, SCError> {
        let (completion, context) = SyncCompletion::<SCShareableContent>::new();

        unsafe {
            crate::ffi::sc_shareable_content_get_below_window(
                self.exclude_desktop_windows,
                reference_window.as_ptr(),
                shareable_content_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::NoShareableContent)
    }

    /// Get shareable content with only windows above a reference window
    ///
    /// This returns windows that are stacked above the specified reference window
    /// in the window layering order.
    ///
    /// # Arguments
    ///
    /// * `reference_window` - The window to use as the reference point
    ///
    /// # Errors
    ///
    /// Returns an error if screen recording permission is not granted or retrieval fails.
    pub fn above_window(self, reference_window: &SCWindow) -> Result<SCShareableContent, SCError> {
        let (completion, context) = SyncCompletion::<SCShareableContent>::new();

        unsafe {
            crate::ffi::sc_shareable_content_get_above_window(
                self.exclude_desktop_windows,
                reference_window.as_ptr(),
                shareable_content_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::NoShareableContent)
    }
}

impl SCShareableContent {
    /// Get shareable content for the current process only (macOS 14.4+)
    ///
    /// This retrieves content that the current process can capture without
    /// requiring user authorization via TCC (Transparency, Consent, and Control).
    ///
    /// # Errors
    ///
    /// Returns an error if retrieval fails.
    #[cfg(feature = "macos_14_4")]
    pub fn current_process() -> Result<Self, SCError> {
        let (completion, context) = SyncCompletion::<Self>::new();

        unsafe {
            crate::ffi::sc_shareable_content_get_current_process_displays(
                shareable_content_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::NoShareableContent)
    }
}

// MARK: - SCShareableContentInfo (macOS 14.0+)

/// Information about shareable content from a filter (macOS 14.0+)
///
/// Provides metadata about the content being captured, including dimensions and scale factor.
#[cfg(feature = "macos_14_0")]
pub struct SCShareableContentInfo(*const c_void);

#[cfg(feature = "macos_14_0")]
impl SCShareableContentInfo {
    /// Get content info for a filter
    ///
    /// Returns information about the content described by the given filter.
    pub fn for_filter(filter: &crate::stream::content_filter::SCContentFilter) -> Option<Self> {
        let ptr = unsafe { crate::ffi::sc_shareable_content_info_for_filter(filter.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// Get the content style
    pub fn style(&self) -> crate::stream::content_filter::SCShareableContentStyle {
        let value = unsafe { crate::ffi::sc_shareable_content_info_get_style(self.0) };
        crate::stream::content_filter::SCShareableContentStyle::from(value)
    }

    /// Get the point-to-pixel scale factor
    ///
    /// Typically 2.0 for Retina displays.
    pub fn point_pixel_scale(&self) -> f32 {
        unsafe { crate::ffi::sc_shareable_content_info_get_point_pixel_scale(self.0) }
    }

    /// Get the content rectangle in points
    pub fn content_rect(&self) -> crate::cg::CGRect {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;
        unsafe {
            crate::ffi::sc_shareable_content_info_get_content_rect(
                self.0,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            );
        }
        crate::cg::CGRect::new(x, y, width, height)
    }

    /// Get the content size in pixels
    ///
    /// Convenience method that multiplies `content_rect` dimensions by `point_pixel_scale`.
    pub fn pixel_size(&self) -> (u32, u32) {
        let rect = self.content_rect();
        let scale = self.point_pixel_scale();
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let width = (rect.width * f64::from(scale)) as u32;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let height = (rect.height * f64::from(scale)) as u32;
        (width, height)
    }
}

#[cfg(feature = "macos_14_0")]
impl Drop for SCShareableContentInfo {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                crate::ffi::sc_shareable_content_info_release(self.0);
            }
        }
    }
}

#[cfg(feature = "macos_14_0")]
impl Clone for SCShareableContentInfo {
    fn clone(&self) -> Self {
        unsafe { Self(crate::ffi::sc_shareable_content_info_retain(self.0)) }
    }
}

#[cfg(feature = "macos_14_0")]
impl fmt::Debug for SCShareableContentInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCShareableContentInfo")
            .field("style", &self.style())
            .field("point_pixel_scale", &self.point_pixel_scale())
            .field("content_rect", &self.content_rect())
            .finish()
    }
}

#[cfg(feature = "macos_14_0")]
impl fmt::Display for SCShareableContentInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (width, height) = self.pixel_size();
        write!(
            f,
            "ContentInfo({:?}, {}x{} px, scale: {})",
            self.style(),
            width,
            height,
            self.point_pixel_scale()
        )
    }
}

#[cfg(feature = "macos_14_0")]
unsafe impl Send for SCShareableContentInfo {}
#[cfg(feature = "macos_14_0")]
unsafe impl Sync for SCShareableContentInfo {}
