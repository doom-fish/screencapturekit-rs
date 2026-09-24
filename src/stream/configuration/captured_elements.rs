//! Captured elements configuration
//!
//! Methods for configuring which elements are included in the capture
//! (cursor, shadows, etc.).

use super::internal::SCStreamConfiguration;
#[cfg(feature = "macos_14_0")]
use crate::error::{SCError, SCResult};

impl SCStreamConfiguration {
    /// Show or hide the cursor in captured frames
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_shows_cursor(true);
    /// assert!(config.shows_cursor());
    /// ```
    pub fn set_shows_cursor(&mut self, shows_cursor: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_shows_cursor(self.as_ptr(), shows_cursor);
        }
        self
    }

    /// Show or hide the cursor (builder pattern)
    #[must_use]
    pub fn with_shows_cursor(mut self, shows_cursor: bool) -> Self {
        self.set_shows_cursor(shows_cursor);
        self
    }

    /// Check if cursor is shown in capture
    pub fn shows_cursor(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_shows_cursor(self.as_ptr()) }
    }

    /// Show mouse click indicators (macOS 15.0+)
    ///
    /// When enabled, draws a circle around the cursor when clicked.
    /// This helps viewers track mouse activity in recordings.
    ///
    /// # Availability
    /// macOS 15.0+. On earlier versions it returns
    /// [`SCError::FeatureNotAvailable`].
    ///
    /// # Examples
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_shows_cursor(true)
    ///     .with_shows_mouse_clicks(true)
    ///     .expect("macOS 15.0 or later");
    /// ```
    #[cfg(feature = "macos_15_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_shows_mouse_clicks(&mut self, shows_mouse_clicks: bool) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_shows_mouse_clicks(
                self.as_ptr(),
                shows_mouse_clicks,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.showMouseClicks", "15.0")
        })
    }

    /// Show mouse click indicators (builder pattern)
    #[cfg(feature = "macos_15_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_shows_mouse_clicks(mut self, shows_mouse_clicks: bool) -> SCResult<Self> {
        self.set_shows_mouse_clicks(shows_mouse_clicks)?;
        Ok(self)
    }

    /// Check if mouse click indicators are shown (macOS 15.0+)
    #[cfg(feature = "macos_15_0")]
    pub fn shows_mouse_clicks(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_shows_mouse_clicks(self.as_ptr()) }
    }

    /// Capture only window shadows (macOS 14.0+)
    ///
    /// When set to `true`, the stream captures only the shadows of windows,
    /// not the actual window content. This is useful for creating transparency
    /// or blur effects in compositing applications.
    ///
    /// # Availability
    /// macOS 14.0+. On earlier versions it returns
    /// [`SCError::FeatureNotAvailable`].
    ///
    /// # Examples
    /// ```no_run
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::new()
    ///     .with_width(1920)
    ///     .with_height(1080)
    ///     .with_captures_shadows_only(true)
    ///     .expect("macOS 14.0 or later");
    /// ```
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_captures_shadows_only(
        &mut self,
        captures_shadows_only: bool,
    ) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_captures_shadows_only(
                self.as_ptr(),
                captures_shadows_only,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.capturesShadowsOnly", "14.0")
        })
    }

    /// Capture only window shadows (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_captures_shadows_only(mut self, captures_shadows_only: bool) -> SCResult<Self> {
        self.set_captures_shadows_only(captures_shadows_only)?;
        Ok(self)
    }

    /// Get whether only window shadows are captured (macOS 14.0+).
    #[cfg(feature = "macos_14_0")]
    pub fn captures_shadows_only(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_captures_shadows_only(self.as_ptr()) }
    }

    /// Ignore shadows for display capture (macOS 14.0+)
    ///
    /// When set to `true`, window shadows are excluded from display capture.
    ///
    /// # Availability
    /// macOS 14.0+. On earlier versions it returns
    /// [`SCError::FeatureNotAvailable`].
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_ignores_shadows_display(&mut self, ignores_shadows: bool) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_ignores_shadows_display(
                self.as_ptr(),
                ignores_shadows,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.ignoreShadowsDisplay", "14.0")
        })
    }

    /// Ignore shadows for display capture (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_ignores_shadows_display(mut self, ignores_shadows: bool) -> SCResult<Self> {
        self.set_ignores_shadows_display(ignores_shadows)?;
        Ok(self)
    }

    /// Check if shadows are ignored for display capture (macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    pub fn ignores_shadows_display(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_ignores_shadows_display(self.as_ptr()) }
    }

    /// Ignore global clip for display capture (macOS 14.0+)
    ///
    /// When set to `true`, the global clip region is ignored for display capture.
    ///
    /// # Availability
    /// macOS 14.0+. On earlier versions it returns
    /// [`SCError::FeatureNotAvailable`].
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_ignore_global_clip_display(&mut self, ignore: bool) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_ignore_global_clip_display(
                self.as_ptr(),
                ignore,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.ignoreGlobalClipDisplay", "14.0")
        })
    }

    /// Ignore global clip for display capture (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_ignore_global_clip_display(mut self, ignore: bool) -> SCResult<Self> {
        self.set_ignore_global_clip_display(ignore)?;
        Ok(self)
    }

    /// Check if global clip is ignored for display capture (macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    pub fn ignore_global_clip_display(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_ignore_global_clip_display(self.as_ptr()) }
    }

    /// Ignore global clip for single window capture (macOS 14.0+)
    ///
    /// When set to `true`, the global clip region is ignored for single window capture.
    ///
    /// # Availability
    /// macOS 14.0+. On earlier versions it returns
    /// [`SCError::FeatureNotAvailable`].
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_ignore_global_clip_single_window(&mut self, ignore: bool) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_ignore_global_clip_single_window(
                self.as_ptr(),
                ignore,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available(
                "SCStreamConfiguration.ignoreGlobalClipSingleWindow",
                "14.0",
            )
        })
    }

    /// Ignore global clip for single window capture (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_ignore_global_clip_single_window(mut self, ignore: bool) -> SCResult<Self> {
        self.set_ignore_global_clip_single_window(ignore)?;
        Ok(self)
    }

    /// Check if global clip is ignored for single window capture (macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    pub fn ignore_global_clip_single_window(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_ignore_global_clip_single_window(self.as_ptr())
        }
    }
}
