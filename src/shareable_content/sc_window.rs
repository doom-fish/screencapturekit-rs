mod internal {
    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{base::*, *};
    #[repr(C)]
    pub struct __SCWindowRef(c_void);
    extern "C" {
        pub fn SCWindowGetTypeID() -> CFTypeID;
    }
    pub type SCWindowRef = *mut __SCWindowRef;

    declare_TCFType! {SCWindow, SCWindowRef}
    impl_TCFType!(SCWindow, SCWindowRef, SCWindowGetTypeID);
}
use core_foundation::{
    base::{TCFType, UInt32},
    string::{CFString, CFStringRef},
};
use core_graphics::geometry::CGRect;
pub use internal::{SCWindow, SCWindowRef};

use objc::{msg_send, *};

use crate::utils::objc::SendableObjc;

use super::sc_running_application::{SCRunningApplication, SCRunningApplicationRef};

impl SCWindow {
    pub fn get_owning_application(&self) -> Option<SCRunningApplication> {
        unsafe {
            let ptr: SCRunningApplicationRef = msg_send![self.to_sendable(), owningApplication];
            if ptr.is_null() {
                None
            } else {
                Some(SCRunningApplication::wrap_under_get_rule(ptr))
            }
        }
    }
    pub fn get_window_layer(&self) -> UInt32 {
        unsafe { msg_send![self.to_sendable(), windowLayer] }
    }
    pub fn get_window_id(&self) -> UInt32 {
        unsafe { msg_send![self.to_sendable(), windowID] }
    }
    pub fn get_frame(&self) -> CGRect {
        unsafe { msg_send![self.to_sendable(), frame] }
    }
    pub fn get_title(&self) -> String {
        unsafe {
            let ptr: CFStringRef = msg_send![self.to_sendable(), title];
            if ptr.is_null() {
                "".to_owned()
            } else {
                CFString::wrap_under_get_rule(ptr).to_string()
            }
        }
    }
    pub fn get_is_on_screen(&self) -> bool {
        unsafe { msg_send![self.to_sendable(), isOnScreen] }
    }
    pub fn get_is_active(&self) -> bool {
        unsafe { msg_send![self.to_sendable(), isActive] }
    }
}
