mod internal {

    #![allow(non_snake_case)]
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};

    use std::ffi::c_void;

    use core_foundation::{
        base::{CFTypeID, TCFType},
        declare_TCFType, impl_TCFType,
    };

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
}

use core_foundation::{boolean::CFBoolean, error::CFError};
pub use internal::SCStreamConfiguration;
use objc::{sel, sel_impl};

use crate::utils::objc::{get_property, set_property};

impl SCStreamConfiguration {
    #[must_use]
    pub fn new() -> Self {
        internal::init()
    }

    /// Sets the width of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_width(mut self, width: u32) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setWidth:), width)?;
        Ok(self)
    }
    /// Sets the height of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_height(mut self, height: u32) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setHeight:), height)?;
        Ok(self)
    }

    /// Sets capturesAudio of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_captures_audio(mut self, captures_audio: bool) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setCapturesAudio:), captures_audio)?;
        Ok(self)
    }
    pub fn get_captures_audio(&self) -> bool {
        get_property(self, sel!(capturesAudio))
    }
    /// Sets capturesAudio of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_excludes_current_process_audio(
        mut self,
        excludes_current_process_audio: bool,
    ) -> Result<Self, CFError> {
        set_property(
            &mut self,
            sel!(setExcludesCurrentProcessAudio:),
            CFBoolean::from(excludes_current_process_audio),
        )?;
        Ok(self)
    }
    /// Sets the channel count of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_channel_count(mut self, channel_count: u8) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setChannelCount:), channel_count)?;
        Ok(self)
    }
    pub fn get_channel_count(&self) -> u8 {
        get_property(self, sel!(channelCount))
    }
}

impl Default for SCStreamConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod sc_stream_configuration_test {
    use core_foundation::error::CFError;

    use super::SCStreamConfiguration;

    #[test]
    fn test_setters() -> Result<(), CFError> {
        SCStreamConfiguration::new()
            .set_captures_audio(true)?
            .set_width(100)?
            .set_height(100)?;
        Ok(())
    }
}
