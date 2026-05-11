//! `SCScreenshotManager` - Single-shot screenshot capture
//!
//! Available on macOS 14.0+.
//! Provides high-quality screenshot capture without the overhead of setting up a stream.
//!
//! ## When to Use
//!
//! Use `SCScreenshotManager` when you need:
//! - A single screenshot rather than continuous capture
//! - Quick capture without stream setup/teardown overhead
//! - Direct saving to image files
//!
//! For continuous capture, use [`SCStream`](crate::stream::SCStream) instead.
//!
//! ## Example
//!
//! ```no_run
//! use screencapturekit::screenshot_manager::{SCScreenshotManager, ImageFormat};
//! use screencapturekit::stream::{content_filter::SCContentFilter, configuration::SCStreamConfiguration};
//! use screencapturekit::shareable_content::SCShareableContent;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let content = SCShareableContent::get()?;
//! let display = &content.displays()[0];
//! let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
//! let config = SCStreamConfiguration::new()
//!     .with_width(1920)
//!     .with_height(1080);
//!
//! // Capture as CGImage
//! let image = SCScreenshotManager::capture_image(&filter, &config)?;
//! println!("Screenshot: {}x{}", image.width(), image.height());
//!
//! // Save to file
//! image.save_png("/tmp/screenshot.png")?;
//!
//! // Or save as JPEG with quality
//! image.save("/tmp/screenshot.jpg", ImageFormat::Jpeg(0.85))?;
//! # Ok(())
//! # }
//! ```

use crate::error::SCError;
use crate::stream::configuration::SCStreamConfiguration;
use crate::stream::content_filter::SCContentFilter;
use crate::utils::completion::{error_from_cstr, SyncCompletion};
use std::ffi::c_void;

#[cfg(feature = "macos_15_2")]
use crate::cg::CGRect;

/// Image output format for saving screenshots
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::screenshot_manager::ImageFormat;
///
/// // PNG for lossless quality
/// let format = ImageFormat::Png;
///
/// // JPEG with 80% quality
/// let format = ImageFormat::Jpeg(0.8);
///
/// // HEIC with 90% quality (smaller file size than JPEG)
/// let format = ImageFormat::Heic(0.9);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    /// PNG format (lossless)
    Png,
    /// JPEG format with quality (0.0-1.0)
    Jpeg(f32),
    /// TIFF format (lossless)
    Tiff,
    /// GIF format
    Gif,
    /// BMP format
    Bmp,
    /// HEIC format with quality (0.0-1.0) - efficient compression
    Heic(f32),
}

impl ImageFormat {
    fn to_format_id(self) -> i32 {
        match self {
            Self::Png => 0,
            Self::Jpeg(_) => 1,
            Self::Tiff => 2,
            Self::Gif => 3,
            Self::Bmp => 4,
            Self::Heic(_) => 5,
        }
    }

    fn quality(self) -> f32 {
        match self {
            Self::Jpeg(q) | Self::Heic(q) => q.clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    /// Get the typical file extension for this format
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg(_) => "jpg",
            Self::Tiff => "tiff",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Heic(_) => "heic",
        }
    }
}

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

