//! Content filter for ScreenCaptureKit streams
//!
//! This module provides a wrapper around SCContentFilter that uses the Swift bridge.
//!
//! # Examples
//!
//! ```no_run
//! use screencapturekit::shareable_content::SCShareableContent;
//! use screencapturekit::stream::content_filter::SCContentFilter;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//! let display = &content.displays()[0];
//!
//! // Capture entire display
//! let filter = SCContentFilter::build()
//!     .display(display)
//!     .exclude_windows(&[])
//!     .build();
//! # Ok(())
//! # }
//! ```

use std::ffi::c_void;
use std::fmt;

use crate::{
    ffi,
    shareable_content::{SCDisplay, SCRunningApplication, SCWindow},
};

/// Content filter for ScreenCaptureKit streams
///
/// Defines what content to capture (displays, windows, or applications).
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::shareable_content::SCShareableContent;
/// use screencapturekit::stream::content_filter::SCContentFilter;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let content = SCShareableContent::get()?;
/// let display = &content.displays()[0];
///
/// // Capture entire display
/// let filter = SCContentFilter::build()
///     .display(display)
///     .exclude_windows(&[])
///     .build();
///
/// // Or capture a specific window
/// let window = &content.windows()[0];
/// let filter = SCContentFilter::build()
///     .window(window)
///     .build();
/// # Ok(())
/// # }
/// ```
pub struct SCContentFilter(*const c_void);

impl PartialEq for SCContentFilter {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SCContentFilter {}

impl std::hash::Hash for SCContentFilter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Default for SCContentFilter {
    fn default() -> Self {
        Self(std::ptr::null())
    }
}

impl SCContentFilter {
    /// Creates a content filter builder
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use screencapturekit::prelude::*;
    /// 
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// let display = &content.displays()[0];
    /// 
    /// let filter = SCContentFilter::build()
    ///     .display(display)
    ///     .exclude_windows(&[])
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn build() -> SCContentFilterBuilder {
        SCContentFilterBuilder::new()
    }

    /// Creates a new content filter (deprecated - use builder pattern)
    /// 
    /// # Deprecated
    /// Use the builder pattern instead:
    /// ```no_run
    /// # use screencapturekit::prelude::*;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let display = &content.displays()[0];
    /// let filter = SCContentFilter::build()
    ///     .display(display)
    ///     .exclude_windows(&[])
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(since = "1.0.0", note = "Use SCContentFilter::build() instead")]
    #[must_use]
    pub fn new() -> Self {
        Self(std::ptr::null())
    }

