//! `CMBlockBuffer` - Block of contiguous data
//!
//! A `CMBlockBuffer` represents a contiguous range of data, typically used
//! for audio samples. It manages memory ownership and is usually accessed
//! indirectly through [`AudioBufferList`](super::AudioBufferList).

use super::ffi;

/// Block buffer containing contiguous media data
///
/// `CMBlockBuffer` is a Core Media type that represents a contiguous block
/// of data, commonly used for audio samples. The data is managed by Core Media
/// and released when the buffer is dropped.
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
