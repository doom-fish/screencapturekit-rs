//! Builder for `SCStreamConfiguration`
//!
//! Provides a fluent builder API for constructing stream configurations.

use super::internal::SCStreamConfiguration;
use super::pixel_format::PixelFormat;
use super::advanced::SCPresenterOverlayAlertSetting;
use super::stream_properties::SCCaptureDynamicRange;
use crate::cg::CGRect;

/// Builder for creating `SCStreamConfiguration` instances
///
/// Use [`SCStreamConfiguration::builder()`] to create a new builder.
///
/// # Examples
///
/// ```rust
/// use screencapturekit::stream::configuration::SCStreamConfiguration;
///
/// let config = SCStreamConfiguration::builder()
///     .width(1920)
///     .height(1080)
///     .pixel_format(screencapturekit::stream::configuration::PixelFormat::BGRA)
///     .captures_audio(true)
///     .build();
/// ```
#[derive(Debug)]
pub struct SCStreamConfigurationBuilder {
    config: SCStreamConfiguration,
}

impl SCStreamConfigurationBuilder {
    /// Create a new builder with default configuration
    pub(crate) fn new() -> Self {
        Self {
            config: SCStreamConfiguration::internal_init(),
        }
    }

    /// Build the final configuration
    pub fn build(self) -> SCStreamConfiguration {
        self.config
    }

    // ============ Dimensions ============

