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

impl SCDisplay {
    /// Create from raw pointer (used internally by shareable content)
    pub(crate) unsafe fn from_ptr(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Create from an FFI-owned (retained) pointer, returning `None` if null.
    ///
    /// # Safety
    /// `ptr` must be null or a valid retained `SCDisplay` pointer transferred
    /// from the Swift FFI bridge (ownership moves into the returned wrapper).
    pub(crate) unsafe fn from_retained_ptr(ptr: *const c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Self::from_ptr(ptr) })
        }
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
        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;
        unsafe {
            crate::ffi::sc_display_get_frame_packed(
                self.0,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            );
        }
        CGRect::new(x, y, width, height)
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

crate::utils::retained::sc_retained!(
    SCDisplay,
    retain = crate::ffi::sc_display_retain,
    release = crate::ffi::sc_display_release,
);

// SAFETY: `SCDisplay` wraps an immutable Objective-C ScreenCaptureKit object.
// ObjC reference counting is atomic and these accessor-only objects are safe to
// send between and share across threads.
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
