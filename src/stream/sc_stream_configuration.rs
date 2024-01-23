mod internal {

    #![allow(non_snake_case)]
    use objc::{
        runtime::{Object, Sel},
        *,
    };

    use std::{collections::HashMap, error::Error, ffi::c_void, sync::OnceLock};

    use core_foundation::{base::*, declare_TCFType, impl_TCFType};

    use crate::utils::objc::impl_objc_compatability;

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
    impl_objc_compatability!(SCConfiguration, __SCConfigurationRef);
    pub(crate) fn init() -> SCConfiguration {
        unsafe {
            let ptr: *mut Object = msg_send![class!(SCConfiguration), alloc];
            let ptr: SCConfigurationRef = msg_send![ptr, init];
            SCConfiguration::wrap_under_create_rule(ptr)
        }
    }
    fn config_selectors() -> &'static HashMap<&'static str, Sel> {
        static SELECTORS: OnceLock<HashMap<&str, Sel>> = OnceLock::new();
        SELECTORS.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("setWidth", sel!(setWidth));
            m.insert("setHeight", sel!(setHeight));
            m
        })
    }
    pub(crate) fn set<T: MessageArguments>(
        config: &SCConfiguration,
        selector: &str,
        value: T,
    ) -> Result<(), String> {
        unsafe {
            if let Some(sel) = config_selectors().get(selector) {
                objc::__send_message(config, *sel, value).map_err(|e| e.to_string())
            } else {
                Err(format!("unknown configuration selector: {selector}"))
            }
        }
    }
}
pub use internal::SCConfiguration;

impl SCConfiguration {
    pub fn new() -> Self {
        internal::init()
    }
    pub fn set_width(&self, width: u32) -> &Self {
        &self
    }
}

impl Default for SCConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
