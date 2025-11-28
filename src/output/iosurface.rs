//! `IOSurface` wrapper for `ScreenCaptureKit`
//!
//! Provides access to IOSurface-backed pixel buffers for efficient frame processing

use std::ffi::c_void;
use std::io;

/// Lock options for `IOSurface`
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IOSurfaceLockOptions {
    /// Read-only lock
    ReadOnly = 0x0000_0001,
    /// Avoid synchronization
    AvoidSync = 0x0000_0002,
}

impl IOSurfaceLockOptions {
    pub const fn as_u32(self) -> u32 {
        self as u32
    }
}

/// A guard that provides access to locked `IOSurface` memory
///
/// The surface is automatically unlocked when this guard is dropped.
pub struct IOSurfaceLockGuard<'a> {
    surface_ptr: *const c_void,
    base_address: std::ptr::NonNull<u8>,
    width: usize,
    height: usize,
    bytes_per_row: usize,
    options: IOSurfaceLockOptions,
    _phantom: std::marker::PhantomData<&'a IOSurface>,
}

impl IOSurfaceLockGuard<'_> {
    /// Create a new lock guard (used internally)
    pub(crate) unsafe fn new(
        surface_ptr: *const c_void,
        options: IOSurfaceLockOptions,
        width: usize,
        height: usize,
        bytes_per_row: usize,
    ) -> Result<Self, i32> {
        // Lock the surface
        let result = crate::ffi::iosurface_lock(surface_ptr, options.as_u32());
        if result != 0 {
            return Err(result);
        }

        // Get base address
        let base_ptr = crate::ffi::iosurface_get_base_address(surface_ptr);
        let base_address = std::ptr::NonNull::new(base_ptr).ok_or(-1)?;

        Ok(Self {
            surface_ptr,
            base_address,
            width,
            height,
            bytes_per_row,
            options,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Get the width in pixels
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Get the height in pixels
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Get bytes per row
    pub const fn bytes_per_row(&self) -> usize {
        self.bytes_per_row
    }

    /// Get raw pointer to buffer data
    pub const fn as_ptr(&self) -> *const u8 {
        self.base_address.as_ptr()
    }

    /// Get mutable raw pointer to buffer data
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.base_address.as_ptr()
    }

    /// Get buffer data as a byte slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.base_address.as_ptr(), self.height * self.bytes_per_row)
        }
    }

    /// Get a specific row as a slice
    pub fn row(&self, row_index: usize) -> Option<&[u8]> {
        if row_index >= self.height {
            return None;
        }
        unsafe {
            let row_ptr = self
                .base_address
                .as_ptr()
                .add(row_index * self.bytes_per_row);
            Some(std::slice::from_raw_parts(row_ptr, self.bytes_per_row))
        }
    }

    /// Access buffer with a standard `std::io::Cursor`
    ///
    /// Returns a cursor over the buffer data that implements `Read` and `Seek` traits.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{Read, Seek, SeekFrom};
    /// use screencapturekit::output::PixelBufferCursorExt;
    /// # use screencapturekit::output::IOSurfaceLockOptions;
    ///
    /// # fn example(guard: screencapturekit::output::IOSurfaceLockGuard) {
    /// let mut cursor = guard.cursor();
    ///
    /// // Read a pixel using the extension trait
    /// let pixel = cursor.read_pixel().unwrap();
    ///
    /// // Or use standard Read trait
    /// let mut buf = [0u8; 4];
    /// cursor.read_exact(&mut buf).unwrap();
    /// # }
    /// ```
    pub fn cursor(&self) -> io::Cursor<&[u8]> {
        io::Cursor::new(self.as_slice())
    }

    /// Access buffer with a cursor using a closure (for backward compatibility)
    ///
    /// This method is provided for backward compatibility. Consider using
    /// `cursor()` directly for more flexibility.
    pub fn with_cursor<F, R>(&self, f: F) -> R
    where
        F: FnOnce(io::Cursor<&[u8]>) -> R,
    {
        f(self.cursor())
    }
}

impl Drop for IOSurfaceLockGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::iosurface_unlock(self.surface_ptr, self.options.as_u32());
        }
    }
}

impl std::ops::Deref for IOSurfaceLockGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// Wrapper around `IOSurface`
///
/// `IOSurface` is a framebuffer object suitable for sharing across process boundaries.
/// `ScreenCaptureKit` uses `IOSurface`-backed `CVPixelBuffer`s for efficient frame delivery.
pub struct IOSurface(*const c_void);

