//! Content filter for `ScreenCaptureKit` streams
//!
//! This module provides a wrapper around `SCContentFilter` that uses the Swift bridge.
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
//! let filter = SCContentFilter::create()
//!     .with_display(display)
//!     .with_excluding_windows(&[])
//!     .build();
//! # Ok(())
//! # }
//! ```

use std::ffi::c_void;
use std::fmt;

#[cfg(feature = "macos_14_2")]
use crate::cg::CGRect;
use crate::{
    error::{SCError, SCResult},
    ffi,
    shareable_content::{SCDisplay, SCRunningApplication, SCWindow},
};

/// Content filter for `ScreenCaptureKit` streams
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
/// let filter = SCContentFilter::create()
///     .with_display(display)
///     .with_excluding_windows(&[])
///     .build();
///
/// // Or capture a specific window
/// let window = &content.windows()[0];
/// let filter = SCContentFilter::create()
///     .with_window(window)
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

// Note: We intentionally do NOT implement Default for SCContentFilter.
// A null filter would cause panics/crashes when used with SCStream.
// Users should always use SCContentFilter::create() to create valid filters.

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
    /// let filter = SCContentFilter::create()
    ///     .with_display(display)
    ///     .with_excluding_windows(&[])
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn create() -> SCContentFilterBuilder {
        SCContentFilterBuilder::new()
    }

    /// Creates a content filter from a picker-returned pointer
    ///
    /// This is used internally when the content sharing picker returns a filter.
    #[cfg(feature = "macos_14_0")]
    pub(crate) fn from_picker_ptr(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Returns the raw pointer to the content filter
    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.0
    }

    /// Sets the content rectangle for this filter (macOS 14.2+)
    ///
    /// Specifies the rectangle within the content filter to capture.
    #[cfg(feature = "macos_14_2")]
    #[must_use]
    pub fn set_content_rect(self, rect: CGRect) -> Self {
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
    #[cfg(feature = "macos_14_2")]
    pub fn content_rect(&self) -> CGRect {
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
            CGRect::new(x, y, width, height)
        }
    }

    /// Get the content style (macOS 14.0+)
    ///
    /// Returns the type of content being captured (window, display, application, or none).
    #[cfg(feature = "macos_14_0")]
    pub fn style(&self) -> SCShareableContentStyle {
        let value = unsafe { ffi::sc_content_filter_get_style(self.0) };
        SCShareableContentStyle::from(value)
    }

    /// Get the stream type (macOS 14.0+)
    ///
    /// Returns whether this filter captures a window or a display.
    #[cfg(feature = "macos_14_0")]
    pub fn stream_type(&self) -> SCStreamType {
        let value = unsafe { ffi::sc_content_filter_get_stream_type(self.0) };
        SCStreamType::from(value)
    }

    /// Get the point-to-pixel scale factor (macOS 14.0+)
    ///
    /// Returns the scaling factor used to convert points to pixels.
    /// Typically 2.0 for Retina displays.
    #[cfg(feature = "macos_14_0")]
    pub fn point_pixel_scale(&self) -> f32 {
        unsafe { ffi::sc_content_filter_get_point_pixel_scale(self.0) }
    }

    /// Include the menu bar in capture (macOS 14.2+)
    ///
    /// When set to `true`, the menu bar is included in display capture.
    /// This property has no effect for window filters.
    #[cfg(feature = "macos_14_2")]
    pub fn set_include_menu_bar(&mut self, include: bool) {
        unsafe {
            ffi::sc_content_filter_set_include_menu_bar(self.0, include);
        }
    }

    /// Check if menu bar is included in capture (macOS 14.2+)
    #[cfg(feature = "macos_14_2")]
    pub fn include_menu_bar(&self) -> bool {
        unsafe { ffi::sc_content_filter_get_include_menu_bar(self.0) }
    }

    /// Get included displays (macOS 15.2+)
    ///
    /// Returns the displays currently included in this filter.
    #[cfg(feature = "macos_15_2")]
    pub fn included_displays(&self) -> Vec<SCDisplay> {
        let count = unsafe { ffi::sc_content_filter_get_included_displays_count(self.0) };
        if count <= 0 {
            return Vec::new();
        }
        #[allow(clippy::cast_sign_loss)]
        (0..count as usize)
            .filter_map(|i| {
                #[allow(clippy::cast_possible_wrap)]
                let ptr =
                    unsafe { ffi::sc_content_filter_get_included_display_at(self.0, i as isize) };
                unsafe { SCDisplay::from_retained_ptr(ptr) }
            })
            .collect()
    }

    /// Get included windows (macOS 15.2+)
    ///
    /// Returns the windows currently included in this filter.
    #[cfg(feature = "macos_15_2")]
    pub fn included_windows(&self) -> Vec<SCWindow> {
        let count = unsafe { ffi::sc_content_filter_get_included_windows_count(self.0) };
        if count <= 0 {
            return Vec::new();
        }
        #[allow(clippy::cast_sign_loss)]
        (0..count as usize)
            .filter_map(|i| {
                #[allow(clippy::cast_possible_wrap)]
                let ptr =
                    unsafe { ffi::sc_content_filter_get_included_window_at(self.0, i as isize) };
                unsafe { SCWindow::from_retained_ptr(ptr) }
            })
            .collect()
    }

    /// Get included applications (macOS 15.2+)
    ///
    /// Returns the applications currently included in this filter.
    #[cfg(feature = "macos_15_2")]
    pub fn included_applications(&self) -> Vec<SCRunningApplication> {
        let count = unsafe { ffi::sc_content_filter_get_included_applications_count(self.0) };
        if count <= 0 {
            return Vec::new();
        }
        #[allow(clippy::cast_sign_loss)]
        (0..count as usize)
            .filter_map(|i| {
                #[allow(clippy::cast_possible_wrap)]
                let ptr = unsafe {
                    ffi::sc_content_filter_get_included_application_at(self.0, i as isize)
                };
                unsafe { SCRunningApplication::from_retained_ptr(ptr) }
            })
            .collect()
    }
}

