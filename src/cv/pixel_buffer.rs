//! `CVPixelBuffer` - Video pixel buffer

use super::ffi;
use crate::cm::IOSurface;
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom};

/// Lock flags for `CVPixelBuffer`
///
/// This is a bitmask type matching Apple's `CVPixelBufferLockFlags`.
///
/// # Examples
///
/// ```
/// use screencapturekit::cv::CVPixelBufferLockFlags;
///
/// // Read-only lock
/// let flags = CVPixelBufferLockFlags::READ_ONLY;
/// assert!(flags.is_read_only());
///
/// // Read-write lock (default)
/// let flags = CVPixelBufferLockFlags::NONE;
/// assert!(!flags.is_read_only());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CVPixelBufferLockFlags(u32);

impl CVPixelBufferLockFlags {
    /// No special options (read-write lock)
    pub const NONE: Self = Self(0);

    /// Read-only lock - use when you only need to read data.
    /// This allows Core Video to keep caches valid.
    pub const READ_ONLY: Self = Self(0x0000_0001);

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

    /// Check if this is a read-only lock
    #[must_use]
    pub const fn is_read_only(self) -> bool {
        (self.0 & Self::READ_ONLY.0) != 0
    }

    /// Check if no flags are set (read-write lock)
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl From<CVPixelBufferLockFlags> for u32 {
    fn from(flags: CVPixelBufferLockFlags) -> Self {
        flags.0
    }
}

#[derive(Debug)]
pub struct CVPixelBuffer(*mut std::ffi::c_void);

impl PartialEq for CVPixelBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CVPixelBuffer {}

impl std::hash::Hash for CVPixelBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cv_pixel_buffer_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl CVPixelBuffer {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CVPixelBuffer` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Create a new pixel buffer with the specified dimensions and pixel format
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the pixel buffer in pixels
    /// * `height` - Height of the pixel buffer in pixels
    /// * `pixel_format` - Pixel format type (e.g., 0x42475241 for BGRA)
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the pixel buffer creation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cv::CVPixelBuffer;
    ///
    /// // Create a 1920x1080 BGRA pixel buffer
    /// let buffer = CVPixelBuffer::create(1920, 1080, 0x42475241)
    ///     .expect("Failed to create pixel buffer");
    ///
    /// assert_eq!(buffer.width(), 1920);
    /// assert_eq!(buffer.height(), 1080);
    /// assert_eq!(buffer.pixel_format(), 0x42475241);
    /// ```
    pub fn create(width: usize, height: usize, pixel_format: u32) -> Result<Self, i32> {
        unsafe {
            let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status =
                ffi::cv_pixel_buffer_create(width, height, pixel_format, &mut pixel_buffer_ptr);

            if status == 0 && !pixel_buffer_ptr.is_null() {
                Ok(Self(pixel_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Create a pixel buffer from existing memory
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the pixel buffer in pixels
    /// * `height` - Height of the pixel buffer in pixels
    /// * `pixel_format` - Pixel format type (e.g., 0x42475241 for BGRA)
    /// * `base_address` - Pointer to pixel data
    /// * `bytes_per_row` - Number of bytes per row
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `base_address` points to valid memory
    /// - Memory remains valid for the lifetime of the pixel buffer
    /// - `bytes_per_row` correctly represents the memory layout
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the pixel buffer creation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cv::CVPixelBuffer;
    ///
    /// // Create pixel data (100x100 BGRA image)
    /// let width = 100;
    /// let height = 100;
    /// let bytes_per_pixel = 4; // BGRA
    /// let bytes_per_row = width * bytes_per_pixel;
    /// let mut pixel_data = vec![0u8; width * height * bytes_per_pixel];
    ///
    /// // Fill with blue color
    /// for y in 0..height {
    ///     for x in 0..width {
    ///         let offset = y * bytes_per_row + x * bytes_per_pixel;
    ///         pixel_data[offset] = 255;     // B
    ///         pixel_data[offset + 1] = 0;   // G
    ///         pixel_data[offset + 2] = 0;   // R
    ///         pixel_data[offset + 3] = 255; // A
    ///     }
    /// }
    ///
    /// // Create pixel buffer from the data
    /// let buffer = unsafe {
    ///     CVPixelBuffer::create_with_bytes(
    ///         width,
    ///         height,
    ///         0x42475241, // BGRA
    ///         pixel_data.as_mut_ptr() as *mut std::ffi::c_void,
    ///         bytes_per_row,
    ///     )
    /// }.expect("Failed to create pixel buffer");
    ///
    /// assert_eq!(buffer.width(), width);
    /// assert_eq!(buffer.height(), height);
    /// ```
    pub unsafe fn create_with_bytes(
        width: usize,
        height: usize,
        pixel_format: u32,
        base_address: *mut std::ffi::c_void,
        bytes_per_row: usize,
    ) -> Result<Self, i32> {
        let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = ffi::cv_pixel_buffer_create_with_bytes(
            width,
            height,
            pixel_format,
            base_address,
            bytes_per_row,
            &mut pixel_buffer_ptr,
        );

        if status == 0 && !pixel_buffer_ptr.is_null() {
            Ok(Self(pixel_buffer_ptr))
        } else {
            Err(status)
        }
    }

    /// Fill the extended pixels of a pixel buffer
    ///
    /// This is useful for pixel buffers that have been created with extended pixels
    /// enabled, to ensure proper edge handling for effects and filters.
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the operation fails.
    pub fn fill_extended_pixels(&self) -> Result<(), i32> {
        unsafe {
            let status = ffi::cv_pixel_buffer_fill_extended_pixels(self.0);
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Create a pixel buffer with planar bytes
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `plane_base_addresses` points to valid memory for each plane
    /// - Memory remains valid for the lifetime of the pixel buffer
    /// - All plane parameters correctly represent the memory layout
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the pixel buffer creation fails.
    pub unsafe fn create_with_planar_bytes(
        width: usize,
        height: usize,
        pixel_format: u32,
        plane_base_addresses: &[*mut std::ffi::c_void],
        plane_widths: &[usize],
        plane_heights: &[usize],
        plane_bytes_per_row: &[usize],
    ) -> Result<Self, i32> {
        if plane_base_addresses.len() != plane_widths.len()
            || plane_widths.len() != plane_heights.len()
            || plane_heights.len() != plane_bytes_per_row.len()
        {
            return Err(-50); // paramErr
        }

        let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = ffi::cv_pixel_buffer_create_with_planar_bytes(
            width,
            height,
            pixel_format,
            plane_base_addresses.len(),
            plane_base_addresses.as_ptr(),
            plane_widths.as_ptr(),
            plane_heights.as_ptr(),
            plane_bytes_per_row.as_ptr(),
            &mut pixel_buffer_ptr,
        );

        if status == 0 && !pixel_buffer_ptr.is_null() {
            Ok(Self(pixel_buffer_ptr))
        } else {
            Err(status)
        }
    }

    /// Create a pixel buffer from an `IOSurface`
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the pixel buffer creation fails.
    pub fn create_with_io_surface(surface: &IOSurface) -> Result<Self, i32> {
        unsafe {
            let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cv_pixel_buffer_create_with_io_surface(
                surface.as_ptr(),
                &mut pixel_buffer_ptr,
            );

            if status == 0 && !pixel_buffer_ptr.is_null() {
                Ok(Self(pixel_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Get the Core Foundation type ID for `CVPixelBuffer`
    pub fn type_id() -> usize {
        unsafe { ffi::cv_pixel_buffer_get_type_id() }
    }

    /// Get the data size of the pixel buffer
    pub fn data_size(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_data_size(self.0) }
    }

    /// Check if the pixel buffer is planar
    pub fn is_planar(&self) -> bool {
        unsafe { ffi::cv_pixel_buffer_is_planar(self.0) }
    }

    /// Get the number of planes in the pixel buffer
    pub fn plane_count(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_plane_count(self.0) }
    }

    /// Get the width of a specific plane
    pub fn width_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_width_of_plane(self.0, plane_index) }
    }

    /// Get the height of a specific plane
    pub fn height_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_height_of_plane(self.0, plane_index) }
    }

    /// Get the base address of a specific plane (internal use only)
    ///
    /// # Safety
    /// Caller must ensure the buffer is locked before accessing the returned pointer.
    fn base_address_of_plane_raw(&self, plane_index: usize) -> Option<*mut u8> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_get_base_address_of_plane(self.0, plane_index);
            if ptr.is_null() {
                None
            } else {
                Some(ptr.cast::<u8>())
            }
        }
    }

    /// Get the bytes per row of a specific plane
    pub fn bytes_per_row_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_bytes_per_row_of_plane(self.0, plane_index) }
    }

    /// Get the extended pixel information (left, right, top, bottom)
    pub fn extended_pixels(&self) -> (usize, usize, usize, usize) {
        unsafe {
            let mut left: usize = 0;
            let mut right: usize = 0;
            let mut top: usize = 0;
            let mut bottom: usize = 0;
            ffi::cv_pixel_buffer_get_extended_pixels(
                self.0,
                &mut left,
                &mut right,
                &mut top,
                &mut bottom,
            );
            (left, right, top, bottom)
        }
    }

    /// Check if the pixel buffer is backed by an `IOSurface`
    pub fn is_backed_by_io_surface(&self) -> bool {
        self.io_surface().is_some()
    }

    /// Get the width of the pixel buffer in pixels
    pub fn width(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_width(self.0) }
    }

    pub fn height(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_height(self.0) }
    }