    /// Creates a content filter with a desktop independent window (deprecated)
    ///
    /// # Deprecated
    /// Use the builder pattern instead:
    /// ```no_run
    /// # use screencapturekit::prelude::*;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let window = &content.windows()[0];
    /// let filter = SCContentFilter::build()
    ///     .window(window)
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(since = "1.0.0", note = "Use SCContentFilter::build().window().build() instead")]
    #[must_use]
    pub fn with_desktop_independent_window(self, window: &SCWindow) -> Self {
        // Drop the old filter if any
        if !self.0.is_null() {
            unsafe { ffi::sc_content_filter_release(self.0); }
        }
        std::mem::forget(self); // Prevent double-free
        
        unsafe {
            let filter = ffi::sc_content_filter_create_with_desktop_independent_window(window.as_ptr());
            Self(filter)
        }
    }

    /// Creates a content filter with a display, excluding specific windows (deprecated)
    ///
    /// # Deprecated
    /// Use the builder pattern instead:
    /// ```no_run
    /// # use screencapturekit::prelude::*;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let display = &content.displays()[0];
    /// # let window = &content.windows()[0];
    /// // Capture entire display
    /// let filter = SCContentFilter::build()
    ///     .display(display)
    ///     .exclude_windows(&[])
    ///     .build();
    ///
    /// // Or exclude specific windows
    /// let filter = SCContentFilter::build()
    ///     .display(display)
    ///     .exclude_windows(&[window])
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(since = "1.0.0", note = "Use SCContentFilter::build().display().exclude_windows().build() instead")]
    #[must_use]
    pub fn with_display_excluding_windows(
        self,
        display: &SCDisplay,
        excluding_windows: &[&SCWindow],
    ) -> Self {
        // Drop the old filter if any
        if !self.0.is_null() {
            unsafe { ffi::sc_content_filter_release(self.0); }
        }
        std::mem::forget(self); // Prevent double-free
        unsafe {
            let window_ptrs: Vec<*const c_void> = excluding_windows
                .iter()
                .map(|w| w.as_ptr())
                .collect();
            
            let filter = if window_ptrs.is_empty() {
                ffi::sc_content_filter_create_with_display_excluding_windows(
                    display.as_ptr(),
                    std::ptr::null(),
                    0,
                )
            } else {
                // FFI expects isize for array length (Objective-C NSInteger)
                #[allow(clippy::cast_possible_wrap)]
                ffi::sc_content_filter_create_with_display_excluding_windows(
                    display.as_ptr(),
                    window_ptrs.as_ptr(),
                    window_ptrs.len() as isize,
                )
            };
            
            Self(filter)
        }
    }

    /// Creates a content filter with a display, including specific windows (deprecated)
    #[deprecated(since = "1.0.0", note = "Use SCContentFilter::build().display().include_windows().build() instead")]
    #[must_use]
    pub fn with_display_including_windows(
        self,
        display: &SCDisplay,
        including_windows: &[&SCWindow],
    ) -> Self {
        // Drop the old filter if any
        if !self.0.is_null() {
            unsafe { ffi::sc_content_filter_release(self.0); }
        }
        std::mem::forget(self); // Prevent double-free
        unsafe {
            let window_ptrs: Vec<*const c_void> = including_windows
                .iter()
                .map(|w| w.as_ptr())
                .collect();
            
            let filter = if window_ptrs.is_empty() {
                ffi::sc_content_filter_create_with_display_including_windows(
                    display.as_ptr(),
                    std::ptr::null(),
                    0,
                )
            } else {
                // FFI expects isize for array length (Objective-C NSInteger)
                #[allow(clippy::cast_possible_wrap)]
                ffi::sc_content_filter_create_with_display_including_windows(
                    display.as_ptr(),
                    window_ptrs.as_ptr(),
                    window_ptrs.len() as isize,
                )
            };
            
            Self(filter)
        }
    }

    /// Creates a content filter with a display, including applications and excepting specific windows (deprecated)
    #[deprecated(since = "1.0.0", note = "Use SCContentFilter::build().display().include_applications().build() instead")]
    #[must_use]
    pub fn with_display_including_applications_excepting_windows(
        self,
        display: &SCDisplay,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) -> Self {
        // Drop the old filter if any
        if !self.0.is_null() {
            unsafe { ffi::sc_content_filter_release(self.0); }
        }
        std::mem::forget(self); // Prevent double-free
        unsafe {
            let app_ptrs: Vec<*const c_void> = applications
                .iter()
                .map(|a| a.as_ptr())
                .collect();
            
            let window_ptrs: Vec<*const c_void> = excepting_windows
                .iter()
                .map(|w| w.as_ptr())
                .collect();
            
            // FFI expects isize for array lengths (Objective-C NSInteger)
            #[allow(clippy::cast_possible_wrap)]
            let filter = ffi::sc_content_filter_create_with_display_including_applications_excepting_windows(
                display.as_ptr(),
                if app_ptrs.is_empty() { std::ptr::null() } else { app_ptrs.as_ptr() },
                app_ptrs.len() as isize,
                if window_ptrs.is_empty() { std::ptr::null() } else { window_ptrs.as_ptr() },
                window_ptrs.len() as isize,
            );
            
            Self(filter)
        }
    }

    /// Returns the raw pointer to the content filter
    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.0
    }

    /// Sets the content rectangle for this filter (macOS 14.2+)
    /// 
    /// Specifies the rectangle within the content filter to capture.
    #[must_use]
    pub fn set_content_rect(self, rect: crate::stream::configuration::Rect) -> Self {
        unsafe {
            ffi::sc_content_filter_set_content_rect(
                self.0,
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            );
        }
        self
    }

    /// Gets the content rectangle for this filter (macOS 14.2+)
    pub fn get_content_rect(&self) -> crate::stream::configuration::Rect {
        unsafe {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut width = 0.0;
            let mut height = 0.0;
            ffi::sc_content_filter_get_content_rect(
                self.0,
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            );
            crate::stream::configuration::Rect::new(
                crate::stream::configuration::Point::new(x, y),
                crate::stream::configuration::Size::new(width, height),
            )
        }
    }
}

