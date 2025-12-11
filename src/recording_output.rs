//! `SCRecordingOutput` - Direct video file recording
//!
//! Available on macOS 15.0+.
//! Provides direct encoding of screen capture to video files with hardware acceleration.
//!
//! Requires the `macos_15_0` feature flag to be enabled.
//!
//! ## When to Use
//!
//! Use `SCRecordingOutput` when you need:
//! - Direct recording to MP4/MOV files without manual encoding
//! - Hardware-accelerated H.264 or HEVC encoding
//! - Recording with automatic file management
//!
//! For custom processing of frames, use [`SCStream`](crate::stream::SCStream) with
//! output handlers instead.
//!
//! ## Example
//!
//! ```no_run
//! use screencapturekit::recording_output::{
//!     SCRecordingOutput, SCRecordingOutputConfiguration, SCRecordingOutputCodec
//! };
//! use screencapturekit::prelude::*;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//! let display = &content.displays()[0];
//! let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
//! let config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080);
//!
//! // Configure recording output
//! let rec_config = SCRecordingOutputConfiguration::new()
//!     .with_output_url(Path::new("/tmp/recording.mp4"))
//!     .with_video_codec(SCRecordingOutputCodec::HEVC);
//!
//! let recording = SCRecordingOutput::new(&rec_config).ok_or("Failed to create recording")?;
//!
//! // Add to stream and start
//! let mut stream = SCStream::new(&filter, &config);
//! stream.add_recording_output(&recording)?;
//! stream.start_capture()?;
//!
//! // ... record for desired duration ...
//!
//! stream.stop_capture()?;
//! stream.remove_recording_output(&recording)?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::ffi::c_void;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::cm::CMTime;

/// Global registry for recording delegates - maps unique ID to delegate entry
static RECORDING_DELEGATE_REGISTRY: Mutex<Option<HashMap<usize, RecordingDelegateEntry>>> =
    Mutex::new(None);

/// Counter for generating unique delegate IDs
static NEXT_DELEGATE_ID: AtomicUsize = AtomicUsize::new(1);

struct RecordingDelegateEntry {
    delegate: Box<dyn SCRecordingOutputDelegate>,
    ref_count: usize,
}

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
    /// ID into the delegate registry, if a delegate was set
    delegate_id: Option<usize>,
}

// C callback trampolines for delegate - ctx is the recording ptr as usize
extern "C" fn recording_started_callback(ctx: *mut c_void) {
    let key = ctx as usize;
    if let Ok(registry) = RECORDING_DELEGATE_REGISTRY.lock() {
        if let Some(ref delegates) = *registry {
            if let Some(entry) = delegates.get(&key) {
                entry.delegate.recording_did_start();
            }
        }
    }
}

extern "C" fn recording_failed_callback(ctx: *mut c_void, error_code: i32, error: *const i8) {
    let key = ctx as usize;
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

    if let Ok(registry) = RECORDING_DELEGATE_REGISTRY.lock() {
        if let Some(ref delegates) = *registry {
            if let Some(entry) = delegates.get(&key) {
                entry.delegate.recording_did_fail(full_error);
            }
        }
    }
}

extern "C" fn recording_finished_callback(ctx: *mut c_void) {
    let key = ctx as usize;
    if let Ok(registry) = RECORDING_DELEGATE_REGISTRY.lock() {
        if let Some(ref delegates) = *registry {
            if let Some(entry) = delegates.get(&key) {
                entry.delegate.recording_did_finish();
            }
        }
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
                delegate_id: None,
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
    ///
    /// # Panics
    /// Panics if the delegate registry mutex is poisoned
    pub fn new_with_delegate<D: SCRecordingOutputDelegate>(
        config: &SCRecordingOutputConfiguration,
        delegate: D,
    ) -> Option<Self> {
        // Generate a unique ID for this delegate
        let delegate_id = NEXT_DELEGATE_ID.fetch_add(1, Ordering::Relaxed);

        // Store delegate in registry before creating recording output
        {
            let mut registry = RECORDING_DELEGATE_REGISTRY.lock().unwrap();
            if registry.is_none() {
                *registry = Some(HashMap::new());
            }
            if let Some(ref mut delegates) = *registry {
                delegates.insert(
                    delegate_id,
                    RecordingDelegateEntry {
                        delegate: Box::new(delegate),
                        ref_count: 1,
                    },
                );
            }
        }

        // Use delegate_id as context
        let ctx = delegate_id as *mut c_void;

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
            // Clean up delegate from registry on failure
            if let Ok(mut registry) = RECORDING_DELEGATE_REGISTRY.lock() {
                if let Some(ref mut delegates) = *registry {
                    delegates.remove(&delegate_id);
                }
            }
            None
        } else {
            Some(Self {
                ptr,
                delegate_id: Some(delegate_id),
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
        // Increment delegate ref count if one exists for this recording
        if let Some(delegate_id) = self.delegate_id {
            if let Ok(mut registry) = RECORDING_DELEGATE_REGISTRY.lock() {
                if let Some(ref mut delegates) = *registry {
                    if let Some(entry) = delegates.get_mut(&delegate_id) {
                        entry.ref_count += 1;
                    }
                }
            }
        }

        unsafe {
            Self {
                ptr: crate::ffi::sc_recording_output_retain(self.ptr),
                delegate_id: self.delegate_id,
            }
        }
    }
}

impl std::fmt::Debug for SCRecordingOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCRecordingOutput")
            .field("recorded_duration", &self.recorded_duration())
            .field("recorded_file_size", &self.recorded_file_size())
            .field("has_delegate", &self.delegate_id.is_some())
            .finish_non_exhaustive()
    }
}

impl Drop for SCRecordingOutput {
    fn drop(&mut self) {
        // Decrement delegate ref count and clean up if this is the last reference
        if let Some(delegate_id) = self.delegate_id {
            let mut should_remove = false;
            if let Ok(mut registry) = RECORDING_DELEGATE_REGISTRY.lock() {
                if let Some(ref mut delegates) = *registry {
                    if let Some(entry) = delegates.get_mut(&delegate_id) {
                        entry.ref_count -= 1;
                        if entry.ref_count == 0 {
                            should_remove = true;
                        }
                    }
                    if should_remove {
                        delegates.remove(&delegate_id);
                    }
                }
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