/// Content style for filters (macOS 14.0+)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg(feature = "macos_14_0")]
pub enum SCShareableContentStyle {
    /// No specific content type
    #[default]
    None = 0,
    /// Window-based content
    Window = 1,
    /// Display-based content
    Display = 2,
    /// Application-based content
    Application = 3,
}

#[cfg(feature = "macos_14_0")]
impl From<i32> for SCShareableContentStyle {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Window,
            2 => Self::Display,
            3 => Self::Application,
            _ => Self::None,
        }
    }
}

#[cfg(feature = "macos_14_0")]
impl std::fmt::Display for SCShareableContentStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Window => write!(f, "Window"),
            Self::Display => write!(f, "Display"),
            Self::Application => write!(f, "Application"),
        }
    }
}

/// Stream type for filters (macOS 14.0+)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg(feature = "macos_14_0")]
pub enum SCStreamType {
    /// Window-based stream
    #[default]
    Window = 0,
    /// Display-based stream
    Display = 1,
}

#[cfg(feature = "macos_14_0")]
impl From<i32> for SCStreamType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Display,
            _ => Self::Window,
        }
    }
}

#[cfg(feature = "macos_14_0")]
impl std::fmt::Display for SCStreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Window => write!(f, "Window"),
            Self::Display => write!(f, "Display"),
        }
    }
}

// `Clone::clone` is not a `memcpy`: it crosses the Swift FFI boundary and calls
// `sc_content_filter_retain` (an Objective-C `retain`). For hot-path code that
// needs many references to the same filter, prefer `Arc<SCContentFilter>` over
// per-call `.clone()`.
crate::utils::retained::sc_retained!(
    SCContentFilter,
    retain = crate::ffi::sc_content_filter_retain,
    release = crate::ffi::sc_content_filter_release,
);

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

/// Builder for creating `SCContentFilter` instances
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
/// let filter = SCContentFilter::create()
///     .with_display(display)
///     .with_excluding_windows(&[])
///     .build();
///
/// // Capture with specific windows excluded
/// let window = &content.windows()[0];
/// let filter = SCContentFilter::create()
///     .with_display(display)
///     .with_excluding_windows(&[window])
///     .build();
///
/// // Capture specific window
/// let filter = SCContentFilter::create()
///     .with_window(window)
///     .build();
/// # Ok(())
/// # }
/// ```
pub struct SCContentFilterBuilder {
    filter_type: FilterType,
    #[cfg(feature = "macos_14_2")]
    content_rect: Option<CGRect>,
}

enum FilterType {
    None,
    Window(SCWindow),
    DisplayExcluding {
        display: SCDisplay,
        windows: Vec<SCWindow>,
    },
    DisplayIncluding {
        display: SCDisplay,
        windows: Vec<SCWindow>,
    },
    DisplayIncludingApplications {
        display: SCDisplay,
        applications: Vec<SCRunningApplication>,
        excepting_windows: Vec<SCWindow>,
    },
    DisplayExcludingApplications {
        display: SCDisplay,
        applications: Vec<SCRunningApplication>,
        excepting_windows: Vec<SCWindow>,
    },
}

impl SCContentFilterBuilder {
    fn new() -> Self {
        Self {
            filter_type: FilterType::None,
            #[cfg(feature = "macos_14_2")]
            content_rect: None,
        }
    }