#[cfg(feature = "macos_26_0")]
extern "C" fn screenshot_output_callback(
    output_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    if !error_ptr.is_null() {
        let error = unsafe { error_from_cstr(error_ptr) };
        unsafe { SyncCompletion::<SCScreenshotOutput>::complete_err(user_data, error) };
    } else if !output_ptr.is_null() {
        unsafe {
            SyncCompletion::complete_ok(user_data, SCScreenshotOutput::from_ptr(output_ptr));
        };
    } else {
        unsafe {
            SyncCompletion::<SCScreenshotOutput>::complete_err(
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
/// let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
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

/// Internal selector for the channel ordering passed to the Swift renderer.
#[derive(Debug, Clone, Copy)]
enum PixelLayout {
    Rgba,
    Bgra,
}

impl PixelLayout {
    const fn name(self) -> &'static str {
        match self {
            Self::Rgba => "RGBA",
            Self::Bgra => "BGRA",
        }
    }

    /// Dispatch into the matching Swift bridge entry point.
    ///
    /// # Safety
    /// The destination must point to at least `capacity` bytes and `ptr` must
    /// be a live retained `CGImage`.
    unsafe fn render(self, ptr: *const c_void, dest: *mut u8, capacity: usize) -> usize {
        match self {
            Self::Rgba => crate::ffi::cgimage_render_rgba_into(ptr, dest, capacity),
            Self::Bgra => crate::ffi::cgimage_render_bgra_into(ptr, dest, capacity),
        }
    }
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
    /// # let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
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
    /// **Performance note:** every `ScreenCaptureKit`-produced `CGImage` is
    /// natively in **BGRA**. Forcing RGBA here makes `CGContext.draw` perform
    /// a per-pixel channel swap that costs ~20 ms on a 4K image. If your
    /// consumer accepts BGRA (Metal / wgpu / ffmpeg / most GPU pipelines),
    /// prefer [`bgra_data`](Self::bgra_data) which skips the conversion.
    ///
    /// **Allocation note:** this allocates a fresh `Vec<u8>` of
    /// `width*height*4` bytes per call (~33 MB for 4K). For sustained
    /// screenshot loops, prefer [`rgba_data_into`](Self::rgba_data_into)
    /// which writes into a caller-supplied buffer and lets you reuse the
    /// allocation across calls.
    ///
    /// # Errors
    /// Returns an error if the pixel data cannot be extracted
    pub fn rgba_data(&self) -> Result<Vec<u8>, SCError> {
        self.render_pixel_data(PixelLayout::Rgba)
    }

    /// Get raw **BGRA** pixel data — the native `ScreenCaptureKit` pixel layout.
    ///
    /// Returns a vector containing BGRA bytes (4 bytes per pixel) in row-major
    /// order. Each pixel is stored as `[B, G, R, A]`.
    ///
    /// This skips the BGRA → RGBA channel-swap that [`rgba_data`](Self::rgba_data)
    /// performs inside `CGContext.draw`, saving roughly **20 ms on a 4K screenshot**.
    /// Use this when the downstream consumer accepts BGRA natively — that
    /// includes Metal (`MTLPixelFormat::BGRA8Unorm`), wgpu (`Bgra8Unorm`),
    /// ffmpeg (`AV_PIX_FMT_BGRA`), and any direct upload to a `kCVPixelFormatType_32BGRA`
    /// pixel buffer.
    ///
    /// For sustained capture loops, see [`bgra_data_into`](Self::bgra_data_into)
    /// which writes into a caller-supplied buffer.
    ///
    /// # Errors
    /// Returns an error if the pixel data cannot be extracted.
    pub fn bgra_data(&self) -> Result<Vec<u8>, SCError> {
        self.render_pixel_data(PixelLayout::Bgra)
    }

    /// Render the image's RGBA bytes into a caller-supplied buffer.
    ///
    /// `dest` must hold at least `width * height * 4` bytes. Returns the
    /// number of bytes written on success. Use this for sustained screenshot
    /// loops to amortise the per-call ~33 MB-at-4K allocation across many
    /// frames — pre-allocate one `Vec<u8>::with_capacity(...)` (or set
    /// `dest.len() = capacity` once after the first call) and reuse it.
    ///
    /// # Errors
    /// Returns `SCError::InternalError` if `dest` is too small or the
    /// `CGContext` draw fails.
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
    /// # let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    /// # let config = SCStreamConfiguration::new().with_width(1920).with_height(1080);
    /// // Pre-allocate once, reuse across many screenshots.
    /// let mut buffer: Vec<u8> = vec![0; 1920 * 1080 * 4];
    /// for _ in 0..100 {
    ///     let img = SCScreenshotManager::capture_image(&filter, &config)?;
    ///     img.rgba_data_into(&mut buffer)?;
    ///     // process `buffer`...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn rgba_data_into(&self, dest: &mut [u8]) -> Result<usize, SCError> {
        self.render_pixel_data_into(dest, PixelLayout::Rgba)
    }

    /// Render the image's **BGRA** bytes into a caller-supplied buffer.
    ///
    /// Same shape as [`rgba_data_into`](Self::rgba_data_into) but in the
    /// native source pixel layout — saves the per-pixel R↔B swap
    /// `rgba_data_into` performs. Combine with a reusable buffer for the
    /// fastest possible sustained-capture loop.
    ///
    /// # Errors
    /// Returns `SCError::InternalError` if `dest` is too small or the
    /// `CGContext` draw fails.
    pub fn bgra_data_into(&self, dest: &mut [u8]) -> Result<usize, SCError> {
        self.render_pixel_data_into(dest, PixelLayout::Bgra)
    }

    fn render_pixel_data(&self, layout: PixelLayout) -> Result<Vec<u8>, SCError> {
        let total_bytes = self.required_byte_size()?;
        if total_bytes == 0 {
            return Ok(Vec::new());
        }

        // Allocate uninitialised — the FFI draws straight into this buffer via
        // CGContext, writing every byte. The previous flow allocated three
        // times the data: CGContext buffer + Swift-owned malloc copy + Rust
        // .to_vec() copy. This single-buffer form measured ~28% end-to-end
        // faster on 4K screenshots; the BGRA variant additionally skips the
        // per-pixel channel swap CGContext.draw performs when targeting RGBA
        // (~20 ms saved on a 4K shot).
        let mut data: Vec<u8> = Vec::with_capacity(total_bytes);
        let written = unsafe { layout.render(self.ptr, data.as_mut_ptr(), total_bytes) };

        if written != total_bytes {
            return Err(SCError::internal_error(format!(
                "Failed to render CGImage into {} buffer",
                layout.name()
            )));
        }

        unsafe { data.set_len(total_bytes) };
        Ok(data)
    }

    fn render_pixel_data_into(
        &self,
        dest: &mut [u8],
        layout: PixelLayout,
    ) -> Result<usize, SCError> {
        let total_bytes = self.required_byte_size()?;
        if dest.len() < total_bytes {
            return Err(SCError::internal_error(format!(
                "Destination buffer too small: need {total_bytes} bytes, got {}",
                dest.len()
            )));
        }
        if total_bytes == 0 {
            return Ok(0);
        }

        let written = unsafe { layout.render(self.ptr, dest.as_mut_ptr(), total_bytes) };
        if written != total_bytes {
            return Err(SCError::internal_error(format!(
                "Failed to render CGImage into {} buffer",
                layout.name()
            )));
        }
        Ok(written)
    }

    fn required_byte_size(&self) -> Result<usize, SCError> {
        self.width()
            .checked_mul(self.height())
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| SCError::internal_error("CGImage dimensions overflow usize"))
    }

    /// Save the image to a PNG file
    ///
    /// # Arguments
    /// * `path` - The file path to save the PNG to
    ///
    /// # Errors
    /// Returns an error if the image cannot be saved
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
    /// # let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    /// # let config = SCStreamConfiguration::new().with_width(1920).with_height(1080);
    /// let image = SCScreenshotManager::capture_image(&filter, &config)?;
    /// image.save_png("/tmp/screenshot.png")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_png(&self, path: &str) -> Result<(), SCError> {
        self.save(path, ImageFormat::Png)
    }

    /// Save the image to a file in the specified format
    ///
    /// # Arguments
    /// * `path` - The file path to save the image to
    /// * `format` - The output format (PNG, JPEG, TIFF, GIF, BMP, or HEIC)
    ///
    /// # Errors
    /// Returns an error if the image cannot be saved
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use screencapturekit::screenshot_manager::{SCScreenshotManager, ImageFormat};
    /// # use screencapturekit::stream::{content_filter::SCContentFilter, configuration::SCStreamConfiguration};
    /// # use screencapturekit::shareable_content::SCShareableContent;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let display = &content.displays()[0];
    /// # let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    /// # let config = SCStreamConfiguration::new().with_width(1920).with_height(1080);
    /// let image = SCScreenshotManager::capture_image(&filter, &config)?;
    ///
    /// // Save as PNG (lossless)
    /// image.save("/tmp/screenshot.png", ImageFormat::Png)?;
    ///
    /// // Save as JPEG with 85% quality
    /// image.save("/tmp/screenshot.jpg", ImageFormat::Jpeg(0.85))?;
    ///
    /// // Save as HEIC with 90% quality (smaller file size)
    /// image.save("/tmp/screenshot.heic", ImageFormat::Heic(0.9))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save(&self, path: &str, format: ImageFormat) -> Result<(), SCError> {
        let c_path = std::ffi::CString::new(path)
            .map_err(|_| SCError::internal_error("Path contains null bytes"))?;

        let success = unsafe {
            crate::ffi::cgimage_save_to_file(
                self.ptr,
                c_path.as_ptr(),
                format.to_format_id(),
                format.quality(),
            )
        };

        if success {
            Ok(())
        } else {
            Err(SCError::internal_error(format!(
                "Failed to save image as {}",
                format.extension().to_uppercase()
            )))
        }
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

impl std::fmt::Debug for CGImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CGImage")
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
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
/// let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
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
    /// ```no_run
    /// use screencapturekit::screenshot_manager::SCScreenshotManager;
    /// use screencapturekit::cg::CGRect;
    ///
    /// fn example() -> Result<(), screencapturekit::utils::error::SCError> {
    ///     let rect = CGRect::new(0.0, 0.0, 1920.0, 1080.0);
    ///     let image = SCScreenshotManager::capture_image_in_rect(rect)?;
    ///     Ok(())
    /// }
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

    /// Capture a screenshot with advanced configuration (macOS 26.0+)
    ///
    /// This method uses the new `SCScreenshotConfiguration` for more control
    /// over the screenshot output, including HDR support and file saving.
    ///
    /// # Arguments
    /// * `content_filter` - The content filter specifying what to capture
    /// * `configuration` - The screenshot configuration
    ///
    /// # Errors
    /// Returns an error if the capture fails
    ///
    /// # Examples
    /// ```no_run
    /// use screencapturekit::screenshot_manager::{SCScreenshotManager, SCScreenshotConfiguration, SCScreenshotDynamicRange};
    /// use screencapturekit::stream::content_filter::SCContentFilter;
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// fn example() -> Option<()> {
    ///     let content = SCShareableContent::get().ok()?;
    ///     let displays = content.displays();
    ///     let display = displays.first()?;
    ///     let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    ///     let config = SCScreenshotConfiguration::new()
    ///         .with_width(1920)
    ///         .with_height(1080)
    ///         .with_dynamic_range(SCScreenshotDynamicRange::BothSDRAndHDR);
    ///
    ///     let output = SCScreenshotManager::capture_screenshot(&filter, &config).ok()?;
    ///     if let Some(sdr) = output.sdr_image() {
    ///         println!("SDR image: {}x{}", sdr.width(), sdr.height());
    ///     }
    ///     Some(())
    /// }
    /// ```
    #[cfg(feature = "macos_26_0")]
    pub fn capture_screenshot(
        content_filter: &SCContentFilter,
        configuration: &SCScreenshotConfiguration,
    ) -> Result<SCScreenshotOutput, SCError> {
        let (completion, context) = SyncCompletion::<SCScreenshotOutput>::new();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_screenshot(
                content_filter.as_ptr(),
                configuration.as_ptr(),
                screenshot_output_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::ScreenshotError)
    }

    /// Capture a screenshot of a specific region with advanced configuration (macOS 26.0+)
    ///
    /// # Arguments
    /// * `rect` - The rectangle to capture, in screen coordinates (points)
    /// * `configuration` - The screenshot configuration
    ///
    /// # Errors
    /// Returns an error if the capture fails
    #[cfg(feature = "macos_26_0")]
    pub fn capture_screenshot_in_rect(
        rect: crate::cg::CGRect,
        configuration: &SCScreenshotConfiguration,
    ) -> Result<SCScreenshotOutput, SCError> {
        let (completion, context) = SyncCompletion::<SCScreenshotOutput>::new();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_screenshot_in_rect(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                configuration.as_ptr(),
                screenshot_output_callback,
                context,
            );
        }

        completion.wait().map_err(SCError::ScreenshotError)
    }
}

