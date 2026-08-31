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
//! let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
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

use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use crate::cm::CMTime;

/// Global registry for recording delegates - maps unique ID to delegate entry
static RECORDING_DELEGATE_REGISTRY: Mutex<Option<HashMap<usize, RecordingDelegateEntry>>> =
    Mutex::new(None);

/// Counter for generating unique delegate IDs
static NEXT_DELEGATE_ID: AtomicUsize = AtomicUsize::new(1);

/// A registry entry.
///
/// The delegate lives behind `Arc` rather than inline in the map so a callback
/// can clone the handle, release the global registry lock, and then run user
/// code without any crate lock held.
struct RecordingDelegateEntry {
    delegate: Arc<dyn SCRecordingOutputDelegate>,
}

/// Look up a delegate handle and release the registry lock before returning.
fn lookup_delegate(key: usize) -> Option<Arc<dyn SCRecordingOutputDelegate>> {
    let registry = RECORDING_DELEGATE_REGISTRY
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    registry
        .as_ref()?
        .get(&key)
        .map(|entry| Arc::clone(&entry.delegate))
}

fn remove_delegate(key: usize) -> Option<RecordingDelegateEntry> {
    let mut registry = RECORDING_DELEGATE_REGISTRY
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    registry
        .as_mut()
        .and_then(|delegates| delegates.remove(&key))
}

/// An `AVVideoCodecType` identifier used for recording.
///
/// The identifier is open-ended: values introduced by future macOS releases
/// remain distinct and can be passed back to the framework.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SCRecordingOutputCodec(Cow<'static, str>);

impl SCRecordingOutputCodec {
    /// H.264 (`AVVideoCodecType.h264`)
    pub const H264: Self = Self(Cow::Borrowed("avc1"));
    /// H.265 / HEVC (`AVVideoCodecType.hevc`)
    pub const HEVC: Self = Self(Cow::Borrowed("hvc1"));
    /// Motion JPEG (`AVVideoCodecType.jpeg`)
    pub const JPEG: Self = Self(Cow::Borrowed("jpeg"));
    /// Apple `ProRes` 422 (`AVVideoCodecType.proRes422`)
    pub const PRO_RES_422: Self = Self(Cow::Borrowed("apcn"));
    /// Apple `ProRes` 4444 (`AVVideoCodecType.proRes4444`)
    pub const PRO_RES_4444: Self = Self(Cow::Borrowed("ap4h"));
    /// HEVC with an alpha channel (`AVVideoCodecType.hevcWithAlpha`)
    pub const HEVC_WITH_ALPHA: Self = Self(Cow::Borrowed("muxa"));
    /// Apple `ProRes` 422 HQ (`AVVideoCodecType.proRes422HQ`)
    pub const PRO_RES_422_HQ: Self = Self(Cow::Borrowed("apch"));
    /// Apple `ProRes` 422 LT (`AVVideoCodecType.proRes422LT`)
    pub const PRO_RES_422_LT: Self = Self(Cow::Borrowed("apcs"));
    /// Apple `ProRes` 422 Proxy (`AVVideoCodecType.proRes422Proxy`)
    pub const PRO_RES_422_PROXY: Self = Self(Cow::Borrowed("apco"));

    /// Construct an arbitrary codec identifier.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRecordingIdentifier`] when the identifier contains an
    /// interior NUL byte.
    pub fn from_identifier(
        identifier: impl Into<String>,
    ) -> Result<Self, InvalidRecordingIdentifier> {
        let identifier = identifier.into();
        if identifier.as_bytes().contains(&0) {
            Err(InvalidRecordingIdentifier)
        } else {
            Ok(Self(Cow::Owned(identifier)))
        }
    }

    /// The underlying `AVVideoCodecType.rawValue`.
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.0.as_ref()
    }
}

impl Default for SCRecordingOutputCodec {
    fn default() -> Self {
        Self::H264
    }
}

