//! Core Media types and wrappers
//!
//! This module provides Rust wrappers for Core Media framework types used in
//! screen capture operations.
//!
//! ## Main Types
//!
//! - [`CMSampleBuffer`] - Container for media samples (audio/video frames)
//! - [`CMTime`] - Time value with rational timescale
//! - [`CVPixelBuffer`] - Video pixel buffer
//! - [`IOSurface`] - Hardware-accelerated surface
//! - [`AudioBuffer`] - Audio data buffer
//! - [`AudioBufferList`] - Collection of audio buffers
//! - [`SCFrameStatus`] - Status of a captured frame

use std::fmt;

/// Frame status for captured screen content
///
/// Indicates the state of a frame captured by ScreenCaptureKit.
/// This maps to Apple's `SCFrameStatus` enum.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SCFrameStatus {
    /// Frame contains complete content
    Complete = 0,
    /// Frame is idle (no changes)
    Idle = 1,
    /// Frame is blank
    Blank = 2,
    /// Frame is suspended
    Suspended = 3,
    /// Started (first frame)
    Started = 4,
    /// Stopped (last frame)
    Stopped = 5,
}

impl SCFrameStatus {
    /// Create from raw i32 value
    pub const fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Complete),
            1 => Some(Self::Idle),
            2 => Some(Self::Blank),
            3 => Some(Self::Suspended),
            4 => Some(Self::Started),
            5 => Some(Self::Stopped),
            _ => None,
        }
    }
    
    /// Returns true if the frame contains actual content
    pub const fn has_content(self) -> bool {
        matches!(self, Self::Complete | Self::Started)
    }
    
    /// Returns true if the frame is complete
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

impl fmt::Display for SCFrameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "Complete"),
            Self::Idle => write!(f, "Idle"),
            Self::Blank => write!(f, "Blank"),
            Self::Suspended => write!(f, "Suspended"),
            Self::Started => write!(f, "Started"),
            Self::Stopped => write!(f, "Stopped"),
        }
    }
}

/// CMTime representation matching Core Media's CMTime
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

/// Opaque handle to CMSampleBuffer
#[repr(transparent)]
#[derive(Debug)]
pub struct CMSampleBuffer(*mut std::ffi::c_void);

impl PartialEq for CMSampleBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CMSampleBuffer {}

