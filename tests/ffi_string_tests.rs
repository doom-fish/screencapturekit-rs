//! Tests for FFI string utilities

/// Helper to create test buffer and call FFI-style callback
unsafe fn ffi_string_from_buffer<F>(size: usize, f: F) -> Option<String>
where
    F: FnOnce(*mut i8, usize) -> bool,
{
    let mut buffer = vec![0i8; size];
    if f(buffer.as_mut_ptr(), size) {
        let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr());
        let s = c_str.to_str().ok()?.to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    }
}

unsafe fn ffi_string_from_buffer_or_empty<F>(size: usize, f: F) -> String
where
    F: FnOnce(*mut i8, usize) -> bool,
{
    ffi_string_from_buffer(size, f).unwrap_or_default()
}

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