impl Drop for SCContentFilter {
    fn drop(&mut self) {
        unsafe {
            ffi::sc_content_filter_release(self.0);
        }
    }
}

// TCFType compatibility for legacy objc-based code

pub type SCContentFilterRef = *const c_void;

extern "C" {
}


impl Clone for SCContentFilter {
    fn clone(&self) -> Self {
        unsafe {
            Self(crate::ffi::sc_content_filter_retain(self.0))
        }
    }
}

impl fmt::Debug for SCContentFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCContentFilter")
            .field("ptr", &self.0)
            .finish()
    }
}

impl fmt::Display for SCContentFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SCContentFilter")
    }
}

// Safety: SCContentFilter wraps an Objective-C object that is thread-safe
// The underlying SCContentFilter object can be safely sent between threads
unsafe impl Send for SCContentFilter {}
unsafe impl Sync for SCContentFilter {}

/// Builder for creating SCContentFilter instances
/// 
/// # Examples
/// 
/// ```no_run
/// use screencapturekit::prelude::*;
/// 
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let content = SCShareableContent::get()?;
/// let display = &content.displays()[0];
/// 
/// // Capture entire display
/// let filter = SCContentFilter::build()
///     .display(display)
///     .exclude_windows(&[])
///     .build();
/// 
/// // Capture with specific windows excluded
/// let window = &content.windows()[0];
/// let filter = SCContentFilter::build()
///     .display(display)
///     .exclude_windows(&[window])
///     .build();
/// 
/// // Capture specific window
/// let filter = SCContentFilter::build()
///     .window(window)
///     .build();
/// # Ok(())
/// # }
/// ```
pub struct SCContentFilterBuilder {
    filter_type: FilterType,
    content_rect: Option<crate::stream::configuration::Rect>,
}

enum FilterType {
    None,
    Window(SCWindow),
    DisplayExcluding { display: SCDisplay, windows: Vec<SCWindow> },
    DisplayIncluding { display: SCDisplay, windows: Vec<SCWindow> },
    DisplayApplications { display: SCDisplay, applications: Vec<SCRunningApplication>, excepting_windows: Vec<SCWindow> },
}

impl SCContentFilterBuilder {
    fn new() -> Self {
        Self {
            filter_type: FilterType::None,
            content_rect: None,
        }
    }

    /// Set the display to capture
    #[must_use]
    pub fn display(mut self, display: &SCDisplay) -> Self {
        self.filter_type = FilterType::DisplayExcluding {
            display: display.clone(),
            windows: Vec::new(),
        };
        self
    }

    /// Set the window to capture
    #[must_use]
    pub fn window(mut self, window: &SCWindow) -> Self {
        self.filter_type = FilterType::Window(window.clone());
        self
    }

    /// Exclude specific windows from the display capture
    #[must_use]
    pub fn exclude_windows(mut self, windows: &[&SCWindow]) -> Self {
        if let FilterType::DisplayExcluding { windows: ref mut excluded, .. } = self.filter_type {
            *excluded = windows.iter().map(|w| (*w).clone()).collect();
        }
        self
    }

    /// Include only specific windows in the display capture
    #[must_use]
    pub fn include_windows(mut self, windows: &[&SCWindow]) -> Self {
        if let FilterType::DisplayExcluding { display, .. } = self.filter_type {
            self.filter_type = FilterType::DisplayIncluding {
                display,
                windows: windows.iter().map(|w| (*w).clone()).collect(),
            };
        }
        self
    }

