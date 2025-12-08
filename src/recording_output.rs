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
    /// Create a new recording output configuration
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { crate::ffi::sc_recording_output_configuration_create() };
        Self { ptr }
    }

    /// Set the output file URL
    #[must_use]
    pub fn with_output_url(self, path: &Path) -> Self {
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
        self
    }

    /// Set the video codec
    #[must_use]
    pub fn with_video_codec(self, codec: SCRecordingOutputCodec) -> Self {
        unsafe {
            crate::ffi::sc_recording_output_configuration_set_video_codec(self.ptr, codec as i32);
        }
        self
    }

    /// Get the video codec
    pub fn video_codec(&self) -> SCRecordingOutputCodec {
        let value =
            unsafe { crate::ffi::sc_recording_output_configuration_get_video_codec(self.ptr) };
        match value {
            1 => SCRecordingOutputCodec::HEVC,
            _ => SCRecordingOutputCodec::H264,
        }
    }

    /// Set the output file type
    #[must_use]
    pub fn with_output_file_type(self, file_type: SCRecordingOutputFileType) -> Self {
        unsafe {
            crate::ffi::sc_recording_output_configuration_set_output_file_type(
                self.ptr,
                file_type as i32,
            );
        }
        self
    }

    /// Get the output file type
    pub fn output_file_type(&self) -> SCRecordingOutputFileType {
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

    /// Get all available video codecs
    ///
    /// Returns a vector of all video codecs that can be used for recording.
    pub fn available_video_codecs(&self) -> Vec<SCRecordingOutputCodec> {
        let count = self.available_video_codecs_count();
        let mut codecs = Vec::with_capacity(count);
        for i in 0..count {
            #[allow(clippy::cast_possible_wrap)]
            let codec_value = unsafe {
                crate::ffi::sc_recording_output_configuration_get_available_video_codec_at(
                    self.ptr, i as isize,
                )
            };
            match codec_value {
                0 => codecs.push(SCRecordingOutputCodec::H264),
                1 => codecs.push(SCRecordingOutputCodec::HEVC),
                _ => {}
            }
        }
        codecs
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

    /// Get all available output file types
    ///
    /// Returns a vector of all file types that can be used for recording output.
    pub fn available_output_file_types(&self) -> Vec<SCRecordingOutputFileType> {
        let count = self.available_output_file_types_count();
        let mut file_types = Vec::with_capacity(count);
        for i in 0..count {
            #[allow(clippy::cast_possible_wrap)]
            let file_type_value = unsafe {
                crate::ffi::sc_recording_output_configuration_get_available_output_file_type_at(
                    self.ptr, i as isize,
                )
            };
            match file_type_value {
                0 => file_types.push(SCRecordingOutputFileType::MP4),
                1 => file_types.push(SCRecordingOutputFileType::MOV),
                _ => {}
            }
        }
        file_types
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
            .field("video_codec", &self.video_codec())
            .field("file_type", &self.output_file_type())
            .finish()
    }
}

/// Delegate for recording output events
///
/// Implement this trait to receive notifications about recording lifecycle events.
///
/// # Examples
///
/// ## Using a struct
///
/// ```
/// use screencapturekit::recording_output::SCRecordingOutputDelegate;
///
/// struct MyRecordingDelegate;
///
/// impl SCRecordingOutputDelegate for MyRecordingDelegate {
///     fn recording_did_start(&self) {
///         println!("Recording started!");
///     }
///     fn recording_did_fail(&self, error: String) {
///         eprintln!("Recording failed: {}", error);
///     }
///     fn recording_did_finish(&self) {
///         println!("Recording finished!");
///     }
/// }
/// ```
///
/// ## Using closures
///
/// Use [`RecordingCallbacks`] to create a delegate from closures:
///
/// ```rust,no_run
/// use screencapturekit::recording_output::{
///     SCRecordingOutput, SCRecordingOutputConfiguration, RecordingCallbacks
/// };
/// use std::path::Path;
///
/// let config = SCRecordingOutputConfiguration::new()
///     .with_output_url(Path::new("/tmp/recording.mp4"));
///
/// let delegate = RecordingCallbacks::new()
///     .on_start(|| println!("Started!"))
///     .on_finish(|| println!("Finished!"))
///     .on_fail(|e| eprintln!("Error: {}", e));
///
/// let recording = SCRecordingOutput::new_with_delegate(&config, delegate);
/// ```
pub trait SCRecordingOutputDelegate: Send + 'static {
    /// Called when recording starts successfully
    fn recording_did_start(&self) {}
    /// Called when recording fails with an error
    fn recording_did_fail(&self, _error: String) {}
    /// Called when recording finishes successfully
    fn recording_did_finish(&self) {}
}

/// Builder for closure-based recording delegate
///
/// Provides a convenient way to create a recording delegate using closures
/// instead of implementing the [`SCRecordingOutputDelegate`] trait.
///
/// # Examples
///
/// ```rust,no_run
/// use screencapturekit::recording_output::{
///     SCRecordingOutput, SCRecordingOutputConfiguration, RecordingCallbacks
/// };
/// use std::path::Path;
///
/// let config = SCRecordingOutputConfiguration::new()
///     .with_output_url(Path::new("/tmp/recording.mp4"));
///
/// // Create delegate with all callbacks
/// let delegate = RecordingCallbacks::new()
///     .on_start(|| println!("Recording started!"))
///     .on_finish(|| println!("Recording finished!"))
///     .on_fail(|error| eprintln!("Recording failed: {}", error));
///
/// let recording = SCRecordingOutput::new_with_delegate(&config, delegate);
///
/// // Or just handle specific events
/// let delegate = RecordingCallbacks::new()
///     .on_fail(|error| eprintln!("Error: {}", error));
/// ```
#[allow(clippy::struct_field_names)]
pub struct RecordingCallbacks {
    on_start: Option<Box<dyn Fn() + Send + 'static>>,
    on_fail: Option<Box<dyn Fn(String) + Send + 'static>>,
    on_finish: Option<Box<dyn Fn() + Send + 'static>>,
}

