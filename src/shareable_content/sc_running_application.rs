mod internal {
    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{base::*, *};
    #[repr(C)]
    pub struct __SCRunningApplicationRef(c_void);
    extern "C" {
        pub fn SCRunningApplicationGetTypeID() -> CFTypeID;
    }
    pub type SCRunningApplicationRef = *mut __SCRunningApplicationRef;

    declare_TCFType! {SCRunningApplication, SCRunningApplicationRef}
    impl_TCFType!(
        SCRunningApplication,
        SCRunningApplicationRef,
        SCRunningApplicationGetTypeID
    );
}
use core_foundation::{
    base::{SInt32, TCFType},
    string::{CFString, CFStringRef},
};
pub use internal::{SCRunningApplication, SCRunningApplicationRef};
use objc::{msg_send, *};

use crate::utils::objc::SendableObjc;

impl SCRunningApplication {
    pub fn get_process_id(&self) -> SInt32 {
        unsafe { msg_send![self.to_sendable(), processID] }
    }
    pub fn get_application_name(&self) -> String {
        unsafe {
            let ptr: CFStringRef = msg_send![self.to_sendable(), applicationName];
            if ptr.is_null() {
                "".to_owned()
            } else {
                CFString::wrap_under_get_rule(ptr).to_string()
            }
        }
    }
    pub fn get_bundle_identifier(&self) -> String {
        unsafe {
            let ptr: CFStringRef = msg_send![self.to_sendable(), bundleIdentifier];
            if ptr.is_null() {
                "".to_owned()
            } else {
                CFString::wrap_under_get_rule(ptr).to_string()
            }
        }
    }
}