impl std::hash::Hash for CMSampleBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cm_sample_buffer_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl CMSampleBuffer {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CMSampleBuffer` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Create a sample buffer for an image buffer (video frame)
    ///
    /// # Arguments
    ///
    /// * `image_buffer` - The pixel buffer containing the video frame
    /// * `presentation_time` - When the frame should be presented
    /// * `duration` - How long the frame should be displayed
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cm::{CMSampleBuffer, CVPixelBuffer, CMTime};
    ///
    /// // Create a pixel buffer
    /// let pixel_buffer = CVPixelBuffer::create(1920, 1080, 0x42475241)
    ///     .expect("Failed to create pixel buffer");
    ///
    /// // Create timing information (30fps video)
    /// let presentation_time = CMTime::new(0, 30); // Frame 0 at 30 fps
    /// let duration = CMTime::new(1, 30);          // 1/30th of a second
    ///
    /// // Create sample buffer
    /// let sample = CMSampleBuffer::create_for_image_buffer(
    ///     &pixel_buffer,
    ///     presentation_time,
    ///     duration,
    /// ).expect("Failed to create sample buffer");
    ///
    /// assert!(sample.is_valid());
    /// assert_eq!(sample.get_presentation_timestamp().value, 0);
    /// assert_eq!(sample.get_presentation_timestamp().timescale, 30);
    /// ```
    pub fn create_for_image_buffer(
        image_buffer: &CVPixelBuffer,
        presentation_time: CMTime,
        duration: CMTime,
    ) -> Result<Self, i32> {
        unsafe {
            let mut sample_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cm_sample_buffer_create_for_image_buffer(
                image_buffer.as_ptr(),
                presentation_time.value,
                presentation_time.timescale,
                duration.value,
                duration.timescale,
                &mut sample_buffer_ptr,
            );
            
            if status == 0 && !sample_buffer_ptr.is_null() {
                Ok(Self(sample_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    pub fn get_image_buffer(&self) -> Option<CVPixelBuffer> {
        unsafe {
            let ptr = ffi::cm_sample_buffer_get_image_buffer(self.0);
            CVPixelBuffer::from_raw(ptr)
        }
    }

    /// Get the frame status from a sample buffer
    ///
    /// Returns the `SCFrameStatus` attachment from the sample buffer,
    /// indicating whether the frame is complete, idle, blank, etc.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::cm::{CMSampleBuffer, SCFrameStatus};
    ///
    /// fn handle_frame(sample: CMSampleBuffer) {
    ///     if let Some(status) = sample.get_frame_status() {
    ///         match status {
    ///             SCFrameStatus::Complete => {
    ///                 println!("Frame is complete, process it");
    ///             }
    ///             SCFrameStatus::Idle => {
    ///                 println!("Frame is idle, no changes");
    ///             }
    ///             _ => {
    ///                 println!("Frame status: {}", status);
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_frame_status(&self) -> Option<SCFrameStatus> {
        unsafe {
            let status = ffi::cm_sample_buffer_get_frame_status(self.0);
            if status >= 0 {
                SCFrameStatus::from_raw(status)
            } else {
                None
            }
        }
    }

    pub fn get_presentation_timestamp(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_presentation_timestamp(
                self.0,
                &mut value,
                &mut timescale,
                &mut flags,
                &mut epoch,
            );
            CMTime { value, timescale, flags, epoch }
        }
    }

    pub fn get_duration(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_duration(
                self.0,
                &mut value,
                &mut timescale,
                &mut flags,
                &mut epoch,
            );
            CMTime { value, timescale, flags, epoch }
        }
    }

    pub fn is_valid(&self) -> bool {
        unsafe { ffi::cm_sample_buffer_is_valid(self.0) }
    }

    pub fn get_num_samples(&self) -> usize {
        unsafe { ffi::cm_sample_buffer_get_num_samples(self.0) }
    }

    pub fn get_audio_buffer_list(&self) -> Option<AudioBufferList> {
        unsafe {
            let mut num_buffers: u32 = 0;
            let mut buffers_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut buffers_len: usize = 0;
            
            ffi::cm_sample_buffer_get_audio_buffer_list(
                self.0,
                &mut num_buffers,
                &mut buffers_ptr,
                &mut buffers_len,
            );
            
            if num_buffers == 0 {
                None
            } else {
                Some(AudioBufferList {
                    inner: AudioBufferListRaw {
                        num_buffers,
                        buffers_ptr: buffers_ptr.cast::<AudioBuffer>(),
                        buffers_len,
                    },
                })
            }
        }
    }

    pub fn get_data_buffer(&self) -> Option<CMBlockBuffer> {
        unsafe {
            let ptr = ffi::cm_sample_buffer_get_data_buffer(self.0);
            CMBlockBuffer::from_raw(ptr)
        }
    }

    /// Get the decode timestamp of the sample buffer
    pub fn get_decode_timestamp(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_decode_timestamp(
                self.0,
                &mut value,
                &mut timescale,
                &mut flags,
                &mut epoch,
            );
            CMTime { value, timescale, flags, epoch }
        }
    }

    /// Get the output presentation timestamp
    pub fn get_output_presentation_timestamp(&self) -> CMTime {
        unsafe {
            let mut value: i64 = 0;
            let mut timescale: i32 = 0;
            let mut flags: u32 = 0;
            let mut epoch: i64 = 0;
            ffi::cm_sample_buffer_get_output_presentation_timestamp(
                self.0,
                &mut value,
                &mut timescale,
                &mut flags,
                &mut epoch,
            );
            CMTime { value, timescale, flags, epoch }
        }
    }

    /// Set the output presentation timestamp
    pub fn set_output_presentation_timestamp(&self, time: CMTime) -> Result<(), i32> {
        unsafe {
            let status = ffi::cm_sample_buffer_set_output_presentation_timestamp(
                self.0,
                time.value,
                time.timescale,
                time.flags,
                time.epoch,
            );
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Get the size of a specific sample
    pub fn get_sample_size(&self, index: usize) -> usize {
        unsafe { ffi::cm_sample_buffer_get_sample_size(self.0, index) }
    }

    /// Get the total size of all samples
    pub fn get_total_sample_size(&self) -> usize {
        unsafe { ffi::cm_sample_buffer_get_total_sample_size(self.0) }
    }

    /// Check if the sample buffer data is ready for access
    pub fn is_data_ready(&self) -> bool {
        unsafe { ffi::cm_sample_buffer_is_ready_for_data_access(self.0) }
    }

    /// Make the sample buffer data ready for access
    pub fn make_data_ready(&self) -> Result<(), i32> {
        unsafe {
            let status = ffi::cm_sample_buffer_make_data_ready(self.0);
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Get the format description
    pub fn get_format_description(&self) -> Option<CMFormatDescription> {
        unsafe {
            let ptr = ffi::cm_sample_buffer_get_format_description(self.0);
            CMFormatDescription::from_raw(ptr)
        }
    }

    /// Get sample timing info for a specific sample
    pub fn get_sample_timing_info(&self, index: usize) -> Result<CMSampleTimingInfo, i32> {
        unsafe {
            let mut timing_info = CMSampleTimingInfo {
                duration: CMTime::INVALID,
                presentation_time_stamp: CMTime::INVALID,
                decode_time_stamp: CMTime::INVALID,
            };
            let status = ffi::cm_sample_buffer_get_sample_timing_info(
                self.0,
                index,
                &mut timing_info.duration.value,
                &mut timing_info.duration.timescale,
                &mut timing_info.duration.flags,
                &mut timing_info.duration.epoch,
                &mut timing_info.presentation_time_stamp.value,
                &mut timing_info.presentation_time_stamp.timescale,
                &mut timing_info.presentation_time_stamp.flags,
                &mut timing_info.presentation_time_stamp.epoch,
                &mut timing_info.decode_time_stamp.value,
                &mut timing_info.decode_time_stamp.timescale,
                &mut timing_info.decode_time_stamp.flags,
                &mut timing_info.decode_time_stamp.epoch,
            );
            if status == 0 {
                Ok(timing_info)
            } else {
                Err(status)
            }
        }
    }

    /// Get all sample timing info as a vector
    pub fn get_sample_timing_info_array(&self) -> Result<Vec<CMSampleTimingInfo>, i32> {
        let num_samples = self.get_num_samples();
        let mut result = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            result.push(self.get_sample_timing_info(i)?);
        }
        Ok(result)
    }

    /// Invalidate the sample buffer
    pub fn invalidate(&self) -> Result<(), i32> {
        unsafe {
            let status = ffi::cm_sample_buffer_invalidate(self.0);
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Create a copy with new timing information
    pub fn create_copy_with_new_timing(&self, timing_info: &[CMSampleTimingInfo]) -> Result<Self, i32> {
        unsafe {
            let mut new_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cm_sample_buffer_create_copy_with_new_timing(
                self.0,
                timing_info.len(),
                timing_info.as_ptr().cast::<std::ffi::c_void>(),
                &mut new_buffer_ptr,
            );
            if status == 0 && !new_buffer_ptr.is_null() {
                Ok(Self(new_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Copy PCM audio data into an audio buffer list
    pub fn copy_pcm_data_into_audio_buffer_list(&self, frame_offset: i32, num_frames: i32, buffer_list: &mut AudioBufferList) -> Result<(), i32> {
        unsafe {
            let status = ffi::cm_sample_buffer_copy_pcm_data_into_audio_buffer_list(
                self.0,
                frame_offset,
                num_frames,
                (buffer_list as *mut AudioBufferList).cast::<std::ffi::c_void>(),
            );
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }
}

impl Drop for CMSampleBuffer {
    fn drop(&mut self) {
        unsafe {
            ffi::cm_sample_buffer_release(self.0);
        }
    }
}

unsafe impl Send for CMSampleBuffer {}
unsafe impl Sync for CMSampleBuffer {}

impl fmt::Display for CMSampleBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CMSampleBuffer(pts: {}, duration: {}, samples: {})",
            self.get_presentation_timestamp(),
            self.get_duration(),
            self.get_num_samples()
        )
    }
}

/// Opaque handle to CVPixelBuffer
#[repr(transparent)]
#[derive(Debug)]
pub struct CVPixelBuffer(*mut std::ffi::c_void);

impl PartialEq for CVPixelBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CVPixelBuffer {}

impl std::hash::Hash for CVPixelBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cv_pixel_buffer_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl CVPixelBuffer {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CVPixelBuffer` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Create a new pixel buffer with the specified dimensions and pixel format
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the pixel buffer in pixels
    /// * `height` - Height of the pixel buffer in pixels
    /// * `pixel_format` - Pixel format type (e.g., 0x42475241 for BGRA)
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cm::CVPixelBuffer;
    ///
    /// // Create a 1920x1080 BGRA pixel buffer
    /// let buffer = CVPixelBuffer::create(1920, 1080, 0x42475241)
    ///     .expect("Failed to create pixel buffer");
    ///
    /// assert_eq!(buffer.width(), 1920);
    /// assert_eq!(buffer.height(), 1080);
    /// assert_eq!(buffer.pixel_format(), 0x42475241);
    /// ```
    pub fn create(width: usize, height: usize, pixel_format: u32) -> Result<Self, i32> {
        unsafe {
            let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cv_pixel_buffer_create(
                width,
                height,
                pixel_format,
                &mut pixel_buffer_ptr,
            );
            
            if status == 0 && !pixel_buffer_ptr.is_null() {
                Ok(Self(pixel_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Create a pixel buffer from existing memory
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the pixel buffer in pixels
    /// * `height` - Height of the pixel buffer in pixels
    /// * `pixel_format` - Pixel format type (e.g., 0x42475241 for BGRA)
    /// * `base_address` - Pointer to pixel data
    /// * `bytes_per_row` - Number of bytes per row
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `base_address` points to valid memory
    /// - Memory remains valid for the lifetime of the pixel buffer
    /// - `bytes_per_row` correctly represents the memory layout
    ///
    /// # Examples
    ///
    /// ```
    /// use screencapturekit::cm::CVPixelBuffer;
    ///
    /// // Create pixel data (100x100 BGRA image)
    /// let width = 100;
    /// let height = 100;
    /// let bytes_per_pixel = 4; // BGRA
    /// let bytes_per_row = width * bytes_per_pixel;
    /// let mut pixel_data = vec![0u8; width * height * bytes_per_pixel];
    ///
    /// // Fill with blue color
    /// for y in 0..height {
    ///     for x in 0..width {
    ///         let offset = y * bytes_per_row + x * bytes_per_pixel;
    ///         pixel_data[offset] = 255;     // B
    ///         pixel_data[offset + 1] = 0;   // G
    ///         pixel_data[offset + 2] = 0;   // R
    ///         pixel_data[offset + 3] = 255; // A
    ///     }
    /// }
    ///
    /// // Create pixel buffer from the data
    /// let buffer = unsafe {
    ///     CVPixelBuffer::create_with_bytes(
    ///         width,
    ///         height,
    ///         0x42475241, // BGRA
    ///         pixel_data.as_mut_ptr() as *mut std::ffi::c_void,
    ///         bytes_per_row,
    ///     )
    /// }.expect("Failed to create pixel buffer");
    ///
    /// assert_eq!(buffer.width(), width);
    /// assert_eq!(buffer.height(), height);
    /// ```
    pub unsafe fn create_with_bytes(
        width: usize,
        height: usize,
        pixel_format: u32,
        base_address: *mut std::ffi::c_void,
        bytes_per_row: usize,
    ) -> Result<Self, i32> {
        let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = ffi::cv_pixel_buffer_create_with_bytes(
            width,
            height,
            pixel_format,
            base_address,
            bytes_per_row,
            &mut pixel_buffer_ptr,
        );
        
        if status == 0 && !pixel_buffer_ptr.is_null() {
            Ok(Self(pixel_buffer_ptr))
        } else {
            Err(status)
        }
    }

    /// Fill the extended pixels of a pixel buffer
    ///
    /// This is useful for pixel buffers that have been created with extended pixels
    /// enabled, to ensure proper edge handling for effects and filters.
    pub fn fill_extended_pixels(&self) -> Result<(), i32> {
        unsafe {
            let status = ffi::cv_pixel_buffer_fill_extended_pixels(self.0);
            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    /// Create a pixel buffer with planar bytes
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `plane_base_addresses` points to valid memory for each plane
    /// - Memory remains valid for the lifetime of the pixel buffer
    /// - All plane parameters correctly represent the memory layout
    pub unsafe fn create_with_planar_bytes(
        width: usize,
        height: usize,
        pixel_format: u32,
        plane_base_addresses: &[*mut std::ffi::c_void],
        plane_widths: &[usize],
        plane_heights: &[usize],
        plane_bytes_per_row: &[usize],
    ) -> Result<Self, i32> {
        if plane_base_addresses.len() != plane_widths.len()
            || plane_widths.len() != plane_heights.len()
            || plane_heights.len() != plane_bytes_per_row.len()
        {
            return Err(-50); // paramErr
        }

        let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = ffi::cv_pixel_buffer_create_with_planar_bytes(
            width,
            height,
            pixel_format,
            plane_base_addresses.len(),
            plane_base_addresses.as_ptr(),
            plane_widths.as_ptr(),
            plane_heights.as_ptr(),
            plane_bytes_per_row.as_ptr(),
            &mut pixel_buffer_ptr,
        );

        if status == 0 && !pixel_buffer_ptr.is_null() {
            Ok(Self(pixel_buffer_ptr))
        } else {
            Err(status)
        }
    }

    /// Create a pixel buffer from an IOSurface
    pub fn create_with_io_surface(surface: &IOSurface) -> Result<Self, i32> {
        unsafe {
            let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cv_pixel_buffer_create_with_io_surface(
                surface.as_ptr(),
                &mut pixel_buffer_ptr,
            );

            if status == 0 && !pixel_buffer_ptr.is_null() {
                Ok(Self(pixel_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Get the Core Foundation type ID for CVPixelBuffer
    pub fn get_type_id() -> usize {
        unsafe { ffi::cv_pixel_buffer_get_type_id() }
    }

    /// Get the data size of the pixel buffer
    pub fn get_data_size(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_data_size(self.0) }
    }

    /// Check if the pixel buffer is planar
    pub fn is_planar(&self) -> bool {
        unsafe { ffi::cv_pixel_buffer_is_planar(self.0) }
    }

    /// Get the number of planes in the pixel buffer
    pub fn get_plane_count(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_plane_count(self.0) }
    }

    /// Get the width of a specific plane
    pub fn get_width_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_width_of_plane(self.0, plane_index) }
    }

    /// Get the height of a specific plane
    pub fn get_height_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_height_of_plane(self.0, plane_index) }
    }

    /// Get the base address of a specific plane
    pub fn get_base_address_of_plane(&self, plane_index: usize) -> Option<*mut u8> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_get_base_address_of_plane(self.0, plane_index);
            if ptr.is_null() {
                None
            } else {
                Some(ptr.cast::<u8>())
            }
        }
    }

    /// Get the bytes per row of a specific plane
    pub fn get_bytes_per_row_of_plane(&self, plane_index: usize) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_bytes_per_row_of_plane(self.0, plane_index) }
    }

    /// Get the extended pixel information (left, right, top, bottom)
    pub fn get_extended_pixels(&self) -> (usize, usize, usize, usize) {
        unsafe {
            let mut left: usize = 0;
            let mut right: usize = 0;
            let mut top: usize = 0;
            let mut bottom: usize = 0;
            ffi::cv_pixel_buffer_get_extended_pixels(
                self.0,
                &mut left,
                &mut right,
                &mut top,
                &mut bottom,
            );
            (left, right, top, bottom)
        }
    }

    /// Check if the pixel buffer is backed by an IOSurface
    pub fn is_backed_by_io_surface(&self) -> bool {
        self.get_io_surface().is_some()
    }

    pub fn get_width(&self) -> usize {
        self.width()
    }

    pub fn get_height(&self) -> usize {
        self.height()
    }

    pub fn get_pixel_format_type(&self) -> u32 {
        self.pixel_format()
    }

    pub fn get_bytes_per_row(&self) -> usize {
        self.bytes_per_row()
    }

    pub fn get_base_address(&self) -> Option<*mut u8> {
        self.base_address()
    }

    pub fn get_iosurface(&self) -> Option<IOSurface> {
        self.get_io_surface()
    }

    pub fn width(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_width(self.0) }
    }

    pub fn height(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_height(self.0) }
    }

    pub fn pixel_format(&self) -> u32 {
        unsafe { ffi::cv_pixel_buffer_get_pixel_format_type(self.0) }
    }

    pub fn bytes_per_row(&self) -> usize {
        unsafe { ffi::cv_pixel_buffer_get_bytes_per_row(self.0) }
    }

    pub fn lock_raw(&self, flags: u32) -> Result<(), i32> {
        unsafe {
            let result = ffi::cv_pixel_buffer_lock_base_address(self.0, flags);
            if result == 0 {
                Ok(())
            } else {
                Err(result)
            }
        }
    }

    pub fn unlock_raw(&self, flags: u32) -> Result<(), i32> {
        unsafe {
            let result = ffi::cv_pixel_buffer_unlock_base_address(self.0, flags);
            if result == 0 {
                Ok(())
            } else {
                Err(result)
            }
        }
    }

    pub fn base_address(&self) -> Option<*mut u8> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_get_base_address(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(ptr.cast::<u8>())
            }
        }
    }

    pub fn get_io_surface(&self) -> Option<IOSurface> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_get_io_surface(self.0);
            IOSurface::from_raw(ptr)
        }
    }

    pub fn lock_base_address(&self, read_only: bool) -> Result<CVPixelBufferLockGuard<'_>, i32> {
        let flags = u32::from(read_only);
        self.lock_raw(flags)?;
        Ok(CVPixelBufferLockGuard { buffer: self, read_only })
    }
}

/// RAII guard for locked CVPixelBuffer base address
pub struct CVPixelBufferLockGuard<'a> {
    buffer: &'a CVPixelBuffer,
    read_only: bool,
}

impl CVPixelBufferLockGuard<'_> {
    pub fn get_base_address(&self) -> *const u8 {
        self.buffer.base_address().unwrap_or(std::ptr::null_mut()).cast_const()
    }

    pub fn get_base_address_mut(&mut self) -> *mut u8 {
        if self.read_only {
            std::ptr::null_mut()
        } else {
            self.buffer.base_address().unwrap_or(std::ptr::null_mut())
        }
    }
}

impl Drop for CVPixelBufferLockGuard<'_> {
    fn drop(&mut self) {
        let flags = u32::from(self.read_only);
        let _ = self.buffer.unlock_raw(flags);
    }
}

impl Clone for CVPixelBuffer {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_retain(self.0);
            Self(ptr)
        }
    }
}

