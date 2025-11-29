//! Dimension and scaling configuration for stream capture
//!
//! This module provides methods to configure the output dimensions, scaling behavior,
//! and source/destination rectangles for captured streams.

use crate::cg::CGRect;

use super::internal::SCStreamConfiguration;

impl SCStreamConfiguration {
    /// Set the output width in pixels
    ///
    /// The width determines the width of captured frames.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_width(1920);
    /// assert_eq!(config.width(), 1920);
    /// ```
    pub fn set_width(&mut self, width: u32) -> &mut Self {
        // FFI expects isize; u32 may wrap on 32-bit platforms (acceptable)
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::sc_stream_configuration_set_width(self.as_ptr(), width as isize);
        }
        self
    }

    /// Set the output width in pixels (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_width(1920)
    ///     .with_height(1080);
    /// ```
    #[must_use]
    pub fn with_width(mut self, width: u32) -> Self {
        self.set_width(width);
        self
    }

    /// Get the configured output width in pixels
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_width(1920);
    /// assert_eq!(config.width(), 1920);
    /// ```
    pub fn width(&self) -> u32 {
        // FFI returns isize but width is always positive and fits in u32
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_stream_configuration_get_width(self.as_ptr()) as u32
        }
    }

    /// Set the output height in pixels
    ///
    /// The height determines the height of captured frames.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_height(1080);
    /// assert_eq!(config.height(), 1080);
    /// ```
    pub fn set_height(&mut self, height: u32) -> &mut Self {
        // FFI expects isize; u32 may wrap on 32-bit platforms (acceptable)
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::sc_stream_configuration_set_height(self.as_ptr(), height as isize);
        }
        self
    }

    /// Set the output height in pixels (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_width(1920)
    ///     .with_height(1080);
    /// ```
    #[must_use]
    pub fn with_height(mut self, height: u32) -> Self {
        self.set_height(height);
        self
    }

    /// Get the configured output height in pixels
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_height(1080);
    /// assert_eq!(config.height(), 1080);
    /// ```
    pub fn height(&self) -> u32 {
        // FFI returns isize but height is always positive and fits in u32
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_stream_configuration_get_height(self.as_ptr()) as u32
        }
    }

    /// Enable or disable scaling to fit the output dimensions
    ///
    /// When enabled, the source content will be scaled to fit within the
    /// configured width and height, potentially changing aspect ratio.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_scales_to_fit(true);
    /// assert!(config.scales_to_fit());
    /// ```
    pub fn set_scales_to_fit(&mut self, scales_to_fit: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_scales_to_fit(self.as_ptr(), scales_to_fit);
        }
        self
    }

    /// Enable or disable scaling to fit (builder pattern)
    #[must_use]
    pub fn with_scales_to_fit(mut self, scales_to_fit: bool) -> Self {
        self.set_scales_to_fit(scales_to_fit);
        self
    }

    /// Check if scaling to fit is enabled
    pub fn scales_to_fit(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_scales_to_fit(self.as_ptr()) }
    }

    /// Set the source rectangle to capture
    ///
    /// Defines which portion of the source content to capture. Coordinates are
    /// relative to the source content's coordinate system.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    /// use screencapturekit::cg::CGRect;
    ///
    /// // Capture only top-left quarter of screen
    /// let rect = CGRect::new(0.0, 0.0, 960.0, 540.0);
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_source_rect(rect);
    /// ```
    pub fn set_source_rect(&mut self, source_rect: CGRect) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_source_rect(
                self.as_ptr(),
                source_rect.x,
                source_rect.y,
                source_rect.width,
                source_rect.height,
            );
        }
        self
    }

    /// Set the source rectangle (builder pattern)
    #[must_use]
    pub fn with_source_rect(mut self, source_rect: CGRect) -> Self {
        self.set_source_rect(source_rect);
        self
    }

    /// Get the configured source rectangle
    pub fn source_rect(&self) -> CGRect {
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut width = 0.0;
            let mut height = 0.0;
            crate::ffi::sc_stream_configuration_get_source_rect(
                self.as_ptr(),
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            );
            CGRect::new(x, y, width, height)
        }
    }

    /// Set the destination rectangle for captured content
    ///
    /// Defines where the captured content will be placed in the output frame.
    /// Useful for picture-in-picture or multi-source compositions.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    /// use screencapturekit::cg::CGRect;
    ///
    /// // Place captured content in top-left corner
    /// let rect = CGRect::new(0.0, 0.0, 640.0, 480.0);
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_destination_rect(rect);
    /// ```
    pub fn set_destination_rect(&mut self, destination_rect: CGRect) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_destination_rect(
                self.as_ptr(),
                destination_rect.x,
                destination_rect.y,
                destination_rect.width,
                destination_rect.height,
            );
        }
        self
    }

    /// Set the destination rectangle (builder pattern)
    #[must_use]
    pub fn with_destination_rect(mut self, destination_rect: CGRect) -> Self {
        self.set_destination_rect(destination_rect);
        self
    }

    /// Get the configured destination rectangle
    pub fn destination_rect(&self) -> CGRect {
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut width = 0.0;
            let mut height = 0.0;
            crate::ffi::sc_stream_configuration_get_destination_rect(
                self.as_ptr(),
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            );
            CGRect::new(x, y, width, height)
        }
    }

    /// Preserve aspect ratio when scaling
    ///
    /// When enabled, the content will be scaled while maintaining its original
    /// aspect ratio, potentially adding letterboxing or pillarboxing.
    ///
    /// Note: This property requires macOS 14.0+. On older versions, the setter
    /// is a no-op and the getter returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_preserves_aspect_ratio(true);
    /// // Returns true on macOS 14.0+, false on older versions
    /// let _ = config.preserves_aspect_ratio();
    /// ```
    pub fn set_preserves_aspect_ratio(&mut self, preserves_aspect_ratio: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_preserves_aspect_ratio(
                self.as_ptr(),
                preserves_aspect_ratio,
            );
        }
        self
    }

    /// Preserve aspect ratio when scaling (builder pattern)
    #[must_use]
    pub fn with_preserves_aspect_ratio(mut self, preserves_aspect_ratio: bool) -> Self {
        self.set_preserves_aspect_ratio(preserves_aspect_ratio);
        self
    }

    /// Check if aspect ratio preservation is enabled
    pub fn preserves_aspect_ratio(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_preserves_aspect_ratio(self.as_ptr()) }
    }

    /// Preserve aspect ratio when scaling (alternative API)
    ///
    /// This is an alternative to `set_preserves_aspect_ratio` for compatibility.
    pub fn set_preserve_aspect_ratio(&mut self, preserve_aspect_ratio: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_preserve_aspect_ratio(
                self.as_ptr(),
                preserve_aspect_ratio,
            );
        }
        self
    }

    /// Preserve aspect ratio (builder pattern, alternative API)
    #[must_use]
    pub fn with_preserve_aspect_ratio(mut self, preserve_aspect_ratio: bool) -> Self {
        self.set_preserve_aspect_ratio(preserve_aspect_ratio);
        self
    }

    /// Check if aspect ratio preservation is enabled (alternative API)
    pub fn preserve_aspect_ratio(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_preserve_aspect_ratio(self.as_ptr()) }
    }

    /// Enable or disable increased resolution for Retina displays
    ///
    /// When enabled, the captured content will be scaled up to match the backing
    /// scale factor of Retina displays, providing higher quality output.
    /// Available on macOS 13.0+
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_increase_resolution_for_retina_displays(true);
    /// // Note: Getter may not return the set value on all macOS versions
    /// let _ = config.increase_resolution_for_retina_displays();
    /// ```
    pub fn set_increase_resolution_for_retina_displays(&mut self, increase_resolution: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_increase_resolution_for_retina_displays(
                self.as_ptr(),
                increase_resolution,
            );
        }
        self
    }

    /// Enable increased resolution for Retina displays (builder pattern)
    #[must_use]
    pub fn with_increase_resolution_for_retina_displays(mut self, increase_resolution: bool) -> Self {
        self.set_increase_resolution_for_retina_displays(increase_resolution);
        self
    }

    /// Check if increased resolution for Retina displays is enabled
    pub fn increase_resolution_for_retina_displays(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_increase_resolution_for_retina_displays(
                self.as_ptr(),
            )
        }
    }
}
