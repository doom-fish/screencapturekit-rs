mod internal {

    #![allow(non_snake_case)]
    use std::os::raw::c_void;

    use core_foundation::{base::*, *};

    use crate::utils::objc::impl_objc_compatability;
    #[repr(C)]
    pub struct __CMSampleBufferRef(c_void);
    extern "C" {
        pub fn CMSampleBufferGetTypeID() -> CFTypeID;
    }
    pub type CMSampleBufferRef = *mut __CMSampleBufferRef;

    declare_TCFType! {CMSampleBuffer, CMSampleBufferRef}
    impl_TCFType!(CMSampleBuffer, CMSampleBufferRef, CMSampleBufferGetTypeID);   impl_objc_compatability!(CMSampleBuffer, __CMSampleBufferRef);
}

pub use internal::{CMSampleBuffer, CMSampleBufferRef};
