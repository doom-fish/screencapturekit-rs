//! `SCScreenshotManager` - Single-shot screenshot capture
//!
//! Available on macOS 14.0+
//! Provides high-quality screenshot capture without the overhead of setting up a stream.

use crate::error::SCError;
use crate::stream::configuration::SCStreamConfiguration;
use crate::stream::content_filter::SCContentFilter;
use crate::utils::sync_completion::{error_from_cstr, SyncCompletion};
use std::ffi::c_void;

#[cfg(feature = "macos_15_2")]
use crate::cg::CGRect;

extern "C" fn image_callback(
    image_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    if !error_ptr.is_null() {
        let error = unsafe { error_from_cstr(error_ptr) };
        unsafe { SyncCompletion::<CGImage>::complete_err(user_data, error) };
    } else if !image_ptr.is_null() {
        unsafe { SyncCompletion::complete_ok(user_data, CGImage::from_ptr(image_ptr)) };
    } else {
        unsafe { SyncCompletion::<CGImage>::complete_err(user_data, "Unknown error".to_string()) };
    }
}

extern "C" fn buffer_callback(
    buffer_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    if !error_ptr.is_null() {
        let error = unsafe { error_from_cstr(error_ptr) };
        unsafe { SyncCompletion::<crate::cm::CMSampleBuffer>::complete_err(user_data, error) };
    } else if !buffer_ptr.is_null() {
        let buffer = unsafe { crate::cm::CMSampleBuffer::from_ptr(buffer_ptr.cast_mut()) };
        unsafe { SyncCompletion::complete_ok(user_data, buffer) };
    } else {
        unsafe {
            SyncCompletion::<crate::cm::CMSampleBuffer>::complete_err(
                user_data,
                "Unknown error".to_string(),
            );
        };
    }
}

/// `CGImage` wrapper for screenshots
///
/// Represents a Core Graphics image returned from screenshot capture.
///
/// # Examples
///
/// ```no_run
/// # use screencapturekit::screenshot_manager::SCScreenshotManager;
/// # use screencapturekit::stream::{content_filter::SCContentFilter, configuration::SCStreamConfiguration};
/// # use screencapturekit::shareable_content::SCShareableContent;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let content = SCShareableContent::get()?;
/// let display = &content.displays()[0];
/// let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
/// let config = SCStreamConfiguration::new()
///     .with_width(1920)
///     .with_height(1080);
///
/// let image = SCScreenshotManager::capture_image(&filter, &config)?;
/// println!("Screenshot size: {}x{}", image.width(), image.height());
/// # Ok(())
/// # }
/// ```
pub struct CGImage {
    ptr: *const c_void,
}

impl CGImage {
    pub(crate) fn from_ptr(ptr: *const c_void) -> Self {
        Self { ptr }
    }

    /// Get image width in pixels
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use screencapturekit::screenshot_manager::SCScreenshotManager;
    /// # use screencapturekit::stream::{content_filter::SCContentFilter, configuration::SCStreamConfiguration};
    /// # use screencapturekit::shareable_content::SCShareableContent;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let display = &content.displays()[0];
    /// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
    /// # let config = SCStreamConfiguration::new().with_width(1920).with_height(1080);
    /// let image = SCScreenshotManager::capture_image(&filter, &config)?;
    /// let width = image.width();
    /// println!("Width: {}", width);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn width(&self) -> usize {
        unsafe { crate::ffi::cgimage_get_width(self.ptr) }
    }

    /// Get image height in pixels
    #[must_use]
    pub fn height(&self) -> usize {
        unsafe { crate::ffi::cgimage_get_height(self.ptr) }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const c_void {
        self.ptr
    }

    /// Get raw RGBA pixel data
    ///
    /// Returns a vector containing RGBA bytes (4 bytes per pixel).
    /// The data is in row-major order.
    ///
    /// # Errors
    /// Returns an error if the pixel data cannot be extracted
    pub fn get_rgba_data(&self) -> Result<Vec<u8>, SCError> {
        let mut data_ptr: *const u8 = std::ptr::null();
        let mut data_length: usize = 0;

        let success = unsafe {
            crate::ffi::cgimage_get_data(
                self.ptr,
                std::ptr::addr_of_mut!(data_ptr),
                std::ptr::addr_of_mut!(data_length),
            )
        };

        if !success || data_ptr.is_null() {
            return Err(SCError::internal_error(
                "Failed to extract pixel data from CGImage",
            ));
        }

        // Copy the data into a Vec
        let data = unsafe { std::slice::from_raw_parts(data_ptr, data_length).to_vec() };

        // Free the allocated data
        unsafe {
            crate::ffi::cgimage_free_data(data_ptr.cast_mut());
        }

        Ok(data)
    }
}