    pub fn pixel_format(&self) -> u32 {
        unsafe { ffi::cv_pixel_buffer_get_pixel_format_type(self.0) }
    }

    pub fn bytes_per_row(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_bytes_per_row(self.0) }
    }

    /// Lock the base address for raw access
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the lock operation fails.
    pub fn lock_raw(&self, flags: CVPixelBufferLockFlags) -> Result<(), i32> {
        unsafe {
            let result = ffi::cv_pixel_buffer_lock_base_address(self.0, flags.as_u32());
            if result == 0 {
                Ok(())
            } else {
                Err(result)
            }
        }
    }

    /// Unlock the base address after raw access
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the unlock operation fails.
    pub fn unlock_raw(&self, flags: CVPixelBufferLockFlags) -> Result<(), i32> {
        unsafe {
            let result = ffi::cv_pixel_buffer_unlock_base_address(self.0, flags.as_u32());
            if result == 0 {
                Ok(())
            } else {
                Err(result)
            }
        }
    }

    /// Get the base address (internal use only)
    ///
    /// # Safety
    /// Caller must ensure the buffer is locked before accessing the returned pointer.
    fn base_address_raw(&self) -> Option<*mut u8> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_get_base_address(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(ptr.cast::<u8>())
            }
        }
    }

    /// Get the `IOSurface` backing this pixel buffer
    pub fn io_surface(&self) -> Option<IOSurface> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_get_io_surface(self.0);
            IOSurface::from_raw(ptr)
        }
    }

    /// Lock the base address and return a guard for RAII-style access
    ///
    /// # Arguments
    ///
    /// * `flags` - Lock flags (use `CVPixelBufferLockFlags::READ_ONLY` for read-only access)
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the lock operation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cv::{CVPixelBuffer, CVPixelBufferLockFlags};
    ///
    /// fn read_buffer(buffer: &CVPixelBuffer) {
    ///     let guard = buffer.lock(CVPixelBufferLockFlags::READ_ONLY).unwrap();
    ///     let data = guard.as_slice();
    ///     println!("Buffer has {} bytes", data.len());
    ///     // Buffer automatically unlocked when guard drops
    /// }
    /// ```
    pub fn lock(&self, flags: CVPixelBufferLockFlags) -> Result<CVPixelBufferLockGuard<'_>, i32> {
        self.lock_raw(flags)?;
        Ok(CVPixelBufferLockGuard {
            buffer: self,
            flags,
        })
    }

    /// Lock the base address for read-only access
    ///
    /// This is a convenience method equivalent to `lock(CVPixelBufferLockFlags::READ_ONLY)`.
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the lock operation fails.
    pub fn lock_read_only(&self) -> Result<CVPixelBufferLockGuard<'_>, i32> {
        self.lock(CVPixelBufferLockFlags::READ_ONLY)
    }

    /// Lock the base address for read-write access
    ///
    /// This is a convenience method equivalent to `lock(CVPixelBufferLockFlags::NONE)`.
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the lock operation fails.
    pub fn lock_read_write(&self) -> Result<CVPixelBufferLockGuard<'_>, i32> {
        self.lock(CVPixelBufferLockFlags::NONE)
    }
}

