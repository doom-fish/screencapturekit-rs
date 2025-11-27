//! Stream identification and HDR configuration
//!
//! This module provides methods to configure stream identification and HDR capture settings.

use crate::error::SCError;
use super::internal::SCStreamConfiguration;

/// Dynamic range mode for capture (macOS 15.0+)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SCCaptureDynamicRange {
    /// Standard Dynamic Range (SDR) - default mode
    SDR = 0,
    /// HDR with local display tone mapping
    HDRLocalDisplay = 1,
    /// HDR with canonical display tone mapping
    HDRCanonicalDisplay = 2,
}

impl SCStreamConfiguration {
    /// Set the stream name for identification
    ///
    /// Assigns a name to the stream that can be used for debugging and identification
    /// purposes. The name appears in system logs and debugging tools.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::build()
    ///     .set_stream_name(Some("MyApp-MainCapture"))?;
    /// # Ok::<(), screencapturekit::error::SCError>(())
    /// ```
    pub fn set_stream_name(self, name: Option<&str>) -> Result<Self, SCError> {
        unsafe {
            if let Some(stream_name) = name {
                if let Ok(c_name) = std::ffi::CString::new(stream_name) {
                    crate::ffi::sc_stream_configuration_set_stream_name(
                        self.as_ptr(),
                        c_name.as_ptr(),
                    );
                }
            } else {
                crate::ffi::sc_stream_configuration_set_stream_name(
                    self.as_ptr(),
                    std::ptr::null(),
                );
            }
        }
        Ok(self)
    }

    /// Get the configured stream name
    ///
    /// Returns the name assigned to this stream, if any.
    pub fn get_stream_name(&self) -> Option<String> {
        let mut buffer = vec![0i8; 256];
        unsafe {
            #[allow(clippy::cast_possible_wrap)]
            if crate::ffi::sc_stream_configuration_get_stream_name(
                self.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as isize,
            ) {
                std::ffi::CStr::from_ptr(buffer.as_ptr())
                    .to_str()
                    .ok()
                    .map(String::from)
            } else {
                None
            }
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
    /// let config = SCStreamConfiguration::build()
    ///     .set_width(1920)?
    ///     .set_height(1080)?
    ///     .set_capture_dynamic_range(SCCaptureDynamicRange::HDRLocalDisplay)?;
    /// # Ok::<(), screencapturekit::error::SCError>(())
    /// ```
    #[cfg(feature = "macos_15_0")]
    pub fn set_capture_dynamic_range(
        self,
        dynamic_range: SCCaptureDynamicRange,
    ) -> Result<Self, SCError> {
        unsafe {
            crate::ffi::sc_stream_configuration_set_capture_dynamic_range(
                self.as_ptr(),
                dynamic_range as i32,
            );
        }
        Ok(self)
    }

    /// Get the configured dynamic range mode (macOS 15.0+)
    ///
    /// Returns the current HDR capture mode setting.
    ///
    /// Requires the `macos_15_0` feature flag to be enabled.
    #[cfg(feature = "macos_15_0")]
    pub fn get_capture_dynamic_range(&self) -> SCCaptureDynamicRange {
        let value = unsafe {
            crate::ffi::sc_stream_configuration_get_capture_dynamic_range(self.as_ptr())
        };
        match value {
            1 => SCCaptureDynamicRange::HDRLocalDisplay,
            2 => SCCaptureDynamicRange::HDRCanonicalDisplay,
            _ => SCCaptureDynamicRange::SDR,
        }
    }
}