impl std::fmt::Display for SCRecordingOutputCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.identifier() {
            "avc1" => f.write_str("H.264"),
            "hvc1" => f.write_str("HEVC"),
            "jpeg" => f.write_str("JPEG"),
            "apcn" => f.write_str("ProRes 422"),
            "ap4h" => f.write_str("ProRes 4444"),
            "muxa" => f.write_str("HEVC with alpha"),
            "apch" => f.write_str("ProRes 422 HQ"),
            "apcs" => f.write_str("ProRes 422 LT"),
            "apco" => f.write_str("ProRes 422 Proxy"),
            other => write!(f, "codec {other}"),
        }
    }
}

/// An `AVFileType` identifier used for recording output.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SCRecordingOutputFileType(Cow<'static, str>);

impl SCRecordingOutputFileType {
    /// MPEG-4 file (`.mp4`)
    pub const MP4: Self = Self(Cow::Borrowed("public.mpeg-4"));
    /// `QuickTime` movie (`.mov`)
    pub const MOV: Self = Self(Cow::Borrowed("com.apple.quicktime-movie"));
    /// iTunes video (`.m4v`)
    pub const M4V: Self = Self(Cow::Borrowed("com.apple.m4v-video"));
    /// iTunes audio (`.m4a`)
    pub const M4A: Self = Self(Cow::Borrowed("com.apple.m4a-audio"));
    /// 3GPP file (`.3gp`)
    pub const MOBILE_3GPP: Self = Self(Cow::Borrowed("public.3gpp"));

    /// Construct an arbitrary file type identifier.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRecordingIdentifier`] when the identifier contains an
    /// interior NUL byte.
    pub fn from_identifier(
        identifier: impl Into<String>,
    ) -> Result<Self, InvalidRecordingIdentifier> {
        let identifier = identifier.into();
        if identifier.as_bytes().contains(&0) {
            Err(InvalidRecordingIdentifier)
        } else {
            Ok(Self(Cow::Owned(identifier)))
        }
    }

    /// The underlying `AVFileType.rawValue`.
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.0.as_ref()
    }

    /// Conventional file extension, when this crate knows the file type.
    #[must_use]
    pub fn extension(&self) -> Option<&'static str> {
        match self.identifier() {
            "public.mpeg-4" => Some("mp4"),
            "com.apple.quicktime-movie" => Some("mov"),
            "com.apple.m4v-video" => Some("m4v"),
            "com.apple.m4a-audio" => Some("m4a"),
            "public.3gpp" => Some("3gp"),
            _ => None,
        }
    }
}

impl Default for SCRecordingOutputFileType {
    fn default() -> Self {
        Self::MP4
    }
}

impl std::fmt::Display for SCRecordingOutputFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.identifier() {
            "public.mpeg-4" => f.write_str("MP4"),
            "com.apple.quicktime-movie" => f.write_str("MOV"),
            "com.apple.m4v-video" => f.write_str("M4V"),
            "com.apple.m4a-audio" => f.write_str("M4A"),
            "public.3gpp" => f.write_str("3GPP"),
            other => write!(f, "file type {other}"),
        }
    }
}

/// A recording codec or file-type identifier contained an interior NUL byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidRecordingIdentifier;

impl std::fmt::Display for InvalidRecordingIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("recording identifier contains an interior NUL byte")
    }
}

impl std::error::Error for InvalidRecordingIdentifier {}

/// Configuration for recording output
pub struct SCRecordingOutputConfiguration {
    ptr: *const c_void,
}

/// Why a path could not be handed to `SCRecordingOutputConfiguration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidOutputPath {
    /// Foundation file URLs require a valid UTF-8 path.
    NotUtf8,
    /// The path contains an interior NUL byte, which would truncate it.
    InteriorNul,
}

impl std::fmt::Display for InvalidOutputPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotUtf8 => f.write_str("output path is not valid UTF-8"),
            Self::InteriorNul => f.write_str("output path contains an interior NUL byte"),
        }
    }
}

impl std::error::Error for InvalidOutputPath {}

impl SCRecordingOutputConfiguration {
    /// Create a new recording output configuration
    ///
    /// # Panics
    ///
    /// Panics when run on macOS older than 15.0. Use [`Self::try_new`] when
    /// runtime availability is not already known.
    #[must_use]
    pub fn new() -> Self {
        Self::try_new().expect("SCRecordingOutput requires macOS 15.0 or later")
    }

