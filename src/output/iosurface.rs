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

    /// Get the base address of a specific plane
    ///
    /// For multi-planar formats like YCbCr 4:2:0, each plane has its own base address:
    /// - Plane 0: Y (luminance) data
    /// - Plane 1: `CbCr` (chrominance) data
    ///
    /// Returns `None` if the plane index is out of bounds.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid while this lock guard is held.
    #[allow(clippy::cast_sign_loss)]
    pub fn base_address_of_plane(&self, plane: usize) -> Option<*mut u8> {
        let plane_count =
            unsafe { crate::ffi::iosurface_get_plane_count(self.surface_ptr) as usize };
        if plane >= plane_count {
            return None;
        }
        let ptr = unsafe {
            crate::cm::ffi::io_surface_get_base_address_of_plane(self.surface_ptr.cast_mut(), plane)
        };
        if ptr.is_null() {
            None
        } else {
            Some(ptr.cast::<u8>())
        }
    }

    /// Get a slice of plane data
    ///
    /// Returns the data for a specific plane as a byte slice. The slice size is
    /// calculated from the plane's height and bytes per row.
    ///
    /// Returns `None` if the plane index is out of bounds.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    pub fn plane_data(&self, plane: usize) -> Option<&[u8]> {
        let base = self.base_address_of_plane(plane)?;
        let height = unsafe {
            crate::ffi::iosurface_get_height_of_plane(self.surface_ptr, plane as isize) as usize
        };
        let bytes_per_row = unsafe {
            crate::ffi::iosurface_get_bytes_per_row_of_plane(self.surface_ptr, plane as isize)
                as usize
        };
        Some(unsafe { std::slice::from_raw_parts(base, height * bytes_per_row) })
    }

    /// Get a specific row from a plane as a slice
    ///
    /// Returns `None` if the plane or row index is out of bounds.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    pub fn plane_row(&self, plane: usize, row_index: usize) -> Option<&[u8]> {
        let height = unsafe {
            crate::ffi::iosurface_get_height_of_plane(self.surface_ptr, plane as isize) as usize
        };
        if row_index >= height {
            return None;
        }
        let base = self.base_address_of_plane(plane)?;
        let bytes_per_row = unsafe {
            crate::ffi::iosurface_get_bytes_per_row_of_plane(self.surface_ptr, plane as isize)
                as usize
        };
        Some(unsafe {
            std::slice::from_raw_parts(base.add(row_index * bytes_per_row), bytes_per_row)
        })
    }
}

impl Drop for IOSurfaceLockGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::iosurface_unlock(self.surface_ptr, self.options.as_u32());
        }
    }
}

impl std::fmt::Debug for IOSurfaceLockGuard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IOSurfaceLockGuard")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bytes_per_row", &self.bytes_per_row)
            .field("options", &self.options)
            .finish_non_exhaustive()
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

/// Properties for a single plane in a multi-planar `IOSurface`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaneProperties {
    /// Width of this plane in pixels
    pub width: usize,
    /// Height of this plane in pixels
    pub height: usize,
    /// Bytes per row for this plane
    pub bytes_per_row: usize,
    /// Bytes per element for this plane
    pub bytes_per_element: usize,
    /// Offset from the start of the surface allocation
    pub offset: usize,
    /// Size of this plane in bytes
    pub size: usize,
}

impl IOSurface {
    /// Create a new `IOSurface` with the given dimensions and pixel format
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `pixel_format` - Pixel format as a `FourCC` code (e.g., 0x42475241 for 'BGRA')
    /// * `bytes_per_element` - Bytes per pixel (e.g., 4 for BGRA)
    ///
    /// # Returns
    ///
    /// `Some(IOSurface)` if creation succeeded, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::output::IOSurface;
    ///
    /// // Create a 100x100 BGRA IOSurface
    /// let surface = IOSurface::create(100, 100, 0x42475241, 4)
    ///     .expect("Failed to create IOSurface");
    /// assert_eq!(surface.width(), 100);
    /// assert_eq!(surface.height(), 100);
    /// ```
    #[must_use]
    pub fn create(
        width: usize,
        height: usize,
        pixel_format: u32,
        bytes_per_element: usize,
    ) -> Option<Self> {
        let mut ptr: *mut c_void = std::ptr::null_mut();
        let status = unsafe {
            crate::ffi::io_surface_create(width, height, pixel_format, bytes_per_element, &mut ptr)
        };
        if status == 0 && !ptr.is_null() {
            Some(Self(ptr.cast_const()))
        } else {
            None
        }
    }

