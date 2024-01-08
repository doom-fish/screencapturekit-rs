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
pub use internal::{SCRunningApplication, SCRunningApplicationRef};
