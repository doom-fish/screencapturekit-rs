//! Audio capture configuration
//!
//! Methods for configuring audio capture, sample rate, and channel count.
//!
//! ## Supported Values
//!
//! `ScreenCaptureKit` supports specific sample rates and channel counts:
//!
//! | Sample Rate | Description |
//! |-------------|-------------|
//! | 8000 Hz | Low quality, telephony |
//! | 16000 Hz | Speech quality |
//! | 24000 Hz | Medium quality |
//! | 48000 Hz | Professional audio (default) |
//!
//! | Channel Count | Description |
//! |---------------|-------------|
//! | 1 | Mono |
//! | 2 | Stereo (default) |

use crate::utils::ffi_string::{ffi_string_from_buffer, SMALL_BUFFER_SIZE};

use super::internal::SCStreamConfiguration;

/// Audio sample rate for capture
///
/// `ScreenCaptureKit` supports a fixed set of sample rates. Using values outside
/// this set will result in the system defaulting to 48000 Hz.
///
/// # Examples
///
/// ```
/// use screencapturekit::stream::configuration::audio::AudioSampleRate;
///
/// // Get the Hz value
/// assert_eq!(AudioSampleRate::Rate48000.as_hz(), 48000);
///
/// // Use default (48000 Hz)
/// let rate = AudioSampleRate::default();
/// assert_eq!(rate, AudioSampleRate::Rate48000);
/// ```
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AudioSampleRate {
    /// 8000 Hz - Low quality, suitable for telephony
    Rate8000 = 8000,
    /// 16000 Hz - Speech quality
    Rate16000 = 16000,
    /// 24000 Hz - Medium quality
    Rate24000 = 24000,
    /// 48000 Hz - Professional audio quality (default)
    #[default]
    Rate48000 = 48000,
}

impl AudioSampleRate {
    /// Get the sample rate in Hz
    #[must_use]
    pub const fn as_hz(self) -> i32 {
        self as i32
    }

    /// Create from Hz value, returning None if unsupported
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::stream::configuration::audio::AudioSampleRate;
    ///
    /// assert_eq!(AudioSampleRate::from_hz(48000), Some(AudioSampleRate::Rate48000));
    /// assert_eq!(AudioSampleRate::from_hz(44100), None); // Not supported
    /// ```
    #[must_use]
    pub const fn from_hz(hz: i32) -> Option<Self> {
        match hz {
            8000 => Some(Self::Rate8000),
            16000 => Some(Self::Rate16000),
            24000 => Some(Self::Rate24000),
            48000 => Some(Self::Rate48000),
            _ => None,
        }
    }
}

impl std::fmt::Display for AudioSampleRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Hz", self.as_hz())
    }
}

impl From<AudioSampleRate> for i32 {
    fn from(rate: AudioSampleRate) -> Self {
        rate.as_hz()
    }
}

/// Audio channel configuration for capture
///
/// `ScreenCaptureKit` supports mono (1 channel) or stereo (2 channels) audio.
/// Using other values will result in the system defaulting to stereo.
///
/// # Examples
///
/// ```
/// use screencapturekit::stream::configuration::audio::AudioChannelCount;
///
/// // Get the channel count
/// assert_eq!(AudioChannelCount::Stereo.as_count(), 2);
///
/// // Use default (stereo)
/// let channels = AudioChannelCount::default();
/// assert_eq!(channels, AudioChannelCount::Stereo);
/// ```
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AudioChannelCount {
    /// Mono - single channel audio
    Mono = 1,
    /// Stereo - two channel audio (default)
    #[default]
    Stereo = 2,
}

impl AudioChannelCount {
    /// Get the channel count as an integer
    #[must_use]
    pub const fn as_count(self) -> i32 {
        self as i32
    }

