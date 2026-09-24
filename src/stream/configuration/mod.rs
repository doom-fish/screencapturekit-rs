mod internal;

pub mod advanced;
pub mod audio;
pub mod captured_elements;
pub mod captured_frames;
pub mod colors;
pub mod dimensions;
pub mod pixel_format;
pub mod stream_properties;

pub use advanced::SCPresenterOverlayAlertSetting;
pub use audio::{AudioChannelCount, AudioSampleRate};
pub use captured_frames::{MAX_QUEUE_DEPTH, MIN_QUEUE_DEPTH};
pub use colors::{color_matrix, color_space, InteriorNulError};
pub use internal::SCStreamConfiguration;
pub use pixel_format::PixelFormat;
pub use stream_properties::SCCaptureDynamicRange;

/// Capture resolution type for stream configuration (macOS 14.0+)
///
/// Controls how the capture resolution is determined relative to the source content.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg(feature = "macos_14_0")]
pub enum SCCaptureResolutionType {
    /// Automatically determines the best resolution
    #[default]
    Automatic = 0,
    /// Uses the best available resolution (highest quality)
    Best = 1,
    /// Uses the nominal resolution of the display
    Nominal = 2,
}

#[cfg(feature = "macos_14_0")]
impl SCCaptureResolutionType {
    pub const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::Automatic),
            1 => Some(Self::Best),
            2 => Some(Self::Nominal),
            _ => None,
        }
    }
}

#[cfg(feature = "macos_14_0")]
impl std::fmt::Display for SCCaptureResolutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Automatic => write!(f, "Automatic"),
            Self::Best => write!(f, "Best"),
            Self::Nominal => write!(f, "Nominal"),
        }
    }
}

impl Default for SCStreamConfiguration {
    fn default() -> Self {
        Self::internal_init()
    }
}

/// Preset for creating stream configurations (macOS 15.0+)
///
/// Use these presets to create configurations optimized for specific use cases,
/// particularly HDR capture scenarios.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg(feature = "macos_15_0")]
pub enum SCStreamConfigurationPreset {
    /// HDR stream optimized for local display
    CaptureHDRStreamLocalDisplay = 0,
    /// HDR stream optimized for canonical display
    CaptureHDRStreamCanonicalDisplay = 1,
    /// HDR screenshot optimized for local display
    CaptureHDRScreenshotLocalDisplay = 2,
    /// HDR screenshot optimized for canonical display
    CaptureHDRScreenshotCanonicalDisplay = 3,
    /// HDR recording optimized for HDR10, preserving SDR range during playback.
    ///
    /// This preset sets values for `captureDynamicRange`, `pixelFormat`, and `colorSpace`
    /// intended for a stream recording in HDR10, optimized for rendering on the
    /// canonical HDR display. It also adds HDR10 metadata to the video recording
    /// that is designed to preserve the SDR range during video playback.
    #[cfg(feature = "macos_26_0")]
    CaptureHDRRecordingPreservedSDRHDR10 = 4,
}

impl SCStreamConfiguration {
    /// Create a new stream configuration with default values
    ///
    /// This is equivalent to `SCStreamConfiguration::default()`.
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration from a preset (macOS 15.0+)
    ///
    /// Presets provide optimized default values for specific use cases,
    /// particularly for HDR capture.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::stream::configuration::{SCStreamConfiguration, SCStreamConfigurationPreset};
    ///
    /// let config = SCStreamConfiguration::from_preset(SCStreamConfigurationPreset::CaptureHDRStreamLocalDisplay)
    ///     .expect("macOS 15.0 or later");
    /// ```
    #[cfg(feature = "macos_15_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn from_preset(preset: SCStreamConfigurationPreset) -> crate::error::SCResult<Self> {
        let ptr = unsafe { crate::ffi::sc_stream_configuration_create_with_preset(preset as i32) };
        if ptr.is_null() {
            #[cfg(feature = "macos_26_0")]
            let required_version = match preset {
                SCStreamConfigurationPreset::CaptureHDRRecordingPreservedSDRHDR10 => "26.0",
                _ => "15.0",
            };
            #[cfg(not(feature = "macos_26_0"))]
            let required_version = "15.0";
            return Err(crate::error::SCError::feature_not_available(
                format!("SCStreamConfiguration preset {preset:?}"),
                required_version,
            ));
        }
        Ok(unsafe { Self::from_ptr(ptr) })
    }

    #[cfg(feature = "macos_15_0")]
    pub(crate) unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }
}

#[cfg(all(test, feature = "macos_15_0"))]
mod tests {
    #[test]
    fn bridge_refuses_unknown_presets() {
        for raw in [5, -1, i32::MAX, i32::MIN] {
            let ptr = unsafe { crate::ffi::sc_stream_configuration_create_with_preset(raw) };
            assert!(
                ptr.is_null(),
                "the bridge built a configuration for preset {raw}"
            );
        }
    }
}