impl Drop for CVPixelBuffer {
    fn drop(&mut self) {
        unsafe {
            ffi::cv_pixel_buffer_release(self.0);
        }
    }
}

unsafe impl Send for CVPixelBuffer {}
unsafe impl Sync for CVPixelBuffer {}

impl fmt::Display for CVPixelBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CVPixelBuffer({}x{}, format: 0x{:08X})",
            self.width(),
            self.height(),
            self.pixel_format()
        )
    }
}

/// Opaque handle to CVPixelBufferPool
#[repr(transparent)]
#[derive(Debug)]
pub struct CVPixelBufferPool(*mut std::ffi::c_void);

impl PartialEq for CVPixelBufferPool {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CVPixelBufferPool {}

impl std::hash::Hash for CVPixelBufferPool {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cv_pixel_buffer_pool_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl CVPixelBufferPool {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CVPixelBufferPool` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Create a new pixel buffer pool
    ///
    /// # Arguments
    ///
    /// * `width` - Width of pixel buffers in the pool
    /// * `height` - Height of pixel buffers in the pool
    /// * `pixel_format` - Pixel format type
    /// * `max_buffers` - Maximum number of buffers in the pool (0 for unlimited)
    pub fn create(
        width: usize,
        height: usize,
        pixel_format: u32,
        max_buffers: usize,
    ) -> Result<Self, i32> {
        unsafe {
            let mut pool_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cv_pixel_buffer_pool_create(
                width,
                height,
                pixel_format,
                max_buffers,
                &mut pool_ptr,
            );

            if status == 0 && !pool_ptr.is_null() {
                Ok(Self(pool_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Create a pixel buffer from the pool
    pub fn create_pixel_buffer(&self) -> Result<CVPixelBuffer, i32> {
        unsafe {
            let mut pixel_buffer_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let status = ffi::cv_pixel_buffer_pool_create_pixel_buffer(
                self.0,
                &mut pixel_buffer_ptr,
            );

            if status == 0 && !pixel_buffer_ptr.is_null() {
                Ok(CVPixelBuffer(pixel_buffer_ptr))
            } else {
                Err(status)
            }
        }
    }

    /// Flush the pixel buffer pool
    ///
    /// Releases all available pixel buffers in the pool
    pub fn flush(&self) {
        unsafe {
            ffi::cv_pixel_buffer_pool_flush(self.0);
        }
    }

    /// Get the Core Foundation type ID for CVPixelBufferPool
    pub fn get_type_id() -> usize {
        unsafe { ffi::cv_pixel_buffer_pool_get_type_id() }
    }

    /// Create a pixel buffer from the pool with auxiliary attributes
    ///
    /// This allows specifying additional attributes for the created buffer
    pub fn create_pixel_buffer_with_aux_attributes(
        &self,
        aux_attributes: Option<&std::collections::HashMap<String, u32>>,
    ) -> Result<CVPixelBuffer, i32> {
        // For now, ignore aux_attributes since we don't have a way to pass them through
        // In a full implementation, this would convert the HashMap to a CFDictionary
        let _ = aux_attributes;
        self.create_pixel_buffer()
    }

    /// Try to create a pixel buffer from the pool without blocking
    ///
    /// Returns None if no buffers are available
    pub fn try_create_pixel_buffer(&self) -> Option<CVPixelBuffer> {
        self.create_pixel_buffer().ok()
    }

    /// Flush the pool with specific options
    ///
    /// Releases buffers based on the provided flags
    pub fn flush_with_options(&self, _flags: u32) {
        // For now, just call regular flush
        // In a full implementation, this would pass flags to the Swift side
        self.flush();
    }

    /// Check if the pool is empty (no available buffers)
    ///
    /// Note: This is an approximation based on whether we can create a buffer
    pub fn is_empty(&self) -> bool {
        self.try_create_pixel_buffer().is_none()
    }

    /// Get the pool attributes
    ///
    /// Returns the raw pointer to the CFDictionary containing pool attributes
    pub fn get_attributes(&self) -> Option<*const std::ffi::c_void> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_pool_get_attributes(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }

    /// Get the pixel buffer attributes
    ///
    /// Returns the raw pointer to the CFDictionary containing pixel buffer attributes
    pub fn get_pixel_buffer_attributes(&self) -> Option<*const std::ffi::c_void> {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_pool_get_pixel_buffer_attributes(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }
}

impl Clone for CVPixelBufferPool {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::cv_pixel_buffer_pool_retain(self.0);
            Self(ptr)
        }
    }
}

impl Drop for CVPixelBufferPool {
    fn drop(&mut self) {
        unsafe {
            ffi::cv_pixel_buffer_pool_release(self.0);
        }
    }
}

unsafe impl Send for CVPixelBufferPool {}
unsafe impl Sync for CVPixelBufferPool {}

impl fmt::Display for CVPixelBufferPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CVPixelBufferPool")
    }
}


/// Audio buffer from an audio sample
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub number_channels: u32,
    pub data_bytes_size: u32,
    data_ptr: *mut std::ffi::c_void,
}

impl PartialEq for AudioBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.number_channels == other.number_channels
            && self.data_bytes_size == other.data_bytes_size
            && self.data_ptr == other.data_ptr
    }
}

impl Eq for AudioBuffer {}

impl std::hash::Hash for AudioBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.number_channels.hash(state);
        self.data_bytes_size.hash(state);
        self.data_ptr.hash(state);
    }
}

impl fmt::Display for AudioBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AudioBuffer({} channels, {} bytes)",
            self.number_channels,
            self.data_bytes_size
        )
    }
}

impl AudioBuffer {
    pub fn data(&self) -> &[u8] {
        if self.data_ptr.is_null() || self.data_bytes_size == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    self.data_ptr as *const u8,
                    self.data_bytes_size as usize,
                )
            }
        }
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        if self.data_ptr.is_null() || self.data_bytes_size == 0 {
            &mut []
        } else {
            unsafe {
                std::slice::from_raw_parts_mut(
                    self.data_ptr.cast::<u8>(),
                    self.data_bytes_size as usize,
                )
            }
        }
    }

    pub fn get_data_byte_size(&self) -> usize {
        self.data_bytes_size as usize
    }
}

/// Reference to an audio buffer with convenience methods
pub struct AudioBufferRef<'a> {
    buffer: &'a AudioBuffer,
}

impl AudioBufferRef<'_> {
    pub fn get_data_byte_size(&self) -> usize {
        self.buffer.get_data_byte_size()
    }

    pub fn data(&self) -> &[u8] {
        self.buffer.data()
    }
}

/// List of audio buffers from an audio sample
#[repr(C)]
#[derive(Debug)]
pub struct AudioBufferListRaw {
    num_buffers: u32,
    buffers_ptr: *mut AudioBuffer,
    buffers_len: usize,
}

pub struct AudioBufferList {
    inner: AudioBufferListRaw,
}

impl AudioBufferList {
    pub fn num_buffers(&self) -> usize {
        self.inner.num_buffers as usize
    }

    pub fn get_number_buffers(&self) -> usize {
        self.num_buffers()
    }

    pub fn get(&self, index: usize) -> Option<&AudioBuffer> {
        if index >= self.num_buffers() {
            None
        } else {
            unsafe {
                Some(&*self.inner.buffers_ptr.add(index))
            }
        }
    }

    pub fn get_buffer(&self, index: usize) -> Option<AudioBufferRef<'_>> {
        self.get(index).map(|buffer| AudioBufferRef { buffer })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut AudioBuffer> {
        if index >= self.num_buffers() {
            None
        } else {
            unsafe {
                Some(&mut *self.inner.buffers_ptr.add(index))
            }
        }
    }

    pub fn iter(&self) -> AudioBufferListIter<'_> {
        AudioBufferListIter {
            list: self,
            index: 0,
        }
    }
}

impl Drop for AudioBufferList {
    fn drop(&mut self) {
        if !self.inner.buffers_ptr.is_null() {
            unsafe {
                Vec::from_raw_parts(
                    self.inner.buffers_ptr,
                    self.inner.buffers_len,
                    self.inner.buffers_len,
                );
            }
        }
    }
}

impl<'a> IntoIterator for &'a AudioBufferList {
    type Item = &'a AudioBuffer;
    type IntoIter = AudioBufferListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl fmt::Display for AudioBufferList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AudioBufferList({} buffers)", self.num_buffers())
    }
}

pub struct AudioBufferListIter<'a> {
    list: &'a AudioBufferList,
    index: usize,
}

impl<'a> Iterator for AudioBufferListIter<'a> {
    type Item = &'a AudioBuffer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.num_buffers() {
            let buffer = self.list.get(self.index);
            self.index += 1;
            buffer
        } else {
            None
        }
    }
}

/// Opaque handle to CMBlockBuffer
#[repr(transparent)]
#[derive(Debug)]
pub struct CMBlockBuffer(*mut std::ffi::c_void);

impl PartialEq for CMBlockBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CMBlockBuffer {}

impl std::hash::Hash for CMBlockBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cm_block_buffer_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl CMBlockBuffer {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CMBlockBuffer` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }
}

unsafe impl Send for CMBlockBuffer {}
unsafe impl Sync for CMBlockBuffer {}

/// Opaque handle to CMFormatDescription
#[repr(transparent)]
#[derive(Debug)]
pub struct CMFormatDescription(*mut std::ffi::c_void);

impl PartialEq for CMFormatDescription {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CMFormatDescription {}

impl std::hash::Hash for CMFormatDescription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::cm_format_description_hash(self.0);
            hash_value.hash(state);
        }
    }
}