    /// Set the output width in pixels
    ///
    /// # Panics
    /// Panics if width is 0.
    #[must_use]
    pub fn width(self, width: u32) -> Self {
        assert!(width > 0, "width must be greater than 0");
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::sc_stream_configuration_set_width(self.config.as_ptr(), width as isize);
        }
        self
    }

    /// Set the output height in pixels
    ///
    /// # Panics
    /// Panics if height is 0.
    #[must_use]
    pub fn height(self, height: u32) -> Self {
        assert!(height > 0, "height must be greater than 0");
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            crate::ffi::sc_stream_configuration_set_height(self.config.as_ptr(), height as isize);
        }
        self
    }

    /// Enable or disable scaling to fit the output dimensions
    #[must_use]
    pub fn scales_to_fit(self, scales: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_scales_to_fit(self.config.as_ptr(), scales);
        }
        self
    }

    /// Set the source rectangle to capture
    #[must_use]
    pub fn source_rect(self, rect: CGRect) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_source_rect(
                self.config.as_ptr(),
                rect.origin().x,
                rect.origin().y,
                rect.size().width,
                rect.size().height,
            );
        }
        self
    }

    /// Set the destination rectangle in output
    #[must_use]
    pub fn destination_rect(self, rect: CGRect) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_destination_rect(
                self.config.as_ptr(),
                rect.origin().x,
                rect.origin().y,
                rect.size().width,
                rect.size().height,
            );
        }
        self
    }

    /// Preserve aspect ratio when scaling (macOS 14.0+)
    #[must_use]
    pub fn preserves_aspect_ratio(self, preserves: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_preserves_aspect_ratio(self.config.as_ptr(), preserves);
        }
        self
    }

    // ============ Colors ============

    /// Set the pixel format for captured frames
    #[must_use]
    pub fn pixel_format(self, format: PixelFormat) -> Self {
        let code: crate::utils::four_char_code::FourCharCode = format.into();
        unsafe {
            crate::ffi::sc_stream_configuration_set_pixel_format(self.config.as_ptr(), code.as_u32());
        }
        self
    }

    /// Set the color matrix for YCbCr formats
    #[must_use]
    pub fn color_matrix(self, matrix: &str) -> Self {
        if let Ok(c_str) = std::ffi::CString::new(matrix) {
            unsafe {
                crate::ffi::sc_stream_configuration_set_color_matrix(self.config.as_ptr(), c_str.as_ptr());
            }
        }
        self
    }

    /// Set the color space name
    #[must_use]
    pub fn color_space_name(self, name: &str) -> Self {
        if let Ok(c_str) = std::ffi::CString::new(name) {
            unsafe {
                crate::ffi::sc_stream_configuration_set_color_space_name(self.config.as_ptr(), c_str.as_ptr());
            }
        }
        self
    }

    /// Set the background color (RGB components 0.0-1.0)
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn background_color(self, red: f32, green: f32, blue: f32) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_background_color(
                self.config.as_ptr(), red, green, blue,
            );
        }
        self
    }

    // ============ Audio ============

    /// Enable or disable audio capture
    #[must_use]
    pub fn captures_audio(self, captures: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_captures_audio(self.config.as_ptr(), captures);
        }
        self
    }

    /// Set the audio sample rate in Hz
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn sample_rate(self, rate: i32) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_sample_rate(self.config.as_ptr(), rate as isize);
        }
        self
    }

    /// Set the number of audio channels
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn channel_count(self, count: i32) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_channel_count(self.config.as_ptr(), count as isize);
        }
        self
    }

    /// Exclude audio from current process
    #[must_use]
    pub fn excludes_current_process_audio(self, excludes: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_excludes_current_process_audio(self.config.as_ptr(), excludes);
        }
        self
    }

    /// Enable or disable microphone capture (macOS 15.0+)
    #[must_use]
    pub fn capture_microphone(self, captures: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_captures_microphone(self.config.as_ptr(), captures);
        }
        self
    }

    /// Set the microphone capture device ID (macOS 15.0+)
    #[must_use]
    pub fn microphone_capture_device_id(self, device_id: Option<&str>) -> Self {
        unsafe {
            if let Some(id) = device_id {
                if let Ok(c_str) = std::ffi::CString::new(id) {
                    crate::ffi::sc_stream_configuration_set_microphone_capture_device_id(
                        self.config.as_ptr(), c_str.as_ptr(),
                    );
                }
            } else {
                crate::ffi::sc_stream_configuration_set_microphone_capture_device_id(
                    self.config.as_ptr(), std::ptr::null(),
                );
            }
        }
        self
    }

    // ============ Captured Elements ============

    /// Show or hide the cursor in captured frames
    #[must_use]
    pub fn shows_cursor(self, shows: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_shows_cursor(self.config.as_ptr(), shows);
        }
        self
    }

    // ============ Captured Frames ============

    /// Set minimum frame interval as `CMTime`
    #[must_use]
    pub fn minimum_frame_interval(self, value: i64, timescale: i32) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_minimum_frame_interval(
                self.config.as_ptr(), value, timescale, 0, 0,
            );
        }
        self
    }

    /// Set the queue depth for frame buffering
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn queue_depth(self, depth: i32) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_queue_depth(self.config.as_ptr(), depth as isize);
        }
        self
    }

    // ============ Advanced ============

    /// Set whether content should be opaque (macOS 13.0+)
    #[must_use]
    pub fn should_be_opaque(self, opaque: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_should_be_opaque(self.config.as_ptr(), opaque);
        }
        self
    }

    /// Ignore global clipboard during capture (macOS 14.0+)
    #[must_use]
    pub fn ignores_global_clipboard(self, ignores: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignore_global_clipboard(self.config.as_ptr(), ignores);
        }
        self
    }

    /// Include child windows in capture (macOS 14.2+)
    #[must_use]
    pub fn includes_child_windows(self, includes: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_includes_child_windows(self.config.as_ptr(), includes);
        }
        self
    }

    /// Ignore shadows for single window capture (macOS 14.2+)
    #[must_use]
    pub fn ignores_shadows_single_window(self, ignores: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignores_shadows_single_window(self.config.as_ptr(), ignores);
        }
        self
    }

    /// Capture resolution (macOS 14.0+)
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn capture_resolution(self, width: usize, height: usize) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_capture_resolution(
                self.config.as_ptr(), width as isize, height as isize,
            );
        }
        self
    }

    /// Set presenter overlay alert setting
    #[must_use]
    pub fn presenter_overlay_privacy_alert_setting(self, setting: SCPresenterOverlayAlertSetting) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_presenter_overlay_privacy_alert_setting(
                self.config.as_ptr(), setting as i32,
            );
        }
        self
    }

    // ============ Stream Properties ============

    /// Set the stream name for identification
    #[must_use]
    pub fn stream_name(self, name: Option<&str>) -> Self {
        unsafe {
            if let Some(n) = name {
                if let Ok(c_str) = std::ffi::CString::new(n) {
                    crate::ffi::sc_stream_configuration_set_stream_name(self.config.as_ptr(), c_str.as_ptr());
                }
            } else {
                crate::ffi::sc_stream_configuration_set_stream_name(self.config.as_ptr(), std::ptr::null());
            }
        }
        self
    }

    /// Set the HDR capture dynamic range (macOS 15.0+)
    #[must_use]
    pub fn capture_dynamic_range(self, range: SCCaptureDynamicRange) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_capture_dynamic_range(self.config.as_ptr(), range as i32);
        }
        self
    }
}

impl Default for SCStreamConfigurationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
