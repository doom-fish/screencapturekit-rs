macro_rules! impl_objc_compatability {
    ($tftype:ident, $tfreftype:ident) => {
        unsafe impl objc::Message for $tftype {}
    };
}
use std::ffi::c_void;

use core_foundation::base::*;

/// .
///
/// # Safety
///
/// .
pub unsafe fn get_concrete_from_void<T: TCFType>(void_ptr: *const c_void) -> T {
    T::wrap_under_get_rule(T::Ref::from_void_ptr(void_ptr))
}

/// .
///
/// # Safety
///
/// .
pub unsafe fn create_concrete_from_void<T: TCFType>(void_ptr: *const c_void) -> T {
    T::wrap_under_get_rule(T::Ref::from_void_ptr(void_ptr))
}
use core_foundation::base::TCFType;
pub(crate) use impl_objc_compatability;