impl RecordingCallbacks {
    /// Create a new empty callbacks builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            on_start: None,
            on_fail: None,
            on_finish: None,
        }
    }

    /// Set the callback for when recording starts
    #[must_use]
    pub fn on_start<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        self.on_start = Some(Box::new(f));
        self
    }

    /// Set the callback for when recording fails
    #[must_use]
    pub fn on_fail<F>(mut self, f: F) -> Self
    where
        F: Fn(String) + Send + 'static,
    {
        self.on_fail = Some(Box::new(f));
        self
    }

    /// Set the callback for when recording finishes
    #[must_use]
    pub fn on_finish<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        self.on_finish = Some(Box::new(f));
        self
    }
}

impl Default for RecordingCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for RecordingCallbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordingCallbacks")
            .field("on_start", &self.on_start.is_some())
            .field("on_fail", &self.on_fail.is_some())
            .field("on_finish", &self.on_finish.is_some())
            .finish()
    }
}

impl SCRecordingOutputDelegate for RecordingCallbacks {
    fn recording_did_start(&self) {
        if let Some(ref f) = self.on_start {
            f();
        }
    }

    fn recording_did_fail(&self, error: String) {
        if let Some(ref f) = self.on_fail {
            f(error);
        }
    }

    fn recording_did_finish(&self) {
        if let Some(ref f) = self.on_finish {
            f();
        }
    }
}

/// Recording output for direct video file encoding
///
/// Available on macOS 15.0+
pub struct SCRecordingOutput {
    ptr: *const c_void,
    /// Raw pointer to the delegate box, kept alive for the lifetime of the recording output.
    /// The Swift side uses this pointer for callbacks.
    delegate_ptr: Option<*mut Box<dyn SCRecordingOutputDelegate>>,
}

// C callback trampolines for delegate
extern "C" fn recording_started_callback(ctx: *mut c_void) {
    if !ctx.is_null() {
        let delegate = unsafe { &**(ctx as *const Box<dyn SCRecordingOutputDelegate>) };
        delegate.recording_did_start();
    }
}

extern "C" fn recording_failed_callback(ctx: *mut c_void, error_code: i32, error: *const i8) {
    if !ctx.is_null() {
        let delegate = unsafe { &**(ctx as *const Box<dyn SCRecordingOutputDelegate>) };
        let error_str = if error.is_null() {
            String::from("Unknown error")
        } else {
            unsafe { std::ffi::CStr::from_ptr(error) }
                .to_string_lossy()
                .into_owned()
        };

        // Include error code in the message if it's a known SCStreamError
        let full_error = if error_code != 0 {
            crate::error::SCStreamErrorCode::from_raw(error_code).map_or_else(
                || format!("{error_str} (code: {error_code})"),
                |code| format!("{error_str} ({code})"),
            )
        } else {
            error_str
        };

        delegate.recording_did_fail(full_error);
    }
}

extern "C" fn recording_finished_callback(ctx: *mut c_void) {
    if !ctx.is_null() {
        let delegate = unsafe { &**(ctx as *const Box<dyn SCRecordingOutputDelegate>) };
        delegate.recording_did_finish();
    }
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
            Some(Self {
                ptr,
                delegate_ptr: None,
            })
        }
    }

    /// Create a new recording output with configuration and delegate
    ///
    /// The delegate receives callbacks for recording lifecycle events:
    /// - `recording_did_start` - Called when recording begins
    /// - `recording_did_fail` - Called if recording fails with an error
    /// - `recording_did_finish` - Called when recording completes successfully
    ///
    /// # Errors
    /// Returns None if the system is not macOS 15.0+ or creation fails
    pub fn new_with_delegate<D: SCRecordingOutputDelegate>(
        config: &SCRecordingOutputConfiguration,
        delegate: D,
    ) -> Option<Self> {
        let boxed_delegate: Box<dyn SCRecordingOutputDelegate> = Box::new(delegate);
        // We need a stable pointer to the Box itself, so we box the box
        let boxed_box = Box::new(boxed_delegate);
        let raw_ptr = Box::into_raw(boxed_box);
        let ctx = raw_ptr.cast::<c_void>();

        let ptr = unsafe {
            crate::ffi::sc_recording_output_create_with_delegate(
                config.as_ptr(),
                Some(recording_started_callback),
                Some(recording_failed_callback),
                Some(recording_finished_callback),
                ctx,
            )
        };
        if ptr.is_null() {
            // Clean up the leaked box on failure
            unsafe {
                let _ = Box::from_raw(raw_ptr);
            }
            None
        } else {
            // Keep the raw pointer so we can clean it up on drop
            Some(Self {
                ptr,
                delegate_ptr: Some(raw_ptr),
            })
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
                delegate_ptr: None, // Delegate is not cloned - only one owner
            }
        }
    }
}

impl std::fmt::Debug for SCRecordingOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCRecordingOutput")
            .field("recorded_duration", &self.recorded_duration())
            .field("recorded_file_size", &self.recorded_file_size())
            .field("has_delegate", &self.delegate_ptr.is_some())
            .finish_non_exhaustive()
    }
}

impl Drop for SCRecordingOutput {
    fn drop(&mut self) {
        // Clean up the delegate if we own it
        if let Some(delegate_ptr) = self.delegate_ptr.take() {
            unsafe {
                let _ = Box::from_raw(delegate_ptr);
            }
        }
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