/// Common media type constants
pub mod media_types {
    use crate::utils::four_char_code::FourCharCode;
    
    /// Video media type ('vide')
    pub const VIDEO: FourCharCode = FourCharCode::from_bytes(*b"vide");
    /// Audio media type ('soun')
    pub const AUDIO: FourCharCode = FourCharCode::from_bytes(*b"soun");
    /// Muxed media type ('mux ')
    pub const MUXED: FourCharCode = FourCharCode::from_bytes(*b"mux ");
    /// Text/subtitle media type ('text')
    pub const TEXT: FourCharCode = FourCharCode::from_bytes(*b"text");
    /// Closed caption media type ('clcp')
    pub const CLOSED_CAPTION: FourCharCode = FourCharCode::from_bytes(*b"clcp");
    /// Metadata media type ('meta')
    pub const METADATA: FourCharCode = FourCharCode::from_bytes(*b"meta");
    /// Timecode media type ('tmcd')
    pub const TIMECODE: FourCharCode = FourCharCode::from_bytes(*b"tmcd");
}

/// Common codec type constants
pub mod codec_types {
    use crate::utils::four_char_code::FourCharCode;
    
    // Video codecs
    /// H.264/AVC ('avc1')
    pub const H264: FourCharCode = FourCharCode::from_bytes(*b"avc1");
    /// HEVC/H.265 ('hvc1')
    pub const HEVC: FourCharCode = FourCharCode::from_bytes(*b"hvc1");
    /// HEVC/H.265 alternative ('hev1')
    pub const HEVC_2: FourCharCode = FourCharCode::from_bytes(*b"hev1");
    /// JPEG ('jpeg')
    pub const JPEG: FourCharCode = FourCharCode::from_bytes(*b"jpeg");
    /// Apple ProRes 422 ('apcn')
    pub const PRORES_422: FourCharCode = FourCharCode::from_bytes(*b"apcn");
    /// Apple ProRes 4444 ('ap4h')
    pub const PRORES_4444: FourCharCode = FourCharCode::from_bytes(*b"ap4h");
    
