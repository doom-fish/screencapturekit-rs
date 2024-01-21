mod internal {

    #![allow(non_snake_case)]
    use objc::*;

    use std::ffi::c_void;

    use core_foundation::{base::*, declare_TCFType, impl_TCFType};

    use crate::utils::objc::impl_deref;

    #[repr(C)]
    pub struct __SCConfigurationRef(c_void);
    extern "C" {
        pub fn SCConfigurationGetTypeID() -> CFTypeID;
    }

    pub type SCConfigurationRef = *mut __SCConfigurationRef;

    declare_TCFType! {SCConfiguration, SCConfigurationRef}
    impl_TCFType!(
        SCConfiguration,
        SCConfigurationRef,
        SCConfigurationGetTypeID
    );
    impl_deref!(SCConfiguration);
    pub(crate) fn init() -> SCConfiguration {
        unsafe {
            let ptr: SCConfigurationRef = msg_send![class!(SCConfiguration), alloc];
            let ptr = msg_send![ptr, init];
            SCConfiguration::wrap_under_create_rule(ptr)
        }
    }
}
pub use internal::SCConfiguration;

impl SCConfiguration {
    fn new() -> Self {
        internal::init()
    }
}
