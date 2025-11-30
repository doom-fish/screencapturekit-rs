//! FFI string utilities
//!
//! Helper functions for retrieving strings from C/Objective-C APIs
//! that use buffer-based string retrieval patterns.

use std::ffi::CStr;

/// Default buffer size for FFI string retrieval
pub const DEFAULT_BUFFER_SIZE: usize = 1024;

/// Smaller buffer size for short strings (e.g., device IDs, stream names)
pub const SMALL_BUFFER_SIZE: usize = 256;

/// Retrieves a string from an FFI function that writes to a buffer.
///
/// This is a common pattern in Objective-C FFI where a function:
/// 1. Takes a buffer pointer and length
/// 2. Writes a null-terminated string to the buffer
/// 3. Returns a boolean indicating success
///
/// # Arguments
/// * `buffer_size` - Size of the buffer to allocate
/// * `ffi_call` - A closure that takes (`buffer_ptr`, `buffer_len`) and returns success bool
///
/// # Returns
/// * `Some(String)` if the FFI call succeeded and the string was valid UTF-8
/// * `None` if the FFI call failed or returned an empty string
///
/// # Safety
/// The caller must ensure the `ffi_call` closure properly writes a null-terminated
/// string to the provided buffer and does not write beyond the buffer length.
///
/// # Example
/// ```
/// use screencapturekit::utils::ffi_string::ffi_string_from_buffer;
///
/// let result = unsafe {
///     ffi_string_from_buffer(64, |buf, len| {
///         // Simulate FFI call that writes "hello" to buffer
///         let src = b"hello\0";
///         if len >= src.len() as isize {
///             std::ptr::copy_nonoverlapping(src.as_ptr(), buf as *mut u8, src.len());
///             true
///         } else {
///             false
///         }
///     })
/// };
/// assert_eq!(result, Some("hello".to_string()));
/// ```
#[allow(clippy::cast_possible_wrap)]
pub unsafe fn ffi_string_from_buffer<F>(buffer_size: usize, ffi_call: F) -> Option<String>
where
    F: FnOnce(*mut i8, isize) -> bool,
{
    let mut buffer = vec![0i8; buffer_size];
    let success = ffi_call(buffer.as_mut_ptr(), buffer.len() as isize);
    if success {
        let c_str = CStr::from_ptr(buffer.as_ptr());
        let s = c_str.to_string_lossy().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    }
}

/// Same as [`ffi_string_from_buffer`] but returns an empty string on failure
/// instead of `None`.
///
/// Useful when the API should always return a string, even if empty.
///
/// # Safety
/// The caller must ensure that the FFI call writes valid UTF-8 data to the buffer.
#[allow(clippy::cast_possible_wrap)]
pub unsafe fn ffi_string_from_buffer_or_empty<F>(buffer_size: usize, ffi_call: F) -> String
where
    F: FnOnce(*mut i8, isize) -> bool,
{
    ffi_string_from_buffer(buffer_size, ffi_call).unwrap_or_default()
}

/// Retrieves a string from an FFI function that returns an owned C string pointer.
///
/// This is more efficient than buffer-based retrieval as it avoids pre-allocation.
/// The FFI function allocates the string (via `strdup`) and this function takes
/// ownership and frees it.
///
/// # Arguments
/// * `ffi_call` - A closure that returns an owned C string pointer (or null)
///
/// # Returns
/// * `Some(String)` if the pointer was non-null and valid UTF-8
/// * `None` if the pointer was null
///
/// # Safety
/// The caller must ensure the returned pointer was allocated by Swift's `strdup`
/// or equivalent, and that `sc_free_string` properly frees it.
pub unsafe fn ffi_string_owned<F>(ffi_call: F) -> Option<String>
where
    F: FnOnce() -> *mut i8,
{
    let ptr = ffi_call();
    if ptr.is_null() {
        return None;
    }
    let c_str = CStr::from_ptr(ptr);
    let result = c_str.to_string_lossy().to_string();
    crate::ffi::sc_free_string(ptr);
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Same as [`ffi_string_owned`] but returns an empty string on failure.
///
/// # Safety
/// Same requirements as [`ffi_string_owned`].
pub unsafe fn ffi_string_owned_or_empty<F>(ffi_call: F) -> String
where
    F: FnOnce() -> *mut i8,
{
    ffi_string_owned(ffi_call).unwrap_or_default()
}


