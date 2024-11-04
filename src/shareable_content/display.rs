use core::fmt;

use core_foundation::base::UInt32;
use core_graphics::geometry::CGRect;
#[allow(clippy::module_name_repetitions)]
pub use internal::{SCDisplay, SCDisplayRef};

use objc::{sel, sel_impl};

use crate::utils::objc::get_property;

mod internal {

    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType, impl_TCFType,
    };

    #[repr(C)]
    pub struct __SCDisplayRef(c_void);
    extern "C" {
        pub fn SCDisplayGetTypeID() -> CFTypeID;
    }
    pub type SCDisplayRef = *mut __SCDisplayRef;

    declare_TCFType! {SCDisplay, SCDisplayRef}
    impl_TCFType!(SCDisplay, SCDisplayRef, SCDisplayGetTypeID);
}

impl fmt::Debug for SCDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCDisplay")
            .field("display_id", &self.display_id())
            .field("frame", &self.frame())
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}

impl SCDisplay {
    pub fn display_id(&self) -> UInt32 {
        get_property(self, sel!(displayID))
    }
    pub fn frame(&self) -> CGRect {
        get_property(self, sel!(frame))
    }
    pub fn height(&self) -> UInt32 {
        get_property(self, sel!(height))
    }
    pub fn width(&self) -> UInt32 {
        get_property(self, sel!(width))
    }
}
#[cfg(test)]
mod sc_display_test {

    use crate::shareable_content::SCShareableContent;

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_properties() {
        let content = SCShareableContent::get().expect("Should work");
        let displays = content.displays();
        assert!(!displays.is_empty());
        for d in displays {
            println!("Display: {d:#?}");
        }
    }
}