impl Drop for CGImage {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                crate::ffi::cgimage_release(self.ptr);
            }
        }
    }
}

unsafe impl Send for CGImage {}
unsafe impl Sync for CGImage {}

/// Manager for capturing single screenshots
///
/// Available on macOS 14.0+. Provides a simpler API than `SCStream` for one-time captures.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::screenshot_manager::SCScreenshotManager;
/// use screencapturekit::stream::{content_filter::SCContentFilter, configuration::SCStreamConfiguration};
/// use screencapturekit::shareable_content::SCShareableContent;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let content = SCShareableContent::get()?;
/// let display = &content.displays()[0];
/// let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
/// let config = SCStreamConfiguration::new()
///     .with_width(1920)
///     .with_height(1080);
///
/// let image = SCScreenshotManager::capture_image(&filter, &config)?;
/// println!("Captured screenshot: {}x{}", image.width(), image.height());
/// # Ok(())
/// # }
/// ```
pub struct SCScreenshotManager;

impl SCScreenshotManager {
    /// Capture a single screenshot as a `CGImage`
    ///
    /// # Errors
    /// Returns an error if:
    /// - The system is not macOS 14.0+
    /// - Screen recording permission is not granted
    /// - The capture fails for any reason
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned.
    pub fn capture_image(
        content_filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> Result<CGImage, SCError> {
        let (completion, context) = SyncCompletion::<CGImage>::new();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_image(
                content_filter.as_ptr(),
                configuration.as_ptr(),
                image_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::ScreenshotError)
    }

    /// Capture a single screenshot as a `CMSampleBuffer`
    ///
    /// Returns the sample buffer for advanced processing.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The system is not macOS 14.0+
    /// - Screen recording permission is not granted
    /// - The capture fails for any reason
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned.
    pub fn capture_sample_buffer(
        content_filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> Result<crate::cm::CMSampleBuffer, SCError> {
        let (completion, context) = SyncCompletion::<crate::cm::CMSampleBuffer>::new();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_sample_buffer(
                content_filter.as_ptr(),
                configuration.as_ptr(),
                buffer_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::ScreenshotError)
    }

    /// Capture a screenshot of a specific screen region (macOS 15.2+)
    ///
    /// This method captures the content within the specified rectangle,
    /// which can span multiple displays.
    ///
    /// # Arguments
    /// * `rect` - The rectangle to capture, in screen coordinates (points)
    ///
    /// # Errors
    /// Returns an error if:
    /// - The system is not macOS 15.2+
    /// - Screen recording permission is not granted
    /// - The capture fails for any reason
    ///
    /// # Examples
    /// ```rust,ignore
    /// use screencapturekit::screenshot_manager::SCScreenshotManager;
    /// use screencapturekit::cg::CGRect;
    ///
    /// let rect = CGRect::new(0.0, 0.0, 1920.0, 1080.0);
    /// let image = SCScreenshotManager::capture_image_in_rect(rect)?;
    /// ```
    #[cfg(feature = "macos_15_2")]
    pub fn capture_image_in_rect(rect: CGRect) -> Result<CGImage, SCError> {
        let (completion, context) = SyncCompletion::<CGImage>::new();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_image_in_rect(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                image_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::ScreenshotError)
    }
}