    /// Create an `IOSurface` with full properties including multi-planar support
    ///
    /// This is the general API for creating `IOSurface`s with any pixel format,
    /// including multi-planar formats like YCbCr 4:2:0.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `pixel_format` - Pixel format as `FourCC` (e.g., 0x42475241 for BGRA)
    /// * `bytes_per_element` - Bytes per pixel element
    /// * `bytes_per_row` - Bytes per row (should be 16-byte aligned for Metal)
    /// * `alloc_size` - Total allocation size in bytes
    /// * `planes` - Optional slice of plane info for multi-planar formats
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::output::IOSurface;
    /// use screencapturekit::output::iosurface::PlaneProperties;
    ///
    /// // Create a YCbCr 420v biplanar surface
    /// let width = 1920usize;
    /// let height = 1080usize;
    /// let plane0_bpr = (width + 15) & !15;  // 16-byte aligned
    /// let plane1_bpr = (width + 15) & !15;
    /// let plane0_size = plane0_bpr * height;
    /// let plane1_size = plane1_bpr * (height / 2);
    ///
    /// let planes = [
    ///     PlaneProperties {
    ///         width,
    ///         height,
    ///         bytes_per_row: plane0_bpr,
    ///         bytes_per_element: 1,
    ///         offset: 0,
    ///         size: plane0_size,
    ///     },
    ///     PlaneProperties {
    ///         width: width / 2,
    ///         height: height / 2,
    ///         bytes_per_row: plane1_bpr,
    ///         bytes_per_element: 2,
    ///         offset: plane0_size,
    ///         size: plane1_size,
    ///     },
    /// ];
    ///
    /// let surface = IOSurface::create_with_properties(
    ///     width,
    ///     height,
    ///     0x34323076,  // '420v'
    ///     1,
    ///     plane0_bpr,
    ///     plane0_size + plane1_size,
    ///     Some(&planes),
    /// );
    /// ```
    #[must_use]
    #[allow(clippy::option_if_let_else)]
    pub fn create_with_properties(
        width: usize,
        height: usize,
        pixel_format: u32,
        bytes_per_element: usize,
        bytes_per_row: usize,
        alloc_size: usize,
        planes: Option<&[PlaneProperties]>,
    ) -> Option<Self> {
        let mut ptr: *mut c_void = std::ptr::null_mut();

        let (
            plane_count,
            plane_widths,
            plane_heights,
            plane_row_bytes,
            plane_elem_bytes,
            plane_offsets,
            plane_sizes,
        ) = if let Some(p) = planes {
            let widths: Vec<usize> = p.iter().map(|x| x.width).collect();
            let heights: Vec<usize> = p.iter().map(|x| x.height).collect();
            let row_bytes: Vec<usize> = p.iter().map(|x| x.bytes_per_row).collect();
            let elem_bytes: Vec<usize> = p.iter().map(|x| x.bytes_per_element).collect();
            let offsets: Vec<usize> = p.iter().map(|x| x.offset).collect();
            let sizes: Vec<usize> = p.iter().map(|x| x.size).collect();
            (
                p.len(),
                widths,
                heights,
                row_bytes,
                elem_bytes,
                offsets,
                sizes,
            )
        } else {
            (0, vec![], vec![], vec![], vec![], vec![], vec![])
        };

        let status = unsafe {
            crate::ffi::io_surface_create_with_properties(
                width,
                height,
                pixel_format,
                bytes_per_element,
                bytes_per_row,
                alloc_size,
                plane_count,
                if plane_count > 0 {
                    plane_widths.as_ptr()
                } else {
                    std::ptr::null()
                },
                if plane_count > 0 {
                    plane_heights.as_ptr()
                } else {
                    std::ptr::null()
                },
                if plane_count > 0 {
                    plane_row_bytes.as_ptr()
                } else {
                    std::ptr::null()
                },
                if plane_count > 0 {
                    plane_elem_bytes.as_ptr()
                } else {
                    std::ptr::null()
                },
                if plane_count > 0 {
                    plane_offsets.as_ptr()
                } else {
                    std::ptr::null()
                },
                if plane_count > 0 {
                    plane_sizes.as_ptr()
                } else {
                    std::ptr::null()
                },
                &mut ptr,
            )
        };

        if status == 0 && !ptr.is_null() {
            Some(Self(ptr.cast_const()))
        } else {
            None
        }
    }

    /// Create from raw pointer (used internally)
    pub(crate) unsafe fn from_ptr(ptr: *const c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// Get the raw `IOSurface` pointer for use with Metal or other frameworks
    ///
    /// # Safety
    /// The returned pointer is only valid for the lifetime of this `IOSurface`.
    #[must_use]
    pub fn as_ptr(&self) -> *const c_void {
        self.0
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

    /// Get the number of planes in the surface
    ///
    /// Multi-planar formats like YCbCr 420 have multiple planes:
    /// - Plane 0: Y (luminance)
    /// - Plane 1: `CbCr` (chrominance)
    ///
    /// Single-plane formats like BGRA return 0.
    pub fn plane_count(&self) -> usize {
        #[allow(clippy::cast_sign_loss)]
        unsafe {
            crate::ffi::iosurface_get_plane_count(self.0) as usize
        }
    }

    /// Get the width of a specific plane in pixels
    ///
    /// For YCbCr 4:2:0 formats, plane 1 (`CbCr`) is half the width of plane 0 (Y).
    pub fn width_of_plane(&self, plane: usize) -> usize {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::iosurface_get_width_of_plane(self.0, plane as isize) as usize
        }
    }

    /// Get the height of a specific plane in pixels
    ///
    /// For YCbCr 4:2:0 formats, plane 1 (`CbCr`) is half the height of plane 0 (Y).
    pub fn height_of_plane(&self, plane: usize) -> usize {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::iosurface_get_height_of_plane(self.0, plane as isize) as usize
        }
    }

    /// Get the bytes per row of a specific plane
    pub fn bytes_per_row_of_plane(&self, plane: usize) -> usize {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::iosurface_get_bytes_per_row_of_plane(self.0, plane as isize) as usize
        }
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
