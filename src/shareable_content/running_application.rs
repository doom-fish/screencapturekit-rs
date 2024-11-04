mod internal {
    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType, impl_TCFType,
    };

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
use core::fmt;

use core_foundation::base::SInt32;
#[allow(clippy::module_name_repetitions)]
pub use internal::{SCRunningApplication, SCRunningApplicationRef};
use objc::{sel, sel_impl};

use crate::utils::objc::{get_property, get_string_property};

impl SCRunningApplication {
    pub fn process_id(&self) -> SInt32 {
        get_property(self, sel!(processID))
    }
    pub fn application_name(&self) -> String {
        get_string_property(self, sel!(applicationName))
    }
    pub fn bundle_identifier(&self) -> String {
        get_string_property(self, sel!(bundleIdentifier))
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

// #[cfg(test)]
// mod sc_running_application_test {
//
//     use crate::shareable_content::sc_shareable_content::SCShareableContent;
//
//     #[test]
//     #[cfg_attr(feature = "ci", ignore)]
//     fn test_properties() {
//         let content = SCShareableContent::get().expect("Should work");
//         let applications = content.applications();
//         assert!(!applications.is_empty());
//         for application in applications {
//             println!("Application: {application:#?}");
//         }
//     }
// }