    /// Create a recording configuration when recording output is available.
    #[must_use]
    pub fn try_new() -> Option<Self> {
        if !SCRecordingOutput::is_available() {
            return None;
        }
        let ptr = unsafe { crate::ffi::sc_recording_output_configuration_create() };
        (!ptr.is_null()).then_some(Self { ptr })
    }

    /// Set the output file URL.
    ///
    /// Paths that are not valid UTF-8 or contain an interior NUL byte are
    /// ignored. Use
    /// [`try_with_output_url`](Self::try_with_output_url) to observe rejection.
    #[must_use]
    pub fn with_output_url(self, path: &Path) -> Self {
        match self.try_with_output_url(path) {
            Ok(config) => config,
            Err((config, error)) => {
                eprintln!("SCRecordingOutputConfiguration: {error}; output URL was not changed");
                config
            }
        }
    }

    /// Set the output file URL, reporting paths that cannot cross the C
    /// boundary.
    ///
    /// # Errors
    ///
    /// Returns the unchanged configuration together with an
    /// [`InvalidOutputPath`] when `path` is not valid UTF-8 or contains an
    /// interior NUL byte.
    pub fn try_with_output_url(self, path: &Path) -> Result<Self, (Self, InvalidOutputPath)> {
        let Some(path) = path.to_str() else {
            return Err((self, InvalidOutputPath::NotUtf8));
        };
        let Ok(c_path) = std::ffi::CString::new(path) else {
            return Err((self, InvalidOutputPath::InteriorNul));
        };
        unsafe {
            crate::ffi::sc_recording_output_configuration_set_output_url(self.ptr, c_path.as_ptr());
        }
        Ok(self)
    }

    /// Get the configured output file URL.
    pub fn output_url(&self) -> Option<PathBuf> {
        use std::os::unix::ffi::OsStringExt;

        unsafe {
            let path =
                crate::ffi::sc_recording_output_configuration_get_output_path_owned(self.ptr);
            if path.is_null() {
                return None;
            }
            let bytes = std::ffi::CStr::from_ptr(path).to_bytes().to_vec();
            crate::ffi::sc_free_string(path);
            Some(PathBuf::from(std::ffi::OsString::from_vec(bytes)))
        }
    }

    /// Set the video codec
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_video_codec(self, codec: SCRecordingOutputCodec) -> Self {
        // SAFETY: the type's private field can only be created by constants or
        // `from_identifier`, which rejects interior NUL bytes.
        let codec = unsafe {
            std::ffi::CString::from_vec_unchecked(codec.identifier().as_bytes().to_vec())
        };
        unsafe {
            crate::ffi::sc_recording_output_configuration_set_video_codec_identifier(
                self.ptr,
                codec.as_ptr(),
            );
        }
        self
    }

    /// Get the video codec
    pub fn video_codec(&self) -> SCRecordingOutputCodec {
        let identifier = unsafe {
            crate::utils::ffi_string::ffi_string_owned(|| {
                crate::ffi::sc_recording_output_configuration_get_video_codec_identifier_owned(
                    self.ptr,
                )
            })
        }
        .unwrap_or_else(|| SCRecordingOutputCodec::H264.identifier().to_string());
        SCRecordingOutputCodec(Cow::Owned(identifier))
    }

    /// Set the output file type
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn with_output_file_type(self, file_type: SCRecordingOutputFileType) -> Self {
        // SAFETY: the type's private field can only be created by constants or
        // `from_identifier`, which rejects interior NUL bytes.
        let file_type = unsafe {
            std::ffi::CString::from_vec_unchecked(file_type.identifier().as_bytes().to_vec())
        };
        unsafe {
            crate::ffi::sc_recording_output_configuration_set_output_file_type_identifier(
                self.ptr,
                file_type.as_ptr(),
            );
        }
        self
    }