// ============================================================================
// SCScreenshotConfiguration (macOS 26.0+)
// ============================================================================

/// Display intent for screenshot rendering (macOS 26.0+)
#[cfg(feature = "macos_26_0")]
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCScreenshotDisplayIntent {
    /// Render on the canonical display
    #[default]
    Canonical = 0,
    /// Render on the local display
    Local = 1,
}

/// Dynamic range for screenshot output (macOS 26.0+)
#[cfg(feature = "macos_26_0")]
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SCScreenshotDynamicRange {
    /// SDR output only
    #[default]
    SDR = 0,
    /// HDR output only
    HDR = 1,
    /// Both SDR and HDR output
    BothSDRAndHDR = 2,
}

/// Configuration for advanced screenshot capture (macOS 26.0+)
///
/// Provides fine-grained control over screenshot output including:
/// - Output dimensions
/// - Source and destination rectangles
/// - Shadow and clipping behavior
/// - HDR/SDR dynamic range
/// - File output
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::screenshot_manager::{SCScreenshotConfiguration, SCScreenshotDynamicRange};
///
/// let config = SCScreenshotConfiguration::new()
///     .with_width(1920)
///     .with_height(1080)
///     .with_shows_cursor(true)
///     .with_dynamic_range(SCScreenshotDynamicRange::BothSDRAndHDR);
/// ```
#[cfg(feature = "macos_26_0")]
pub struct SCScreenshotConfiguration {
    ptr: *const c_void,
}

