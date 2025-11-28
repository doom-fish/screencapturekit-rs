//! Audio capture configuration
//!
//! Methods for configuring audio capture, sample rate, and channel count.

use crate::utils::ffi_string::{ffi_string_from_buffer, SMALL_BUFFER_SIZE};

use super::internal::SCStreamConfiguration;

impl SCStreamConfiguration {
    /// Enable or disable audio capture
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::default()
    ///     .set_captures_audio(true);
    /// assert!(config.get_captures_audio());
    /// ```
    pub fn set_captures_audio(self, captures_audio: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_captures_audio(self.as_ptr(), captures_audio);
        }
        self
    }

    /// Check if audio capture is enabled
    pub fn get_captures_audio(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_captures_audio(self.as_ptr()) }
    }

    /// Set the audio sample rate in Hz
    ///
    /// Common values: 44100, 48000
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::default()
    ///     .set_sample_rate(48000);
    /// assert_eq!(config.get_sample_rate(), 48000);
    /// ```
    pub fn set_sample_rate(self, sample_rate: i32) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_sample_rate(
                self.as_ptr(),
                sample_rate as isize,
            );
        }
        self
    }

    /// Get the configured audio sample rate
    pub fn get_sample_rate(&self) -> i32 {
        // FFI returns isize but sample rate fits in i32 (typical values: 44100, 48000)
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_stream_configuration_get_sample_rate(self.as_ptr()) as i32
        }
    }

    /// Set the number of audio channels
    ///
    /// Common values: 1 (mono), 2 (stereo)
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::default()
    ///     .set_channel_count(2); // Stereo
    /// assert_eq!(config.get_channel_count(), 2);
    /// ```
    pub fn set_channel_count(self, channel_count: i32) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_channel_count(
                self.as_ptr(),
                channel_count as isize,
            );
        }
        self
    }

    /// Get the configured channel count
    pub fn get_channel_count(&self) -> i32 {
        // FFI returns isize but channel count fits in i32 (typical values: 1-8)
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_stream_configuration_get_channel_count(self.as_ptr()) as i32
        }
    }

    /// Enable microphone capture (macOS 15.0+)
    ///
    /// When set to `true`, the stream will capture audio from the microphone
    /// in addition to system/application audio (if `captures_audio` is also enabled).
    ///
    /// **Note**: Requires `NSMicrophoneUsageDescription` in your app's Info.plist
    /// for microphone access permission.
    ///
    /// # Availability
    /// macOS 15.0+. On earlier versions, this setting has no effect.
    ///
    /// # Example
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::default()
    ///     .set_captures_audio(true)       // System audio
    ///     .set_captures_microphone(true)  // Microphone audio (macOS 15.0+)
    ///     .set_sample_rate(48000)
    ///     .set_channel_count(2);
    /// ```
    pub fn set_captures_microphone(self, captures_microphone: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_captures_microphone(
                self.as_ptr(),
                captures_microphone,
            );
        }
        self
    }

    /// Get whether microphone capture is enabled (macOS 15.0+).
    pub fn get_captures_microphone(&self) -> bool {
        unsafe { crate::ffi::sc_stream_configuration_get_captures_microphone(self.as_ptr()) }
    }

    /// Exclude current process audio from capture.
    ///
    /// When set to `true`, the stream will not capture audio from the current
    /// process, preventing feedback loops in recording applications.
    ///
    /// # Example
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::default()
    ///     .set_captures_audio(true)
    ///     .set_excludes_current_process_audio(true); // Prevent feedback
    /// ```
    pub fn set_excludes_current_process_audio(self, excludes: bool) -> Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_excludes_current_process_audio(
                self.as_ptr(),
                excludes,
            );
        }
        self
    }

    /// Get whether current process audio is excluded from capture.
    pub fn get_excludes_current_process_audio(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_excludes_current_process_audio(self.as_ptr())
        }
    }

    /// Set microphone capture device ID (macOS 15.0+).
    ///
    /// Specifies which microphone device to capture from. Use `None` for the
    /// default system microphone.
    ///
    /// # Availability
    /// macOS 15.0+. On earlier versions, this setting has no effect.
    ///
    /// # Example
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// let config = SCStreamConfiguration::default()
    ///     .set_captures_microphone(true)
    ///     .set_microphone_capture_device_id(Some("AppleHDAEngineInput:1B,0,1,0:1"));
    /// ```
    pub fn set_microphone_capture_device_id(self, device_id: Option<&str>) -> Self {
        unsafe {
            if let Some(id) = device_id {
                if let Ok(c_id) = std::ffi::CString::new(id) {
                    crate::ffi::sc_stream_configuration_set_microphone_capture_device_id(
                        self.as_ptr(),
                        c_id.as_ptr(),
                    );
                }
            } else {
                crate::ffi::sc_stream_configuration_set_microphone_capture_device_id(
                    self.as_ptr(),
                    std::ptr::null(),
                );
            }
        }
        self
    }

    /// Get microphone capture device ID (macOS 15.0+).
    pub fn get_microphone_capture_device_id(&self) -> Option<String> {
        unsafe {
            ffi_string_from_buffer(SMALL_BUFFER_SIZE, |buf, len| {
                crate::ffi::sc_stream_configuration_get_microphone_capture_device_id(
                    self.as_ptr(),
                    buf,
                    len,
                )
            })
        }
    }
}
