pub(crate) mod internal {
    use std::{ptr, str::FromStr};

    use core_foundation::{
        base::{CFAllocatorGetDefault, TCFType},
        error::{CFError, CFErrorCreate},
        string::CFString,
    };

    pub(crate) fn create_cf_error(domain: &str, code: isize) -> CFError {
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
}
