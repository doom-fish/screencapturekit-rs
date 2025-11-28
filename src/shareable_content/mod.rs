//! Shareable content types - displays, windows, and applications
//!
//! This module provides access to the system's displays, windows, and running
//! applications that can be captured by `ScreenCaptureKit`.
//!
//! # Examples
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
//!     println!("App: {}", app.application_name());
//! }
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
use crate::utils::sync_completion::{error_from_cstr, SyncCompletion};
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

    pub fn as_ptr(&self) -> *const c_void {
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

        completion
            .wait()
            .map_err(SCError::NoShareableContent)
    }
}
