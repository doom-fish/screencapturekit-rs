//! `SCRecordingOutput` - Direct video file recording
//!
//! Available on macOS 15.0+
//! Provides direct encoding of screen capture to video files.
//!
//! Requires the `macos_15_0` feature flag to be enabled.

use std::ffi::c_void;
use std::path::Path;

use crate::cm::CMTime;

/// Video codec for recording
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCRecordingOutputCodec {
    /// H.264 codec
    #[default]
    H264 = 0,
    /// H.265/HEVC codec
    HEVC = 1,
}

/// Output file type for recording
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCRecordingOutputFileType {
    /// MPEG-4 file (.mp4)
    #[default]
    MP4 = 0,
    /// `QuickTime` movie (.mov)
    MOV = 1,
}

/// Configuration for recording output
pub struct SCRecordingOutputConfiguration {
    ptr: *const c_void,
}

impl SCRecordingOutputConfiguration {
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { crate::ffi::sc_recording_output_configuration_create() };
        Self { ptr }
    }

    /// Set the output file URL
    pub fn set_output_url(&mut self, path: &Path) {
        if let Some(path_str) = path.to_str() {
            if let Ok(c_path) = std::ffi::CString::new(path_str) {
                unsafe {
                    crate::ffi::sc_recording_output_configuration_set_output_url(
                        self.ptr,
                        c_path.as_ptr(),
                    );
                }
            }
        }
    }

    /// Set the video codec
    pub fn set_video_codec(&mut self, codec: SCRecordingOutputCodec) {
        unsafe {
            crate::ffi::sc_recording_output_configuration_set_video_codec(self.ptr, codec as i32);
        }
    }

    /// Get the video codec
    pub fn get_video_codec(&self) -> SCRecordingOutputCodec {
        let value =
            unsafe { crate::ffi::sc_recording_output_configuration_get_video_codec(self.ptr) };
        match value {
            1 => SCRecordingOutputCodec::HEVC,
            _ => SCRecordingOutputCodec::H264,
        }
    }

    /// Set the output file type
    pub fn set_output_file_type(&mut self, file_type: SCRecordingOutputFileType) {
        unsafe {
            crate::ffi::sc_recording_output_configuration_set_output_file_type(
                self.ptr,
                file_type as i32,
            );
        }
    }

    /// Get the output file type
    pub fn get_output_file_type(&self) -> SCRecordingOutputFileType {
        let value =
            unsafe { crate::ffi::sc_recording_output_configuration_get_output_file_type(self.ptr) };
        match value {
            1 => SCRecordingOutputFileType::MOV,
            _ => SCRecordingOutputFileType::MP4,
        }
    }

    /// Get the number of available video codecs
    pub fn available_video_codecs_count(&self) -> usize {
        let count = unsafe {
            crate::ffi::sc_recording_output_configuration_get_available_video_codecs_count(self.ptr)
        };
        #[allow(clippy::cast_sign_loss)]
        if count > 0 {
            count as usize
        } else {
            0
        }
    }

    /// Get the number of available output file types
    pub fn available_output_file_types_count(&self) -> usize {
        let count = unsafe {
            crate::ffi::sc_recording_output_configuration_get_available_output_file_types_count(
                self.ptr,
            )
        };
        #[allow(clippy::cast_sign_loss)]
        if count > 0 {
            count as usize
        } else {
            0
        }
    }

    /// Set the average bitrate in bits per second
    pub fn set_average_bitrate(&mut self, bitrate: i64) {
        unsafe {
            crate::ffi::sc_recording_output_configuration_set_average_bitrate(self.ptr, bitrate);
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const c_void {
        self.ptr
    }
}

impl Default for SCRecordingOutputConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SCRecordingOutputConfiguration {
    fn clone(&self) -> Self {
        unsafe {
            Self {
                ptr: crate::ffi::sc_recording_output_configuration_retain(self.ptr),
            }
        }
    }
}

impl Drop for SCRecordingOutputConfiguration {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                crate::ffi::sc_recording_output_configuration_release(self.ptr);
            }
        }
    }
}

impl std::fmt::Debug for SCRecordingOutputConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCRecordingOutputConfiguration")
            .field("video_codec", &self.get_video_codec())
            .field("file_type", &self.get_output_file_type())
            .finish()
    }
}

/// Delegate for recording output events
///
/// Implement this trait to receive notifications about recording lifecycle events.
pub trait SCRecordingOutputDelegate: Send + 'static {
    /// Called when recording starts successfully
    fn recording_did_start(&self) {}
    /// Called when recording fails with an error
    fn recording_did_fail(&self, _error: String) {}
    /// Called when recording finishes successfully
    fn recording_did_finish(&self) {}
}

/// Recording output for direct video file encoding
///
/// Available on macOS 15.0+
pub struct SCRecordingOutput {
    ptr: *const c_void,
}

impl SCRecordingOutput {
    /// Create a new recording output with configuration
    ///
    /// # Errors
    /// Returns None if the system is not macOS 15.0+ or creation fails
    pub fn new(config: &SCRecordingOutputConfiguration) -> Option<Self> {
        let ptr = unsafe { crate::ffi::sc_recording_output_create(config.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    /// Get the current recorded duration
    pub fn recorded_duration(&self) -> CMTime {
        let mut value: i64 = 0;
        let mut timescale: i32 = 0;
        unsafe {
            crate::ffi::sc_recording_output_get_recorded_duration(
                self.ptr,
                &mut value,
                &mut timescale,
            );
        }
        CMTime {
            value,
            timescale,
            flags: 0,
            epoch: 0,
        }
    }

    /// Get the current recorded file size in bytes
    pub fn recorded_file_size(&self) -> i64 {
        unsafe { crate::ffi::sc_recording_output_get_recorded_file_size(self.ptr) }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const c_void {
        self.ptr
    }
}

impl Clone for SCRecordingOutput {
    fn clone(&self) -> Self {
        unsafe {
            Self {
                ptr: crate::ffi::sc_recording_output_retain(self.ptr),
            }
        }
    }
}

impl Drop for SCRecordingOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                crate::ffi::sc_recording_output_release(self.ptr);
            }
        }
    }
}

// Safety: SCRecordingOutput wraps an Objective-C object that is thread-safe
unsafe impl Send for SCRecordingOutput {}
unsafe impl Sync for SCRecordingOutput {}

// Safety: SCRecordingOutputConfiguration wraps an Objective-C object that is thread-safe
unsafe impl Send for SCRecordingOutputConfiguration {}
unsafe impl Sync for SCRecordingOutputConfiguration {}
