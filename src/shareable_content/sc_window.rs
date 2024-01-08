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
pub use internal::{SCWindow, SCWindowRef};
