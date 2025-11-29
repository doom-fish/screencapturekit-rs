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
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_captures_audio(true);
    /// assert!(config.captures_audio());
    /// ```
    pub fn set_captures_audio(&mut self, captures_audio: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_captures_audio(self.as_ptr(), captures_audio);
        }
        self
    }

    /// Enable or disable audio capture (builder pattern)
    #[must_use]
    pub fn with_captures_audio(mut self, captures_audio: bool) -> Self {
        self.set_captures_audio(captures_audio);
        self
    }

    /// Check if audio capture is enabled
    pub fn captures_audio(&self) -> bool {
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
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_sample_rate(48000);
    /// assert_eq!(config.sample_rate(), 48000);
    /// ```
    pub fn set_sample_rate(&mut self, sample_rate: i32) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_sample_rate(
                self.as_ptr(),
                sample_rate as isize,
            );
        }
        self
    }

    /// Set the audio sample rate (builder pattern)
    #[must_use]
    pub fn with_sample_rate(mut self, sample_rate: i32) -> Self {
        self.set_sample_rate(sample_rate);
        self
    }

    /// Get the configured audio sample rate
    pub fn sample_rate(&self) -> i32 {
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
    /// let mut config = SCStreamConfiguration::default();
    /// config.set_channel_count(2); // Stereo
    /// assert_eq!(config.channel_count(), 2);
    /// ```
    pub fn set_channel_count(&mut self, channel_count: i32) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_channel_count(
                self.as_ptr(),
                channel_count as isize,
            );
        }
        self
    }

    /// Set the number of audio channels (builder pattern)
    #[must_use]
    pub fn with_channel_count(mut self, channel_count: i32) -> Self {
        self.set_channel_count(channel_count);
        self
    }

    /// Get the configured channel count
    pub fn channel_count(&self) -> i32 {
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
    /// let config = SCStreamConfiguration::new()
    ///     .with_captures_audio(true)       // System audio
    ///     .with_captures_microphone(true)  // Microphone audio (macOS 15.0+)
    ///     .with_sample_rate(48000)
    ///     .with_channel_count(2);
    /// ```
    pub fn set_captures_microphone(&mut self, captures_microphone: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_captures_microphone(
                self.as_ptr(),
                captures_microphone,
            );
        }
        self
    }

    /// Enable microphone capture (builder pattern)
    #[must_use]
    pub fn with_captures_microphone(mut self, captures_microphone: bool) -> Self {
        self.set_captures_microphone(captures_microphone);
        self
    }

    /// Get whether microphone capture is enabled (macOS 15.0+).
    pub fn captures_microphone(&self) -> bool {
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
    /// let config = SCStreamConfiguration::new()
    ///     .with_captures_audio(true)
    ///     .with_excludes_current_process_audio(true); // Prevent feedback
    /// ```
    pub fn set_excludes_current_process_audio(&mut self, excludes: bool) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_excludes_current_process_audio(
                self.as_ptr(),
                excludes,
            );
        }
        self
    }

    /// Exclude current process audio (builder pattern)
    #[must_use]
    pub fn with_excludes_current_process_audio(mut self, excludes: bool) -> Self {
        self.set_excludes_current_process_audio(excludes);
        self
    }

    /// Get whether current process audio is excluded from capture.
    pub fn excludes_current_process_audio(&self) -> bool {
        unsafe {
            crate::ffi::sc_stream_configuration_get_excludes_current_process_audio(self.as_ptr())
        }
    }

    /// Set microphone capture device ID (macOS 15.0+).
    ///
    /// Specifies which microphone device to capture from.
    ///
    /// # Availability
    /// macOS 15.0+. On earlier versions, this setting has no effect.
    ///
    /// # Example
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// let mut config = SCStreamConfiguration::new()
    ///     .with_captures_microphone(true);
    /// config.set_microphone_capture_device_id("AppleHDAEngineInput:1B,0,1,0:1");
    /// ```
    pub fn set_microphone_capture_device_id(&mut self, device_id: &str) -> &mut Self {
        unsafe {
            if let Ok(c_id) = std::ffi::CString::new(device_id) {
                crate::ffi::sc_stream_configuration_set_microphone_capture_device_id(
                    self.as_ptr(),
                    c_id.as_ptr(),
                );
            }
        }
        self
    }

    /// Set microphone capture device ID (builder pattern)
    #[must_use]
    pub fn with_microphone_capture_device_id(mut self, device_id: &str) -> Self {
        self.set_microphone_capture_device_id(device_id);
        self
    }

    /// Clear microphone capture device ID, reverting to default system microphone
    pub fn clear_microphone_capture_device_id(&mut self) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_microphone_capture_device_id(
                self.as_ptr(),
                std::ptr::null(),
            );
        }
        self
    }

    /// Get microphone capture device ID (macOS 15.0+).
    pub fn microphone_capture_device_id(&self) -> Option<String> {
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
