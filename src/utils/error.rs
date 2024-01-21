pub(crate) mod internal {
    use std::ptr;

    use core_foundation::{
        base::{CFAllocatorGetDefault, TCFType},
        error::{CFError, CFErrorCreate},
    };

    pub(crate) fn create_cf_error(domain: String, code: u32) -> CFError {
        unsafe {
            CFError::wrap_under_create_rule(CFErrorCreate(
                CFAllocatorGetDefault(),
                domain,
                code,
                ptr::null(),
            ))
        }
    }
}
