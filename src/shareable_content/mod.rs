//! Shareable content types - displays, windows, and applications
//!
//! This module provides access to the system's displays, windows, and running
//! applications that can be captured by ScreenCaptureKit.
//!
//! # Examples
//!
//! ```no_run
//! use screencapturekit::shareable_content::SCShareableContent;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Get all shareable content
//! let content = SCShareableContent::get()?;
//!
//! // List displays
//! for display in content.displays() {
//!     println!("Display {}: {}x{}", 
//!         display.display_id(), 
//!         display.width(), 
//!         display.height()
//!     );
//! }
//!
//! // List windows
//! for window in content.windows() {
//!     if let Some(title) = window.title() {
//!         println!("Window: {}", title);
//!     }
//! }
//!
//! // List applications
//! for app in content.applications() {
//!     println!("App: {}", app.application_name());
//! }
//! # Ok(())
//! # }
//! ```

pub mod display;
pub mod running_application;
pub mod window;
pub use display::SCDisplay;
pub use running_application::SCRunningApplication;
pub use window::SCWindow;

use core::fmt;
use crate::error::SCError;
use std::ffi::{c_void, CStr};
use std::sync::{Mutex, Arc, Condvar};
use std::time::Duration;

static CALLBACK_DATA: Mutex<Option<usize>> = Mutex::new(None);

#[repr(transparent)]
pub struct SCShareableContent(*const c_void);

unsafe impl Send for SCShareableContent {}
unsafe impl Sync for SCShareableContent {}

impl PartialEq for SCShareableContent {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SCShareableContent {}

impl std::hash::Hash for SCShareableContent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Clone for SCShareableContent {
    fn clone(&self) -> Self {
        unsafe {
            Self(crate::ffi::sc_shareable_content_retain(self.0))
        }
    }
}

impl SCShareableContent {
    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get shareable content (displays, windows, and applications)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// println!("Found {} displays", content.displays().len());
    /// println!("Found {} windows", content.windows().len());
    /// println!("Found {} apps", content.applications().len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if screen recording permission is not granted.
    pub fn get() -> Result<Self, SCError> {
        Self::with_options().get()
    }

    /// Create options builder for customizing shareable content retrieval
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::with_options()
    ///     .on_screen_windows_only(true)
    ///     .exclude_desktop_windows(true)
    ///     .get()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_options() -> SCShareableContentOptions {
        SCShareableContentOptions::default()
    }

    /// Get all available displays
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// for display in content.displays() {
    ///     println!("Display: {}x{}", display.width(), display.height());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn displays(&self) -> Vec<SCDisplay> {
        unsafe {
            let count = crate::ffi::sc_shareable_content_get_displays_count(self.0);
            // FFI returns isize but count is always positive
            #[allow(clippy::cast_sign_loss)]
            let mut displays = Vec::with_capacity(count as usize);
            
            for i in 0..count {
                let display_ptr = crate::ffi::sc_shareable_content_get_display_at(self.0, i);
                if !display_ptr.is_null() {
                    displays.push(SCDisplay::from_ptr(display_ptr));
                }
            }
            
            displays
        }
    }

    /// Get all available windows
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// for window in content.windows() {
    ///     if let Some(title) = window.title() {
    ///         println!("Window: {}", title);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn windows(&self) -> Vec<SCWindow> {
        unsafe {
            let count = crate::ffi::sc_shareable_content_get_windows_count(self.0);
            // FFI returns isize but count is always positive
            #[allow(clippy::cast_sign_loss)]
            let mut windows = Vec::with_capacity(count as usize);
            
            for i in 0..count {
                let window_ptr = crate::ffi::sc_shareable_content_get_window_at(self.0, i);
                if !window_ptr.is_null() {
                    windows.push(SCWindow::from_ptr(window_ptr));
                }
            }
            
            windows
        }
    }

    pub fn applications(&self) -> Vec<SCRunningApplication> {
        unsafe {
            let count = crate::ffi::sc_shareable_content_get_applications_count(self.0);
            // FFI returns isize but count is always positive
            #[allow(clippy::cast_sign_loss)]
            let mut apps = Vec::with_capacity(count as usize);
            
            for i in 0..count {
                let app_ptr = crate::ffi::sc_shareable_content_get_application_at(self.0, i);
                if !app_ptr.is_null() {
                    apps.push(SCRunningApplication::from_ptr(app_ptr));
                }
            }
            
            apps
        }
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.0
    }
}

impl Drop for SCShareableContent {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                crate::ffi::sc_shareable_content_release(self.0);
            }
        }
    }
}

