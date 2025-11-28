use crate::cg::CGRect;
use core::fmt;
use std::ffi::c_void;

/// Opaque wrapper around `SCDisplay` from `ScreenCaptureKit`
///
/// Represents a physical or virtual display that can be captured.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::shareable_content::SCShareableContent;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let content = SCShareableContent::get()?;
/// for display in content.displays() {
///     println!("Display {}: {}x{}",
///         display.display_id(),
///         display.width(),
///         display.height()
///     );
/// }
/// # Ok(())
/// # }
/// ```
#[repr(transparent)]
pub struct SCDisplay(*const c_void);

impl PartialEq for SCDisplay {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SCDisplay {}

impl std::hash::Hash for SCDisplay {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Raw pointer type for `SCDisplay` (for FFI compatibility)
pub type SCDisplayRef = *const c_void;

impl SCDisplay {
    /// Create from raw pointer (used internally by shareable content)
    pub(crate) unsafe fn from_ptr(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer (used internally)
    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.0
    }

    /// Get display ID
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use screencapturekit::shareable_content::SCShareableContent;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// if let Some(display) = content.displays().first() {
    ///     println!("Display ID: {}", display.display_id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_id(&self) -> u32 {
        unsafe { crate::ffi::sc_display_get_display_id(self.0) }
    }

    /// Get display frame (position and size)
    pub fn frame(&self) -> CGRect {
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut width = 0.0;
            let mut height = 0.0;
            crate::ffi::sc_display_get_frame(self.0, &mut x, &mut y, &mut width, &mut height);
            CGRect::new(x, y, width, height)
        }
    }

    /// Get display height in pixels
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use screencapturekit::shareable_content::SCShareableContent;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// if let Some(display) = content.displays().first() {
    ///     println!("Display resolution: {}x{}", display.width(), display.height());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn height(&self) -> u32 {
        // FFI returns isize but display dimensions are always positive and fit in u32
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_display_get_height(self.0) as u32
        }
    }

    /// Get display width in pixels
    pub fn width(&self) -> u32 {
        // FFI returns isize but display dimensions are always positive and fit in u32
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_display_get_width(self.0) as u32
        }
    }
}

impl Drop for SCDisplay {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::sc_display_release(self.0);
        }
    }
}

impl Clone for SCDisplay {
    fn clone(&self) -> Self {
        unsafe { Self(crate::ffi::sc_display_retain(self.0)) }
    }
}

unsafe impl Send for SCDisplay {}
unsafe impl Sync for SCDisplay {}

impl fmt::Debug for SCDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCDisplay")
            .field("display_id", &self.display_id())
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}

impl fmt::Display for SCDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Display {} ({}x{})",
            self.display_id(),
            self.width(),
            self.height()
        )
    }
}
