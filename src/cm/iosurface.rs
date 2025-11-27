//! IOSurface - Hardware-accelerated surface

use std::fmt;
use super::ffi;

pub struct IOSurface(*mut std::ffi::c_void);

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
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `IOSurface` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    pub fn get_width(&self) -> usize {
        unsafe { ffi::io_surface_get_width(self.0) }
    }

    pub fn get_height(&self) -> usize {
        unsafe { ffi::io_surface_get_height(self.0) }
    }

    pub fn get_bytes_per_row(&self) -> usize {
        unsafe { ffi::io_surface_get_bytes_per_row(self.0) }
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

impl fmt::Display for IOSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IOSurface({}x{}, {} bytes/row)",
            self.get_width(),
            self.get_height(),
            self.get_bytes_per_row()
        )
    }
}

