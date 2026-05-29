use std::ffi::c_void;
use std::fmt;

/// Opaque wrapper around `SCStreamConfiguration`
///
/// Configuration for a screen capture stream, including dimensions,
/// pixel format, audio settings, and other capture parameters.
///
/// # Examples
///
/// ```
/// use screencapturekit::stream::configuration::SCStreamConfiguration;
///
/// let config = SCStreamConfiguration::new()
///     .with_width(1920)
///     .with_height(1080);
/// ```
#[repr(transparent)]
pub struct SCStreamConfiguration(pub(crate) *const c_void);

impl PartialEq for SCStreamConfiguration {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SCStreamConfiguration {}

impl std::hash::Hash for SCStreamConfiguration {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl SCStreamConfiguration {
    pub(crate) fn internal_init() -> Self {
        unsafe {
            let ptr = crate::ffi::sc_stream_configuration_create();
            Self(ptr)
        }
    }

    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.0
    }
}

// `Clone::clone` is not a `memcpy`: it crosses the Swift FFI boundary and calls
// `sc_stream_configuration_retain` (an Objective-C `retain`). For most callers
// the cost is irrelevant, but if you're cloning an `SCStreamConfiguration` per
// frame on the hot path, prefer sharing an `Arc<SCStreamConfiguration>` (or
// `&SCStreamConfiguration`) and cloning *that* instead.
crate::utils::retained::sc_retained!(
    SCStreamConfiguration,
    retain = crate::ffi::sc_stream_configuration_retain,
    release = crate::ffi::sc_stream_configuration_release,
);

unsafe impl Send for SCStreamConfiguration {}
unsafe impl Sync for SCStreamConfiguration {}

impl fmt::Debug for SCStreamConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCStreamConfiguration")
            .field("ptr", &self.0)
            .finish()
    }
}

impl fmt::Display for SCStreamConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SCStreamConfiguration")
    }
}
