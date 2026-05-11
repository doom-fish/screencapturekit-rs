//! FFI string utilities
//!
//! Helper functions for retrieving strings from C/Objective-C APIs
//! that use buffer-based string retrieval patterns.

use std::ffi::CStr;

/// Default buffer size for FFI string retrieval
pub const DEFAULT_BUFFER_SIZE: usize = 1024;

/// Smaller buffer size for short strings (e.g., device IDs, stream names)
pub const SMALL_BUFFER_SIZE: usize = 256;

/// Stack-allocate up to this many bytes — anything bigger falls back to a
/// heap `Vec`. 256 bytes covers every real call site (`SMALL_BUFFER_SIZE`,
/// audio device IDs, stream names, microphone IDs); the 1 KiB callers are
/// rare and currently absent from the codebase, so the heap fallback path
/// is essentially dead code today but kept for forward-compat with
/// future longer-string APIs.
const STACK_BUFFER_BYTES: usize = 256;

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
/// The caller must ensure the `ffi_call` closure does not write beyond the
/// provided `buffer_len`. This function defends against the closure writing
/// a non-NUL-terminated string by scanning the buffer up to its declared
/// length and treating the absence of a terminator as failure (returns
/// `None`) rather than reading past the buffer with `CStr::from_ptr`.
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
    // Fast path: the typical small-getter case (audio device IDs, stream
    // names, microphone IDs) fits comfortably in 256 bytes and is called
    // often enough that the per-call `vec![0i8; 256]` heap allocation
    // adds up. Use a stack buffer for those and only fall back to a Vec
    // for unusually-large requests.
    if buffer_size <= STACK_BUFFER_BYTES {
        let mut buffer = [0i8; STACK_BUFFER_BYTES];
        let success = ffi_call(buffer.as_mut_ptr(), buffer_size as isize);
        if !success {
            return None;
        }
        return parse_buffer(&buffer[..buffer_size]);
    }

    let mut buffer = vec![0i8; buffer_size];
    let success = ffi_call(buffer.as_mut_ptr(), buffer.len() as isize);
    if !success {
        return None;
    }
    parse_buffer(&buffer)
}

/// Scan for the NUL terminator and decode the string portion.
/// Defensive: do NOT use `CStr::from_ptr` here. If the FFI closure
/// returned `true` but failed to write a NUL terminator, `CStr::from_ptr`
/// would read past the buffer until it found a zero byte — UB and a
/// potential information leak. Instead, scan only the buffer we
/// allocated and treat a missing terminator as failure.
fn parse_buffer(buffer: &[i8]) -> Option<String> {
    // SAFETY: `i8` and `u8` have identical layout; the cast is purely a
    // signed/unsigned reinterpretation.
    let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), buffer.len()) };
    let nul_pos = bytes.iter().position(|&b| b == 0)?;
    let s = String::from_utf8_lossy(&bytes[..nul_pos]).into_owned();
    if s.is_empty() {
        None
    } else {
        Some(s)
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
/// or equivalent, and that `sc_free_string` properly frees it. The pointer is
/// freed via an RAII guard, so a panic in `to_string_lossy` (extremely rare —
/// only OOM) does not leak the Swift-allocated buffer.
pub unsafe fn ffi_string_owned<F>(ffi_call: F) -> Option<String>
where
    F: FnOnce() -> *mut i8,
{
    /// RAII guard: releases the Swift-allocated buffer on drop, including
    /// during panic unwind. Without this, a panic between `CStr::from_ptr`
    /// and the explicit `sc_free_string` call (e.g. allocator failure
    /// inside `to_string_lossy`) would leak the buffer.
    struct FreeGuard(*mut i8);
    impl Drop for FreeGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { crate::ffi::sc_free_string(self.0) };
            }
        }
    }

    let ptr = ffi_call();
    if ptr.is_null() {
        return None;
    }
    let _guard = FreeGuard(ptr);
    // `to_string_lossy().to_string()` allocates twice on the valid-UTF-8
    // path: once for the borrowed Cow, then again for the explicit
    // `to_string`. `from_utf8_lossy(...).into_owned()` allocates once
    // and skips the redundant copy. For invalid UTF-8 (extremely rare
    // for AppKit strings) both paths allocate the replacement-char string.
    let bytes = CStr::from_ptr(ptr).to_bytes();
    if bytes.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(bytes).into_owned())
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
