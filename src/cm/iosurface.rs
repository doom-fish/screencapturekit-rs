//! `IOSurface` - Hardware-accelerated surface
//!
//! Provides safe Rust bindings for Apple's `IOSurface` framework.
//! `IOSurface` objects are framebuffers suitable for sharing across process boundaries
//! and are the primary mechanism for zero-copy frame delivery in `ScreenCaptureKit`.
//!
//! # Safety
//!
//! Base address access is only available through lock guards to ensure proper
//! memory synchronization. The surface must be locked before accessing pixel data.

use super::ffi;
use std::ffi::c_void;
use std::fmt;
use std::io;

/// Lock options for `IOSurface`
///
/// This is a bitmask type that supports combining multiple options using the `|` operator.
///
/// # Examples
///
/// ```
/// use screencapturekit::cm::IOSurfaceLockOptions;
///
/// // Single option
/// let read_only = IOSurfaceLockOptions::READ_ONLY;
///
/// // Combined options
/// let combined = IOSurfaceLockOptions::READ_ONLY | IOSurfaceLockOptions::AVOID_SYNC;
/// assert!(combined.contains(IOSurfaceLockOptions::READ_ONLY));
/// assert!(combined.contains(IOSurfaceLockOptions::AVOID_SYNC));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IOSurfaceLockOptions(u32);

impl IOSurfaceLockOptions {
    /// No special options (read-write lock with sync)
    pub const NONE: Self = Self(0);

    /// Read-only lock - use when you only need to read data.
    /// This allows the system to keep caches valid.
    pub const READ_ONLY: Self = Self(0x0000_0001);

    /// Avoid synchronization - use with caution.
    /// Skip waiting for pending operations before completing the lock.
    pub const AVOID_SYNC: Self = Self(0x0000_0002);

    /// Create from a raw u32 value
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Convert to u32 for FFI
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Check if these options contain the given option
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check if this is a read-only lock
    #[must_use]
    pub const fn is_read_only(self) -> bool {
        self.contains(Self::READ_ONLY)
    }

    /// Check if this avoids synchronization
    #[must_use]
    pub const fn is_avoid_sync(self) -> bool {
        self.contains(Self::AVOID_SYNC)
    }

