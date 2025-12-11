//! `CMBlockBuffer` - Block of contiguous data
//!
//! A `CMBlockBuffer` represents a contiguous range of data, typically used
//! for audio samples or compressed video data. It manages memory ownership
//! and provides access to the underlying data bytes.

use super::ffi;
use std::io;

/// Block buffer containing contiguous media data
///
/// `CMBlockBuffer` is a Core Media type that represents a block of data,
/// commonly used for audio samples or compressed video data. The data is
/// managed by Core Media and released when the buffer is dropped.
///
/// Unlike `CVPixelBuffer` or `IOSurface`, `CMBlockBuffer` does not require
/// locking for data access - the data pointer is valid as long as the buffer
/// is retained.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::cm::CMBlockBuffer;
///
/// fn process_block_buffer(buffer: &CMBlockBuffer) {
///     // Check if there's any data
///     if buffer.is_empty() {
///         return;
///     }
///
///     println!("Buffer has {} bytes", buffer.data_length());
///
///     // Get a pointer to the data
///     if let Some((ptr, length)) = buffer.data_pointer(0) {
///         println!("Got {} bytes at offset 0", length);
///     }
///
///     // Or copy data to a Vec
///     if let Some(data) = buffer.copy_data_bytes(0, buffer.data_length()) {
///         println!("Copied {} bytes", data.len());
///     }
/// }
/// ```
pub struct CMBlockBuffer(*mut std::ffi::c_void);

impl PartialEq for CMBlockBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CMBlockBuffer {}

