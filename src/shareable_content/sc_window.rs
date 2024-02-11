mod internal {
    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType, impl_TCFType,
    };

    #[repr(C)]
    pub struct __SCWindowRef(c_void);
    extern "C" {
        pub fn SCWindowGetTypeID() -> CFTypeID;
    }
    pub type SCWindowRef = *mut __SCWindowRef;

    declare_TCFType! {SCWindow, SCWindowRef}
    impl_TCFType!(SCWindow, SCWindowRef, SCWindowGetTypeID);
}
pub use internal::{SCWindow, SCWindowRef};
use std::fmt::{self};

use core_foundation::base::{TCFType, UInt32};
use core_graphics::geometry::CGRect;

use objc::{msg_send, sel, sel_impl};

use crate::utils::objc::{get_bool_property, get_property, get_string_property, MessageForTFType};

use super::sc_running_application::{SCRunningApplication, SCRunningApplicationRef};

impl SCWindow {
    pub fn owning_application(&self) -> SCRunningApplication {
        unsafe {
            let ptr: SCRunningApplicationRef = msg_send![self.as_sendable(), owningApplication];
            SCRunningApplication::wrap_under_get_rule(ptr)
        }
    }
    pub fn window_layer(&self) -> UInt32 {
        get_property(self, sel!(windowLayer))
    }
    pub fn window_id(&self) -> UInt32 {
        get_property(self, sel!(windowID))
    }
    pub fn get_frame(&self) -> CGRect {
        get_property(self, sel!(frame))
    }
    pub fn title(&self) -> String {
        get_string_property(self, sel!(title))
    }

    pub fn is_on_screen(&self) -> bool {
        get_bool_property(self, sel!(isOnScreen))
    }
    pub fn is_active(&self) -> bool {
        get_bool_property(self, sel!(isActive))
    }
}

impl fmt::Debug for SCWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCWindow")
            .field("title", &self.title())
            .field("window_id", &self.window_id())
            .field("window_layer", &self.window_layer())
            .field("is_on_screen", &self.is_on_screen())
            .field("is_active", &self.is_active())
            .field("owning_application", &self.owning_application())
            .finish()
    }
}

#[cfg(test)]
mod sc_window_test {

    use crate::shareable_content::{sc_shareable_content::SCShareableContent, sc_window::SCWindow};

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_properties() {
        let content = SCShareableContent::get().expect("Should work");
        let windows: Vec<SCWindow> = content.windows();
        assert!(!windows.is_empty());
        for window in windows {
            println!("Window: {window:#?}");
        }
    }
}