    /// Set the display to capture
    #[must_use]
    pub fn with_display(mut self, display: &SCDisplay) -> Self {
        self.filter_type = FilterType::DisplayExcluding {
            display: display.clone(),
            windows: Vec::new(),
        };
        self
    }

    /// Set the window to capture
    #[must_use]
    pub fn with_window(mut self, window: &SCWindow) -> Self {
        self.filter_type = FilterType::Window(window.clone());
        self
    }

    /// Exclude specific windows from the display capture
    #[must_use]
    pub fn with_excluding_windows(mut self, windows: &[&SCWindow]) -> Self {
        if let FilterType::DisplayExcluding {
            windows: ref mut excluded,
            ..
        } = self.filter_type
        {
            // `clone()` on SCWindow is a Swift retain (FFI). Pre-size the Vec
            // so we don't reallocate while pushing — at 200 windows this is
            // ~half the per-element cost.
            let mut v = Vec::with_capacity(windows.len());
            v.extend(windows.iter().map(|w| (*w).clone()));
            *excluded = v;
        }
        self
    }

    /// Include only specific windows in the display capture
    #[must_use]
    pub fn with_including_windows(mut self, windows: &[&SCWindow]) -> Self {
        if let FilterType::DisplayExcluding { display, .. } = self.filter_type {
            let mut v = Vec::with_capacity(windows.len());
            v.extend(windows.iter().map(|w| (*w).clone()));
            self.filter_type = FilterType::DisplayIncluding {
                display,
                windows: v,
            };
        }
        self
    }

    /// Include specific applications and optionally except certain windows
    #[must_use]
    pub fn with_including_applications(
        mut self,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) -> Self {
        if let FilterType::DisplayExcluding { display, .. }
        | FilterType::DisplayIncluding { display, .. } = self.filter_type
        {
            let mut apps = Vec::with_capacity(applications.len());
            apps.extend(applications.iter().map(|a| (*a).clone()));
            let mut wins = Vec::with_capacity(excepting_windows.len());
            wins.extend(excepting_windows.iter().map(|w| (*w).clone()));
            self.filter_type = FilterType::DisplayIncludingApplications {
                display,
                applications: apps,
                excepting_windows: wins,
            };
        }
        self
    }

    /// Exclude specific applications and optionally except certain windows
    ///
    /// Captures everything on the display except the specified applications.
    /// Windows in `excepting_windows` will still be captured even if their
    /// owning application is excluded.
    #[must_use]
    pub fn with_excluding_applications(
        mut self,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) -> Self {
        if let FilterType::DisplayExcluding { display, .. }
        | FilterType::DisplayIncluding { display, .. } = self.filter_type
        {
            let mut apps = Vec::with_capacity(applications.len());
            apps.extend(applications.iter().map(|a| (*a).clone()));
            let mut wins = Vec::with_capacity(excepting_windows.len());
            wins.extend(excepting_windows.iter().map(|w| (*w).clone()));
            self.filter_type = FilterType::DisplayExcludingApplications {
                display,
                applications: apps,
                excepting_windows: wins,
            };
        }
        self
    }

    /// Set the content rectangle (macOS 14.2+)
    #[cfg(feature = "macos_14_2")]
    #[must_use]
    pub fn with_content_rect(mut self, rect: CGRect) -> Self {
        self.content_rect = Some(rect);
        self
    }

    // =========================================================================
    // Deprecated methods - use with_* versions instead
    // =========================================================================

    /// Set the display to capture
    #[must_use]
    #[deprecated(since = "1.5.0", note = "Use with_display() instead")]
    pub fn display(self, display: &SCDisplay) -> Self {
        self.with_display(display)
    }

    /// Set the window to capture
    #[must_use]
    #[deprecated(since = "1.5.0", note = "Use with_window() instead")]
    pub fn window(self, window: &SCWindow) -> Self {
        self.with_window(window)
    }

    /// Exclude specific windows from the display capture
    #[must_use]
    #[deprecated(since = "1.5.0", note = "Use with_excluding_windows() instead")]
    pub fn exclude_windows(self, windows: &[&SCWindow]) -> Self {
        self.with_excluding_windows(windows)
    }

    /// Include only specific windows in the display capture
    #[must_use]
    #[deprecated(since = "1.5.0", note = "Use with_including_windows() instead")]
    pub fn include_windows(self, windows: &[&SCWindow]) -> Self {
        self.with_including_windows(windows)
    }

    /// Include specific applications and optionally except certain windows
    #[must_use]
    #[deprecated(since = "1.5.0", note = "Use with_including_applications() instead")]
    pub fn include_applications(
        self,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) -> Self {
        self.with_including_applications(applications, excepting_windows)
    }

