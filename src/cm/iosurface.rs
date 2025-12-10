//! `IOSurface` - Hardware-accelerated surface

use super::ffi;
use std::fmt;
use std::io;

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
    pub fn plane_count(&self) -> usize {
        unsafe { ffi::io_surface_get_plane_count(self.0) }
    }

    /// Get the width of a specific plane
    pub fn width_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::io_surface_get_width_of_plane(self.0, plane_index) }
    }

    /// Get the height of a specific plane
    pub fn height_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::io_surface_get_height_of_plane(self.0, plane_index) }
    }

    /// Get the bytes per row of a specific plane
    pub fn bytes_per_row_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::io_surface_get_bytes_per_row_of_plane(self.0, plane_index) }
    }

    /// Get the base address of a specific plane
    ///
    /// Note: The surface must be locked before accessing the base address.
    pub fn base_address_of_plane(&self, plane_index: usize) -> *mut u8 {
        unsafe { ffi::io_surface_get_base_address_of_plane(self.0, plane_index).cast::<u8>() }
    }

    /// Get the base address of the surface
    ///
    /// Note: The surface must be locked before accessing the base address.
    pub fn base_address(&self) -> *mut u8 {
        unsafe { ffi::io_surface_get_base_address(self.0).cast::<u8>() }
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

    /// Lock the surface for CPU access
    ///
    /// # Arguments
    /// * `read_only` - If true, locks for read-only access
    ///
    /// # Errors
    /// Returns `kern_return_t` error code if the lock fails.
    pub fn lock(&self, read_only: bool) -> Result<u32, i32> {
        let options = u32::from(read_only); // kIOSurfaceLockReadOnly = 1
        let mut seed: u32 = 0;
        let status = unsafe { ffi::io_surface_lock(self.0, options, &mut seed) };
        if status == 0 {
            Ok(seed)
        } else {
            Err(status)
        }
    }

    /// Unlock the surface after CPU access
    ///
    /// # Arguments
    /// * `read_only` - Must match the value used in the corresponding `lock()` call
    ///
    /// # Errors
    /// Returns `kern_return_t` error code if the unlock fails.
    pub fn unlock(&self, read_only: bool) -> Result<u32, i32> {
        let options = u32::from(read_only);
        let mut seed: u32 = 0;
        let status = unsafe { ffi::io_surface_unlock(self.0, options, &mut seed) };
        if status == 0 {
            Ok(seed)
        } else {
            Err(status)
        }
    }

    /// Lock the surface and return a guard for RAII-style access
    ///
    /// # Arguments
    /// * `read_only` - If true, locks for read-only access
    ///
    /// # Errors
    /// Returns `kern_return_t` error code if the lock fails.
    pub fn lock_guard(&self, read_only: bool) -> Result<IOSurfaceLockGuard<'_>, i32> {
        self.lock(read_only)?;
        Ok(IOSurfaceLockGuard {
            surface: self,
            read_only,
        })
    }
}

/// RAII guard for locked `IOSurface`
pub struct IOSurfaceLockGuard<'a> {
    surface: &'a IOSurface,
    read_only: bool,
}

impl IOSurfaceLockGuard<'_> {
    /// Get the base address of the locked surface
    pub fn base_address(&self) -> *const u8 {
        self.surface.base_address().cast_const()
    }

    /// Get the mutable base address (only valid for read-write locks)
    pub fn base_address_mut(&mut self) -> Option<*mut u8> {
        if self.read_only {
            None
        } else {
            Some(self.surface.base_address())
        }
    }

    /// Get the bytes per row of the surface
    pub fn bytes_per_row(&self) -> usize {
        self.surface.bytes_per_row()
    }

    /// Get a slice view of the surface data
    ///
    /// # Safety
    /// The caller must ensure the surface data is valid for the duration of the slice.
    pub unsafe fn as_slice(&self) -> &[u8] {
        let ptr = self.base_address();
        let len = self.surface.alloc_size();
        if ptr.is_null() || len == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(ptr, len)
        }
    }

    /// Get a mutable slice view of the surface data (only valid for read-write locks)
    ///
    /// # Safety
    /// The caller must ensure the surface data is valid for the duration of the slice.
    pub unsafe fn as_slice_mut(&mut self) -> Option<&mut [u8]> {
        if self.read_only {
            return None;
        }
        let ptr = self.surface.base_address();
        let len = self.surface.alloc_size();
        if ptr.is_null() || len == 0 {
            Some(&mut [])
        } else {
            Some(std::slice::from_raw_parts_mut(ptr, len))
        }
    }

    /// Access surface with a standard `std::io::Cursor`
    ///
    /// Returns a cursor over the surface data that implements `Read` and `Seek`.
    ///
    /// # Safety
    /// The caller must ensure the surface data is valid for the lifetime of the cursor.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{Read, Seek, SeekFrom};
    /// use screencapturekit::cm::IOSurface;
    ///
    /// fn read_surface(surface: &IOSurface) {
    ///     let guard = surface.lock_guard(true).unwrap();
    ///     let mut cursor = unsafe { guard.cursor() };
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
    pub unsafe fn cursor(&self) -> io::Cursor<&[u8]> {
        io::Cursor::new(self.as_slice())
    }
}

impl Drop for IOSurfaceLockGuard<'_> {
    fn drop(&mut self) {
        let _ = self.surface.unlock(self.read_only);
    }
}

impl std::fmt::Debug for IOSurfaceLockGuard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IOSurfaceLockGuard")
            .field("read_only", &self.read_only)
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