    /// Check if no options are set
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for IOSurfaceLockOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for IOSurfaceLockOptions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for IOSurfaceLockOptions {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for IOSurfaceLockOptions {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl From<IOSurfaceLockOptions> for u32 {
    fn from(options: IOSurfaceLockOptions) -> Self {
        options.0
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

/// Hardware-accelerated surface for efficient frame delivery
///
/// `IOSurface` is Apple's cross-process framebuffer type. It provides:
/// - Zero-copy sharing between processes
/// - Direct GPU texture creation via Metal
/// - Multi-planar format support (YCbCr, etc.)
///
/// # Memory Access Safety
///
/// The surface must be locked before accessing pixel data. Use [`lock`](Self::lock)
/// to get a RAII guard that ensures proper locking/unlocking.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::cm::{IOSurface, IOSurfaceLockOptions};
///
/// fn access_surface(surface: &IOSurface) -> Result<(), i32> {
///     // Lock for read-only access
///     let guard = surface.lock(IOSurfaceLockOptions::READ_ONLY)?;
///     
///     // Access pixel data through the guard
///     let data = guard.as_slice();
///     println!("Surface has {} bytes", data.len());
///     
///     // Surface automatically unlocked when guard drops
///     Ok(())
/// }
/// ```
pub struct IOSurface(*mut c_void);

impl PartialEq for IOSurface {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for IOSurface {}

impl std::hash::Hash for IOSurface {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::io_surface_hash(self.0);
            hash_value.hash(state);
        }
    }
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
    /// use screencapturekit::cm::IOSurface;
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
            Some(Self(ptr))
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
    /// use screencapturekit::cm::iosurface::PlaneProperties;
    /// use screencapturekit::cm::IOSurface;
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
            Some(Self(ptr))
        } else {
            None
        }
    }

    /// Create from raw pointer
    pub fn from_raw(ptr: *mut c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `IOSurface` pointer.
    pub unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut c_void {
        self.0
    }

    /// Get the width of the surface in pixels
    pub fn width(&self) -> usize {
        unsafe { ffi::io_surface_get_width(self.0) }
    }

    /// Get the height of the surface in pixels
    pub fn height(&self) -> usize {
        unsafe { ffi::io_surface_get_height(self.0) }
    }

    /// Get the bytes per row of the surface
    pub fn bytes_per_row(&self) -> usize {
        unsafe { ffi::io_surface_get_bytes_per_row(self.0) }
    }

    /// Get the total allocation size of the surface in bytes
    pub fn alloc_size(&self) -> usize {
        unsafe { ffi::io_surface_get_alloc_size(self.0) }
    }

    /// Get the data size of the surface in bytes (alias for `alloc_size`)
    ///
    /// This method provides API parity with `CVPixelBuffer::data_size()`.
    pub fn data_size(&self) -> usize {
        self.alloc_size()
    }

    /// Get the pixel format of the surface (OSType/FourCC)
    pub fn pixel_format(&self) -> u32 {
        unsafe { ffi::io_surface_get_pixel_format(self.0) }
    }

    /// Get the unique `IOSurfaceID` for this surface
    pub fn id(&self) -> u32 {
        unsafe { ffi::io_surface_get_id(self.0) }
    }

    /// Get the modification seed value
    ///
    /// This value changes each time the surface is modified, useful for
    /// detecting whether the surface contents have changed.
    pub fn seed(&self) -> u32 {
        unsafe { ffi::io_surface_get_seed(self.0) }
    }

    /// Get the number of planes in this surface
    ///
    /// Multi-planar formats like YCbCr 420 have multiple planes:
    /// - Plane 0: Y (luminance)
    /// - Plane 1: `CbCr` (chrominance)
    ///
    /// Single-plane formats like BGRA return 0.
    pub fn plane_count(&self) -> usize {
        unsafe { ffi::io_surface_get_plane_count(self.0) }
    }

    /// Get the width of a specific plane
    ///
    /// For YCbCr 4:2:0 formats, plane 1 (`CbCr`) is half the width of plane 0 (Y).
    pub fn width_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::io_surface_get_width_of_plane(self.0, plane_index) }
    }

    /// Get the height of a specific plane
    ///
    /// For YCbCr 4:2:0 formats, plane 1 (`CbCr`) is half the height of plane 0 (Y).
    pub fn height_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::io_surface_get_height_of_plane(self.0, plane_index) }
    }

    /// Get the bytes per row of a specific plane
    pub fn bytes_per_row_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::io_surface_get_bytes_per_row_of_plane(self.0, plane_index) }
    }

    /// Get the bytes per element of the surface
    pub fn bytes_per_element(&self) -> usize {
        unsafe { ffi::io_surface_get_bytes_per_element(self.0) }
    }

    /// Get the element width of the surface
    pub fn element_width(&self) -> usize {
        unsafe { ffi::io_surface_get_element_width(self.0) }
    }

    /// Get the element height of the surface
    pub fn element_height(&self) -> usize {
        unsafe { ffi::io_surface_get_element_height(self.0) }
    }

    /// Check if the surface is currently in use
    pub fn is_in_use(&self) -> bool {
        unsafe { ffi::io_surface_is_in_use(self.0) }
    }

    /// Increment the use count of the surface
    pub fn increment_use_count(&self) {
        unsafe { ffi::io_surface_increment_use_count(self.0) }
    }

    /// Decrement the use count of the surface
    pub fn decrement_use_count(&self) {
        unsafe { ffi::io_surface_decrement_use_count(self.0) }
    }

    /// Get the base address (internal use only)
    ///
    /// # Safety
    /// Caller must ensure the surface is locked before accessing the returned pointer.
    pub(crate) fn base_address_raw(&self) -> *mut u8 {
        unsafe { ffi::io_surface_get_base_address(self.0).cast::<u8>() }
    }

    /// Get the base address of a specific plane (internal use only)
    ///
    /// # Safety
    /// Caller must ensure the surface is locked before accessing the returned pointer.
    pub(crate) fn base_address_of_plane_raw(&self, plane_index: usize) -> Option<*mut u8> {
        let plane_count = self.plane_count();
        if plane_count == 0 || plane_index >= plane_count {
            return None;
        }
        let ptr = unsafe { ffi::io_surface_get_base_address_of_plane(self.0, plane_index) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr.cast::<u8>())
        }
    }

    /// Lock the surface for CPU access (low-level API)
    ///
    /// Prefer using [`lock`](Self::lock) for RAII-style access.
    ///
    /// # Arguments
    /// * `options` - Lock options (e.g., `IOSurfaceLockOptions::READ_ONLY`)
    ///
    /// # Errors
    /// Returns `kern_return_t` error code if the lock fails.
    pub fn lock_raw(&self, options: IOSurfaceLockOptions) -> Result<u32, i32> {
        let mut seed: u32 = 0;
        let status = unsafe { ffi::io_surface_lock(self.0, options.as_u32(), &mut seed) };
        if status == 0 {
            Ok(seed)
        } else {
            Err(status)
        }
    }

    /// Unlock the surface after CPU access (low-level API)
    ///
    /// # Arguments
    /// * `options` - Must match the options used in the corresponding `lock_raw()` call
    ///
    /// # Errors
    /// Returns `kern_return_t` error code if the unlock fails.
    pub fn unlock_raw(&self, options: IOSurfaceLockOptions) -> Result<u32, i32> {
        let mut seed: u32 = 0;
        let status = unsafe { ffi::io_surface_unlock(self.0, options.as_u32(), &mut seed) };
        if status == 0 {
            Ok(seed)
        } else {
            Err(status)
        }
    }

    /// Lock the surface and return a guard for RAII-style access
    ///
    /// This is the recommended way to access surface memory. The guard ensures
    /// the surface is properly unlocked when it goes out of scope.
    ///
    /// # Arguments
    /// * `options` - Lock options (e.g., `IOSurfaceLockOptions::READ_ONLY`)
    ///
    /// # Errors
    /// Returns `kern_return_t` error code if the lock fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::{IOSurface, IOSurfaceLockOptions};
    ///
    /// fn read_surface(surface: &IOSurface) -> Result<(), i32> {
    ///     let guard = surface.lock(IOSurfaceLockOptions::READ_ONLY)?;
    ///     let data = guard.as_slice();
    ///     println!("Read {} bytes", data.len());
    ///     Ok(())
    /// }
    /// ```
    pub fn lock(&self, options: IOSurfaceLockOptions) -> Result<IOSurfaceLockGuard<'_>, i32> {
        self.lock_raw(options)?;
        Ok(IOSurfaceLockGuard {
            surface: self,
            options,
        })
    }

    /// Lock the surface for read-only access
    ///
    /// This is a convenience method equivalent to `lock(IOSurfaceLockOptions::READ_ONLY)`.
    ///
    /// # Errors
    /// Returns `kern_return_t` error code if the lock fails.
    pub fn lock_read_only(&self) -> Result<IOSurfaceLockGuard<'_>, i32> {
        self.lock(IOSurfaceLockOptions::READ_ONLY)
    }

    /// Lock the surface for read-write access
    ///
    /// This is a convenience method equivalent to `lock(IOSurfaceLockOptions::NONE)`.
    ///
    /// # Errors
    /// Returns `kern_return_t` error code if the lock fails.
    pub fn lock_read_write(&self) -> Result<IOSurfaceLockGuard<'_>, i32> {
        self.lock(IOSurfaceLockOptions::NONE)
    }
}

