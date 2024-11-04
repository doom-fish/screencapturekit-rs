use core_foundation::error::CFError;
use core_graphics::display::CGRect;
use objc::{sel, sel_impl};

use super::internal::SCStreamConfiguration;
use crate::utils::objc::{get_property, set_property};

impl SCStreamConfiguration {
    /// Sets the width of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_width(mut self, width: u32) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setWidth:), width)?;
        Ok(self)
    }
    pub fn get_width(&self) -> u32 {
        get_property(self, sel!(width))
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
    pub fn get_height(&self) -> u32 {
        get_property(self, sel!(height))
    }

    /// Sets scalesToFit for this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_scales_to_fit(mut self, scales_to_fit: bool) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setScalesToFit:), scales_to_fit)?;
        Ok(self)
    }
    pub fn get_scales_to_fit(&self) -> bool {
        get_property(self, sel!(scalesToFit))
    }

    /// Sets sourceRect for this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_source_rect(mut self, source_rect: CGRect) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setSourceRect:), source_rect)?;
        Ok(self)
    }
    pub fn get_source_rect(&self) -> CGRect {
        get_property(self, sel!(sourceRect))
    }

    /// Sets destinationRect for this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_destination_rect(mut self, destination_rect: CGRect) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setDestinationRect:), destination_rect)?;
        Ok(self)
    }
    pub fn get_destination_rect(&self) -> CGRect {
        get_property(self, sel!(destinationRect))
    }

    /// Sets preservesAspectRatio for this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_preserves_aspect_ratio(
        mut self,
        preserves_aspect_ratio: bool,
    ) -> Result<Self, CFError> {
        set_property(
            &mut self,
            sel!(setPreservesAspectRatio:),
            preserves_aspect_ratio,
        )?;
        Ok(self)
    }
    pub fn get_preserves_aspect_ratio(&self) -> bool {
        get_property(self, sel!(preservesAspectRatio))
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