/// RAII guard for locked `CVPixelBuffer` base address
pub struct CVPixelBufferLockGuard<'a> {
    buffer: &'a CVPixelBuffer,
    flags: CVPixelBufferLockFlags,
}

impl CVPixelBufferLockGuard<'_> {
    /// Get the base address of the locked buffer
    pub fn base_address(&self) -> *const u8 {
        self.buffer
            .base_address_raw()
            .unwrap_or(std::ptr::null_mut())
            .cast_const()
    }

    /// Get mutable base address (only valid for read-write locks)
    ///
    /// Returns `None` if this is a read-only lock.
    pub fn base_address_mut(&mut self) -> Option<*mut u8> {
        if self.flags.is_read_only() {
            None
        } else {
            self.buffer.base_address_raw()
        }
    }

    /// Get the base address of a specific plane
    ///
    /// For multi-planar formats like YCbCr 4:2:0:
    /// - Plane 0: Y (luminance) data
    /// - Plane 1: `CbCr` (chrominance) data
    ///
    /// Returns `None` if the plane index is out of bounds.
    pub fn base_address_of_plane(&self, plane_index: usize) -> Option<*const u8> {
        self.buffer
            .base_address_of_plane_raw(plane_index)
            .map(<*mut u8>::cast_const)
    }

    /// Get the mutable base address of a specific plane
    ///
    /// Returns `None` if this is a read-only lock or the plane index is out of bounds.
    pub fn base_address_of_plane_mut(&mut self, plane_index: usize) -> Option<*mut u8> {
        if self.flags.is_read_only() {
            return None;
        }
        self.buffer.base_address_of_plane_raw(plane_index)
    }

    /// Get the width of the buffer
    pub fn width(&self) -> usize {
        self.buffer.width()
    }

    /// Get the height of the buffer
    pub fn height(&self) -> usize {
        self.buffer.height()
    }

    /// Get bytes per row
    pub fn bytes_per_row(&self) -> usize {
        self.buffer.bytes_per_row()
    }

    /// Get the data size in bytes
    ///
    /// This provides API parity with `IOSurfaceLockGuard::data_size()`.
    pub fn data_size(&self) -> usize {
        self.buffer.data_size()
    }

    /// Get the number of planes
    pub fn plane_count(&self) -> usize {
        self.buffer.plane_count()
    }

    /// Get the width of a specific plane
    pub fn width_of_plane(&self, plane_index: usize) -> usize {
        self.buffer.width_of_plane(plane_index)
    }

    /// Get the height of a specific plane
    pub fn height_of_plane(&self, plane_index: usize) -> usize {
        self.buffer.height_of_plane(plane_index)
    }

    /// Get the bytes per row of a specific plane
    pub fn bytes_per_row_of_plane(&self, plane_index: usize) -> usize {
        self.buffer.bytes_per_row_of_plane(plane_index)
    }

    /// Get data as a byte slice
    ///
    /// The lock guard ensures the buffer is locked for the lifetime of the slice.
    pub fn as_slice(&self) -> &[u8] {
        let ptr = self.base_address();
        let len = self.buffer.height() * self.buffer.bytes_per_row();
        if ptr.is_null() || len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(ptr, len) }
        }
    }

    /// Get data as a mutable byte slice (only valid for read-write locks)
    ///
    /// Returns `None` if this is a read-only lock.
    pub fn as_slice_mut(&mut self) -> Option<&mut [u8]> {
        let ptr = self.base_address_mut()?;
        let len = self.buffer.height() * self.buffer.bytes_per_row();
        if ptr.is_null() || len == 0 {
            Some(&mut [])
        } else {
            Some(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
        }
    }

    /// Get a slice of plane data
    ///
    /// Returns the data for a specific plane as a byte slice.
    ///
    /// Returns `None` if the plane index is out of bounds.
    pub fn plane_data(&self, plane_index: usize) -> Option<&[u8]> {
        let base = self.base_address_of_plane(plane_index)?;
        let height = self.buffer.height_of_plane(plane_index);
        let bytes_per_row = self.buffer.bytes_per_row_of_plane(plane_index);
        Some(unsafe { std::slice::from_raw_parts(base, height * bytes_per_row) })
    }

    /// Get a specific row from a plane as a slice
    ///
    /// Returns `None` if the plane or row index is out of bounds.
    pub fn plane_row(&self, plane_index: usize, row_index: usize) -> Option<&[u8]> {
        if !self.buffer.is_planar() || plane_index >= self.buffer.plane_count() {
            return None;
        }
        let height = self.buffer.height_of_plane(plane_index);
        if row_index >= height {
            return None;
        }
        let base = self.base_address_of_plane(plane_index)?;
        let bytes_per_row = self.buffer.bytes_per_row_of_plane(plane_index);
        Some(unsafe {
            std::slice::from_raw_parts(base.add(row_index * bytes_per_row), bytes_per_row)
        })
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

    /// Access buffer with a standard `std::io::Cursor`
    ///
    /// Returns a cursor over the buffer data that implements `Read` and `Seek`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cv::{CVPixelBuffer, CVPixelBufferLockFlags};
    /// use std::io::{Read, Seek, SeekFrom};
    ///
    /// fn read_buffer(buffer: &CVPixelBuffer) {
    ///     let guard = buffer.lock(CVPixelBufferLockFlags::READ_ONLY).unwrap();
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

    /// Get raw pointer to buffer data
    pub fn as_ptr(&self) -> *const u8 {
        self.base_address()
    }

    /// Get mutable raw pointer to buffer data (only valid for read-write locks)
    ///
    /// Returns `None` if this is a read-only lock.
    pub fn as_mut_ptr(&mut self) -> Option<*mut u8> {
        self.base_address_mut()
    }

    /// Check if this is a read-only lock
    pub const fn is_read_only(&self) -> bool {
        self.flags.is_read_only()
    }

    /// Get the lock options
    pub const fn options(&self) -> CVPixelBufferLockFlags {
        self.flags
    }

    /// Get the pixel format
    pub fn pixel_format(&self) -> u32 {
        self.buffer.pixel_format()
    }
}

impl Drop for CVPixelBufferLockGuard<'_> {
    fn drop(&mut self) {
        let _ = self.buffer.unlock_raw(self.flags);
    }
}

