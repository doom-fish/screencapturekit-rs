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
    /// Identity comparison: two `SCStreamConfiguration`s are equal only when
    /// they wrap the same native object.
    ///
    /// Because [`Clone`] deep-copies (see below), `config != config.clone()`
    /// even though the two carry identical settings.
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

// `Clone::clone` is not a `memcpy`: it crosses the Swift FFI boundary and
// deep-copies the underlying `SCStreamConfiguration`. That is deliberate.
// `SCStreamConfiguration` is a *mutable* Objective-C class, so a retain-based
// clone would hand out aliases: `config.clone().set_width(..)` would silently
// resize the original, and two threads configuring "their own" clone would
// race on the same non-atomic properties — which the `Send`/`Sync` impls below
// promise cannot happen. If you're cloning per frame on a hot path, share an
// `Arc<SCStreamConfiguration>` (or a `&SCStreamConfiguration`) instead.
impl Clone for SCStreamConfiguration {
    fn clone(&self) -> Self {
        Self(unsafe { crate::ffi::sc_stream_configuration_copy(self.0) })
    }
}

crate::utils::retained::sc_retained!(
    SCStreamConfiguration,
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