    // Audio codecs
    /// AAC ('aac ')
    pub const AAC: FourCharCode = FourCharCode::from_bytes(*b"aac ");
    /// Linear PCM ('lpcm')
    pub const LPCM: FourCharCode = FourCharCode::from_bytes(*b"lpcm");
    /// Apple Lossless ('alac')
    pub const ALAC: FourCharCode = FourCharCode::from_bytes(*b"alac");
    /// Opus ('opus')
    pub const OPUS: FourCharCode = FourCharCode::from_bytes(*b"opus");
    /// FLAC ('flac')
    pub const FLAC: FourCharCode = FourCharCode::from_bytes(*b"flac");
}

impl CMFormatDescription {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `CMFormatDescription` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Get the media type (video, audio, etc.)
    pub fn get_media_type(&self) -> u32 {
        unsafe { ffi::cm_format_description_get_media_type(self.0) }
    }

    /// Get the media type as FourCharCode
    pub fn media_type(&self) -> crate::utils::four_char_code::FourCharCode {
        crate::utils::four_char_code::FourCharCode::from(self.get_media_type())
    }

    /// Get the media subtype (codec type)
    pub fn get_media_subtype(&self) -> u32 {
        unsafe { ffi::cm_format_description_get_media_subtype(self.0) }
    }

