mod internal {
    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{base::*, *};
    #[repr(C)]
    pub struct __SCDisplayRef(c_void);
    extern "C" {
        pub fn SCDisplayGetTypeID() -> CFTypeID;
    }
    pub type SCDisplayRef = *mut __SCDisplayRef;

    declare_TCFType! {SCDisplay, SCDisplayRef}
    impl_TCFType!(SCDisplay, SCDisplayRef, SCDisplayGetTypeID);
}
pub use internal::{SCDisplay, SCDisplayRef};
