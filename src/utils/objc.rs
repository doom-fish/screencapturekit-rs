use core_foundation::{
    array::CFArray,
    base::{TCFType, TCFTypeRef},
    string::CFString,
};
use objc::runtime::{Object, Sel};
use std::ffi::c_void;

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

/// .
///
/// # Errors
///
/// This function will return an error if .
#[allow(clippy::module_name_repetitions)]
pub fn objc_set_property<TSubject: TCFType, TValue>(
    subject: &mut TSubject,
    selector: Sel,
    value: TValue,
) -> Result<(), String> {
    unsafe {
        objc::__send_message(subject.as_sendable(), selector, (value,)).map_err(|e| e.to_string())
    }
}
/// .
///
/// # Panics
///
/// Panics if .
#[allow(clippy::module_name_repetitions)]
pub fn objc_get_property<TSubject: TCFType, TReturn: 'static>(
    subject: &TSubject,
    selector: Sel,
) -> TReturn {
    unsafe {
        objc::__send_message(subject.as_sendable(), selector, ())
            .expect("should work! Otherwise illegal selector!")
    }
}
#[allow(clippy::module_name_repetitions)]
pub fn objc_get_cftype_property<TReturn: 'static + TCFType, TSubject: TCFType>(
    subject: &TSubject,
    selector: Sel,
) -> Option<TReturn> {
    unsafe {
        let return_ref: *const c_void = objc_get_property(subject, selector);
        if return_ref.is_null() {
            None
        } else {
            Some(TReturn::wrap_under_get_rule(TReturn::Ref::from_void_ptr(
                return_ref,
            )))
        }
    }
}
#[allow(clippy::module_name_repetitions)]
pub fn objc_get_string_property<TSubject: TCFType>(subject: &TSubject, selector: Sel) -> String {
    objc_get_cftype_property(subject, selector)
        .map_or(String::new(), |cfstring: CFString| cfstring.to_string())
}
#[allow(clippy::module_name_repetitions)]
pub fn objc_get_vec_property<TSubject: TCFType, TReturn: 'static + TCFType>(
    subject: &TSubject,
    selector: Sel,
) -> Vec<TReturn> {
    unsafe {
        CFArray::<TReturn::Ref>::wrap_under_get_rule(objc_get_property(subject, selector))
            .into_untyped()
            .iter()
            .map(|ptr| TReturn::wrap_under_get_rule(TReturn::Ref::from_void_ptr(*ptr)))
            .collect()
    }
}