    /// Get the media subtype as FourCharCode
    pub fn media_subtype(&self) -> crate::utils::four_char_code::FourCharCode {
        crate::utils::four_char_code::FourCharCode::from(self.get_media_subtype())
    }

    /// Get format description extensions
    pub fn get_extensions(&self) -> Option<*const std::ffi::c_void> {
        unsafe {
            let ptr = ffi::cm_format_description_get_extensions(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }

    /// Check if this is a video format description
    pub fn is_video(&self) -> bool {
        self.media_type() == media_types::VIDEO
    }

    /// Check if this is an audio format description
    pub fn is_audio(&self) -> bool {
        self.media_type() == media_types::AUDIO
    }

    /// Check if this is a muxed format description
    pub fn is_muxed(&self) -> bool {
        self.media_type() == media_types::MUXED
    }

    /// Check if this is a text/subtitle format description
    pub fn is_text(&self) -> bool {
        self.media_type() == media_types::TEXT
    }

    /// Check if this is a closed caption format description
    pub fn is_closed_caption(&self) -> bool {
        self.media_type() == media_types::CLOSED_CAPTION
    }

    /// Check if this is a metadata format description
    pub fn is_metadata(&self) -> bool {
        self.media_type() == media_types::METADATA
    }

    /// Check if this is a timecode format description
    pub fn is_timecode(&self) -> bool {
        self.media_type() == media_types::TIMECODE
    }

    /// Get a human-readable string for the media type
    pub fn media_type_string(&self) -> String {
        self.media_type().display()
    }

    /// Get a human-readable string for the media subtype (codec)
    pub fn media_subtype_string(&self) -> String {
        self.media_subtype().display()
    }

    /// Check if the codec is H.264
    pub fn is_h264(&self) -> bool {
        self.media_subtype() == codec_types::H264
    }

    /// Check if the codec is HEVC/H.265
    pub fn is_hevc(&self) -> bool {
        let subtype = self.media_subtype();
        subtype == codec_types::HEVC || subtype == codec_types::HEVC_2
    }

    /// Check if the codec is AAC
    pub fn is_aac(&self) -> bool {
        self.media_subtype() == codec_types::AAC
    }

    /// Check if the codec is PCM
    pub fn is_pcm(&self) -> bool {
        self.media_subtype() == codec_types::LPCM
    }

    /// Check if the codec is ProRes
    pub fn is_prores(&self) -> bool {
        let subtype = self.media_subtype();
        subtype == codec_types::PRORES_422 || subtype == codec_types::PRORES_4444
    }

    /// Check if the codec is Apple Lossless (ALAC)
    pub fn is_alac(&self) -> bool {
        self.media_subtype() == codec_types::ALAC
    }
}

impl Clone for CMFormatDescription {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::cm_format_description_retain(self.0);
            Self(ptr)
        }
    }
}