impl std::fmt::Debug for CVPixelBufferLockGuard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CVPixelBufferLockGuard")
            .field("flags", &self.flags)
            .field("buffer_size", &(self.buffer.width(), self.buffer.height()))
            .finish()
    }
}

impl std::ops::Deref for CVPixelBufferLockGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Clone for CVPixelBuffer {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_retain(self.0);
            Self(ptr)
        }
    }
}

impl Drop for CVPixelBuffer {
    fn drop(&mut self) {
        unsafe {
            ffi::cv_pixel_buffer_release(self.0);
        }
    }
}

unsafe impl Send for CVPixelBuffer {}
unsafe impl Sync for CVPixelBuffer {}

impl fmt::Display for CVPixelBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CVPixelBuffer({}x{}, format: 0x{:08X})",
            self.width(),
            self.height(),
            self.pixel_format()
        )
    }
}

/// Opaque handle to `CVPixelBufferPool`
#[repr(transparent)]
#[derive(Debug)]
pub struct CVPixelBufferPool(*mut std::ffi::c_void);

impl PartialEq for CVPixelBufferPool {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CVPixelBufferPool {}

impl std::hash::Hash for CVPixelBufferPool {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cv_pixel_buffer_pool_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl CVPixelBufferPool {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CVPixelBufferPool` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Create a new pixel buffer pool
    ///
    /// # Arguments
    ///
    /// * `width` - Width of pixel buffers in the pool
    /// * `height` - Height of pixel buffers in the pool
    /// * `pixel_format` - Pixel format type
    /// * `max_buffers` - Maximum number of buffers in the pool (0 for unlimited)
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the pool creation fails.
    pub fn create(
        width: usize,
        height: usize,
        pixel_format: u32,
        max_buffers: usize,
    ) -> Result<Self, i32> {
        unsafe {
            let mut pool_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cv_pixel_buffer_pool_create(
                width,
                height,
                pixel_format,
                max_buffers,
                &mut pool_ptr,
            );

            if status == 0 && !pool_ptr.is_null() {
                Ok(Self(pool_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Create a pixel buffer from the pool
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the buffer creation fails.
    pub fn create_pixel_buffer(&self) -> Result<CVPixelBuffer, i32> {
        unsafe {
            let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status =
                ffi::cv_pixel_buffer_pool_create_pixel_buffer(self.0, &mut pixel_buffer_ptr);

            if status == 0 && !pixel_buffer_ptr.is_null() {
                Ok(CVPixelBuffer(pixel_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Flush the pixel buffer pool
    ///
    /// Releases all available pixel buffers in the pool
    pub fn flush(&self) {
        unsafe {
            ffi::cv_pixel_buffer_pool_flush(self.0);
        }
    }

    /// Get the Core Foundation type ID for `CVPixelBufferPool`
    pub fn type_id() -> usize {
        unsafe { ffi::cv_pixel_buffer_pool_get_type_id() }
    }

    /// Create a pixel buffer from the pool with auxiliary attributes
    ///
    /// This allows specifying additional attributes for the created buffer
    ///
    /// # Errors
    ///
    /// Returns a Core Video error code if the buffer creation fails.
    pub fn create_pixel_buffer_with_aux_attributes(
        &self,
        aux_attributes: Option<&std::collections::HashMap<String, u32>>,
    ) -> Result<CVPixelBuffer, i32> {
        // For now, ignore aux_attributes since we don't have a way to pass them through
        // In a full implementation, this would convert the HashMap to a CFDictionary
        let _ = aux_attributes;
        self.create_pixel_buffer()
    }

    /// Try to create a pixel buffer from the pool without blocking
    ///
    /// Returns None if no buffers are available
    pub fn try_create_pixel_buffer(&self) -> Option<CVPixelBuffer> {
        self.create_pixel_buffer().ok()
    }

    /// Flush the pool with specific options
    ///
    /// Releases buffers based on the provided flags
    pub fn flush_with_options(&self, _flags: u32) {
        // For now, just call regular flush
        // In a full implementation, this would pass flags to the Swift side
        self.flush();
    }

    /// Check if the pool is empty (no available buffers)
    ///
    /// Note: This is an approximation based on whether we can create a buffer
    pub fn is_empty(&self) -> bool {
        self.try_create_pixel_buffer().is_none()
    }

    /// Get the pool attributes
    ///
    /// Returns the raw pointer to the `CFDictionary` containing pool attributes
    pub fn attributes(&self) -> Option<*const std::ffi::c_void> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_pool_get_attributes(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }

    /// Get the pixel buffer attributes
    ///
    /// Returns the raw pointer to the `CFDictionary` containing pixel buffer attributes
    pub fn pixel_buffer_attributes(&self) -> Option<*const std::ffi::c_void> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_pool_get_pixel_buffer_attributes(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }
}

impl Clone for CVPixelBufferPool {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_pool_retain(self.0);
            Self(ptr)
        }
    }
}

impl Drop for CVPixelBufferPool {
    fn drop(&mut self) {
        unsafe {
            ffi::cv_pixel_buffer_pool_release(self.0);
        }
    }
}

unsafe impl Send for CVPixelBufferPool {}
unsafe impl Sync for CVPixelBufferPool {}

impl fmt::Display for CVPixelBufferPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CVPixelBufferPool")
    }
}

/// Extension trait for `io::Cursor` to add pixel buffer specific operations
pub trait PixelBufferCursorExt {
    /// Seek to a specific pixel coordinate (x, y)
    ///
    /// Assumes 4 bytes per pixel (BGRA format).
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the seek operation fails.
    fn seek_to_pixel(&mut self, x: usize, y: usize, bytes_per_row: usize) -> io::Result<u64>;

    /// Read a single pixel (4 bytes: BGRA)
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the read operation fails.
    fn read_pixel(&mut self) -> io::Result<[u8; 4]>;
}

impl<T: AsRef<[u8]>> PixelBufferCursorExt for io::Cursor<T> {
    fn seek_to_pixel(&mut self, x: usize, y: usize, bytes_per_row: usize) -> io::Result<u64> {
        let pos = y * bytes_per_row + x * 4; // 4 bytes per pixel (BGRA)
        self.seek(SeekFrom::Start(pos as u64))
    }

    fn read_pixel(&mut self) -> io::Result<[u8; 4]> {
        let mut pixel = [0u8; 4];
        self.read_exact(&mut pixel)?;
        Ok(pixel)
    }
}
