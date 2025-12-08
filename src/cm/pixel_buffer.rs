//! `CVPixelBuffer` - Video pixel buffer

use super::ffi;
use super::IOSurface;
use std::fmt;

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
    /// use screencapturekit::cm::CVPixelBuffer;
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
    /// use screencapturekit::cm::CVPixelBuffer;
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

    /// Get the base address of a specific plane
    pub fn base_address_of_plane(&self, plane_index: usize) -> Option<*mut u8> {
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
    pub fn lock_raw(&self, flags: u32) -> Result<(), i32> {
        unsafe {
            let result = ffi::cv_pixel_buffer_lock_base_address(self.0, flags);
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
    pub fn unlock_raw(&self, flags: u32) -> Result<(), i32> {
        unsafe {
            let result = ffi::cv_pixel_buffer_unlock_base_address(self.0, flags);
            if result == 0 {
                Ok(())
            } else {
                Err(result)
            }
        }
    }

    pub fn base_address(&self) -> Option<*mut u8> {
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
    /// # Errors
    ///
    /// Returns a Core Video error code if the lock operation fails.
    pub fn lock_base_address(&self, read_only: bool) -> Result<CVPixelBufferLockGuard<'_>, i32> {
        let flags = u32::from(read_only);
        self.lock_raw(flags)?;
        Ok(CVPixelBufferLockGuard {
            buffer: self,
            read_only,
        })
    }
}

/// RAII guard for locked `CVPixelBuffer` base address
pub struct CVPixelBufferLockGuard<'a> {
    buffer: &'a CVPixelBuffer,
    read_only: bool,
}

impl CVPixelBufferLockGuard<'_> {
    pub fn base_address(&self) -> *const u8 {
        self.buffer
            .base_address()
            .unwrap_or(std::ptr::null_mut())
            .cast_const()
    }

    pub fn base_address_mut(&mut self) -> *mut u8 {
        if self.read_only {
            std::ptr::null_mut()
        } else {
            self.buffer.base_address().unwrap_or(std::ptr::null_mut())
        }
    }
}

impl Drop for CVPixelBufferLockGuard<'_> {
    fn drop(&mut self) {
        let flags = u32::from(self.read_only);
        let _ = self.buffer.unlock_raw(flags);
    }
}

impl std::fmt::Debug for CVPixelBufferLockGuard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CVPixelBufferLockGuard")
            .field("read_only", &self.read_only)
            .field("buffer_size", &(self.buffer.width(), self.buffer.height()))
            .finish()
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