impl PartialEq for IOSurface {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for IOSurface {}

impl std::hash::Hash for IOSurface {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl IOSurface {
    /// Create from raw pointer (used internally)
    pub(crate) unsafe fn from_ptr(ptr: *const c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// Get the width of the `IOSurface` in pixels
    pub fn width(&self) -> usize {
        // FFI returns isize but dimensions are always positive
        #[allow(clippy::cast_sign_loss)]
        unsafe {
            crate::ffi::iosurface_get_width(self.0) as usize
        }
    }

    /// Get the height of the `IOSurface` in pixels
    pub fn height(&self) -> usize {
        // FFI returns isize but dimensions are always positive
        #[allow(clippy::cast_sign_loss)]
        unsafe {
            crate::ffi::iosurface_get_height(self.0) as usize
        }
    }

    /// Get the number of bytes per row
    pub fn bytes_per_row(&self) -> usize {
        // FFI returns isize but byte count is always positive
        #[allow(clippy::cast_sign_loss)]
        unsafe {
            crate::ffi::iosurface_get_bytes_per_row(self.0) as usize
        }
    }

    /// Get the pixel format (OSType/FourCC)
    pub fn pixel_format(&self) -> u32 {
        unsafe { crate::ffi::iosurface_get_pixel_format(self.0) }
    }

    /// Get the base address of the `IOSurface` buffer
    ///
    /// **Important:** You must lock the `IOSurface` before accessing memory!
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid while the `IOSurface` is locked.
    /// Accessing unlocked memory or after unlock is undefined behavior.
    pub unsafe fn base_address(&self) -> *mut u8 {
        crate::ffi::iosurface_get_base_address(self.0)
    }

    /// Lock the `IOSurface` and get a guard for safe access
    ///
    /// The surface will be automatically unlocked when the guard is dropped.
    ///
    /// # Errors
    ///
    /// Returns an `IOSurface` error code if the lock operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use screencapturekit::output::{IOSurface, IOSurfaceLockOptions};
    /// # fn example(surface: IOSurface) -> Result<(), i32> {
    /// let guard = surface.lock(IOSurfaceLockOptions::ReadOnly)?;
    /// let data = guard.as_slice();
    /// // Use data...
    /// // Surface is automatically unlocked when guard goes out of scope
    /// # Ok(())
    /// # }
    /// ```
    pub fn lock(&self, options: IOSurfaceLockOptions) -> Result<IOSurfaceLockGuard<'_>, i32> {
        unsafe {
            IOSurfaceLockGuard::new(
                self.0,
                options,
                self.width(),
                self.height(),
                self.bytes_per_row(),
            )
        }
    }

    /// Check if the `IOSurface` is currently in use
    pub fn is_in_use(&self) -> bool {
        unsafe { crate::ffi::iosurface_is_in_use(self.0) }
    }
}

impl Drop for IOSurface {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                crate::ffi::iosurface_release(self.0);
            }
        }
    }
}

unsafe impl Send for IOSurface {}
unsafe impl Sync for IOSurface {}

impl std::fmt::Debug for IOSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IOSurface")
            .field("width", &self.width())
            .field("height", &self.height())
            .field("bytes_per_row", &self.bytes_per_row())
            .field("pixel_format", &format!("0x{:08X}", self.pixel_format()))
            .field("is_in_use", &self.is_in_use())
            .finish()
    }
}

/// Extension trait for `CVPixelBuffer` to access `IOSurface`
pub trait CVPixelBufferIOSurface {
    /// Get the underlying `IOSurface` if the pixel buffer is backed by one
    fn iosurface(&self) -> Option<IOSurface>;

    /// Check if this pixel buffer is backed by an `IOSurface`
    fn is_backed_by_iosurface(&self) -> bool;
}

impl CVPixelBufferIOSurface for crate::output::CVPixelBuffer {
    fn iosurface(&self) -> Option<IOSurface> {
        unsafe {
            let ptr = crate::ffi::cv_pixel_buffer_get_iosurface(self.as_ptr());
            IOSurface::from_ptr(ptr)
        }
    }

    fn is_backed_by_iosurface(&self) -> bool {
        unsafe { crate::ffi::cv_pixel_buffer_is_backed_by_iosurface(self.as_ptr()) }
    }
}