/// RAII guard for locked `IOSurface`
///
/// Provides safe access to surface memory while the lock is held.
/// The surface is automatically unlocked when this guard is dropped.
///
/// # Memory Access
///
/// All base address access is through this guard to ensure proper locking.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::cm::{IOSurface, IOSurfaceLockOptions};
///
/// fn access_surface(surface: &IOSurface) -> Result<(), i32> {
///     let guard = surface.lock(IOSurfaceLockOptions::READ_ONLY)?;
///     
///     // Access the entire buffer
///     let data = guard.as_slice();
///     
///     // Access a specific row
///     if let Some(row) = guard.row(0) {
///         println!("First row: {} bytes", row.len());
///     }
///     
///     // Access a specific plane (for multi-planar formats)
///     if let Some(plane_data) = guard.plane_data(0) {
///         println!("Plane 0: {} bytes", plane_data.len());
///     }
///     
///     Ok(())
/// }
/// ```
pub struct IOSurfaceLockGuard<'a> {
    surface: &'a IOSurface,
    options: IOSurfaceLockOptions,
}

impl IOSurfaceLockGuard<'_> {
    /// Get the width of the surface in pixels
    pub fn width(&self) -> usize {
        self.surface.width()
    }

    /// Get the height of the surface in pixels
    pub fn height(&self) -> usize {
        self.surface.height()
    }

    /// Get the bytes per row of the surface
    pub fn bytes_per_row(&self) -> usize {
        self.surface.bytes_per_row()
    }

    /// Get the total allocation size in bytes
    pub fn alloc_size(&self) -> usize {
        self.surface.alloc_size()
    }

    /// Get the data size of the surface (alias for `alloc_size`)
    pub fn data_size(&self) -> usize {
        self.alloc_size()
    }

    /// Get the pixel format of the surface
    pub fn pixel_format(&self) -> u32 {
        self.surface.pixel_format()
    }

    /// Get the number of planes in the surface
    pub fn plane_count(&self) -> usize {
        self.surface.plane_count()
    }

    /// Get the base address of the locked surface
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid while this guard is held.
    pub fn base_address(&self) -> *const u8 {
        self.surface.base_address_raw().cast_const()
    }

    /// Get the mutable base address (only valid for read-write locks)
    ///
    /// Returns `None` if this is a read-only lock.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid while this guard is held.
    pub fn base_address_mut(&mut self) -> Option<*mut u8> {
        if self.options.is_read_only() {
            None
        } else {
            Some(self.surface.base_address_raw())
        }
    }

    /// Get the base address of a specific plane
    ///
    /// For multi-planar formats like YCbCr 4:2:0:
    /// - Plane 0: Y (luminance) data
    /// - Plane 1: `CbCr` (chrominance) data
    ///
    /// Returns `None` if the plane index is out of bounds.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid while this guard is held.
    pub fn base_address_of_plane(&self, plane_index: usize) -> Option<*const u8> {
        self.surface
            .base_address_of_plane_raw(plane_index)
            .map(<*mut u8>::cast_const)
    }

    /// Get the mutable base address of a specific plane
    ///
    /// Returns `None` if this is a read-only lock or the plane index is out of bounds.
    pub fn base_address_of_plane_mut(&mut self, plane_index: usize) -> Option<*mut u8> {
        if self.options.is_read_only() {
            return None;
        }
        self.surface.base_address_of_plane_raw(plane_index)
    }

    /// Get a slice view of the surface data
    ///
    /// The lock guard ensures the surface is locked for the lifetime of the slice.
    pub fn as_slice(&self) -> &[u8] {
        let ptr = self.base_address();
        let len = self.alloc_size();
        if ptr.is_null() || len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(ptr, len) }
        }
    }

    /// Get a mutable slice view of the surface data (only valid for read-write locks)
    ///
    /// Returns `None` if this is a read-only lock.
    pub fn as_slice_mut(&mut self) -> Option<&mut [u8]> {
        if self.options.is_read_only() {
            return None;
        }
        let ptr = self.base_address_mut()?;
        let len = self.alloc_size();
        if ptr.is_null() || len == 0 {
            Some(&mut [])
        } else {
            Some(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
        }
    }

    /// Get a specific row as a slice
    ///
    /// Returns `None` if the row index is out of bounds.
    pub fn row(&self, row_index: usize) -> Option<&[u8]> {
        if row_index >= self.height() {
            return None;
        }
        let ptr = self.base_address();
        if ptr.is_null() {
            return None;
        }
        unsafe {
            let row_ptr = ptr.add(row_index * self.bytes_per_row());
            Some(std::slice::from_raw_parts(row_ptr, self.bytes_per_row()))
        }
    }

    /// Get a slice of plane data
    ///
    /// Returns the data for a specific plane as a byte slice. The slice size is
    /// calculated from the plane's height and bytes per row.
    ///
    /// Returns `None` if the plane index is out of bounds.
    pub fn plane_data(&self, plane_index: usize) -> Option<&[u8]> {
        let base = self.base_address_of_plane(plane_index)?;
        let height = self.surface.height_of_plane(plane_index);
        let bytes_per_row = self.surface.bytes_per_row_of_plane(plane_index);
        Some(unsafe { std::slice::from_raw_parts(base, height * bytes_per_row) })
    }

    /// Get a specific row from a plane as a slice
    ///
    /// Returns `None` if the plane or row index is out of bounds.
    pub fn plane_row(&self, plane_index: usize, row_index: usize) -> Option<&[u8]> {
        let height = self.surface.height_of_plane(plane_index);
        if row_index >= height {
            return None;
        }
        let base = self.base_address_of_plane(plane_index)?;
        let bytes_per_row = self.surface.bytes_per_row_of_plane(plane_index);
        Some(unsafe {
            std::slice::from_raw_parts(base.add(row_index * bytes_per_row), bytes_per_row)
        })
    }

    /// Access surface with a standard `std::io::Cursor`
    ///
    /// Returns a cursor over the surface data that implements `Read` and `Seek`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{Read, Seek, SeekFrom};
    /// use screencapturekit::cm::{IOSurface, IOSurfaceLockOptions};
    ///
    /// fn read_surface(surface: &IOSurface) {
    ///     let guard = surface.lock(IOSurfaceLockOptions::READ_ONLY).unwrap();
    ///     let mut cursor = guard.cursor();
    ///
    ///     // Read first 4 bytes
    ///     let mut pixel = [0u8; 4];
    ///     cursor.read_exact(&mut pixel).unwrap();
    ///
    ///     // Seek to row 10
    ///     let offset = 10 * guard.bytes_per_row();
    ///     cursor.seek(SeekFrom::Start(offset as u64)).unwrap();
    /// }
    /// ```
    pub fn cursor(&self) -> io::Cursor<&[u8]> {
        io::Cursor::new(self.as_slice())
    }

    /// Get raw pointer to surface data
    pub fn as_ptr(&self) -> *const u8 {
        self.base_address()
    }

    /// Get mutable raw pointer to surface data (only valid for read-write locks)
    ///
    /// Returns `None` if this is a read-only lock.
    pub fn as_mut_ptr(&mut self) -> Option<*mut u8> {
        self.base_address_mut()
    }

    /// Check if this is a read-only lock
    pub const fn is_read_only(&self) -> bool {
        self.options.is_read_only()
    }

    /// Get the lock options
    pub const fn options(&self) -> IOSurfaceLockOptions {
        self.options
    }
}

