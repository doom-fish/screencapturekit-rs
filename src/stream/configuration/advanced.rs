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
    pub fn set_ignore_fraction_of_screen(&mut self, ignore_fraction: f64) {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignore_fraction_of_screen(
                self.as_ptr(),
                ignore_fraction,
            );
        }
    }

    #[cfg(feature = "macos_14_2")]
    pub fn get_ignore_fraction_of_screen(&self) -> f64 {
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
    pub fn set_ignores_shadows_single_window(&mut self, ignores_shadows: bool) {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignores_shadows_single_window(
                self.as_ptr(),
                ignores_shadows,
            );
        }
    }

    #[cfg(feature = "macos_14_2")]
    pub fn get_ignores_shadows_single_window(&self) -> bool {
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
    pub fn set_should_be_opaque(&mut self, should_be_opaque: bool) {
        unsafe {
            crate::ffi::sc_stream_configuration_set_should_be_opaque(
                self.as_ptr(),
                should_be_opaque,
            );
        }
    }

    #[cfg(feature = "macos_13_0")]
    pub fn get_should_be_opaque(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_should_be_opaque(self.as_ptr()) }
    }

    /// Sets whether to include child windows in capture.
    ///
    /// A Boolean value that indicates whether the content includes child windows.
    /// Available on macOS 14.2+
    ///
    /// Requires the `macos_14_2` feature flag to be enabled.
    #[cfg(feature = "macos_14_2")]
    pub fn set_includes_child_windows(&mut self, includes_child_windows: bool) {
        unsafe {
            crate::ffi::sc_stream_configuration_set_includes_child_windows(
                self.as_ptr(),
                includes_child_windows,
            );
        }
    }

    #[cfg(feature = "macos_14_2")]
    pub fn get_includes_child_windows(&self) -> bool {
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
        self,
        setting: SCPresenterOverlayAlertSetting,
    ) {
        unsafe {
            crate::ffi::sc_stream_configuration_set_presenter_overlay_privacy_alert_setting(
                self.as_ptr(),
                setting as i32,
            );
        }
    }

    #[cfg(feature = "macos_14_2")]
    pub fn get_presenter_overlay_privacy_alert_setting(&self) -> SCPresenterOverlayAlertSetting {
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
    pub fn set_ignore_global_clipboard(&mut self, ignore_global_clipboard: bool) {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignore_global_clipboard(
                self.as_ptr(),
                ignore_global_clipboard,
            );
        }
    }

    #[cfg(feature = "macos_14_0")]
    pub fn get_ignore_global_clipboard(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_ignore_global_clipboard(self.as_ptr()) }
    }

    /// Sets whether to ignore shadow display configuration.
    ///
    /// Available on macOS 14.0+
    ///
    /// Requires the `macos_14_0` feature flag to be enabled.
    #[cfg(feature = "macos_14_0")]
    pub fn set_ignores_shadow_display_configuration(&mut self, ignores_shadow: bool) {
        unsafe {
            crate::ffi::sc_stream_configuration_set_ignores_shadow_display_configuration(
                self.as_ptr(),
                ignores_shadow,
            );
        }
    }

    #[cfg(feature = "macos_14_0")]
    pub fn get_ignores_shadow_display_configuration(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_ignores_shadow_display_configuration(
                self.as_ptr(),
            )
        }
    }
}
