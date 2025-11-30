use crate::cg::CGRect;
use crate::utils::ffi_string::ffi_string_owned;
use core::fmt;
use std::ffi::c_void;

use super::SCRunningApplication;

/// Wrapper around `SCWindow` from `ScreenCaptureKit`
///
/// Represents a window that can be captured.
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
///         println!("Window: {} (ID: {})", title, window.window_id());
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[repr(transparent)]
pub struct SCWindow(*const c_void);

impl PartialEq for SCWindow {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SCWindow {}

impl std::hash::Hash for SCWindow {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl SCWindow {
    /// Create from raw pointer (used internally by shareable content)
    pub(crate) unsafe fn from_ptr(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Create from FFI-owned pointer (caller transfers ownership)
    #[allow(dead_code)]
    pub(crate) fn from_ffi_owned(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer (used internally)
    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.0
    }

    /// Get the owning application
    pub fn owning_application(&self) -> Option<SCRunningApplication> {
        unsafe {
            let app_ptr = crate::ffi::sc_window_get_owning_application(self.0);
            if app_ptr.is_null() {
                None
            } else {
                Some(SCRunningApplication::from_ptr(app_ptr))
            }
        }
    }

    /// Get the window ID
    pub fn window_id(&self) -> u32 {
        unsafe { crate::ffi::sc_window_get_window_id(self.0) }
    }

    /// Get the window frame (position and size)
    pub fn frame(&self) -> CGRect {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;
        unsafe {
            crate::ffi::sc_window_get_frame_packed(self.0, &mut x, &mut y, &mut width, &mut height);
        }
        CGRect::new(x, y, width, height)
    }

    /// Get the window title (if available)
    pub fn title(&self) -> Option<String> {
        unsafe { ffi_string_owned(|| crate::ffi::sc_window_get_title_owned(self.0)) }
    }

    /// Get window layer
    pub fn window_layer(&self) -> i32 {
        // FFI returns isize but window layer fits in i32
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_window_get_window_layer(self.0) as i32
        }
    }

    /// Check if window is on screen
    pub fn is_on_screen(&self) -> bool {
        unsafe { crate::ffi::sc_window_is_on_screen(self.0) }
    }

    /// Check if window is active (macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    pub fn is_active(&self) -> bool {
        unsafe { crate::ffi::sc_window_is_active(self.0) }
    }
}

impl Drop for SCWindow {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                crate::ffi::sc_window_release(self.0);
            }
        }
    }
}

impl Clone for SCWindow {
    fn clone(&self) -> Self {
        unsafe { Self(crate::ffi::sc_window_retain(self.0)) }
    }
}

impl fmt::Debug for SCWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("SCWindow");
        debug
            .field("window_id", &self.window_id())
            .field("title", &self.title())
            .field("frame", &self.frame())
            .field("window_layer", &self.window_layer())
            .field("is_on_screen", &self.is_on_screen());
        #[cfg(feature = "macos_14_0")]
        debug.field("is_active", &self.is_active());
        debug.finish()
    }
}

impl fmt::Display for SCWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Window {} \"{}\" ({})",
            self.window_id(),
            self.title().unwrap_or_else(|| String::from("<untitled>")),
            self.frame()
        )
    }
}

unsafe impl Send for SCWindow {}
unsafe impl Sync for SCWindow {}
