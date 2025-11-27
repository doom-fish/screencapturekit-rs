//! `SCScreenshotManager` - Single-shot screenshot capture
//!
//! Available on macOS 14.0+
//! Provides high-quality screenshot capture without the overhead of setting up a stream.

use crate::error::SCError;
use crate::stream::content_filter::SCContentFilter;
use crate::stream::configuration::SCStreamConfiguration;
use std::ffi::c_void;

extern "C" fn image_callback(
    image_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    let tx = unsafe { Box::from_raw(user_data.cast::<std::sync::mpsc::Sender<Result<CGImage, SCError>>>()) };
    
    if !error_ptr.is_null() {
        let error_msg = unsafe {
            std::ffi::CStr::from_ptr(error_ptr)
                .to_string_lossy()
                .into_owned()
        };
        let _ = tx.send(Err(crate::utils::error::create_sc_error(&error_msg)));
    } else if !image_ptr.is_null() {
        let _ = tx.send(Ok(CGImage::from_ptr(image_ptr)));
    } else {
        let _ = tx.send(Err(crate::utils::error::create_sc_error("Unknown error")));
    }
}

extern "C" fn buffer_callback(
    buffer_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    let tx = unsafe { Box::from_raw(user_data.cast::<std::sync::mpsc::Sender<Result<crate::cm::CMSampleBuffer, SCError>>>()) };
    
    if !error_ptr.is_null() {
        let error_msg = unsafe {
            std::ffi::CStr::from_ptr(error_ptr)
                .to_string_lossy()
                .into_owned()
        };
        let _ = tx.send(Err(crate::utils::error::create_sc_error(&error_msg)));
    } else if !buffer_ptr.is_null() {
        let buffer = unsafe { crate::cm::CMSampleBuffer::from_ptr(buffer_ptr.cast_mut()) };
        let _ = tx.send(Ok(buffer));
    } else {
        let _ = tx.send(Err(crate::utils::error::create_sc_error("Unknown error")));
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
/// let config = SCStreamConfiguration::builder().build();
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
    /// # let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();
    /// # let config = SCStreamConfiguration::default();
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
/// let config = SCStreamConfiguration::builder()
///     .width(1920)
///     .height(1080)
///     .build();
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
    pub fn capture_image(
        content_filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> Result<CGImage, SCError> {
        let (tx, rx) = std::sync::mpsc::channel();

        let user_data = Box::into_raw(Box::new(tx)).cast::<c_void>();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_image(
                content_filter.as_ptr(),
                configuration.as_ptr(),
                image_callback,
                user_data,
            );
        }

        rx.recv()
            .map_err(|e| crate::utils::error::create_sc_error(&format!("Channel receive error: {e}")))?
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
    pub fn capture_sample_buffer(
        content_filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> Result<crate::cm::CMSampleBuffer, SCError> {
        let (tx, rx) = std::sync::mpsc::channel();

        let user_data = Box::into_raw(Box::new(tx)).cast::<c_void>();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_sample_buffer(
                content_filter.as_ptr(),
                configuration.as_ptr(),
                buffer_callback,
                user_data,
            );
        }

        rx.recv()
            .map_err(|e| crate::utils::error::create_sc_error(&format!("Channel receive error: {e}")))?
    }
}

