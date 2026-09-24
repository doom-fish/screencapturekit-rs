use super::internal::SCStreamConfiguration;
#[cfg(feature = "macos_14_0")]
use crate::error::{SCError, SCResult};

/// Presenter overlay privacy alert setting (macOS 14.0+)
///
/// Controls when the system displays a privacy alert for presenter overlay.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCPresenterOverlayAlertSetting {
    /// Let the system decide when to show the alert
    #[default]
    System = 0,
    /// Never show the privacy alert
    Never = 1,
    /// Always show the privacy alert
    Always = 2,
}

impl SCPresenterOverlayAlertSetting {
    pub const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::System),
            1 => Some(Self::Never),
            2 => Some(Self::Always),
            _ => None,
        }
    }
}

impl SCStreamConfiguration {
    /// Sets whether to ignore shadows for single window capture.
    ///
    /// A Boolean value that indicates whether the stream omits the shadow effects
    /// of the windows it captures.
    /// Available on macOS 14.0+
    ///
    /// Requires the `macos_14_0` feature flag to be enabled.
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_ignores_shadows_single_window(
        &mut self,
        ignores_shadows: bool,
    ) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_ignores_shadows_single_window(
                self.as_ptr(),
                ignores_shadows,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available(
                "SCStreamConfiguration.ignoreShadowsSingleWindow",
                "14.0",
            )
        })
    }

    /// Sets whether to ignore shadows for single window capture (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_ignores_shadows_single_window(mut self, ignores_shadows: bool) -> SCResult<Self> {
        self.set_ignores_shadows_single_window(ignores_shadows)?;
        Ok(self)
    }

    /// Get whether shadows are ignored for single-window capture (macOS 14.0+).
    #[cfg(feature = "macos_14_0")]
    pub fn ignores_shadows_single_window(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_ignores_shadows_single_window(self.as_ptr())
        }
    }

    /// Sets whether captured content should be treated as opaque.
    ///
    /// A Boolean value that indicates whether the stream treats the transparency
    /// of the captured content as opaque.
    ///
    /// Available on macOS 14.0+ (`SCStreamConfiguration.shouldBeOpaque` is
    /// annotated `API_AVAILABLE(macos(14.0))`), so this requires the
    /// `macos_14_0` feature flag. On older systems it returns
    /// [`SCError::FeatureNotAvailable`].
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_should_be_opaque(&mut self, should_be_opaque: bool) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_should_be_opaque(
                self.as_ptr(),
                should_be_opaque,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.shouldBeOpaque", "14.0")
        })
    }

    /// Sets whether captured content should be treated as opaque (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_should_be_opaque(mut self, should_be_opaque: bool) -> SCResult<Self> {
        self.set_should_be_opaque(should_be_opaque)?;
        Ok(self)
    }

    /// Get whether captured content is treated as opaque (macOS 14.0+).
    #[cfg(feature = "macos_14_0")]
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
    #[allow(clippy::missing_errors_doc)]
    pub fn set_includes_child_windows(
        &mut self,
        includes_child_windows: bool,
    ) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_includes_child_windows(
                self.as_ptr(),
                includes_child_windows,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.includeChildWindows", "14.2")
        })
    }

    /// Sets whether to include child windows (builder pattern)
    #[cfg(feature = "macos_14_2")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_includes_child_windows(mut self, includes_child_windows: bool) -> SCResult<Self> {
        self.set_includes_child_windows(includes_child_windows)?;
        Ok(self)
    }

    /// Get whether child windows are included (macOS 14.2+).
    #[cfg(feature = "macos_14_2")]
    pub fn includes_child_windows(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_includes_child_windows(self.as_ptr()) }
    }

    /// Sets the presenter overlay privacy alert setting.
    ///
    /// A configuration for the privacy alert that the capture session displays.
    ///
    /// Available on macOS 14.0+ — `presenterOverlayPrivacyAlertSetting` is
    /// annotated `API_AVAILABLE(macos(14.0))`, not 14.2 as this binding
    /// previously assumed — so it only needs the `macos_14_0` feature flag.
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_presenter_overlay_privacy_alert_setting(
        &mut self,
        setting: SCPresenterOverlayAlertSetting,
    ) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_presenter_overlay_privacy_alert_setting(
                self.as_ptr(),
                setting as i32,
            )
        };
        if applied {
            Ok(self)
        } else {
            Err(SCError::feature_not_available(
                "SCStreamConfiguration.presenterOverlayPrivacyAlertSetting",
                "14.0",
            ))
        }
    }

    /// Sets the presenter overlay privacy alert setting (builder pattern, macOS 14.0+)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_presenter_overlay_privacy_alert_setting(
        mut self,
        setting: SCPresenterOverlayAlertSetting,
    ) -> SCResult<Self> {
        self.set_presenter_overlay_privacy_alert_setting(setting)?;
        Ok(self)
    }

    /// Get the presenter overlay privacy alert setting (macOS 14.0+).
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn presenter_overlay_privacy_alert_setting(
        &self,
    ) -> SCResult<SCPresenterOverlayAlertSetting> {
        let mut raw = 0_i32;
        let available = unsafe {
            crate::ffi::sc_stream_configuration_get_presenter_overlay_privacy_alert_setting(
                self.as_ptr(),
                &raw mut raw,
            )
        };
        if !available {
            return Err(SCError::feature_not_available(
                "SCStreamConfiguration.presenterOverlayPrivacyAlertSetting",
                "14.0",
            ));
        }
        SCPresenterOverlayAlertSetting::from_raw(raw).ok_or_else(|| SCError::UnknownValue {
            type_name: "SCPresenterOverlayAlertSetting",
            raw: i64::from(raw),
        })
    }

    /// Sets whether to ignore shadow display configuration.
    ///
    /// Available on macOS 14.0+
    ///
    /// Requires the `macos_14_0` feature flag to be enabled.
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn set_ignores_shadow_display_configuration(
        &mut self,
        ignores_shadow: bool,
    ) -> SCResult<&mut Self> {
        let applied = unsafe {
            crate::ffi::sc_stream_configuration_set_ignores_shadow_display_configuration(
                self.as_ptr(),
                ignores_shadow,
            )
        };
        applied.then_some(self).ok_or_else(|| {
            SCError::feature_not_available("SCStreamConfiguration.ignoreShadowsDisplay", "14.0")
        })
    }

    /// Sets whether to ignore shadow display configuration (builder pattern)
    #[cfg(feature = "macos_14_0")]
    #[allow(clippy::missing_errors_doc)]
    pub fn with_ignores_shadow_display_configuration(
        mut self,
        ignores_shadow: bool,
    ) -> SCResult<Self> {
        self.set_ignores_shadow_display_configuration(ignores_shadow)?;
        Ok(self)
    }

    /// Get whether the shadow display configuration is ignored (macOS 14.0+).
    #[cfg(feature = "macos_14_0")]
    pub fn ignores_shadow_display_configuration(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_ignores_shadow_display_configuration(
                self.as_ptr(),
            )
        }
    }
}

#[cfg(all(test, feature = "macos_14_0"))]
mod tests {
    use super::{SCPresenterOverlayAlertSetting, SCStreamConfiguration};

    #[test]
    fn bridge_rejects_unknown_presenter_overlay_raw_values() {
        let mut config = SCStreamConfiguration::new();
        config
            .set_presenter_overlay_privacy_alert_setting(SCPresenterOverlayAlertSetting::Never)
            .expect("set the presenter overlay privacy alert setting");
        for raw in [3, -1, i32::MAX, i32::MIN] {
            let applied = unsafe {
                crate::ffi::sc_stream_configuration_set_presenter_overlay_privacy_alert_setting(
                    config.as_ptr(),
                    raw,
                )
            };
            assert!(!applied, "the bridge accepted raw value {raw}");
            assert_eq!(
                config.presenter_overlay_privacy_alert_setting(),
                Ok(SCPresenterOverlayAlertSetting::Never)
            );
        }
    }
}
