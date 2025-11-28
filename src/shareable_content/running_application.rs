use core::fmt;
use std::ffi::c_void;

use crate::utils::ffi_string::{ffi_string_from_buffer_or_empty, DEFAULT_BUFFER_SIZE};

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

/// Raw pointer type for `SCRunningApplication` (for FFI compatibility)
pub type SCRunningApplicationRef = *const c_void;

impl SCRunningApplication {
    /// Create from raw pointer (used internally by shareable content)
    pub(crate) unsafe fn from_ptr(ptr: *const c_void) -> Self {
        Self(ptr)
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
            ffi_string_from_buffer_or_empty(DEFAULT_BUFFER_SIZE, |buf, len| {
                crate::ffi::sc_running_application_get_application_name(self.0, buf, len)
            })
        }
    }

    /// Get bundle identifier
    pub fn bundle_identifier(&self) -> String {
        unsafe {
            ffi_string_from_buffer_or_empty(DEFAULT_BUFFER_SIZE, |buf, len| {
                crate::ffi::sc_running_application_get_bundle_identifier(self.0, buf, len)
            })
        }
    }
}

impl Drop for SCRunningApplication {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                crate::ffi::sc_running_application_release(self.0);
            }
        }
    }
}

impl Clone for SCRunningApplication {
    fn clone(&self) -> Self {
        unsafe { Self(crate::ffi::sc_running_application_retain(self.0)) }
    }
}

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

unsafe impl Send for SCRunningApplication {}
unsafe impl Sync for SCRunningApplication {}
