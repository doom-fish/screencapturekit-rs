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
/// * `ffi_call` - A closure that takes (buffer_ptr, buffer_len) and returns success bool
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_string_from_buffer_success() {
        let result = unsafe {
            ffi_string_from_buffer(64, |buf, _len| {
                let test_str = b"hello\0";
                std::ptr::copy_nonoverlapping(test_str.as_ptr(), buf.cast::<u8>(), test_str.len());
                true
            })
        };
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_ffi_string_from_buffer_failure() {
        let result = unsafe { ffi_string_from_buffer(64, |_buf, _len| false) };
        assert_eq!(result, None);
    }

    #[test]
    fn test_ffi_string_from_buffer_empty() {
        let result = unsafe {
            ffi_string_from_buffer(64, |buf, _len| {
                *buf = 0; // empty string
                true
            })
        };
        assert_eq!(result, None);
    }

    #[test]
    fn test_ffi_string_or_empty() {
        let result = unsafe { ffi_string_from_buffer_or_empty(64, |_buf, _len| false) };
        assert_eq!(result, String::new());
    }
}
