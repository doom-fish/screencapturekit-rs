use std::{fmt::Display, ptr, str::FromStr};

use core_foundation::{
    base::{CFAllocatorGetDefault, TCFType},
    error::{CFError, CFErrorCreate},
    string::CFString,
};
/// .
///
/// # Panics
///
/// Panics if .
#[allow(clippy::module_name_repetitions)]
pub fn create_cf_error(domain: &str, code: isize) -> CFError {
    unsafe {
        CFError::wrap_under_create_rule(CFErrorCreate(
            CFAllocatorGetDefault(),
            CFString::from_str(domain)
                .expect("should work!")
                .as_concrete_TypeRef(),
            code,
            ptr::null(),
        ))
    }
}
/// .
///
/// # Panics
///
/// Panics if .
#[allow(clippy::module_name_repetitions)]
pub fn create_sc_error(message: impl Display) -> CFError {
    unsafe {
        CFError::wrap_under_create_rule(CFErrorCreate(
            CFAllocatorGetDefault(),
            CFString::from_str(&message.to_string())
                .expect("should work!")
                .as_concrete_TypeRef(),
            1337,
            ptr::null(),
        ))
    }
}