    /// Exclude specific applications and optionally except certain windows
    #[must_use]
    #[deprecated(since = "1.5.0", note = "Use with_excluding_applications() instead")]
    pub fn exclude_applications(
        self,
        applications: &[&SCRunningApplication],
        excepting_windows: &[&SCWindow],
    ) -> Self {
        self.with_excluding_applications(applications, excepting_windows)
    }

    /// Set the content rectangle (macOS 14.2+)
    #[cfg(feature = "macos_14_2")]
    #[must_use]
    #[deprecated(since = "1.5.0", note = "Use with_content_rect() instead")]
    pub fn content_rect(self, rect: CGRect) -> Self {
        self.with_content_rect(rect)
    }

    /// Build the content filter.
    ///
    /// # Panics
    ///
    /// Panics if no filter type was set. Call `.display()` or `.window()` before `.build()`.
    /// For a non-panicking alternative that reports this as a recoverable error, use
    /// [`try_build`](Self::try_build).
    #[must_use]
    pub fn build(self) -> SCContentFilter {
        self.try_build()
            .expect("SCContentFilterBuilder: No filter type set. Call .display() or .window() before .build()")
    }

    /// Build the content filter, returning an error instead of panicking when no
    /// filter type was set.
    ///
    /// # Errors
    ///
    /// Returns [`SCError::InvalidConfiguration`] if neither `.display()` nor `.window()`
    /// was called before building.
    #[allow(clippy::too_many_lines)]
    pub fn try_build(self) -> SCResult<SCContentFilter> {
        let filter = match self.filter_type {
            FilterType::Window(window) => unsafe {
                let ptr =
                    ffi::sc_content_filter_create_with_desktop_independent_window(window.as_ptr());
                SCContentFilter(ptr)
            },
            FilterType::DisplayExcluding { display, windows } => {
                let window_refs: Vec<&SCWindow> = windows.iter().collect();
                unsafe {
                    let window_ptrs: Vec<*const c_void> =
                        window_refs.iter().map(|w| w.as_ptr()).collect();

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
                    let window_ptrs: Vec<*const c_void> =
                        window_refs.iter().map(|w| w.as_ptr()).collect();

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
            FilterType::DisplayIncludingApplications {
                display,
                applications,
                excepting_windows,
            } => {
                let app_refs: Vec<&SCRunningApplication> = applications.iter().collect();
                let window_refs: Vec<&SCWindow> = excepting_windows.iter().collect();
                unsafe {
                    let app_ptrs: Vec<*const c_void> =
                        app_refs.iter().map(|a| a.as_ptr()).collect();

                    let window_ptrs: Vec<*const c_void> =
                        window_refs.iter().map(|w| w.as_ptr()).collect();

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
            FilterType::DisplayExcludingApplications {
                display,
                applications,
                excepting_windows,
            } => {
                let app_refs: Vec<&SCRunningApplication> = applications.iter().collect();
                let window_refs: Vec<&SCWindow> = excepting_windows.iter().collect();
                unsafe {
                    let app_ptrs: Vec<*const c_void> =
                        app_refs.iter().map(|a| a.as_ptr()).collect();

                    let window_ptrs: Vec<*const c_void> =
                        window_refs.iter().map(|w| w.as_ptr()).collect();

                    #[allow(clippy::cast_possible_wrap)]
                    let ptr = ffi::sc_content_filter_create_with_display_excluding_applications_excepting_windows(
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
                return Err(SCError::invalid_config(
                    "SCContentFilterBuilder: No filter type set. \
                     Call .display() or .window() before building.",
                ));
            }
        };

        // Apply content rect if set (macOS 14.2+)
        #[cfg(feature = "macos_14_2")]
        let filter = if let Some(rect) = self.content_rect {
            filter.set_content_rect(rect)
        } else {
            filter
        };

        Ok(filter)
    }
}

impl std::fmt::Debug for SCContentFilterBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let filter_type_name = match &self.filter_type {
            FilterType::None => "None",
            FilterType::Window(_) => "Window",
            FilterType::DisplayExcluding { .. } => "DisplayExcluding",
            FilterType::DisplayIncluding { .. } => "DisplayIncluding",
            FilterType::DisplayIncludingApplications { .. } => "DisplayIncludingApplications",
            FilterType::DisplayExcludingApplications { .. } => "DisplayExcludingApplications",
        };

        let mut debug = f.debug_struct("SCContentFilterBuilder");
        debug.field("filter_type", &filter_type_name);

        #[cfg(feature = "macos_14_2")]
        debug.field("content_rect", &self.content_rect);

        debug.finish()
    }
}
