mod internal {

    #![allow(non_snake_case)]
    use objc::{
        class, msg_send,
        runtime::{Object, Sel},
        sel, sel_impl,
    };

    use std::{collections::HashMap, ffi::c_void, sync::OnceLock};

    use core_foundation::{base::*, declare_TCFType, impl_TCFType};

    use crate::utils::objc::MessageForTFType;

    #[repr(C)]
    pub struct __SCStreamConfigurationRef(c_void);
    extern "C" {
        pub fn SCStreamConfigurationGetTypeID() -> CFTypeID;
    }

    pub type SCStreamConfigurationRef = *mut __SCStreamConfigurationRef;

    declare_TCFType! {SCStreamConfiguration, SCStreamConfigurationRef}
    impl_TCFType!(
        SCStreamConfiguration,
        SCStreamConfigurationRef,
        SCStreamConfigurationGetTypeID
    );

    pub fn init() -> SCStreamConfiguration {
        unsafe {
            let ptr: *mut Object = msg_send![class!(SCStreamConfiguration), alloc];
            let ptr: SCStreamConfigurationRef = msg_send![ptr, init];
            SCStreamConfiguration::wrap_under_create_rule(ptr)
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
    pub fn set<T>(
        config: &mut SCStreamConfiguration,
        selector: &str,
        value: T,
    ) -> Result<(), String> {
        unsafe {
            config_selectors().get(selector).map_or_else(
                || Err(format!("unknown configuration selector: {selector}")),
                |sel| {
                    objc::__send_message(config.as_sendable(), *sel, (value,))
                        .map_err(|e| e.to_string())
                },
            )
        }
    }
}

pub use internal::SCStreamConfiguration;

impl SCStreamConfiguration {
    #[must_use]
    pub fn new() -> Self {
        internal::init()
    }

    #[must_use]
    pub fn set_width(mut self, width: u32) -> Self {
        internal::set(&mut self, "setWidth", width).unwrap_or_else(|e| {
            println!("{e}");
        });
        self
    }
    #[must_use]
    pub fn set_height(mut self, height: u32) -> Self {
        internal::set(&mut self, "setHeight", height).unwrap_or_else(|e| {
            println!("{e}");
        });
        self
    }
}

impl Default for SCStreamConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod sc_stream_configuration_test {
    use super::SCStreamConfiguration;

    #[test]
    fn test_setters() {
        let _ = SCStreamConfiguration::new().set_width(100).set_height(100);
    }
}
