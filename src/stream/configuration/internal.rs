#![allow(non_snake_case)]
use objc::{class, msg_send, runtime::Object, sel, sel_impl};

use std::ffi::c_void;

use core_foundation::{
    base::{CFTypeID, TCFType},
    declare_TCFType, impl_TCFType,
};

#[repr(C)]
pub struct __SCStreamConfigurationRef(c_void);
extern "C" {
    pub fn SCStreamConfigurationGetTypeID() -> CFTypeID;
}

pub type SCStreamConfigurationRef = *mut __SCStreamConfigurationRef;

declare_TCFType! {SCStreamConfiguration, SCStreamConfigurationRef}
impl_TCFType!(
    SCStreamConfiguration,
    SCStreamConfigurationRef,
    SCStreamConfigurationGetTypeID
);

impl SCStreamConfiguration {
    pub fn internal_init() -> Self {
        unsafe {
            let ptr: *mut Object = msg_send![class!(SCStreamConfiguration), alloc];
            let ptr: SCStreamConfigurationRef = msg_send![ptr, init];
            Self::wrap_under_create_rule(ptr)
        }
    }
}
