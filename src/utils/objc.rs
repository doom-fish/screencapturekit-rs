use std::ffi::c_void;

use core_foundation::base::*;

pub trait MessageForTFType {
    fn as_sendable(&self) -> *mut Object;
}
pub trait MessageForTFTypeRef {
    fn as_sendable(&self) -> *mut Object;
}

impl<T: TCFType> MessageForTFType for T {
    fn as_sendable(&self) -> *mut Object {
        self.as_CFTypeRef() as *mut Object
    }
}
impl<T: TCFTypeRef> MessageForTFTypeRef for T {
    fn as_sendable(&self) -> *mut Object {
        self as *const _ as *mut Object
    }
}

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
use objc::runtime::Object;