#[cfg(feature = "macos_26_0")]
impl SCScreenshotConfiguration {
    /// Create a new screenshot configuration
    ///
    /// # Panics
    /// Panics if the configuration cannot be created (requires macOS 26.0+)
    #[must_use]
    pub fn new() -> Self {
        let ptr = unsafe { crate::ffi::sc_screenshot_configuration_create() };
        assert!(!ptr.is_null(), "Failed to create SCScreenshotConfiguration");
        Self { ptr }
    }

    /// Set the output width in pixels
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn with_width(self, width: usize) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_width(self.ptr, width as isize);
        }
        self
    }

    /// Set the output height in pixels
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn with_height(self, height: usize) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_height(self.ptr, height as isize);
        }
        self
    }

    /// Set whether to show the cursor
    #[must_use]
    pub fn with_shows_cursor(self, shows_cursor: bool) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_shows_cursor(self.ptr, shows_cursor);
        }
        self
    }

    /// Set the source rectangle (subset of capture area)
    #[must_use]
    pub fn with_source_rect(self, rect: crate::cg::CGRect) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_source_rect(
                self.ptr,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
            );
        }
        self
    }

    /// Set the destination rectangle (output area)
    #[must_use]
    pub fn with_destination_rect(self, rect: crate::cg::CGRect) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_destination_rect(
                self.ptr,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
            );
        }
        self
    }

    /// Set whether to ignore shadows
    #[must_use]
    pub fn with_ignore_shadows(self, ignore_shadows: bool) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_ignore_shadows(self.ptr, ignore_shadows);
        }
        self
    }

    /// Set whether to ignore clipping
    #[must_use]
    pub fn with_ignore_clipping(self, ignore_clipping: bool) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_ignore_clipping(self.ptr, ignore_clipping);
        }
        self
    }

    /// Set whether to include child windows
    #[must_use]
    pub fn with_include_child_windows(self, include_child_windows: bool) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_include_child_windows(
                self.ptr,
                include_child_windows,
            );
        }
        self
    }

    /// Set the display intent
    #[must_use]
    pub fn with_display_intent(self, display_intent: SCScreenshotDisplayIntent) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_display_intent(
                self.ptr,
                display_intent as i32,
            );
        }
        self
    }

    /// Set the dynamic range
    #[must_use]
    pub fn with_dynamic_range(self, dynamic_range: SCScreenshotDynamicRange) -> Self {
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_dynamic_range(
                self.ptr,
                dynamic_range as i32,
            );
        }
        self
    }

    /// Set the output file URL
    ///
    /// # Panics
    /// Panics if the path contains null bytes
    #[must_use]
    pub fn with_file_path(self, path: &str) -> Self {
        let c_path = std::ffi::CString::new(path).expect("path should not contain null bytes");
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_file_url(self.ptr, c_path.as_ptr());
        }
        self
    }

    /// Set the content type (output format) using `UTType` identifier
    ///
    /// Common identifiers include:
    /// - `"public.png"` - PNG format
    /// - `"public.jpeg"` - JPEG format
    /// - `"public.heic"` - HEIC format
    /// - `"public.tiff"` - TIFF format
    ///
    /// Use [`supported_content_types()`](Self::supported_content_types) to get
    /// available formats.
    ///
    /// # Panics
    /// Panics if the identifier contains null bytes
    #[must_use]
    pub fn with_content_type(self, identifier: &str) -> Self {
        let c_id =
            std::ffi::CString::new(identifier).expect("identifier should not contain null bytes");
        unsafe {
            crate::ffi::sc_screenshot_configuration_set_content_type(self.ptr, c_id.as_ptr());
        }
        self
    }

    /// Get the current content type as `UTType` identifier
    pub fn content_type(&self) -> Option<String> {
        let mut buffer = vec![0i8; 256];
        let success = unsafe {
            crate::ffi::sc_screenshot_configuration_get_content_type(
                self.ptr,
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        };
        if success {
            let c_str = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) };
            c_str.to_str().ok().map(ToString::to_string)
        } else {
            None
        }
    }

    /// Get the list of supported content types (`UTType` identifiers)
    ///
    /// Returns a list of `UTType` identifiers that can be used with
    /// [`with_content_type()`](Self::with_content_type).
    ///
    /// Common types include:
    /// - `"public.png"` - PNG format
    /// - `"public.jpeg"` - JPEG format
    /// - `"public.heic"` - HEIC format
    pub fn supported_content_types() -> Vec<String> {
        let count =
            unsafe { crate::ffi::sc_screenshot_configuration_get_supported_content_types_count() };
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let mut buffer = vec![0i8; 256];
            let success = unsafe {
                crate::ffi::sc_screenshot_configuration_get_supported_content_type_at(
                    i,
                    buffer.as_mut_ptr(),
                    buffer.len(),
                )
            };
            if success {
                let c_str = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) };
                if let Ok(s) = c_str.to_str() {
                    result.push(s.to_string());
                }
            }
        }
        result
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *const c_void {
        self.ptr
    }
}

