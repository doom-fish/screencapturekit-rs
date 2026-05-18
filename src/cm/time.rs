//! Core Media time types

use std::ffi::c_void;
use std::fmt;

pub use apple_cf::cm::CMTime;

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

/// `CMClock` wrapper for synchronization clock
///
/// Represents a Core Media clock used for time synchronization.
/// Available on macOS 13.0+.
pub struct CMClock {
    ptr: *const c_void,
}

impl PartialEq for CMClock {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl Eq for CMClock {}

impl std::hash::Hash for CMClock {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl CMClock {
    /// Create from raw pointer, returning None if null
    pub fn from_raw(ptr: *const c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    /// Create from raw pointer (used internally)
    ///
    /// # Safety
    /// The caller must ensure the pointer is a valid, retained `CMClock` pointer.
    #[allow(dead_code)]
    pub(crate) fn from_ptr(ptr: *const c_void) -> Self {
        Self { ptr }
    }

    /// Returns the raw pointer to the underlying `CMClock`
    pub fn as_ptr(&self) -> *const c_void {
        self.ptr
    }

    /// Get the current time from this clock
    ///
    /// Note: Returns invalid time. Use `as_ptr()` with Core Media APIs directly
    /// for full clock functionality.
    pub fn time(&self) -> CMTime {
        // This would require FFI to CMClockGetTime - for now return invalid
        // Users can use the pointer directly with Core Media APIs
        CMTime::INVALID
    }
}

impl Drop for CMClock {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // CMClock is a CFType, needs CFRelease
            extern "C" {
                fn CFRelease(cf: *const c_void);
            }
            unsafe {
                CFRelease(self.ptr);
            }
        }
    }
}

impl Clone for CMClock {
    fn clone(&self) -> Self {
        if self.ptr.is_null() {
            Self {
                ptr: std::ptr::null(),
            }
        } else {
            extern "C" {
                fn CFRetain(cf: *const c_void) -> *const c_void;
            }
            unsafe {
                Self {
                    ptr: CFRetain(self.ptr),
                }
            }
        }
    }
}

impl std::fmt::Debug for CMClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CMClock").field("ptr", &self.ptr).finish()
    }
}

impl fmt::Display for CMClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ptr.is_null() {
            write!(f, "CMClock(null)")
        } else {
            write!(f, "CMClock({:p})", self.ptr)
        }
    }
}

// Safety: CMClock is a CoreFoundation type that is thread-safe
unsafe impl Send for CMClock {}
unsafe impl Sync for CMClock {}
