//! Color and pixel format configuration
//!
//! Methods for configuring color space, pixel format, and background color.

use crate::utils::four_char_code::FourCharCode;

use super::{internal::SCStreamConfiguration, pixel_format::PixelFormat};

impl SCStreamConfiguration {
    /// Set the pixel format for captured frames
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::stream::configuration::{SCStreamConfiguration, PixelFormat};
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_pixel_format(PixelFormat::BGRA);
    /// ```
    pub fn set_pixel_format(&mut self, pixel_format: PixelFormat) -> &mut Self {
        let four_char_code: FourCharCode = pixel_format.into();
        unsafe {
            crate::ffi::sc_stream_configuration_set_pixel_format(
                self.as_ptr(),
                four_char_code.as_u32(),
            );
        }
        self
    }

    /// Set the pixel format (builder pattern)
    #[must_use]
    pub fn with_pixel_format(mut self, pixel_format: PixelFormat) -> Self {
        self.set_pixel_format(pixel_format);
        self
    }

    /// Get the current pixel format
    pub fn pixel_format(&self) -> PixelFormat {
        unsafe {
            let value = crate::ffi::sc_stream_configuration_get_pixel_format(self.as_ptr());
            PixelFormat::from(value)
        }
    }

    /// Set the background color for captured content
    ///
    /// Available on macOS 13.0+
    ///
    /// # Parameters
    ///
    /// - `r`: Red component (0.0 - 1.0)
    /// - `g`: Green component (0.0 - 1.0)
    /// - `b`: Blue component (0.0 - 1.0)
    pub fn set_background_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_background_color(self.as_ptr(), r, g, b);
        }
        self
    }

    /// Set the background color (builder pattern)
    #[must_use]
    pub fn with_background_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.set_background_color(r, g, b);
        self
    }

    /// Set the color space name for captured content
    ///
    /// Available on macOS 13.0+
    pub fn set_color_space_name(&mut self, name: &str) -> &mut Self {
        if let Ok(c_name) = std::ffi::CString::new(name) {
            unsafe {
                crate::ffi::sc_stream_configuration_set_color_space_name(
                    self.as_ptr(),
                    c_name.as_ptr(),
                );
            }
        }
        self
    }

    /// Set the color space name (builder pattern)
    #[must_use]
    pub fn with_color_space_name(mut self, name: &str) -> Self {
        self.set_color_space_name(name);
        self
    }

    /// Set the color matrix for captured content
    ///
    /// Available on macOS 13.0+. The matrix should be a 3x3 array in row-major order.
    pub fn set_color_matrix(&mut self, matrix: &str) -> &mut Self {
        if let Ok(c_matrix) = std::ffi::CString::new(matrix) {
            unsafe {
                crate::ffi::sc_stream_configuration_set_color_matrix(
                    self.as_ptr(),
                    c_matrix.as_ptr(),
                );
            }
        }
        self
    }

    /// Set the color matrix (builder pattern)
    #[must_use]
    pub fn with_color_matrix(mut self, matrix: &str) -> Self {
        self.set_color_matrix(matrix);
        self
    }
}