#[cfg(feature = "macos_26_0")]
impl std::fmt::Debug for SCScreenshotConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCScreenshotConfiguration")
            .field("content_type", &self.content_type())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "macos_26_0")]
impl Default for SCScreenshotConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "macos_26_0")]
impl Drop for SCScreenshotConfiguration {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                crate::ffi::sc_screenshot_configuration_release(self.ptr);
            }
        }
    }
}

#[cfg(feature = "macos_26_0")]
unsafe impl Send for SCScreenshotConfiguration {}
#[cfg(feature = "macos_26_0")]
unsafe impl Sync for SCScreenshotConfiguration {}

// ============================================================================
// SCScreenshotOutput (macOS 26.0+)
// ============================================================================

/// Output from advanced screenshot capture (macOS 26.0+)
///
/// Contains SDR and/or HDR images depending on the configuration,
/// and optionally the file URL where the image was saved.
#[cfg(feature = "macos_26_0")]
pub struct SCScreenshotOutput {
    ptr: *const c_void,
}

#[cfg(feature = "macos_26_0")]
impl SCScreenshotOutput {
    pub(crate) fn from_ptr(ptr: *const c_void) -> Self {
        Self { ptr }
    }

    /// Get the SDR image if available
    #[must_use]
    pub fn sdr_image(&self) -> Option<CGImage> {
        let ptr = unsafe { crate::ffi::sc_screenshot_output_get_sdr_image(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(CGImage::from_ptr(ptr))
        }
    }

    /// Get the HDR image if available
    #[must_use]
    pub fn hdr_image(&self) -> Option<CGImage> {
        let ptr = unsafe { crate::ffi::sc_screenshot_output_get_hdr_image(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(CGImage::from_ptr(ptr))
        }
    }

    /// Get the file URL where the image was saved, if applicable
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn file_url(&self) -> Option<String> {
        let mut buffer = vec![0i8; 4096];
        let success = unsafe {
            crate::ffi::sc_screenshot_output_get_file_url(
                self.ptr,
                buffer.as_mut_ptr(),
                buffer.len() as isize,
            )
        };
        if success {
            let c_str = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) };
            c_str.to_str().ok().map(String::from)
        } else {
            None
        }
    }
}

#[cfg(feature = "macos_26_0")]
impl std::fmt::Debug for SCScreenshotOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCScreenshotOutput")
            .field(
                "sdr_image",
                &self.sdr_image().map(|i| (i.width(), i.height())),
            )
            .field(
                "hdr_image",
                &self.hdr_image().map(|i| (i.width(), i.height())),
            )
            .field("file_url", &self.file_url())
            .finish()
    }
}

#[cfg(feature = "macos_26_0")]
impl Drop for SCScreenshotOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                crate::ffi::sc_screenshot_output_release(self.ptr);
            }
        }
    }
}

#[cfg(feature = "macos_26_0")]
unsafe impl Send for SCScreenshotOutput {}
#[cfg(feature = "macos_26_0")]
unsafe impl Sync for SCScreenshotOutput {}
