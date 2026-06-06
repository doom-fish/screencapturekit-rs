use core::fmt;
use std::ffi::c_void;

use crate::utils::ffi_string::ffi_string_owned_or_empty;

/// Wrapper around `SCRunningApplication` from `ScreenCaptureKit`
///
/// Represents a running application that can be captured.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::shareable_content::SCShareableContent;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let content = SCShareableContent::get()?;
/// for app in content.applications() {
///     println!("App: {} (PID: {})",
///         app.application_name(),
///         app.process_id()
///     );
/// }
/// # Ok(())
/// # }
/// ```
#[repr(transparent)]
pub struct SCRunningApplication(*const c_void);

impl PartialEq for SCRunningApplication {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SCRunningApplication {}

impl std::hash::Hash for SCRunningApplication {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl SCRunningApplication {
    /// Create from raw pointer (used internally by shareable content)
    pub(crate) unsafe fn from_ptr(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Create from an FFI-owned (retained) pointer, returning `None` if null.
    ///
    /// # Safety
    /// `ptr` must be null or a valid retained `SCRunningApplication` pointer
    /// transferred from the Swift FFI bridge (ownership moves into the wrapper).
    pub(crate) unsafe fn from_retained_ptr(ptr: *const c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Self::from_ptr(ptr) })
        }
    }

    /// Get the raw pointer (used internally)
    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.0
    }

    /// Get process ID
    pub fn process_id(&self) -> i32 {
        unsafe { crate::ffi::sc_running_application_get_process_id(self.0) }
    }

    /// Get application name
    pub fn application_name(&self) -> String {
        unsafe {
            ffi_string_owned_or_empty(|| {
                crate::ffi::sc_running_application_get_application_name_owned(self.0)
            })
        }
    }

    /// Get bundle identifier
    pub fn bundle_identifier(&self) -> String {
        unsafe {
            ffi_string_owned_or_empty(|| {
                crate::ffi::sc_running_application_get_bundle_identifier_owned(self.0)
            })
        }
    }
}

crate::utils::retained::sc_retained!(
    SCRunningApplication,
    retain = crate::ffi::sc_running_application_retain,
    release = crate::ffi::sc_running_application_release,
);

impl fmt::Debug for SCRunningApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCRunningApplication")
            .field("bundle_identifier", &self.bundle_identifier())
            .field("application_name", &self.application_name())
            .field("process_id", &self.process_id())
            .finish()
    }
}

impl fmt::Display for SCRunningApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) [PID: {}]",
            self.application_name(),
            self.bundle_identifier(),
            self.process_id()
        )
    }
}

// SAFETY: `SCRunningApplication` wraps an immutable Objective-C ScreenCaptureKit
// object. ObjC reference counting is atomic and these accessor-only objects are
// safe to send between and share across threads.
unsafe impl Send for SCRunningApplication {}
unsafe impl Sync for SCRunningApplication {}