    /// Get the output file type
    pub fn output_file_type(&self) -> SCRecordingOutputFileType {
        let identifier = unsafe {
            crate::utils::ffi_string::ffi_string_owned(|| {
                crate::ffi::sc_recording_output_configuration_get_output_file_type_identifier_owned(
                    self.ptr,
                )
            })
        }
        .unwrap_or_else(|| SCRecordingOutputFileType::MP4.identifier().to_string());
        SCRecordingOutputFileType(Cow::Owned(identifier))
    }

    /// Get the number of available video codecs
    pub fn available_video_codecs_count(&self) -> usize {
        let count = unsafe {
            crate::ffi::sc_recording_output_configuration_get_available_video_codecs_count(self.ptr)
        };
        usize::try_from(count).unwrap_or(0)
    }

    /// Get all available video codecs
    ///
    /// Returns a vector of all video codecs that can be used for recording.
    /// The length always matches
    /// [`available_video_codecs_count`](Self::available_video_codecs_count):
    /// a codec this crate has no constant for is preserved by identifier.
    pub fn available_video_codecs(&self) -> Vec<SCRecordingOutputCodec> {
        let count = self.available_video_codecs_count();
        let mut codecs = Vec::with_capacity(count);
        for i in 0..count {
            let Ok(index) = isize::try_from(i) else { break };
            let identifier = unsafe {
                crate::utils::ffi_string::ffi_string_owned(|| {
                    crate::ffi::sc_recording_output_configuration_get_available_video_codec_identifier_at_owned(
                        self.ptr,
                        index,
                    )
                })
            };
            if let Some(identifier) = identifier {
                codecs.push(SCRecordingOutputCodec(Cow::Owned(identifier)));
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
        usize::try_from(count).unwrap_or(0)
    }

    /// Get all available output file types
    ///
    /// Returns a vector of all file types that can be used for recording
    /// output. The length always matches
    /// [`available_output_file_types_count`](Self::available_output_file_types_count).
    pub fn available_output_file_types(&self) -> Vec<SCRecordingOutputFileType> {
        let count = self.available_output_file_types_count();
        let mut file_types = Vec::with_capacity(count);
        for i in 0..count {
            let Ok(index) = isize::try_from(i) else { break };
            let identifier = unsafe {
                crate::utils::ffi_string::ffi_string_owned(|| {
                    crate::ffi::sc_recording_output_configuration_get_available_output_file_type_identifier_at_owned(
                        self.ptr,
                        index,
                    )
                })
            };
            if let Some(identifier) = identifier {
                file_types.push(SCRecordingOutputFileType(Cow::Owned(identifier)));
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

crate::utils::retained::sc_retained!(
    SCRecordingOutputConfiguration,
    field = ptr,
    release = crate::ffi::sc_recording_output_configuration_release,
);

impl Clone for SCRecordingOutputConfiguration {
    /// Deep-copies the underlying `SCRecordingOutputConfiguration`.
    ///
    /// The native object is a mutable class. Retaining it would make every
    /// clone an alias — reconfiguring one handle would silently reconfigure
    /// the others, and two threads configuring "their own" clone would race on
    /// the same non-atomic properties, which `Send`/`Sync` on this type
    /// promises cannot happen.
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { crate::ffi::sc_recording_output_configuration_copy(self.ptr) },
        }
    }
}

impl std::fmt::Debug for SCRecordingOutputConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCRecordingOutputConfiguration")
            .field("video_codec", &format_args!("{}", self.video_codec()))
            .field("file_type", &format_args!("{}", self.output_file_type()))
            .finish()
    }
}

/// Delegate for recording output events
///
/// Implement this trait to receive notifications about recording lifecycle events.
/// Callbacks may arrive on different system threads, so implementations must
/// synchronize shared mutable state internally.
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
pub trait SCRecordingOutputDelegate: Send + Sync + 'static {
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
    on_start: Option<Box<dyn Fn() + Send + Sync + 'static>>,
    on_fail: Option<Box<dyn Fn(String) + Send + Sync + 'static>>,
    on_finish: Option<Box<dyn Fn() + Send + Sync + 'static>>,
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
        F: Fn() + Send + Sync + 'static,
    {
        self.on_start = Some(Box::new(f));
        self
    }

    /// Set the callback for when recording fails
    #[must_use]
    pub fn on_fail<F>(mut self, f: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.on_fail = Some(Box::new(f));
        self
    }

    /// Set the callback for when recording finishes
    #[must_use]
    pub fn on_finish<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
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

// C callback trampolines for delegate - ctx is the delegate registry id as usize.
//
// Each body is fully enclosed in a panic barrier: a panic escaping an
// `extern "C"` function is undefined behaviour, and the registry lookup itself
// can panic (allocation, poisoned-lock recovery) before user code even runs.
// The registry lock is always released before the user delegate is invoked —
// see `RecordingDelegateEntry`.
extern "C" fn recording_started_callback(ctx: *mut c_void) {
    crate::utils::panic_safe::catch_user_panic(
        "SCRecordingOutputDelegate::recording_did_start",
        || {
            if let Some(delegate) = lookup_delegate(ctx as usize) {
                delegate.recording_did_start();
            }
        },
    );
}

extern "C" fn recording_failed_callback(ctx: *mut c_void, error_code: i32, error: *const i8) {
    crate::utils::panic_safe::catch_user_panic(
        "SCRecordingOutputDelegate::recording_did_fail",
        || {
            let error_str = if error.is_null() {
                String::from("Unknown error")
            } else {
                unsafe { std::ffi::CStr::from_ptr(error) }
                    .to_string_lossy()
                    .into_owned()
            };

            // Include error code in the message if it's a known SCStreamError
            let full_error = if error_code == 0 {
                error_str
            } else {
                crate::error::SCStreamErrorCode::from_raw(error_code).map_or_else(
                    || format!("{error_str} (code: {error_code})"),
                    |code| format!("{error_str} ({code})"),
                )
            };
            if let Some(delegate) = lookup_delegate(ctx as usize) {
                delegate.recording_did_fail(full_error);
            }
        },
    );
}

extern "C" fn recording_finished_callback(ctx: *mut c_void) {
    crate::utils::panic_safe::catch_user_panic(
        "SCRecordingOutputDelegate::recording_did_finish",
        || {
            if let Some(delegate) = lookup_delegate(ctx as usize) {
                delegate.recording_did_finish();
            }
        },
    );
}

extern "C" fn recording_context_release_callback(ctx: *mut c_void) {
    crate::utils::panic_safe::catch_user_panic("SCRecordingOutputDelegate::release", || {
        drop(remove_delegate(ctx as usize));
    });
}

impl SCRecordingOutput {
    /// Whether recording-output APIs are available on this system.
    #[must_use]
    pub fn is_available() -> bool {
        unsafe { crate::ffi::sc_recording_output_is_available() }
    }

    /// Create a new recording output with configuration
    ///
    /// # Errors
    /// Returns None if the system is not macOS 15.0+ or creation fails
    pub fn new(config: &SCRecordingOutputConfiguration) -> Option<Self> {
        if !Self::is_available() {
            return None;
        }
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
    pub fn new_with_delegate<D: SCRecordingOutputDelegate>(
        config: &SCRecordingOutputConfiguration,
        delegate: D,
    ) -> Option<Self> {
        if !Self::is_available() {
            return None;
        }
        let entry = RecordingDelegateEntry {
            delegate: Arc::new(delegate),
        };
        let delegate_id = {
            let mut registry = RECORDING_DELEGATE_REGISTRY
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let delegates = registry.get_or_insert_with(HashMap::new);
            loop {
                let id = NEXT_DELEGATE_ID.fetch_add(1, Ordering::Relaxed);
                if id != 0 && !delegates.contains_key(&id) {
                    delegates.insert(id, entry);
                    drop(registry);
                    break id;
                }
            }
        };

        // Use delegate_id as context
        let ctx = delegate_id as *mut c_void;

        let ptr = unsafe {
            crate::ffi::sc_recording_output_create_with_delegate(
                config.as_ptr(),
                Some(recording_started_callback),
                Some(recording_failed_callback),
                Some(recording_finished_callback),
                Some(recording_context_release_callback),
                ctx,
            )
        };

        if ptr.is_null() {
            drop(remove_delegate(delegate_id));
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
        CMTime::new(value, timescale)
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