impl std::hash::Hash for CMBlockBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cm_block_buffer_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl CMBlockBuffer {
    /// Create a new `CMBlockBuffer` with the given data
    ///
    /// # Arguments
    ///
    /// * `data` - The data to copy into the block buffer
    ///
    /// # Returns
    ///
    /// `Some(CMBlockBuffer)` if successful, `None` if creation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// let data = vec![1u8, 2, 3, 4, 5];
    /// let buffer = CMBlockBuffer::create(&data).expect("Failed to create buffer");
    /// assert_eq!(buffer.data_length(), 5);
    /// ```
    #[must_use]
    pub fn create(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return Self::create_empty();
        }
        let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = unsafe {
            ffi::cm_block_buffer_create_with_data(data.as_ptr().cast(), data.len(), &mut ptr)
        };
        if status == 0 && !ptr.is_null() {
            Some(Self(ptr))
        } else {
            None
        }
    }

    /// Create an empty `CMBlockBuffer`
    ///
    /// # Returns
    ///
    /// `Some(CMBlockBuffer)` if successful, `None` if creation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// let buffer = CMBlockBuffer::create_empty().expect("Failed to create empty buffer");
    /// assert!(buffer.is_empty());
    /// ```
    #[must_use]
    pub fn create_empty() -> Option<Self> {
        let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = unsafe { ffi::cm_block_buffer_create_empty(&mut ptr) };
        if status == 0 && !ptr.is_null() {
            Some(Self(ptr))
        } else {
            None
        }
    }

    /// Create from a raw pointer, returning `None` if null
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CMBlockBuffer` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer to the block buffer
    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Get the total data length of the buffer in bytes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// fn check_size(buffer: &CMBlockBuffer) {
    ///     let size = buffer.data_length();
    ///     println!("Buffer contains {} bytes", size);
    /// }
    /// ```
    pub fn data_length(&self) -> usize {
        unsafe { ffi::cm_block_buffer_get_data_length(self.0) }
    }

    /// Check if the buffer is empty (contains no data)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// fn process(buffer: &CMBlockBuffer) {
    ///     if buffer.is_empty() {
    ///         println!("No data to process");
    ///         return;
    ///     }
    ///     // Process data...
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        unsafe { ffi::cm_block_buffer_is_empty(self.0) }
    }

    /// Check if a range of bytes is stored contiguously in memory
    ///
    /// # Arguments
    ///
    /// * `offset` - Starting offset in the buffer
    /// * `length` - Length of the range to check
    ///
    /// # Returns
    ///
    /// `true` if the specified range is contiguous in memory
    pub fn is_range_contiguous(&self, offset: usize, length: usize) -> bool {
        unsafe { ffi::cm_block_buffer_is_range_contiguous(self.0, offset, length) }
    }

    /// Get a pointer to the data at the specified offset
    ///
    /// Returns a tuple of (data pointer, length available at that offset) if successful.
    /// The pointer is valid as long as this `CMBlockBuffer` is retained.
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset into the buffer
    ///
    /// # Returns
    ///
    /// `Some((pointer, length_at_offset))` if the data pointer was obtained successfully,
    /// `None` if the operation failed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// fn read_data(buffer: &CMBlockBuffer) {
    ///     if let Some((ptr, length)) = buffer.data_pointer(0) {
    ///         // SAFETY: ptr is valid for `length` bytes while buffer is alive
    ///         let slice = unsafe { std::slice::from_raw_parts(ptr, length) };
    ///         println!("First byte: {:02x}", slice[0]);
    ///     }
    /// }
    /// ```
    pub fn data_pointer(&self, offset: usize) -> Option<(*const u8, usize)> {
        unsafe {
            let mut length_at_offset: usize = 0;
            let mut total_length: usize = 0;
            let mut data_pointer: *mut std::ffi::c_void = std::ptr::null_mut();

            let status = ffi::cm_block_buffer_get_data_pointer(
                self.0,
                offset,
                &mut length_at_offset,
                &mut total_length,
                &mut data_pointer,
            );

            if status == 0 && !data_pointer.is_null() {
                Some((data_pointer.cast::<u8>().cast_const(), length_at_offset))
            } else {
                None
            }
        }
    }

    /// Get a mutable pointer to the data at the specified offset
    ///
    /// # Safety
    ///
    /// The caller must ensure that modifying the data is safe and that no other
    /// references to this data exist.
    pub unsafe fn data_pointer_mut(&self, offset: usize) -> Option<(*mut u8, usize)> {
        let mut length_at_offset: usize = 0;
        let mut total_length: usize = 0;
        let mut data_pointer: *mut std::ffi::c_void = std::ptr::null_mut();

        let status = ffi::cm_block_buffer_get_data_pointer(
            self.0,
            offset,
            &mut length_at_offset,
            &mut total_length,
            &mut data_pointer,
        );

        if status == 0 && !data_pointer.is_null() {
            Some((data_pointer.cast::<u8>(), length_at_offset))
        } else {
            None
        }
    }

    /// Copy data bytes from the buffer into a new `Vec<u8>`
    ///
    /// This is the safest way to access buffer data as it copies the bytes
    /// into owned memory.
    ///
    /// # Arguments
    ///
    /// * `offset` - Starting offset in the buffer
    /// * `length` - Number of bytes to copy
    ///
    /// # Returns
    ///
    /// `Some(Vec<u8>)` containing the copied data, or `None` if the copy failed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// fn extract_data(buffer: &CMBlockBuffer) -> Option<Vec<u8>> {
    ///     // Copy all data from the buffer
    ///     buffer.copy_data_bytes(0, buffer.data_length())
    /// }
    /// ```
    pub fn copy_data_bytes(&self, offset: usize, length: usize) -> Option<Vec<u8>> {
        if length == 0 {
            return Some(Vec::new());
        }

        let mut data = vec![0u8; length];
        unsafe {
            let status = ffi::cm_block_buffer_copy_data_bytes(
                self.0,
                offset,
                length,
                data.as_mut_ptr().cast::<std::ffi::c_void>(),
            );

            if status == 0 {
                Some(data)
            } else {
                None
            }
        }
    }

    /// Copy data bytes from the buffer into an existing slice
    ///
    /// # Arguments
    ///
    /// * `offset` - Starting offset in the buffer
    /// * `destination` - Mutable slice to copy data into
    ///
    /// # Errors
    ///
    /// Returns a Core Media error code if the copy fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// fn read_header(buffer: &CMBlockBuffer) -> Result<[u8; 4], i32> {
    ///     let mut header = [0u8; 4];
    ///     buffer.copy_data_bytes_into(0, &mut header)?;
    ///     Ok(header)
    /// }
    /// ```
    pub fn copy_data_bytes_into(&self, offset: usize, destination: &mut [u8]) -> Result<(), i32> {
        if destination.is_empty() {
            return Ok(());
        }

        unsafe {
            let status = ffi::cm_block_buffer_copy_data_bytes(
                self.0,
                offset,
                destination.len(),
                destination.as_mut_ptr().cast::<std::ffi::c_void>(),
            );

            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Get a slice view of the data if the entire buffer is contiguous
    ///
    /// This is a zero-copy way to access the data, but only works if the
    /// buffer's data is stored contiguously in memory.
    ///
    /// # Returns
    ///
    /// `Some(&[u8])` if the buffer is contiguous, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// fn process_contiguous(buffer: &CMBlockBuffer) {
    ///     if let Some(data) = buffer.as_slice() {
    ///         println!("Processing {} contiguous bytes", data.len());
    ///     } else {
    ///         // Fall back to copying
    ///         if let Some(data) = buffer.copy_data_bytes(0, buffer.data_length()) {
    ///             println!("Processing {} copied bytes", data.len());
    ///         }
    ///     }
    /// }
    /// ```
    pub fn as_slice(&self) -> Option<&[u8]> {
        let len = self.data_length();
        if len == 0 {
            return Some(&[]);
        }

        // Check if the entire buffer is contiguous
        if !self.is_range_contiguous(0, len) {
            return None;
        }

        self.data_pointer(0).map(|(ptr, length)| {
            // Use the minimum of reported length and data_length for safety
            let safe_len = length.min(len);
            unsafe { std::slice::from_raw_parts(ptr, safe_len) }
        })
    }

    /// Access buffer with a standard `std::io::Cursor`
    ///
    /// Returns a cursor over a copy of the buffer data. The cursor implements
    /// `Read` and `Seek` traits for convenient sequential data access.
    ///
    /// Note: This copies the data because `CMBlockBuffer` may not be contiguous.
    /// For zero-copy access to contiguous buffers, use [`as_slice()`](Self::as_slice).
    ///
    /// # Returns
    ///
    /// `Some(Cursor)` if data could be copied, `None` if the copy failed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{Read, Seek, SeekFrom};
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// fn read_data(buffer: &CMBlockBuffer) {
    ///     if let Some(mut cursor) = buffer.cursor() {
    ///         // Read first 4 bytes
    ///         let mut header = [0u8; 4];
    ///         cursor.read_exact(&mut header).unwrap();
    ///
    ///         // Seek to a position
    ///         cursor.seek(SeekFrom::Start(100)).unwrap();
    ///
    ///         // Read more data
    ///         let mut buf = [0u8; 16];
    ///         cursor.read_exact(&mut buf).unwrap();
    ///     }
    /// }
    /// ```
    pub fn cursor(&self) -> Option<io::Cursor<Vec<u8>>> {
        self.copy_data_bytes(0, self.data_length())
            .map(io::Cursor::new)
    }

    /// Access contiguous buffer with a zero-copy `std::io::Cursor`
    ///
    /// Returns a cursor over the buffer data without copying, but only works
    /// if the buffer is contiguous in memory.
    ///
    /// # Returns
    ///
    /// `Some(Cursor)` if the buffer is contiguous, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{Read, Seek, SeekFrom};
    /// use screencapturekit::cm::CMBlockBuffer;
    ///
    /// fn read_contiguous(buffer: &CMBlockBuffer) {
    ///     // Try zero-copy first
    ///     if let Some(mut cursor) = buffer.cursor_ref() {
    ///         let mut header = [0u8; 4];
    ///         cursor.read_exact(&mut header).unwrap();
    ///     } else {
    ///         // Fall back to copying cursor
    ///         if let Some(mut cursor) = buffer.cursor() {
    ///             let mut header = [0u8; 4];
    ///             cursor.read_exact(&mut header).unwrap();
    ///         }
    ///     }
    /// }
    /// ```
    pub fn cursor_ref(&self) -> Option<io::Cursor<&[u8]>> {
        self.as_slice().map(io::Cursor::new)
    }
}

impl Clone for CMBlockBuffer {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::cm_block_buffer_retain(self.0);
            Self(ptr)
        }
    }
}

impl Drop for CMBlockBuffer {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ffi::cm_block_buffer_release(self.0);
            }
        }
    }
}

unsafe impl Send for CMBlockBuffer {}
unsafe impl Sync for CMBlockBuffer {}

impl std::fmt::Debug for CMBlockBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CMBlockBuffer")
            .field("ptr", &self.0)
            .field("data_length", &self.data_length())
            .field("is_empty", &self.is_empty())
            .finish()
    }
}

impl std::fmt::Display for CMBlockBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CMBlockBuffer({} bytes)", self.data_length())
    }
}
