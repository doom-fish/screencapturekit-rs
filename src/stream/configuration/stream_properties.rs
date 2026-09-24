//! Stream identification and HDR configuration
//!
//! This module provides methods to configure stream identification and HDR capture settings.

use super::internal::SCStreamConfiguration;
#[cfg(feature = "macos_14_0")]
use super::InteriorNulError;
#[cfg(feature = "macos_14_0")]
use crate::error::{SCError, SCResult};
#[cfg(feature = "macos_14_0")]
use crate::utils::ffi_string::{ffi_string_from_buffer, SMALL_BUFFER_SIZE};

/// Dynamic range mode for capture (macOS 15.0+)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCCaptureDynamicRange {
    /// Standard Dynamic Range (SDR) - default mode
    #[default]
    SDR = 0,
    /// HDR with local display tone mapping
    HDRLocalDisplay = 1,
    /// HDR with canonical display tone mapping
    HDRCanonicalDisplay = 2,
}

impl SCCaptureDynamicRange {
    pub const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::SDR),
            1 => Some(Self::HDRLocalDisplay),
            2 => Some(Self::HDRCanonicalDisplay),
            _ => None,
        }
    }
}

impl SCStreamConfiguration {
    /// Set the stream name for identification
    ///
    /// Assigns a name to the stream that can be used for debugging and identification
    /// purposes. The name appears in system logs and debugging tools.
    ///
    /// Available on macOS 14.0+.
    ///
    /// # Errors
    ///
    /// Returns [`SCError::InvalidConfiguration`] — leaving the configuration
    /// unchanged — if `name` contains an interior NUL byte, and
    /// [`SCError::FeatureNotAvailable`] before macOS 14.0.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_stream_name(Some("MyApp-MainCapture"))
    ///     .expect("stream name has no NUL byte");
    /// ```
    #[cfg(feature = "macos_14_0")]
    pub fn set_stream_name(&mut self, name: Option<&str>) -> SCResult<&mut Self> {
        let c_name = name
            .map(|stream_name| std::ffi::CString::new(stream_name).map_err(|_| InteriorNulError))
            .transpose()?;
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_stream_name(
                self.as_ptr(),
                c_name.as_ref().map_or(std::ptr::null(), |n| n.as_ptr()),
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.streamName", "14.0")
        })
    }

    /// Set the stream name (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_stream_name(mut self, name: Option<&str>) -> SCResult<Self> {
        self.set_stream_name(name)?;
        Ok(self)
    }

    /// Get the configured stream name
    ///
    /// Returns the name assigned to this stream, if any.
    #[cfg(feature = "macos_14_0")]
    pub fn stream_name(&self) -> Option<String> {
        unsafe {
            ffi_string_from_buffer(SMALL_BUFFER_SIZE, |buf, len| {
                crate::ffi::sc_stream_configuration_get_stream_name(self.as_ptr(), buf, len)
            })
        }
    }

    /// Set the dynamic range mode for capture (macOS 15.0+)
    ///
    /// Controls whether to capture in SDR or HDR mode and how HDR content
    /// should be tone-mapped for display.
    ///
    /// # Availability
    /// macOS 15.0+. Requires the `macos_15_0` feature flag to be enabled.
    ///
    /// # Modes
    /// - `SDR`: Standard dynamic range capture (default)
    /// - `HDRLocalDisplay`: HDR with tone mapping optimized for the local display
    /// - `HDRCanonicalDisplay`: HDR with canonical tone mapping for portability
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    /// use screencapturekit::stream::configuration::stream_properties::SCCaptureDynamicRange;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_width(1920)
    ///     .with_height(1080)
    ///     .with_capture_dynamic_range(SCCaptureDynamicRange::HDRLocalDisplay)
    ///     .expect("macOS 15.0 or later");
    /// ```
    #[cfg(feature = "macos_15_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_capture_dynamic_range(
        &mut self,
        dynamic_range: SCCaptureDynamicRange,
    ) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_capture_dynamic_range(
                self.as_ptr(),
                dynamic_range as i32,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.captureDynamicRange", "15.0")
        })
    }

    /// Set the dynamic range mode (builder pattern)
    #[cfg(feature = "macos_15_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_capture_dynamic_range(
        mut self,
        dynamic_range: SCCaptureDynamicRange,
    ) -> SCResult<Self> {
        self.set_capture_dynamic_range(dynamic_range)?;
        Ok(self)
    }

    /// Get the configured dynamic range mode (macOS 15.0+)
    ///
    /// Returns the current HDR capture mode setting.
    ///
    /// Requires the `macos_15_0` feature flag to be enabled.
    #[cfg(feature = "macos_15_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn capture_dynamic_range(&self) -> SCResult<SCCaptureDynamicRange> {
        let mut raw = 0_i32;
        let available = unsafe {
            crate::ffi::sc_stream_configuration_get_capture_dynamic_range(
                self.as_ptr(),
                &raw mut raw,
            )
        };
        if !available {
            return Err(SCError::feature_not_available(
                "SCStreamConfiguration.captureDynamicRange",
                "15.0",
            ));
        }
        SCCaptureDynamicRange::from_raw(raw).ok_or_else(|| SCError::UnknownValue {
            type_name: "SCCaptureDynamicRange",
            raw: i64::from(raw),
        })
    }
}

#[cfg(all(test, feature = "macos_15_0"))]
mod tests {
    use super::{SCCaptureDynamicRange, SCStreamConfiguration};

    #[test]
    fn bridge_rejects_unknown_capture_dynamic_range_raw_values() {
        let mut config = SCStreamConfiguration::new();
        config
            .set_capture_dynamic_range(SCCaptureDynamicRange::HDRCanonicalDisplay)
            .expect("macOS 15.0 or later");
        for raw in [3, -1, i32::MAX, i32::MIN] {
            let applied = unsafe {
                crate::ffi::sc_stream_configuration_set_capture_dynamic_range(config.as_ptr(), raw)
            };
            assert!(!applied, "the bridge accepted raw value {raw}");
            assert_eq!(
                config.capture_dynamic_range(),
                Ok(SCCaptureDynamicRange::HDRCanonicalDisplay)
            );
        }
    }
}
