//! Core Media time types

use std::fmt;

/// `CMTime` representation matching Core Media's `CMTime`
///
/// Represents a rational time value with a 64-bit numerator and 32-bit denominator.
///
/// # Examples
///
/// ```
/// use screencapturekit::cm::CMTime;
///
/// // Create a time of 1 second (30/30)
/// let time = CMTime::new(30, 30);
/// assert_eq!(time.as_seconds(), Some(1.0));
///
/// // Create a time of 2.5 seconds at 1000 Hz timescale
/// let time = CMTime::new(2500, 1000);
/// assert_eq!(time.value, 2500);
/// assert_eq!(time.timescale, 1000);
/// assert_eq!(time.as_seconds(), Some(2.5));
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CMTime {
    pub value: i64,
    pub timescale: i32,
    pub flags: u32,
    pub epoch: i64,
}

impl std::hash::Hash for CMTime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.timescale.hash(state);
        self.flags.hash(state);
        self.epoch.hash(state);
    }
}

/// Sample timing information
///
/// Contains timing data for a media sample (audio or video frame).
///
/// # Examples
///
/// ```
/// use screencapturekit::cm::{CMSampleTimingInfo, CMTime};
///
/// let timing = CMSampleTimingInfo::new();
/// assert!(!timing.is_valid());
///
/// let duration = CMTime::new(1, 30);
/// let pts = CMTime::new(100, 30);
/// let dts = CMTime::new(100, 30);
/// let timing = CMSampleTimingInfo::with_times(duration, pts, dts);
/// assert!(timing.is_valid());
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CMSampleTimingInfo {
    pub duration: CMTime,
    pub presentation_time_stamp: CMTime,
    pub decode_time_stamp: CMTime,
}

impl std::hash::Hash for CMSampleTimingInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.duration.hash(state);
        self.presentation_time_stamp.hash(state);
        self.decode_time_stamp.hash(state);
    }
}

impl CMSampleTimingInfo {
    /// Create a new timing info with all times set to invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cm::CMSampleTimingInfo;
    ///
    /// let timing = CMSampleTimingInfo::new();
    /// assert!(!timing.is_valid());
    /// ```
    pub const fn new() -> Self {
        Self {
            duration: CMTime::INVALID,
            presentation_time_stamp: CMTime::INVALID,
            decode_time_stamp: CMTime::INVALID,
        }
    }

    /// Create timing info with specific values
    pub const fn with_times(
        duration: CMTime,
        presentation_time_stamp: CMTime,
        decode_time_stamp: CMTime,
    ) -> Self {
        Self {
            duration,
            presentation_time_stamp,
            decode_time_stamp,
        }
    }

    /// Check if all timing fields are valid
    pub const fn is_valid(&self) -> bool {
        self.duration.is_valid()
            && self.presentation_time_stamp.is_valid()
            && self.decode_time_stamp.is_valid()
    }

    /// Check if presentation timestamp is valid
    pub const fn has_valid_presentation_time(&self) -> bool {
        self.presentation_time_stamp.is_valid()
    }

    /// Check if decode timestamp is valid
    pub const fn has_valid_decode_time(&self) -> bool {
        self.decode_time_stamp.is_valid()
    }

    /// Check if duration is valid
    pub const fn has_valid_duration(&self) -> bool {
        self.duration.is_valid()
    }

    /// Get the presentation timestamp in seconds
    pub fn presentation_seconds(&self) -> Option<f64> {
        self.presentation_time_stamp.as_seconds()
    }

    /// Get the decode timestamp in seconds
    pub fn decode_seconds(&self) -> Option<f64> {
        self.decode_time_stamp.as_seconds()
    }

    /// Get the duration in seconds
    pub fn duration_seconds(&self) -> Option<f64> {
        self.duration.as_seconds()
    }
}

impl Default for CMSampleTimingInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CMSampleTimingInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CMSampleTimingInfo(pts: {}, dts: {}, duration: {})",
            self.presentation_time_stamp, self.decode_time_stamp, self.duration
        )
    }
}

impl CMTime {
    pub const ZERO: Self = Self {
        value: 0,
        timescale: 0,
        flags: 1,
        epoch: 0,
    };

    pub const INVALID: Self = Self {
        value: 0,
        timescale: 0,
        flags: 0,
        epoch: 0,
    };

    pub const fn new(value: i64, timescale: i32) -> Self {
        Self {
            value,
            timescale,
            flags: 1,
            epoch: 0,
        }
    }

    pub const fn is_valid(&self) -> bool {
        self.flags & 0x1 != 0
    }

    /// Check if this time represents zero
    pub const fn is_zero(&self) -> bool {
        self.value == 0 && self.is_valid()
    }

    /// Check if this time is indefinite
    pub const fn is_indefinite(&self) -> bool {
        self.flags & 0x2 != 0
    }

    /// Check if this time is positive infinity
    pub const fn is_positive_infinity(&self) -> bool {
        self.flags & 0x4 != 0
    }

    /// Check if this time is negative infinity
    pub const fn is_negative_infinity(&self) -> bool {
        self.flags & 0x8 != 0
    }

    /// Check if this time has been rounded
    pub const fn has_been_rounded(&self) -> bool {
        self.flags & 0x10 != 0
    }

    /// Compare two times for equality (value and timescale)
    pub const fn equals(&self, other: &Self) -> bool {
        if !self.is_valid() || !other.is_valid() {
            return false;
        }
        self.value == other.value && self.timescale == other.timescale
    }

    /// Create a time representing positive infinity
    pub const fn positive_infinity() -> Self {
        Self {
            value: 0,
            timescale: 0,
            flags: 0x5, // kCMTimeFlags_Valid | kCMTimeFlags_PositiveInfinity
            epoch: 0,
        }
    }

    /// Create a time representing negative infinity
    pub const fn negative_infinity() -> Self {
        Self {
            value: 0,
            timescale: 0,
            flags: 0x9, // kCMTimeFlags_Valid | kCMTimeFlags_NegativeInfinity
            epoch: 0,
        }
    }

    /// Create an indefinite time
    pub const fn indefinite() -> Self {
        Self {
            value: 0,
            timescale: 0,
            flags: 0x3, // kCMTimeFlags_Valid | kCMTimeFlags_Indefinite
            epoch: 0,
        }
    }

    pub fn as_seconds(&self) -> Option<f64> {
        if self.is_valid() && self.timescale != 0 {
            // Precision loss is acceptable for time conversion to seconds
            #[allow(clippy::cast_precision_loss)]
            Some(self.value as f64 / f64::from(self.timescale))
        } else {
            None
        }
    }
}

impl Default for CMTime {
    fn default() -> Self {
        Self::INVALID
    }
}

impl fmt::Display for CMTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(seconds) = self.as_seconds() {
            write!(f, "{seconds:.3}s")
        } else {
            write!(f, "invalid")
        }
    }
}