impl Drop for CMFormatDescription {
    fn drop(&mut self) {
        unsafe {
            ffi::cm_format_description_release(self.0);
        }
    }
}

unsafe impl Send for CMFormatDescription {}
unsafe impl Sync for CMFormatDescription {}

impl fmt::Display for CMFormatDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CMFormatDescription(type: 0x{:08X}, subtype: 0x{:08X})",
            self.get_media_type(),
            self.get_media_subtype()
        )
    }
}

/// Opaque handle to IOSurface
#[repr(transparent)]
#[derive(Debug)]
pub struct IOSurface(*mut std::ffi::c_void);

impl PartialEq for IOSurface {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for IOSurface {}

impl std::hash::Hash for IOSurface {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            let hash_value = ffi::io_surface_hash(self.0);
            hash_value.hash(state);
        }
    }
}

impl IOSurface {
    pub fn from_raw(ptr: *mut std::ffi::c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    /// # Safety
    /// The caller must ensure the pointer is a valid `IOSurface` pointer.
    pub unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    pub fn get_width(&self) -> usize {
        unsafe { ffi::io_surface_get_width(self.0) }
    }

    pub fn get_height(&self) -> usize {
        unsafe { ffi::io_surface_get_height(self.0) }
    }

    pub fn get_bytes_per_row(&self) -> usize {
        unsafe { ffi::io_surface_get_bytes_per_row(self.0) }
    }
}

impl Drop for IOSurface {
    fn drop(&mut self) {
        unsafe {
            ffi::io_surface_release(self.0);
        }
    }
}

impl Clone for IOSurface {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = ffi::io_surface_retain(self.0);
            Self(ptr)
        }
    }
}

unsafe impl Send for IOSurface {}
unsafe impl Sync for IOSurface {}

impl fmt::Display for IOSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IOSurface({}x{}, {} bytes/row)",
            self.get_width(),
            self.get_height(),
            self.get_bytes_per_row()
        )
    }
}

pub mod ffi {