    /// Create from channel count, returning None if unsupported
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::stream::configuration::audio::AudioChannelCount;
    ///
    /// assert_eq!(AudioChannelCount::from_count(2), Some(AudioChannelCount::Stereo));
    /// assert_eq!(AudioChannelCount::from_count(6), None); // Not supported
    /// ```
    #[must_use]
    pub const fn from_count(count: i32) -> Option<Self> {
        match count {
            1 => Some(Self::Mono),
            2 => Some(Self::Stereo),
            _ => None,
        }
    }
}

impl std::fmt::Display for AudioChannelCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mono => write!(f, "Mono (1 channel)"),
            Self::Stereo => write!(f, "Stereo (2 channels)"),
        }
    }
}

impl From<AudioChannelCount> for i32 {
    fn from(count: AudioChannelCount) -> Self {
        count.as_count()
    }
}

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

    /// Set the audio sample rate
    ///
    /// Accepts either an [`AudioSampleRate`] enum or a raw `i32` Hz value.
    ///
    /// # Supported Values
    ///
    /// `ScreenCaptureKit` supports: 8000, 16000, 24000, 48000 Hz.
    /// Other values will default to 48000 Hz.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    /// use screencapturekit::stream::configuration::audio::AudioSampleRate;
    ///
    /// // Using the enum (recommended)
    /// let config = SCStreamConfiguration::new()
    ///     .with_sample_rate(AudioSampleRate::Rate48000);
    ///
    /// // Using raw value (still works)
    /// let config = SCStreamConfiguration::new()
    ///     .with_sample_rate(48000);
    /// ```
    pub fn set_sample_rate(&mut self, sample_rate: impl Into<i32>) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_sample_rate(
                self.as_ptr(),
                sample_rate.into() as isize,
            );
        }
        self
    }

    /// Set the audio sample rate (builder pattern)
    #[must_use]
    pub fn with_sample_rate(mut self, sample_rate: impl Into<i32>) -> Self {
        self.set_sample_rate(sample_rate);
        self
    }

    /// Get the configured audio sample rate in Hz
    pub fn sample_rate(&self) -> i32 {
        // FFI returns isize but sample rate fits in i32 (typical values: 44100, 48000)
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            crate::ffi::sc_stream_configuration_get_sample_rate(self.as_ptr()) as i32
        }
    }

    /// Get the configured audio sample rate as an enum
    ///
    /// Returns `None` if the current sample rate is not a supported value.
    pub fn audio_sample_rate(&self) -> Option<AudioSampleRate> {
        AudioSampleRate::from_hz(self.sample_rate())
    }

    /// Set the number of audio channels
    ///
    /// Accepts either an [`AudioChannelCount`] enum or a raw `i32` value.
    ///
    /// # Supported Values
    ///
    /// `ScreenCaptureKit` supports: 1 (mono), 2 (stereo).
    /// Other values will default to stereo.
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::prelude::*;
    /// use screencapturekit::stream::configuration::audio::AudioChannelCount;
    ///
    /// // Using the enum (recommended)
    /// let config = SCStreamConfiguration::new()
    ///     .with_channel_count(AudioChannelCount::Stereo);
    ///
    /// // Using raw value (still works)
    /// let config = SCStreamConfiguration::new()
    ///     .with_channel_count(2);
    /// ```
    pub fn set_channel_count(&mut self, channel_count: impl Into<i32>) -> &mut Self {
        unsafe {
            crate::ffi::sc_stream_configuration_set_channel_count(
                self.as_ptr(),
                channel_count.into() as isize,
            );
        }
        self
    }

    /// Set the number of audio channels (builder pattern)
    #[must_use]
    pub fn with_channel_count(mut self, channel_count: impl Into<i32>) -> Self {
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

    /// Get the configured channel count as an enum
    ///
    /// Returns `None` if the current channel count is not a supported value.
    pub fn audio_channel_count(&self) -> Option<AudioChannelCount> {
        AudioChannelCount::from_count(self.channel_count())
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
