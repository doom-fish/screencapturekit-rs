use core_foundation::{boolean::CFBoolean, error::CFError};
use objc::{sel, sel_impl};

use super::internal::SCStreamConfiguration;
use crate::utils::objc::{get_property, set_property};

impl SCStreamConfiguration {
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
