use super::internal::SCStreamConfiguration;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCPresenterOverlayAlertSetting {
    #[default]
    Never = 0,
    Once = 1,
    Always = 2,
}

impl SCStreamConfiguration {
    /// Sets the ignore fraction of screen for this [`SCStreamConfiguration`].
    ///
    /// Specifies the percentage of the content filter that the stream omits from the captured image.
    /// Available on macOS 14.2+
    ///
    /// Requires the `macos_14_2` feature flag to be enabled.
    #[cfg(feature = "macos_14_2")]
    pub fn set_ignore_fraction_of_screen(&mut self, ignore_fraction: f64) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignore_fraction_of_screen(
                self.as_ptr(),
                ignore_fraction,
            );
        }
        self
    }

    /// Sets the ignore fraction of screen (builder pattern)
    #[cfg(feature = "macos_14_2")]
    #[must_use]
    pub fn with_ignore_fraction_of_screen(mut self, ignore_fraction: f64) -> Self {
        self.set_ignore_fraction_of_screen(ignore_fraction);
        self
    }

    #[cfg(feature = "macos_14_2")]
    pub fn ignore_fraction_of_screen(&self) -> f64 {
        unsafe { crate::ffi::sc_stream_configuration_get_ignore_fraction_of_screen(self.as_ptr()) }
    }

    /// Sets whether to ignore shadows for single window capture.
    ///
    /// A Boolean value that indicates whether the stream omits the shadow effects
    /// of the windows it captures.
    /// Available on macOS 14.2+
    ///
    /// Requires the `macos_14_2` feature flag to be enabled.
    #[cfg(feature = "macos_14_2")]
    pub fn set_ignores_shadows_single_window(&mut self, ignores_shadows: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignores_shadows_single_window(
                self.as_ptr(),
                ignores_shadows,
            );
        }
        self
    }

    /// Sets whether to ignore shadows for single window capture (builder pattern)
    #[cfg(feature = "macos_14_2")]
    #[must_use]
    pub fn with_ignores_shadows_single_window(mut self, ignores_shadows: bool) -> Self {
        self.set_ignores_shadows_single_window(ignores_shadows);
        self
    }

    #[cfg(feature = "macos_14_2")]
    pub fn ignores_shadows_single_window(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_ignores_shadows_single_window(self.as_ptr())
        }
    }

    /// Sets whether captured content should be treated as opaque.
    ///
    /// A Boolean value that indicates whether the stream treats the transparency
    /// of the captured content as opaque.
    /// Available on macOS 13.0+
    ///
    /// Requires the `macos_13_0` feature flag to be enabled.
    #[cfg(feature = "macos_13_0")]
    pub fn set_should_be_opaque(&mut self, should_be_opaque: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_should_be_opaque(
                self.as_ptr(),
                should_be_opaque,
            );
        }
        self
    }

    /// Sets whether captured content should be treated as opaque (builder pattern)
    #[cfg(feature = "macos_13_0")]
    #[must_use]
    pub fn with_should_be_opaque(mut self, should_be_opaque: bool) -> Self {
        self.set_should_be_opaque(should_be_opaque);
        self
    }

    #[cfg(feature = "macos_13_0")]
    pub fn should_be_opaque(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_should_be_opaque(self.as_ptr()) }
    }

    /// Sets whether to include child windows in capture.
    ///
    /// A Boolean value that indicates whether the content includes child windows.
    /// Available on macOS 14.2+
    ///
    /// Requires the `macos_14_2` feature flag to be enabled.
    #[cfg(feature = "macos_14_2")]
    pub fn set_includes_child_windows(&mut self, includes_child_windows: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_includes_child_windows(
                self.as_ptr(),
                includes_child_windows,
            );
        }
        self
    }

    /// Sets whether to include child windows (builder pattern)
    #[cfg(feature = "macos_14_2")]
    #[must_use]
    pub fn with_includes_child_windows(mut self, includes_child_windows: bool) -> Self {
        self.set_includes_child_windows(includes_child_windows);
        self
    }

    #[cfg(feature = "macos_14_2")]
    pub fn includes_child_windows(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_includes_child_windows(self.as_ptr()) }
    }

    /// Sets the presenter overlay privacy alert setting.
    ///
    /// A configuration for the privacy alert that the capture session displays.
    /// Available on macOS 14.2+
    ///
    /// Requires the `macos_14_2` feature flag to be enabled.
    #[cfg(feature = "macos_14_2")]
    pub fn set_presenter_overlay_privacy_alert_setting(
        &mut self,
        setting: SCPresenterOverlayAlertSetting,
    ) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_presenter_overlay_privacy_alert_setting(
                self.as_ptr(),
                setting as i32,
            );
        }
        self
    }

    /// Sets the presenter overlay privacy alert setting (builder pattern)
    #[cfg(feature = "macos_14_2")]
    #[must_use]
    pub fn with_presenter_overlay_privacy_alert_setting(
        mut self,
        setting: SCPresenterOverlayAlertSetting,
    ) -> Self {
        self.set_presenter_overlay_privacy_alert_setting(setting);
        self
    }

    #[cfg(feature = "macos_14_2")]
    pub fn presenter_overlay_privacy_alert_setting(&self) -> SCPresenterOverlayAlertSetting {
        let value = unsafe {
            crate::ffi::sc_stream_configuration_get_presenter_overlay_privacy_alert_setting(
                self.as_ptr(),
            )
        };
        match value {
            0 => SCPresenterOverlayAlertSetting::Never,
            2 => SCPresenterOverlayAlertSetting::Always,
            _ => SCPresenterOverlayAlertSetting::Once,
        }
    }

    /// Sets whether to ignore the global clipboard when capturing.
    ///
    /// Available on macOS 14.0+
    ///
    /// Requires the `macos_14_0` feature flag to be enabled.
    #[cfg(feature = "macos_14_0")]
    pub fn set_ignore_global_clipboard(&mut self, ignore_global_clipboard: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignore_global_clipboard(
                self.as_ptr(),
                ignore_global_clipboard,
            );
        }
        self
    }

    /// Sets whether to ignore the global clipboard (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[must_use]
    pub fn with_ignore_global_clipboard(mut self, ignore_global_clipboard: bool) -> Self {
        self.set_ignore_global_clipboard(ignore_global_clipboard);
        self
    }

    #[cfg(feature = "macos_14_0")]
    pub fn ignore_global_clipboard(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_ignore_global_clipboard(self.as_ptr()) }
    }

    /// Sets whether to ignore shadow display configuration.
    ///
    /// Available on macOS 14.0+
    ///
    /// Requires the `macos_14_0` feature flag to be enabled.
    #[cfg(feature = "macos_14_0")]
    pub fn set_ignores_shadow_display_configuration(&mut self, ignores_shadow: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignores_shadow_display_configuration(
                self.as_ptr(),
                ignores_shadow,
            );
        }
        self
    }

    /// Sets whether to ignore shadow display configuration (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[must_use]
    pub fn with_ignores_shadow_display_configuration(mut self, ignores_shadow: bool) -> Self {
        self.set_ignores_shadow_display_configuration(ignores_shadow);
        self
    }

    #[cfg(feature = "macos_14_0")]
    pub fn ignores_shadow_display_configuration(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_ignores_shadow_display_configuration(
                self.as_ptr(),
            )
        }
    }
}