    /// Include specific applications and optionally except certain windows
    #[must_use]
    pub fn include_applications(mut self, applications: &[&SCRunningApplication], excepting_windows: &[&SCWindow]) -> Self {
        if let FilterType::DisplayExcluding { display, .. } | FilterType::DisplayIncluding { display, .. } = self.filter_type {
            self.filter_type = FilterType::DisplayApplications {
                display,
                applications: applications.iter().map(|a| (*a).clone()).collect(),
                excepting_windows: excepting_windows.iter().map(|w| (*w).clone()).collect(),
            };
        }
        self
    }

    /// Set the content rectangle (macOS 14.2+)
    #[must_use]
    pub fn content_rect(mut self, rect: crate::stream::configuration::Rect) -> Self {
        self.content_rect = Some(rect);
        self
    }

    /// Build the content filter
    #[must_use]
    pub fn build(self) -> SCContentFilter {
        let filter = match self.filter_type {
            FilterType::Window(window) => {
                unsafe {
                    let ptr = ffi::sc_content_filter_create_with_desktop_independent_window(window.as_ptr());
                    SCContentFilter(ptr)
                }
            }
            FilterType::DisplayExcluding { display, windows } => {
                let window_refs: Vec<&SCWindow> = windows.iter().collect();
                unsafe {
                    let window_ptrs: Vec<*const c_void> = window_refs
                        .iter()
                        .map(|w| w.as_ptr())
                        .collect();
                    
                    let ptr = if window_ptrs.is_empty() {
                        ffi::sc_content_filter_create_with_display_excluding_windows(
                            display.as_ptr(),
                            std::ptr::null(),
                            0,
                        )
                    } else {
                        #[allow(clippy::cast_possible_wrap)]
                        ffi::sc_content_filter_create_with_display_excluding_windows(
                            display.as_ptr(),
                            window_ptrs.as_ptr(),
                            window_ptrs.len() as isize,
                        )
                    };
                    SCContentFilter(ptr)
                }
            }
            FilterType::DisplayIncluding { display, windows } => {
                let window_refs: Vec<&SCWindow> = windows.iter().collect();
                unsafe {
                    let window_ptrs: Vec<*const c_void> = window_refs
                        .iter()
                        .map(|w| w.as_ptr())
                        .collect();
                    
                    let ptr = if window_ptrs.is_empty() {
                        ffi::sc_content_filter_create_with_display_including_windows(
                            display.as_ptr(),
                            std::ptr::null(),
                            0,
                        )
                    } else {
                        #[allow(clippy::cast_possible_wrap)]
                        ffi::sc_content_filter_create_with_display_including_windows(
                            display.as_ptr(),
                            window_ptrs.as_ptr(),
                            window_ptrs.len() as isize,
                        )
                    };
                    SCContentFilter(ptr)
                }
            }
            FilterType::DisplayApplications { display, applications, excepting_windows } => {
                let app_refs: Vec<&SCRunningApplication> = applications.iter().collect();
                let window_refs: Vec<&SCWindow> = excepting_windows.iter().collect();
                unsafe {
                    let app_ptrs: Vec<*const c_void> = app_refs
                        .iter()
                        .map(|a| a.as_ptr())
                        .collect();
                    
                    let window_ptrs: Vec<*const c_void> = window_refs
                        .iter()
                        .map(|w| w.as_ptr())
                        .collect();
                    
                    #[allow(clippy::cast_possible_wrap)]
                    let ptr = ffi::sc_content_filter_create_with_display_including_applications_excepting_windows(
                        display.as_ptr(),
                        if app_ptrs.is_empty() { std::ptr::null() } else { app_ptrs.as_ptr() },
                        app_ptrs.len() as isize,
                        if window_ptrs.is_empty() { std::ptr::null() } else { window_ptrs.as_ptr() },
                        window_ptrs.len() as isize,
                    );
                    SCContentFilter(ptr)
                }
            }
            FilterType::None => {
                // Return a null filter
                SCContentFilter(std::ptr::null())
            }
        };

        // Apply content rect if set
        if let Some(rect) = self.content_rect {
            filter.set_content_rect(rect)
        } else {
            filter
        }
    }
}

