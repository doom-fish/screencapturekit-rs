//! Color and pixel format configuration
//!
//! Methods for configuring color space, pixel format, and background color.

use crate::utils::{
    ffi_string::{ffi_string_from_buffer, SMALL_BUFFER_SIZE},
    four_char_code::FourCharCode,
};

const DEFAULT_ALPHA: f32 = 1.0;
type BackgroundColor = (f32, f32, f32, f32);

use super::{internal::SCStreamConfiguration, pixel_format::PixelFormat};

impl SCStreamConfiguration {
    /// Set the pixel format for captured frames
    ///
    /// Streams created via [`Self::new`] / [`Self::default`] are pinned to
    /// [`PixelFormat::BGRA`] at construction time, so calling this method is
    /// only required when you want a non-BGRA format (e.g. YUV `420v` for
    /// video encoding, or `l10r` for HDR). Apple's runtime default for
    /// `SCStreamConfiguration()` varies by macOS release — see
    /// [`PixelFormat::BGRA`] for context.
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

    /// Set the background color for captured content with an explicit alpha value.
    ///
    /// Available on macOS 13.0+
    pub fn set_background_color_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_background_color(self.as_ptr(), r, g, b, a);
        }
        self
    }

    /// Set the background color for captured content.
    ///
    /// This convenience overload uses an opaque alpha channel (`1.0`).
    pub fn set_background_color(&mut self, r: f32, g: f32, b: f32) -> &mut Self {
        self.set_background_color_rgba(r, g, b, DEFAULT_ALPHA)
    }

    /// Set the background color with an explicit alpha value (builder pattern).
    #[must_use]
    pub fn with_background_color_rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.set_background_color_rgba(r, g, b, a);
        self
    }

    /// Set the background color (builder pattern).
    ///
    /// This convenience overload uses an opaque alpha channel (`1.0`).
    #[must_use]
    pub fn with_background_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.set_background_color(r, g, b);
        self
    }

    /// Get the current background color, if it was set through this wrapper.
    pub fn background_color(&self) -> Option<BackgroundColor> {
        let mut r = 0.0f32;
        let mut g = 0.0f32;
        let mut b = 0.0f32;
        let mut a = 0.0f32;
        // The value is read back from the Swift bridge's per-configuration
        // state (keyed by object identity and released with the configuration),
        // so there is no Rust-side cache to leak or to go stale on pointer reuse.
        let was_set = unsafe {
            crate::ffi::sc_stream_configuration_get_background_color(
                self.as_ptr(),
                &mut r,
                &mut g,
                &mut b,
                &mut a,
            )
        };
        was_set.then_some((r, g, b, a))
    }

    /// Set the color space name for captured content.
    ///
    /// Available on macOS 13.0+
    ///
    /// If `name` contains an interior NUL byte it cannot be converted to a C
    /// string and the call is silently ignored (the configuration is left
    /// unchanged). Valid color-space names never contain NUL bytes.
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

    /// Set the color space name (builder pattern).
    #[must_use]
    pub fn with_color_space_name(mut self, name: &str) -> Self {
        self.set_color_space_name(name);
        self
    }

    /// Get the color space name for captured content.
    pub fn color_space_name(&self) -> Option<String> {
        unsafe {
            ffi_string_from_buffer(SMALL_BUFFER_SIZE, |buf, len| {
                crate::ffi::sc_stream_configuration_get_color_space_name(self.as_ptr(), buf, len)
            })
        }
    }

    /// Set the color matrix for captured content
    ///
    /// Available on macOS 13.0+. The matrix should be a 3x3 array in row-major order.
    ///
    /// If `matrix` contains an interior NUL byte it cannot be converted to a C
    /// string and the call is silently ignored (the configuration is left
    /// unchanged). Valid matrix-name strings never contain NUL bytes.
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

    /// Get the color matrix for captured content.
    ///
    /// Returns the color matrix as a string, or `None` if not set.
    pub fn color_matrix(&self) -> Option<String> {
        unsafe {
            ffi_string_from_buffer(SMALL_BUFFER_SIZE, |buf, len| {
                crate::ffi::sc_stream_configuration_get_color_matrix(self.as_ptr(), buf, len)
            })
        }
    }

    /// Set the color matrix (builder pattern)
    #[must_use]
    pub fn with_color_matrix(mut self, matrix: &str) -> Self {
        self.set_color_matrix(matrix);
        self
    }
}