impl std::ops::Deref for IOSurfaceLockGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Drop for IOSurfaceLockGuard<'_> {
    fn drop(&mut self) {
        let _ = self.surface.unlock_raw(self.options);
    }
}

impl std::fmt::Debug for IOSurfaceLockGuard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IOSurfaceLockGuard")
            .field("options", &self.options)
            .field(
                "surface_size",
                &(self.surface.width(), self.surface.height()),
            )
            .finish()
    }
}

impl Drop for IOSurface {
    fn drop(&mut self) {
        unsafe {
            ffi::io_surface_release(self.0);
        }
    }
}

impl Clone for IOSurface {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::io_surface_retain(self.0);
            Self(ptr)
        }
    }
}

unsafe impl Send for IOSurface {}
unsafe impl Sync for IOSurface {}

impl fmt::Debug for IOSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IOSurface")
            .field("id", &self.id())
            .field("width", &self.width())
            .field("height", &self.height())
            .field("bytes_per_row", &self.bytes_per_row())
            .field("pixel_format", &self.pixel_format())
            .field("plane_count", &self.plane_count())
            .finish()
    }
}

impl fmt::Display for IOSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IOSurface({}x{}, {} bytes/row)",
            self.width(),
            self.height(),
            self.bytes_per_row()
        )
    }
}