impl fmt::Debug for SCShareableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCShareableContent")
            .field("displays", &self.displays().len())
            .field("windows", &self.windows().len())
            .field("applications", &self.applications().len())
            .finish()
    }
}

impl fmt::Display for SCShareableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SCShareableContent ({} displays, {} windows, {} applications)",
            self.displays().len(),
            self.windows().len(),
            self.applications().len()
        )
    }
}

extern "C" fn shareable_content_callback(content: *const c_void, error: *const i8) {
    let arc_ptr = {
        let Ok(mut guard) = CALLBACK_DATA.lock() else {
            eprintln!("Failed to lock CALLBACK_DATA");
            return;
        };
        guard.take()
    };

    if let Some(ptr) = arc_ptr {
        let arc = unsafe { Arc::from_raw(ptr as *const (Mutex<Option<Result<SCShareableContent, SCError>>>, Condvar)) };
        let (lock, cvar) = &*arc;
        let Ok(mut result) = lock.lock() else {
            eprintln!("Failed to lock result");
            return;
        };
        
        *result = Some(if content.is_null() {
            let error_msg = if error.is_null() {
                "Unknown error".to_string()
            } else {
                unsafe { CStr::from_ptr(error).to_string_lossy().to_string() }
            };
            Err(io_error_to_cferror(std::io::Error::new(std::io::ErrorKind::Other, error_msg)))
        } else {
            Ok(unsafe { SCShareableContent::from_ptr(content) })
        });
        
        cvar.notify_one();
    }
}

#[derive(Default)]
pub struct SCShareableContentOptions {
    timeout: Option<Duration>,
    exclude_desktop_windows: bool,
    on_screen_windows_only: bool,
    current_process_only: bool,
}

impl SCShareableContentOptions {
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Exclude desktop windows from the shareable content.
    /// 
    /// When set to `true`, desktop-level windows (like the desktop background)
    /// are excluded from the returned window list.
    #[must_use]
    pub fn exclude_desktop_windows(mut self, exclude: bool) -> Self {
        self.exclude_desktop_windows = exclude;
        self
    }

    /// Include only on-screen windows in the shareable content.
    /// 
    /// When set to `true`, only windows that are currently visible on screen
    /// are included. Minimized or off-screen windows are excluded.
    #[must_use]
    pub fn on_screen_windows_only(mut self, on_screen_only: bool) -> Self {
        self.on_screen_windows_only = on_screen_only;
        self
    }

    /// Get shareable content for the current process only (macOS 14.0+).
    /// 
    /// When set to `true`, only displays and windows accessible to the current
    /// process are returned. On macOS versions prior to 14.0, this falls back
    /// to the standard shareable content retrieval.
    #[must_use]
    pub fn current_process_only(mut self, current_process: bool) -> Self {
        self.current_process_only = current_process;
        self
    }

    pub fn get(self) -> Result<SCShareableContent, SCError> {
        let pair = Arc::new((
            Mutex::new(None::<Result<SCShareableContent, SCError>>),
            Condvar::new(),
        ));
        
        let arc_ptr = Arc::into_raw(pair.clone()) as usize;
        {
            let mut guard = CALLBACK_DATA.lock().map_err(|_| {
                io_error_to_cferror(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to lock callback data",
                ))
            })?;
            *guard = Some(arc_ptr);
        }

        unsafe {
            if self.current_process_only {
                crate::ffi::sc_shareable_content_get_current_process_displays(shareable_content_callback);
            } else {
                crate::ffi::sc_shareable_content_get_with_options(
                    self.exclude_desktop_windows,
                    self.on_screen_windows_only,
                    shareable_content_callback,
                );
            }
        }

        let (lock, cvar) = &*pair;
        
        let timeout = self.timeout.unwrap_or(Duration::from_secs(5));
        let mut wait_result = {
            let result = lock.lock().map_err(|_| {
                io_error_to_cferror(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to lock result",
                ))
            })?;
            
            cvar.wait_timeout_while(
                result,
                timeout,
                |r| r.is_none(),
            ).map_err(|_| {
                io_error_to_cferror(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Wait condition error",
                ))
            })?
        };
        
        if wait_result.1.timed_out() {
            return Err(io_error_to_cferror(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Timeout waiting for shareable content",
            )));
        }

        wait_result.0.take().ok_or_else(|| {
            io_error_to_cferror(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No result received",
            ))
        })?
    }
}

// Helper to convert errors  
fn io_error_to_cferror(_err: std::io::Error) -> SCError {
    crate::utils::error::create_sc_error("IO Error")
}
