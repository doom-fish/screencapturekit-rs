//! Pixel buffer wrapper with RAII lock guards
//!
//! Provides safe access to `CVPixelBuffer` and `IOSurface` with automatic locking/unlocking.
//! The lock guard pattern ensures buffers are always properly unlocked, even in case of panics.
//!
//! # Examples
//!
//! ```
//! use screencapturekit::prelude::*;
//! use screencapturekit::output::{CVImageBufferLockExt, PixelBufferLockFlags};
//!
//! # fn example() -> SCResult<()> {
//! // Create a test pixel buffer
//! let buffer = screencapturekit::cm::CVPixelBuffer::create(100, 100, 0x42475241)
//!     .map_err(|_| SCError::internal_error("Failed to create buffer"))?;
//!
//! // Lock for reading (automatically unlocks on drop)
//! let guard = buffer.lock(PixelBufferLockFlags::ReadOnly)?;
//!
//! // Access pixel data
//! let width = guard.width();
//! let height = guard.height();
//! let pixels = guard.as_slice();
//!
//! println!("Got {}x{} frame with {} bytes", width, height, pixels.len());
//!
//! // Buffer automatically unlocked here when guard drops
//! # Ok(())
//! # }
//! # example().unwrap();
//! ```

use std::ffi::c_void;
use std::io::{self, Read, Seek, SeekFrom};
use std::ops::Deref;
use std::ptr::NonNull;

/// Lock options for pixel buffer access
///
/// Specifies the access mode when locking a pixel buffer.
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelBufferLockFlags {
    /// Read-only access to the buffer
    ///
    /// Use this flag when you only need to read pixel data, not modify it.
    /// This is the most common use case for screen capture.
    ReadOnly = 0x0000_0001,
}

impl PixelBufferLockFlags {
    /// Convert to u64 representation
    pub const fn as_u64(self) -> u64 {
        self as u64
    }

    /// Convert to u32 representation (used by FFI)
    pub const fn as_u32(self) -> u32 {
        self as u32
    }
}

/// A guard that provides access to locked pixel buffer memory
///
/// This guard implements RAII (Resource Acquisition Is Initialization) pattern.
/// The buffer is automatically unlocked when this guard is dropped, ensuring
/// proper cleanup even if an error occurs or panic happens.
///
/// # Safety
///
/// The guard ensures:
/// - Buffer is locked before access
/// - Buffer is unlocked when guard is dropped
/// - No data races (single owner at a time)
/// - Memory is valid for the guard's lifetime
///
/// # Examples
///
/// ```
/// use screencapturekit::output::{CVImageBufferLockExt, PixelBufferLockFlags};
/// # use screencapturekit::cm::CVPixelBuffer;
/// # use screencapturekit::prelude::*;
///
/// # fn example() -> SCResult<()> {
/// // Create a test buffer
/// let buffer = CVPixelBuffer::create(100, 100, 0x42475241)
///     .map_err(|_| SCError::internal_error("Failed to create buffer"))?;
///
/// // Lock the buffer
/// let guard = buffer.lock(PixelBufferLockFlags::ReadOnly)?;
///
/// // Access properties
/// println!("Width: {}", guard.width());
/// println!("Height: {}", guard.height());
/// println!("Bytes per row: {}", guard.bytes_per_row());
///
/// // Access raw pixel data
/// let pixels: &[u8] = guard.as_slice();
/// println!("Total bytes: {}", pixels.len());
///
/// // Buffer automatically unlocked here
/// # Ok(())
/// # }
/// # example().unwrap();
/// ```
pub struct PixelBufferLockGuard<'a> {
    buffer_ptr: *mut c_void,
    base_address: NonNull<u8>,
    width: usize,
    height: usize,
    bytes_per_row: usize,
    flags: PixelBufferLockFlags,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl PixelBufferLockGuard<'_> {
    /// Create a new lock guard (used internally)
    pub(crate) unsafe fn new(
        buffer_ptr: *mut c_void,
        flags: PixelBufferLockFlags,
    ) -> Result<Self, i32> {
        // Lock the buffer
        let result = crate::cm::ffi::cv_pixel_buffer_lock_base_address(buffer_ptr, flags.as_u32());
        if result != 0 {
            return Err(result);
        }

        // Get buffer info
        let base_ptr = crate::cm::ffi::cv_pixel_buffer_get_base_address(buffer_ptr);
        let base_address = NonNull::new(base_ptr.cast::<u8>()).ok_or(-1)?;
        let width = crate::cm::ffi::cv_pixel_buffer_get_width(buffer_ptr) as usize;
        let height = crate::cm::ffi::cv_pixel_buffer_get_height(buffer_ptr) as usize;
        let bytes_per_row = crate::cm::ffi::cv_pixel_buffer_get_bytes_per_row(buffer_ptr) as usize;

        Ok(Self {
            buffer_ptr,
            base_address,
            width,
            height,
            bytes_per_row,
            flags,
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
    pub fn as_ptr(&self) -> *const u8 {
        self.base_address.as_ptr()
    }

    /// Get mutable raw pointer to buffer data (for read-only locks, use carefully)
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

    /// Access buffer with a cursor for reading bytes
    ///
    /// Returns a standard `std::io::Cursor` over the buffer data.
    /// The cursor implements `Read` and `Seek` traits for convenient data access.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Read, Seek, SeekFrom};
    /// use screencapturekit::output::{CVImageBufferLockExt, PixelBufferLockFlags};
    /// # use screencapturekit::cm::CVPixelBuffer;
    /// # use screencapturekit::prelude::*;
    ///
    /// # fn example() -> SCResult<()> {
    /// let buffer = CVPixelBuffer::create(100, 100, 0x42475241)
    ///     .map_err(|_| SCError::internal_error("Failed to create buffer"))?;
    /// let guard = buffer.lock(PixelBufferLockFlags::ReadOnly)?;
    ///
    /// let mut cursor = guard.cursor();
    ///
    /// // Read using standard Read trait
    /// let mut pixel = [0u8; 4];
    /// cursor.read_exact(&mut pixel).unwrap();
    ///
    /// // Seek using standard Seek trait
    /// cursor.seek(SeekFrom::Start(0)).unwrap();
    /// # Ok(())
    /// # }
    /// # example().unwrap();
    /// ```
    pub fn cursor(&self) -> io::Cursor<&[u8]> {
        io::Cursor::new(self.as_slice())
    }
}

impl Drop for PixelBufferLockGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            crate::cm::ffi::cv_pixel_buffer_unlock_base_address(
                self.buffer_ptr,
                self.flags.as_u32(),
            );
        }
    }
}

impl Deref for PixelBufferLockGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
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

/// Extension trait for `CVImageBuffer` with lock guards
/// Extension trait for locking pixel buffers
pub trait CVImageBufferLockExt {
    /// Lock the buffer and provide a guard for safe access
    ///
    /// # Errors
    ///
    /// Returns an `SCError` if the lock operation fails.
    fn lock(
        &self,
        flags: PixelBufferLockFlags,
    ) -> Result<PixelBufferLockGuard<'_>, crate::error::SCError>;
}

// Implementation for our CVPixelBuffer
impl CVImageBufferLockExt for crate::cm::CVPixelBuffer {
    fn lock(
        &self,
        flags: PixelBufferLockFlags,
    ) -> Result<PixelBufferLockGuard<'_>, crate::error::SCError> {
        unsafe {
            PixelBufferLockGuard::new(self.as_ptr(), flags).map_err(|code| {
                crate::error::SCError::buffer_lock_error(format!(
                    "Failed to lock pixel buffer (error code: {code})"
                ))
            })
        }
    }
}
