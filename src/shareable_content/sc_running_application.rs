mod internal {
    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{base::*, *};

    use crate::utils::objc::impl_objc_compatability;
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
    impl_objc_compatability!(SCRunningApplication, __SCRunningApplicationRef);
}
use core::fmt;

use core_foundation::{
    base::{SInt32, TCFType},
    string::{CFString, CFStringRef},
};
pub use internal::{SCRunningApplication, SCRunningApplicationRef};
use objc::{msg_send, *};

impl SCRunningApplication {
    pub fn process_id(&self) -> SInt32 {
        unsafe { msg_send![self, processID] }
    }
    pub fn application_name(&self) -> String {
        unsafe {
            let ptr: CFStringRef = msg_send![self, applicationName];
            if ptr.is_null() {
                "".to_owned()
            } else {
                CFString::wrap_under_get_rule(ptr).to_string()
            }
        }
    }
    pub fn bundle_identifier(&self) -> String {
        unsafe {
            let ptr: CFStringRef = msg_send![self, bundleIdentifier];
            if ptr.is_null() {
                "".to_owned()
            } else {
                CFString::wrap_under_get_rule(ptr).to_string()
            }
        }
    }
}

impl fmt::Debug for SCRunningApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCRunningApplication")
            .field("process_id", &self.process_id())
            .field("application_name", &self.application_name())
            .field("bundle_identifier", &self.bundle_identifier())
            .finish()
    }
}

#[cfg(test)]
mod sc_running_application_test {

    use crate::shareable_content::sc_shareable_content::SCShareableContent;

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_properties() {
        let content = SCShareableContent::get().expect("Should work");
        let applications = content.applications();
        assert!(!applications.is_empty());
        for application in applications {
            println!("Application: {:#?}", application);
        }
    }
}
