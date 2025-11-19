//! Captured elements configuration
//!
//! Methods for configuring which elements are included in the capture
//! (cursor, shadows, etc.).

use crate::error::SCError;

use super::internal::SCStreamConfiguration;

impl SCStreamConfiguration {
    /// Show or hide the cursor in captured frames
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::build()
    ///     .set_shows_cursor(true)?;
    /// assert!(config.get_shows_cursor());
    /// # Ok::<(), screencapturekit::error::SCError>(())
    /// ```
    pub fn set_shows_cursor(self, shows_cursor: bool) -> Result<Self, SCError> {
        unsafe {
            crate::ffi::sc_stream_configuration_set_shows_cursor(self.as_ptr(), shows_cursor);
        }
        Ok(self)
    }

    /// Check if cursor is shown in capture
    pub fn get_shows_cursor(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_shows_cursor(self.as_ptr())
        }
    }

    /// Capture only window shadows (macOS 14.0+)
    /// 
    /// When set to `true`, the stream captures only the shadows of windows,
    /// not the actual window content. This is useful for creating transparency
    /// or blur effects in compositing applications.
    /// 
    /// # Availability
    /// macOS 14.0+. On earlier versions, this setting has no effect.
    /// 
    /// # Examples
    /// ```
    /// use screencapturekit::prelude::*;
    /// 
    /// let config = SCStreamConfiguration::build()
    ///     .set_width(1920)?
    ///     .set_height(1080)?
    ///     .set_captures_shadows_only(true)?; // Only capture shadows
    /// # Ok::<(), screencapturekit::error::SCError>(())
    /// ```
    pub fn set_captures_shadows_only(self, captures_shadows_only: bool) -> Result<Self, SCError> {
        unsafe {
            crate::ffi::sc_stream_configuration_set_captures_shadows_only(self.as_ptr(), captures_shadows_only);
        }
        Ok(self)
    }

    /// Get whether only window shadows are captured (macOS 14.0+).
    pub fn get_captures_shadows_only(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_captures_shadows_only(self.as_ptr())
        }
    }
}

#[cfg(test)]
mod sc_stream_configuration_test {
    use crate::stream::configuration::SCStreamConfiguration;

    #[test]
    fn test_setters_and_getters() {
        let config = SCStreamConfiguration::default();
        let config = config
            .set_shows_cursor(true)
            .expect("Failed to set showsCursor");
        assert!(config.get_shows_cursor());
    }
}