    extern "C" {
        pub fn cm_sample_buffer_get_image_buffer(
            sample_buffer: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        pub fn cm_sample_buffer_get_frame_status(
            sample_buffer: *mut std::ffi::c_void,
        ) -> i32;
        pub fn cm_sample_buffer_get_presentation_timestamp(
            sample_buffer: *mut std::ffi::c_void,
            out_value: *mut i64,
            out_timescale: *mut i32,
            out_flags: *mut u32,
            out_epoch: *mut i64,
        );
        pub fn cm_sample_buffer_get_duration(
            sample_buffer: *mut std::ffi::c_void,
            out_value: *mut i64,
            out_timescale: *mut i32,
            out_flags: *mut u32,
            out_epoch: *mut i64,
        );
        pub fn cm_sample_buffer_release(sample_buffer: *mut std::ffi::c_void);
        pub fn cm_sample_buffer_retain(sample_buffer: *mut std::ffi::c_void);
        pub fn cm_sample_buffer_is_valid(sample_buffer: *mut std::ffi::c_void) -> bool;
        pub fn cm_sample_buffer_get_num_samples(sample_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cm_sample_buffer_get_audio_buffer_list(
            sample_buffer: *mut std::ffi::c_void,
            out_num_buffers: *mut u32,
            out_buffers_ptr: *mut *mut std::ffi::c_void,
            out_buffers_len: *mut usize,
        );
        pub fn cm_sample_buffer_get_data_buffer(
            sample_buffer: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;

        pub fn cm_sample_buffer_get_decode_timestamp(
            sample_buffer: *mut std::ffi::c_void,
            out_value: *mut i64,
            out_timescale: *mut i32,
            out_flags: *mut u32,
            out_epoch: *mut i64,
        );
        pub fn cm_sample_buffer_get_output_presentation_timestamp(
            sample_buffer: *mut std::ffi::c_void,
            out_value: *mut i64,
            out_timescale: *mut i32,
            out_flags: *mut u32,
            out_epoch: *mut i64,
        );
        pub fn cm_sample_buffer_set_output_presentation_timestamp(
            sample_buffer: *mut std::ffi::c_void,
            value: i64,
            timescale: i32,
            flags: u32,
            epoch: i64,
        ) -> i32;
        pub fn cm_sample_buffer_get_sample_size(
            sample_buffer: *mut std::ffi::c_void,
            sample_index: usize,
        ) -> usize;
        pub fn cm_sample_buffer_get_total_sample_size(sample_buffer: *mut std::ffi::c_void)
            -> usize;
        pub fn cm_sample_buffer_is_ready_for_data_access(
            sample_buffer: *mut std::ffi::c_void,
        ) -> bool;
        pub fn cm_sample_buffer_make_data_ready(sample_buffer: *mut std::ffi::c_void) -> i32;

        // New CMSampleBuffer APIs
        pub fn cm_sample_buffer_get_format_description(
            sample_buffer: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        pub fn cm_sample_buffer_get_sample_timing_info(
            sample_buffer: *mut std::ffi::c_void,
            sample_index: usize,
            out_duration_value: *mut i64,
            out_duration_timescale: *mut i32,
            out_duration_flags: *mut u32,
            out_duration_epoch: *mut i64,
            out_pts_value: *mut i64,
            out_pts_timescale: *mut i32,
            out_pts_flags: *mut u32,
            out_pts_epoch: *mut i64,
            out_dts_value: *mut i64,
            out_dts_timescale: *mut i32,
            out_dts_flags: *mut u32,
            out_dts_epoch: *mut i64,
        ) -> i32;
        pub fn cm_sample_buffer_invalidate(sample_buffer: *mut std::ffi::c_void) -> i32;
        pub fn cm_sample_buffer_create_copy_with_new_timing(
            sample_buffer: *mut std::ffi::c_void,
            num_timing_infos: usize,
            timing_info_array: *const std::ffi::c_void,
            sample_buffer_out: *mut *mut std::ffi::c_void,
        ) -> i32;
        pub fn cm_sample_buffer_copy_pcm_data_into_audio_buffer_list(
            sample_buffer: *mut std::ffi::c_void,
            frame_offset: i32,
            num_frames: i32,
            buffer_list: *mut std::ffi::c_void,
        ) -> i32;

        // CMFormatDescription APIs
        pub fn cm_format_description_get_media_type(
            format_description: *mut std::ffi::c_void,
        ) -> u32;
        pub fn cm_format_description_get_media_subtype(
            format_description: *mut std::ffi::c_void,
        ) -> u32;
        pub fn cm_format_description_get_extensions(
            format_description: *mut std::ffi::c_void,
        ) -> *const std::ffi::c_void;
        pub fn cm_format_description_retain(
            format_description: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        pub fn cm_format_description_release(format_description: *mut std::ffi::c_void);

        // Hash functions
        pub fn cm_sample_buffer_hash(sample_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cv_pixel_buffer_hash(pixel_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cv_pixel_buffer_pool_hash(pool: *mut std::ffi::c_void) -> usize;
        pub fn cm_block_buffer_hash(block_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cm_format_description_hash(format_description: *mut std::ffi::c_void) -> usize;
        pub fn io_surface_hash(surface: *mut std::ffi::c_void) -> usize;

        pub fn cv_pixel_buffer_get_width(pixel_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cv_pixel_buffer_get_height(pixel_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cv_pixel_buffer_get_pixel_format_type(pixel_buffer: *mut std::ffi::c_void) -> u32;
        pub fn cv_pixel_buffer_get_bytes_per_row(pixel_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cv_pixel_buffer_lock_base_address(
            pixel_buffer: *mut std::ffi::c_void,
            flags: u32,
        ) -> i32;
        pub fn cv_pixel_buffer_unlock_base_address(
            pixel_buffer: *mut std::ffi::c_void,
            flags: u32,
        ) -> i32;
        pub fn cv_pixel_buffer_get_base_address(
            pixel_buffer: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        pub fn cv_pixel_buffer_get_io_surface(
            pixel_buffer: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        pub fn cv_pixel_buffer_release(pixel_buffer: *mut std::ffi::c_void);
        pub fn cv_pixel_buffer_retain(
            pixel_buffer: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        pub fn cv_pixel_buffer_create(
            width: usize,
            height: usize,
            pixel_format_type: u32,
            pixel_buffer_out: *mut *mut std::ffi::c_void,
        ) -> i32;
        pub fn cv_pixel_buffer_create_with_bytes(
            width: usize,
            height: usize,
            pixel_format_type: u32,
            base_address: *mut std::ffi::c_void,
            bytes_per_row: usize,
            pixel_buffer_out: *mut *mut std::ffi::c_void,
        ) -> i32;
        pub fn cv_pixel_buffer_fill_extended_pixels(pixel_buffer: *mut std::ffi::c_void) -> i32;

        // New CVPixelBuffer APIs
        pub fn cv_pixel_buffer_create_with_planar_bytes(
            width: usize,
            height: usize,
            pixel_format_type: u32,
            num_planes: usize,
            plane_base_addresses: *const *mut std::ffi::c_void,
            plane_widths: *const usize,
            plane_heights: *const usize,
            plane_bytes_per_row: *const usize,
            pixel_buffer_out: *mut *mut std::ffi::c_void,
        ) -> i32;
        pub fn cv_pixel_buffer_create_with_io_surface(
            io_surface: *mut std::ffi::c_void,
            pixel_buffer_out: *mut *mut std::ffi::c_void,
        ) -> i32;
        pub fn cv_pixel_buffer_get_type_id() -> usize;

        // CVPixelBufferPool APIs
        pub fn cv_pixel_buffer_pool_create(
            width: usize,
            height: usize,
            pixel_format_type: u32,
            max_buffers: usize,
            pool_out: *mut *mut std::ffi::c_void,
        ) -> i32;
        pub fn cv_pixel_buffer_pool_create_pixel_buffer(
            pool: *mut std::ffi::c_void,
            pixel_buffer_out: *mut *mut std::ffi::c_void,
        ) -> i32;
        pub fn cv_pixel_buffer_pool_flush(pool: *mut std::ffi::c_void);
        pub fn cv_pixel_buffer_pool_get_type_id() -> usize;
        pub fn cv_pixel_buffer_pool_retain(
            pool: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        pub fn cv_pixel_buffer_pool_release(pool: *mut std::ffi::c_void);

        // Additional pool APIs
        pub fn cv_pixel_buffer_pool_get_attributes(
            pool: *mut std::ffi::c_void,
        ) -> *const std::ffi::c_void;
        pub fn cv_pixel_buffer_pool_get_pixel_buffer_attributes(
            pool: *mut std::ffi::c_void,
        ) -> *const std::ffi::c_void;

        pub fn cv_pixel_buffer_get_data_size(pixel_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cv_pixel_buffer_is_planar(pixel_buffer: *mut std::ffi::c_void) -> bool;
        pub fn cv_pixel_buffer_get_plane_count(pixel_buffer: *mut std::ffi::c_void) -> usize;
        pub fn cv_pixel_buffer_get_width_of_plane(
            pixel_buffer: *mut std::ffi::c_void,
            plane_index: usize,
        ) -> usize;
        pub fn cv_pixel_buffer_get_height_of_plane(
            pixel_buffer: *mut std::ffi::c_void,
            plane_index: usize,
        ) -> usize;
        pub fn cv_pixel_buffer_get_base_address_of_plane(
            pixel_buffer: *mut std::ffi::c_void,
            plane_index: usize,
        ) -> *mut std::ffi::c_void;
        pub fn cv_pixel_buffer_get_bytes_per_row_of_plane(
            pixel_buffer: *mut std::ffi::c_void,
            plane_index: usize,
        ) -> usize;
        pub fn cv_pixel_buffer_get_extended_pixels(
            pixel_buffer: *mut std::ffi::c_void,
            extra_columns_on_left: *mut usize,
            extra_columns_on_right: *mut usize,
            extra_rows_on_top: *mut usize,
            extra_rows_on_bottom: *mut usize,
        );

        pub fn cm_sample_buffer_create_for_image_buffer(
            image_buffer: *mut std::ffi::c_void,
            presentation_time_value: i64,
            presentation_time_scale: i32,
            duration_value: i64,
            duration_scale: i32,
            sample_buffer_out: *mut *mut std::ffi::c_void,
        ) -> i32;

        pub fn io_surface_get_width(surface: *mut std::ffi::c_void) -> usize;
        pub fn io_surface_get_height(surface: *mut std::ffi::c_void) -> usize;
        pub fn io_surface_get_bytes_per_row(surface: *mut std::ffi::c_void) -> usize;
        pub fn io_surface_release(surface: *mut std::ffi::c_void);
        pub fn io_surface_retain(surface: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    }
}
