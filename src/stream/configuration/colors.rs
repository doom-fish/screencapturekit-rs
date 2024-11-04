use core_foundation::{base::TCFType, error::CFError, string::CFStringRef};
use core_graphics::color::CGColor;
use core_utils_rs::four_char_code::FourCharCode;
use objc::{sel, sel_impl};

use super::{internal::SCStreamConfiguration, pixel_format::PixelFormat};
use crate::utils::{
    error::create_cf_error,
    objc::{get_cftype_property, get_property, set_property},
};

// Color Matrixes
extern "C" {
    // Specifies the YCbCr to RGB conversion matrix for HDTV digital television (ITU R 709) images.
    pub static kCGDisplayStreamYCbCrMatrix_ITU_R_709_2: CFStringRef;
    //Specifies the YCbCr to RGB conversion matrix for standard digital television (ITU R 601) images.
    pub static kCGDisplayStreamYCbCrMatrix_ITU_R_601_4: CFStringRef;
    // Specifies the YCbCR to RGB conversion matrix for 1920 x 1135 HDTV (SMPTE 240M 1995).
    pub static kCGDisplayStreamYCbCrMatrix_SMPTE_240M_1995: CFStringRef;
}

extern "C" {
    // The Display P3 color space, created by Apple.
    pub static kCGColorSpaceDisplayP3: CFStringRef;
    // The Display P3 color space, using the HLG transfer function.
    pub static kCGColorSpaceDisplayP3_HLG: CFStringRef;
    // Deprecated The Display P3 color space, using the PQ transfer function.
    pub static kCGColorSpaceDisplayP3_PQ_EOTF: CFStringRef;
    // The Display P3 color space with a linear transfer function and extended-range values.
    pub static kCGColorSpaceExtendedLinearDisplayP3: CFStringRef;
    // The standard Red Green Blue (sRGB) color space.
    pub static kCGColorSpaceSRGB: CFStringRef;
    // The sRGB color space with a linear transfer function.
    pub static kCGColorSpaceLinearSRGB: CFStringRef;
    // The extended sRGB color space.
    pub static kCGColorSpaceExtendedSRGB: CFStringRef;
    // The sRGB color space with a linear transfer function and extended-range values.
    pub static kCGColorSpaceExtendedLinearSRGB: CFStringRef;
    // The generic gray color space that has an exponential transfer function with a power of 2.2.
    pub static kCGColorSpaceGenericGrayGamma2_2: CFStringRef;
    // The gray color space using a linear transfer function.
    pub static kCGColorSpaceLinearGray: CFStringRef;
    // The extended gray color space.
    pub static kCGColorSpaceExtendedGray: CFStringRef;
    // The extended gray color space with a linear transfer function.
    pub static kCGColorSpaceExtendedLinearGray: CFStringRef;
    // The generic RGB color space with a linear transfer function.
    pub static kCGColorSpaceGenericRGBLinear: CFStringRef;
    // The generic CMYK color space.
    pub static kCGColorSpaceGenericCMYK: CFStringRef;
    // The XYZ color space, as defined by the CIE 1931 standard.
    pub static kCGColorSpaceGenericXYZ: CFStringRef;
    // The generic LAB color space.
    pub static kCGColorSpaceGenericLab: CFStringRef;
    // The ACEScg color space.
    pub static kCGColorSpaceACESCGLinear: CFStringRef;
    // The Adobe RGB (1998) color space.
    pub static kCGColorSpaceAdobeRGB1998: CFStringRef;
    // The DCI P3 color space, which is the digital cinema standard.
    pub static kCGColorSpaceDCIP3: CFStringRef;
    // The recommendation of the International Telecommunication Union (ITU) Radiocommunication sector for the BT.709 color space.
    pub static kCGColorSpaceITUR_709: CFStringRef;
    // The Reference Output Medium Metric (ROMM) RGB color space.
    pub static kCGColorSpaceROMMRGB: CFStringRef;
    // The recommendation of the International Telecommunication Union (ITU) Radiocommunication sector for the BT.2020 color space.
    pub static kCGColorSpaceITUR_2020: CFStringRef;
    // Deprecated The recommendation of the International Telecommunication Union (ITU) Radiocommunication sector for the BT.2020 color space, with the HLG transfer function.
    pub static kCGColorSpaceITUR_2020_HLG: CFStringRef;
    // Deprecated The recommendation of the International Telecommunication Union (ITU) Radiocommunication sector for the BT.2020 color space, with the PQ transfer function.
    pub static kCGColorSpaceITUR_2020_PQ_EOTF: CFStringRef;
    // The recommendation of the International Telecommunication Union (ITU) Radiocommunication sector for the BT.2020 color space, with a linear transfer function and extended range values.
    pub static kCGColorSpaceExtendedLinearITUR_2020: CFStringRef;
    // The name of the generic RGB color space.
    pub static kCGColorSpaceGenericRGB: CFStringRef;
    // The name of the generic gray color space.
    pub static kCGColorSpaceGenericGray: CFStringRef;
}

impl SCStreamConfiguration {
    /// Sets the pixel format of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_pixel_format(mut self, pixel_format: PixelFormat) -> Result<Self, CFError> {
        let four_char_code: FourCharCode = pixel_format.into();
        set_property(&mut self, sel!(setPixelFormat:), four_char_code.as_u32())?;
        Ok(self)
    }
    pub fn get_pixel_format(&self) -> PixelFormat {
        let value: u32 = get_property(self, sel!(pixelFormat));
        PixelFormat::from(value)
    }

    /// Sets the color matrix of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_color_matrix(mut self, color_matrix: CFStringRef) -> Result<Self, CFError> {
        let pixel_format = self.get_pixel_format();
        if pixel_format != PixelFormat::YCbCr_420f && pixel_format != PixelFormat::YCbCr_420v {
            return Err(create_cf_error(
                format!("color matrix can only be set for 420f and 420v formats: {pixel_format}")
                    .as_str(),
                -1,
            ));
        }
        set_property(&mut self, sel!(setColorMatrix:), color_matrix)?;
        Ok(self)
    }
    pub fn get_color_matrix(&self) -> CFStringRef {
        get_property(self, sel!(colorMatrix))
    }

    /// Sets the color space name of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_color_space_name(mut self, color_space_name: CFStringRef) -> Result<Self, CFError> {
        set_property(&mut self, sel!(setColorSpaceName:), color_space_name)?;
        Ok(self)
    }
    pub fn get_color_space_name(&self) -> CFStringRef {
        get_property(self, sel!(colorSpaceName))
    }
    /// Sets the background color of this [`SCStreamConfiguration`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_background_color(mut self, background_color: &CGColor) -> Result<Self, CFError> {
        set_property(
            &mut self,
            sel!(setBackgroundColor:),
            background_color.clone().as_CFTypeRef(),
        )?;
        Ok(self)
    }
    pub fn get_background_color(&self) -> Option<CGColor> {
        get_cftype_property(self, sel!(backgroundColor))
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
