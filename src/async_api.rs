//! Async API for `ScreenCaptureKit`
//!
//! This module provides async versions of operations when the `async` feature is enabled.
//! The async API is **executor-agnostic** and works with any async runtime (Tokio, async-std, smol, etc.).
//!
//! ## Available Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncSCShareableContent`] | Async content queries |
//! | [`AsyncSCStream`] | Async stream with frame iteration |
//! | [`AsyncSCScreenshotManager`] | Async screenshot capture (macOS 14.0+) |
//! | [`AsyncSCContentSharingPicker`] | Async content picker UI (macOS 14.0+) |
//! | [`AsyncSCRecordingOutput`] | Async recording with events (macOS 15.0+) |
//!
//! ## Runtime Agnostic Design
//!
//! This async API uses only `std` types and works with **any** async runtime:
//! - Uses callback-based Swift FFI for true async operations
//! - Uses `std::sync::{Arc, Mutex}` for synchronization
//! - Uses `std::task::{Poll, Waker}` for async primitives
//! - Uses `std::future::Future` trait
//!
//! ## Examples
//!
//! ### Basic Async Content Query
//!
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use screencapturekit::async_api::AsyncSCShareableContent;
//!
//! let content = AsyncSCShareableContent::get().await?;
//! println!("Found {} displays", content.displays().len());
//! println!("Found {} windows", content.windows().len());
//! # Ok(())
//! # }
//! ```
//!
//! ### Async Stream with Frame Iteration
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
//! use screencapturekit::stream::configuration::SCStreamConfiguration;
//! use screencapturekit::stream::content_filter::SCContentFilter;
//! use screencapturekit::stream::output_type::SCStreamOutputType;
//!
//! let content = AsyncSCShareableContent::get().await?;
//! let display = &content.displays()[0];
//! let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
//! let config = SCStreamConfiguration::new().with_width(1920).with_height(1080);
//!
//! let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
//! stream.start_capture()?;
//!
//! // Process frames asynchronously
//! for _ in 0..100 {
//!     if let Some(frame) = stream.next().await {
//!         println!("Got frame at {:?}", frame.presentation_timestamp());
//!     }
//! }
//!
//! stream.stop_capture()?;
//! # Ok(())
//! # }
//! ```

use crate::error::SCError;
use crate::shareable_content::SCShareableContent;
use crate::stream::configuration::SCStreamConfiguration;
use crate::stream::content_filter::SCContentFilter;
use crate::utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

// ============================================================================
// AsyncSCShareableContent - True async with callback-based FFI
// ============================================================================

/// Callback from Swift FFI for shareable content
extern "C" fn shareable_content_callback(
    content: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    if !error.is_null() {
        let error_msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<SCShareableContent>::complete_err(user_data, error_msg) };
    } else if !content.is_null() {
        let sc = unsafe { SCShareableContent::from_ptr(content) };
        unsafe { AsyncCompletion::complete_ok(user_data, sc) };
    } else {
        unsafe {
            AsyncCompletion::<SCShareableContent>::complete_err(
                user_data,
                "Unknown error".to_string(),
            );
        };
    }
}

/// Future for async shareable content retrieval
pub struct AsyncShareableContentFuture {
    inner: AsyncCompletionFuture<SCShareableContent>,
}

impl std::fmt::Debug for AsyncShareableContentFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncShareableContentFuture")
            .finish_non_exhaustive()
    }
}

impl Future for AsyncShareableContentFuture {
    type Output = Result<SCShareableContent, SCError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(SCError::NoShareableContent))
    }
}

/// Async wrapper for `SCShareableContent`
///
/// Provides async methods to retrieve displays, windows, and applications
/// without blocking. **Executor-agnostic** - works with any async runtime.
#[derive(Debug, Clone, Copy)]
pub struct AsyncSCShareableContent;

impl AsyncSCShareableContent {
    /// Asynchronously get the shareable content (displays, windows, applications)
    ///
    /// Uses callback-based Swift FFI for true async operation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Screen recording permission is not granted
    /// - The system fails to retrieve shareable content
    pub fn get() -> AsyncShareableContentFuture {
        Self::create().get()
    }

    /// Create options builder for customizing shareable content retrieval
    #[must_use]
    pub fn create() -> AsyncSCShareableContentOptions {
        AsyncSCShareableContentOptions::default()
    }
}

/// Options for async shareable content retrieval
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AsyncSCShareableContentOptions {
    exclude_desktop_windows: bool,
    on_screen_windows_only: bool,
}

impl AsyncSCShareableContentOptions {
    /// Exclude desktop windows from the shareable content
    #[must_use]
    pub fn with_exclude_desktop_windows(mut self, exclude: bool) -> Self {
        self.exclude_desktop_windows = exclude;
        self
    }

    /// Include only on-screen windows in the shareable content
    #[must_use]
    pub fn with_on_screen_windows_only(mut self, on_screen_only: bool) -> Self {
        self.on_screen_windows_only = on_screen_only;
        self
    }

    /// Asynchronously get the shareable content with these options
    pub fn get(self) -> AsyncShareableContentFuture {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_shareable_content_get_with_options(
                self.exclude_desktop_windows,
                self.on_screen_windows_only,
                shareable_content_callback,
                context,
            );
        }

        AsyncShareableContentFuture { inner: future }
    }

    /// Asynchronously get shareable content with only windows below a reference window
    ///
    /// This returns windows that are stacked below the specified reference window
    /// in the window layering order.
    ///
    /// # Arguments
    ///
    /// * `reference_window` - The window to use as the reference point
    pub fn below_window(
        self,
        reference_window: &crate::shareable_content::SCWindow,
    ) -> AsyncShareableContentFuture {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_shareable_content_get_below_window(
                self.exclude_desktop_windows,
                reference_window.as_ptr(),
                shareable_content_callback,
                context,
            );
        }

        AsyncShareableContentFuture { inner: future }
    }

    /// Asynchronously get shareable content with only windows above a reference window
    ///
    /// This returns windows that are stacked above the specified reference window
    /// in the window layering order.
    ///
    /// # Arguments
    ///
    /// * `reference_window` - The window to use as the reference point
    pub fn above_window(
        self,
        reference_window: &crate::shareable_content::SCWindow,
    ) -> AsyncShareableContentFuture {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_shareable_content_get_above_window(
                self.exclude_desktop_windows,
                reference_window.as_ptr(),
                shareable_content_callback,
                context,
            );
        }

        AsyncShareableContentFuture { inner: future }
    }
}

impl AsyncSCShareableContent {
    /// Asynchronously get shareable content for the current process only (macOS 14.4+)
    ///
    /// This retrieves content that the current process can capture without
    /// requiring user authorization via TCC (Transparency, Consent, and Control).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use screencapturekit::async_api::AsyncSCShareableContent;
    ///
    /// // Get content capturable without TCC authorization
    /// let content = AsyncSCShareableContent::current_process().await?;
    /// println!("Found {} windows for current process", content.windows().len());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "macos_14_4")]
    pub fn current_process() -> AsyncShareableContentFuture {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_shareable_content_get_current_process_displays(
                shareable_content_callback,
                context,
            );
        }

        AsyncShareableContentFuture { inner: future }
    }
}

// ============================================================================
// AsyncSCStream - Async stream with integrated frame iteration
// ============================================================================

/// Async iterator over sample buffers
struct AsyncSampleIteratorState {
    buffer: std::collections::VecDeque<crate::cm::CMSampleBuffer>,
    waker: Option<Waker>,
    closed: bool,
    capacity: usize,
}

/// Internal sender for async sample iterator
struct AsyncSampleSender {
    inner: Arc<Mutex<AsyncSampleIteratorState>>,
}

impl crate::stream::output_trait::SCStreamOutputTrait for AsyncSampleSender {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: crate::cm::CMSampleBuffer,
        _of_type: crate::stream::output_type::SCStreamOutputType,
    ) {
        let Ok(mut state) = self.inner.lock() else {
            return;
        };

        // Drop oldest if at capacity
        if state.buffer.len() >= state.capacity {
            state.buffer.pop_front();
        }

        state.buffer.push_back(sample_buffer);

        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

impl Drop for AsyncSampleSender {
    fn drop(&mut self) {
        if let Ok(mut state) = self.inner.lock() {
            state.closed = true;
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }
}

/// Future for getting the next sample buffer
pub struct NextSample<'a> {
    state: &'a Arc<Mutex<AsyncSampleIteratorState>>,
}

impl std::fmt::Debug for NextSample<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NextSample").finish_non_exhaustive()
    }
}

impl Future for NextSample<'_> {
    type Output = Option<crate::cm::CMSampleBuffer>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Ok(mut state) = self.state.lock() else {
            return Poll::Ready(None);
        };

        if let Some(sample) = state.buffer.pop_front() {
            return Poll::Ready(Some(sample));
        }

        if state.closed {
            Poll::Ready(None)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

unsafe impl Send for AsyncSampleSender {}
unsafe impl Sync for AsyncSampleSender {}

/// Async wrapper for `SCStream` with integrated frame iteration
///
/// Provides async methods for stream lifecycle and frame iteration.
/// **Executor-agnostic** - works with any async runtime.
///
/// # Examples
///
/// ```rust,no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCStream};
/// use screencapturekit::stream::configuration::SCStreamConfiguration;
/// use screencapturekit::stream::content_filter::SCContentFilter;
/// use screencapturekit::stream::output_type::SCStreamOutputType;
///
/// let content = AsyncSCShareableContent::get().await?;
/// let display = &content.displays()[0];
/// let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
/// let config = SCStreamConfiguration::new()
///     .with_width(1920)
///     .with_height(1080);
///
/// let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
/// stream.start_capture()?;
///
/// // Process frames asynchronously
/// while let Some(frame) = stream.next().await {
///     println!("Got frame!");
/// }
/// # Ok(())
/// # }
/// ```
pub struct AsyncSCStream {
    stream: crate::stream::SCStream,
    iterator_state: Arc<Mutex<AsyncSampleIteratorState>>,
}

impl AsyncSCStream {
    /// Create a new async stream
    ///
    /// # Arguments
    ///
    /// * `filter` - Content filter specifying what to capture
    /// * `config` - Stream configuration
    /// * `buffer_capacity` - Max frames to buffer (oldest dropped when full)
    /// * `output_type` - Type of output (Screen, Audio, Microphone)
    #[must_use]
    pub fn new(
        filter: &SCContentFilter,
        config: &SCStreamConfiguration,
        buffer_capacity: usize,
        output_type: crate::stream::output_type::SCStreamOutputType,
    ) -> Self {
        let state = Arc::new(Mutex::new(AsyncSampleIteratorState {
            buffer: std::collections::VecDeque::with_capacity(buffer_capacity),
            waker: None,
            closed: false,
            capacity: buffer_capacity,
        }));

        let sender = AsyncSampleSender {
            inner: Arc::clone(&state),
        };

        let mut stream = crate::stream::SCStream::new(filter, config);
        stream.add_output_handler(sender, output_type);

        Self {
            stream,
            iterator_state: state,
        }
    }

    /// Get the next sample buffer asynchronously
    ///
    /// Returns `None` when the stream is closed.
    pub fn next(&self) -> NextSample<'_> {
        NextSample {
            state: &self.iterator_state,
        }
    }

    /// Try to get a sample without waiting
    #[must_use]
    pub fn try_next(&self) -> Option<crate::cm::CMSampleBuffer> {
        self.iterator_state.lock().ok()?.buffer.pop_front()
    }

    /// Check if the stream has been closed
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.iterator_state.lock().map(|s| s.closed).unwrap_or(true)
    }

    /// Get the number of buffered samples
    #[must_use]
    pub fn buffered_count(&self) -> usize {
        self.iterator_state
            .lock()
            .map(|s| s.buffer.len())
            .unwrap_or(0)
    }

    /// Clear all buffered samples
    pub fn clear_buffer(&self) {
        if let Ok(mut state) = self.iterator_state.lock() {
            state.buffer.clear();
        }
    }

    /// Start capture (synchronous - returns immediately)
    ///
    /// # Errors
    ///
    /// Returns an error if capture fails to start.
    pub fn start_capture(&self) -> Result<(), SCError> {
        self.stream.start_capture()
    }

    /// Stop capture (synchronous - returns immediately)
    ///
    /// # Errors
    ///
    /// Returns an error if capture fails to stop.
    pub fn stop_capture(&self) -> Result<(), SCError> {
        self.stream.stop_capture()
    }

    /// Update stream configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub fn update_configuration(&self, config: &SCStreamConfiguration) -> Result<(), SCError> {
        self.stream.update_configuration(config)
    }

    /// Update content filter
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub fn update_content_filter(&self, filter: &SCContentFilter) -> Result<(), SCError> {
        self.stream.update_content_filter(filter)
    }

    /// Get a reference to the underlying stream
    #[must_use]
    pub fn inner(&self) -> &crate::stream::SCStream {
        &self.stream
    }
}

impl std::fmt::Debug for AsyncSCStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncSCStream")
            .field("stream", &self.stream)
            .field("buffered_count", &self.buffered_count())
            .field("is_closed", &self.is_closed())
            .finish_non_exhaustive()
    }
}

// ============================================================================
// AsyncSCScreenshotManager - Async screenshot capture (macOS 14.0+)
// ============================================================================

/// Async wrapper for `SCScreenshotManager`
///
/// Provides async methods for single-frame screenshot capture.
/// **Executor-agnostic** - works with any async runtime.
///
/// Requires the `macos_14_0` feature flag.
///
/// # Examples
///
/// ```rust,no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCScreenshotManager};
/// use screencapturekit::stream::configuration::SCStreamConfiguration;
/// use screencapturekit::stream::content_filter::SCContentFilter;
///
/// let content = AsyncSCShareableContent::get().await?;
/// let display = &content.displays()[0];
/// let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
/// let config = SCStreamConfiguration::new()
///     .with_width(1920)
///     .with_height(1080);
///
/// let image = AsyncSCScreenshotManager::capture_image(&filter, &config).await?;
/// println!("Screenshot: {}x{}", image.width(), image.height());
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "macos_14_0")]
#[derive(Debug, Clone, Copy)]
pub struct AsyncSCScreenshotManager;

/// Callback for async `CGImage` capture
#[cfg(feature = "macos_14_0")]
extern "C" fn screenshot_image_callback(
    image_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    if !error_ptr.is_null() {
        let error = unsafe { error_from_cstr(error_ptr) };
        unsafe {
            AsyncCompletion::<crate::screenshot_manager::CGImage>::complete_err(user_data, error);
        }
    } else if !image_ptr.is_null() {
        let image = crate::screenshot_manager::CGImage::from_ptr(image_ptr);
        unsafe { AsyncCompletion::complete_ok(user_data, image) };
    } else {
        unsafe {
            AsyncCompletion::<crate::screenshot_manager::CGImage>::complete_err(
                user_data,
                "Unknown error".to_string(),
            );
        };
    }
}

/// Callback for async `CMSampleBuffer` capture
#[cfg(feature = "macos_14_0")]
extern "C" fn screenshot_buffer_callback(
    buffer_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    if !error_ptr.is_null() {
        let error = unsafe { error_from_cstr(error_ptr) };
        unsafe { AsyncCompletion::<crate::cm::CMSampleBuffer>::complete_err(user_data, error) };
    } else if !buffer_ptr.is_null() {
        let buffer = unsafe { crate::cm::CMSampleBuffer::from_ptr(buffer_ptr.cast_mut()) };
        unsafe { AsyncCompletion::complete_ok(user_data, buffer) };
    } else {
        unsafe {
            AsyncCompletion::<crate::cm::CMSampleBuffer>::complete_err(
                user_data,
                "Unknown error".to_string(),
            );
        };
    }
}

/// Future for async screenshot capture
#[cfg(feature = "macos_14_0")]
pub struct AsyncScreenshotFuture<T> {
    inner: AsyncCompletionFuture<T>,
}

#[cfg(feature = "macos_14_0")]
impl<T> std::fmt::Debug for AsyncScreenshotFuture<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncScreenshotFuture")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "macos_14_0")]
impl<T> Future for AsyncScreenshotFuture<T> {
    type Output = Result<T, SCError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(SCError::ScreenshotError))
    }
}

#[cfg(feature = "macos_14_0")]
impl AsyncSCScreenshotManager {
    /// Capture a single screenshot as a `CGImage` asynchronously
    ///
    /// # Errors
    /// Returns an error if:
    /// - Screen recording permission is not granted
    /// - The capture fails for any reason
    pub fn capture_image(
        content_filter: &crate::stream::content_filter::SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> AsyncScreenshotFuture<crate::screenshot_manager::CGImage> {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_image(
                content_filter.as_ptr(),
                configuration.as_ptr(),
                screenshot_image_callback,
                context,
            );
        }

        AsyncScreenshotFuture { inner: future }
    }

    /// Capture a single screenshot as a `CMSampleBuffer` asynchronously
    ///
    /// # Errors
    /// Returns an error if:
    /// - Screen recording permission is not granted
    /// - The capture fails for any reason
    pub fn capture_sample_buffer(
        content_filter: &crate::stream::content_filter::SCContentFilter,
        configuration: &SCStreamConfiguration,
    ) -> AsyncScreenshotFuture<crate::cm::CMSampleBuffer> {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_sample_buffer(
                content_filter.as_ptr(),
                configuration.as_ptr(),
                screenshot_buffer_callback,
                context,
            );
        }

        AsyncScreenshotFuture { inner: future }
    }

    /// Capture a screenshot of a specific screen region asynchronously (macOS 15.2+)
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
    #[cfg(feature = "macos_15_2")]
    pub fn capture_image_in_rect(
        rect: crate::cg::CGRect,
    ) -> AsyncScreenshotFuture<crate::screenshot_manager::CGImage> {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_image_in_rect(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                screenshot_image_callback,
                context,
            );
        }

        AsyncScreenshotFuture { inner: future }
    }

    /// Capture a screenshot with advanced configuration asynchronously (macOS 26.0+)
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
    #[cfg(feature = "macos_26_0")]
    pub fn capture_screenshot(
        content_filter: &crate::stream::content_filter::SCContentFilter,
        configuration: &crate::screenshot_manager::SCScreenshotConfiguration,
    ) -> AsyncScreenshotFuture<crate::screenshot_manager::SCScreenshotOutput> {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_screenshot_manager_capture_screenshot(
                content_filter.as_ptr(),
                configuration.as_ptr(),
                screenshot_output_callback,
                context,
            );
        }

        AsyncScreenshotFuture { inner: future }
    }

    /// Capture a screenshot of a specific region with advanced configuration asynchronously (macOS 26.0+)
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
        configuration: &crate::screenshot_manager::SCScreenshotConfiguration,
    ) -> AsyncScreenshotFuture<crate::screenshot_manager::SCScreenshotOutput> {
        let (future, context) = AsyncCompletion::create();

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

        AsyncScreenshotFuture { inner: future }
    }
}

/// Callback for async `SCScreenshotOutput` capture (macOS 26.0+)
#[cfg(feature = "macos_26_0")]
extern "C" fn screenshot_output_callback(
    output_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    if !error_ptr.is_null() {
        let error = unsafe { error_from_cstr(error_ptr) };
        unsafe {
            AsyncCompletion::<crate::screenshot_manager::SCScreenshotOutput>::complete_err(
                user_data, error,
            );
        }
    } else if !output_ptr.is_null() {
        let output = crate::screenshot_manager::SCScreenshotOutput::from_ptr(output_ptr);
        unsafe { AsyncCompletion::complete_ok(user_data, output) };
    } else {
        unsafe {
            AsyncCompletion::<crate::screenshot_manager::SCScreenshotOutput>::complete_err(
                user_data,
                "Unknown error".to_string(),
            );
        };
    }
}

// ============================================================================
// AsyncSCContentSharingPicker - Async content sharing picker (macOS 14.0+)
// ============================================================================

/// Result from the async picker callback
#[cfg(feature = "macos_14_0")]
struct AsyncPickerCallbackResult {
    code: i32,
    ptr: *const c_void,
}

#[cfg(feature = "macos_14_0")]
unsafe impl Send for AsyncPickerCallbackResult {}

/// Callback for async picker
#[cfg(feature = "macos_14_0")]
extern "C" fn async_picker_callback(result_code: i32, ptr: *const c_void, user_data: *mut c_void) {
    let result = AsyncPickerCallbackResult {
        code: result_code,
        ptr,
    };
    unsafe { AsyncCompletion::complete_ok(user_data, result) };
}

/// Future for async picker with full result
#[cfg(feature = "macos_14_0")]
pub struct AsyncPickerFuture {
    inner: AsyncCompletionFuture<AsyncPickerCallbackResult>,
}

#[cfg(feature = "macos_14_0")]
impl std::fmt::Debug for AsyncPickerFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncPickerFuture").finish_non_exhaustive()
    }
}

#[cfg(feature = "macos_14_0")]
impl Future for AsyncPickerFuture {
    type Output = crate::content_sharing_picker::SCPickerOutcome;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use crate::content_sharing_picker::{SCPickerOutcome, SCPickerResult};

        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(result)) => {
                let outcome = match result.code {
                    1 if !result.ptr.is_null() => {
                        SCPickerOutcome::Picked(SCPickerResult::from_ptr(result.ptr))
                    }
                    0 => SCPickerOutcome::Cancelled,
                    _ => SCPickerOutcome::Error("Picker failed".to_string()),
                };
                Poll::Ready(outcome)
            }
            Poll::Ready(Err(e)) => Poll::Ready(SCPickerOutcome::Error(e)),
        }
    }
}

/// Future for async picker returning filter only
#[cfg(feature = "macos_14_0")]
pub struct AsyncPickerFilterFuture {
    inner: AsyncCompletionFuture<AsyncPickerCallbackResult>,
}

#[cfg(feature = "macos_14_0")]
impl std::fmt::Debug for AsyncPickerFilterFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncPickerFilterFuture")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "macos_14_0")]
impl Future for AsyncPickerFilterFuture {
    type Output = crate::content_sharing_picker::SCPickerFilterOutcome;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use crate::content_sharing_picker::SCPickerFilterOutcome;

        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(result)) => {
                let outcome = match result.code {
                    1 if !result.ptr.is_null() => {
                        SCPickerFilterOutcome::Filter(SCContentFilter::from_picker_ptr(result.ptr))
                    }
                    0 => SCPickerFilterOutcome::Cancelled,
                    _ => SCPickerFilterOutcome::Error("Picker failed".to_string()),
                };
                Poll::Ready(outcome)
            }
            Poll::Ready(Err(e)) => Poll::Ready(SCPickerFilterOutcome::Error(e)),
        }
    }
}

/// Async wrapper for `SCContentSharingPicker` (macOS 14.0+)
///
/// Provides async methods to show the system content sharing picker UI.
/// **Executor-agnostic** - works with any async runtime.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::async_api::AsyncSCContentSharingPicker;
/// use screencapturekit::content_sharing_picker::*;
///
/// async fn pick_content() {
///     let config = SCContentSharingPickerConfiguration::new();
///     match AsyncSCContentSharingPicker::show(&config).await {
///         SCPickerOutcome::Picked(result) => {
///             let (width, height) = result.pixel_size();
///             let filter = result.filter();
///             println!("Selected content: {}x{}", width, height);
///         }
///         SCPickerOutcome::Cancelled => println!("User cancelled"),
///         SCPickerOutcome::Error(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
#[cfg(feature = "macos_14_0")]
#[derive(Debug, Clone, Copy)]
pub struct AsyncSCContentSharingPicker;

#[cfg(feature = "macos_14_0")]
impl AsyncSCContentSharingPicker {
    /// Show the picker UI asynchronously and return `SCPickerResult` with filter and metadata
    ///
    /// This is the main API - use when you need content dimensions or want to build custom filters.
    /// The picker UI will be shown on the main thread, and the future will resolve when the user
    /// makes a selection or cancels.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::async_api::AsyncSCContentSharingPicker;
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// async fn example() {
    ///     let config = SCContentSharingPickerConfiguration::new();
    ///     if let SCPickerOutcome::Picked(result) = AsyncSCContentSharingPicker::show(&config).await {
    ///         let (width, height) = result.pixel_size();
    ///         let filter = result.filter();
    ///     }
    /// }
    /// ```
    pub fn show(
        config: &crate::content_sharing_picker::SCContentSharingPickerConfiguration,
    ) -> AsyncPickerFuture {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_with_result(
                config.as_ptr(),
                async_picker_callback,
                context,
            );
        }

        AsyncPickerFuture { inner: future }
    }

    /// Show the picker UI asynchronously and return an `SCContentFilter` directly
    ///
    /// This is the simple API - use when you just need the filter without metadata.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::async_api::AsyncSCContentSharingPicker;
    /// use screencapturekit::content_sharing_picker::*;
    ///
    /// async fn example() {
    ///     let config = SCContentSharingPickerConfiguration::new();
    ///     if let SCPickerFilterOutcome::Filter(filter) = AsyncSCContentSharingPicker::show_filter(&config).await {
    ///         // Use filter with SCStream
    ///     }
    /// }
    /// ```
    pub fn show_filter(
        config: &crate::content_sharing_picker::SCContentSharingPickerConfiguration,
    ) -> AsyncPickerFilterFuture {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show(
                config.as_ptr(),
                async_picker_callback,
                context,
            );
        }

        AsyncPickerFilterFuture { inner: future }
    }

    /// Show the picker UI for an existing stream to change source while capturing
    ///
    /// Use this when you have an active `SCStream` and want to let the user
    /// select a new content source. The result can be used with `stream.update_content_filter()`.
    ///
    /// # Example
    /// ```no_run
    /// use screencapturekit::async_api::AsyncSCContentSharingPicker;
    /// use screencapturekit::content_sharing_picker::*;
    /// use screencapturekit::stream::SCStream;
    /// use screencapturekit::stream::configuration::SCStreamConfiguration;
    /// use screencapturekit::stream::content_filter::SCContentFilter;
    /// use screencapturekit::shareable_content::SCShareableContent;
    ///
    /// async fn example() -> Option<()> {
    ///     let content = SCShareableContent::get().ok()?;
    ///     let displays = content.displays();
    ///     let display = displays.first()?;
    ///     let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    ///     let stream_config = SCStreamConfiguration::new();
    ///     let stream = SCStream::new(&filter, &stream_config);
    ///
    ///     // When stream is active and user wants to change source
    ///     let config = SCContentSharingPickerConfiguration::new();
    ///     if let SCPickerOutcome::Picked(result) = AsyncSCContentSharingPicker::show_for_stream(&config, &stream).await {
    ///         // Use result.filter() with stream.update_content_filter()
    ///         let _ = result.filter();
    ///     }
    ///     Some(())
    /// }
    /// ```
    pub fn show_for_stream(
        config: &crate::content_sharing_picker::SCContentSharingPickerConfiguration,
        stream: &crate::stream::SCStream,
    ) -> AsyncPickerFuture {
        let (future, context) = AsyncCompletion::create();

        unsafe {
            crate::ffi::sc_content_sharing_picker_show_for_stream(
                config.as_ptr(),
                stream.as_ptr(),
                async_picker_callback,
                context,
            );
        }

        AsyncPickerFuture { inner: future }
    }
}

// ============================================================================
// AsyncSCRecordingOutput - Async recording with event stream (macOS 15.0+)
// ============================================================================

/// Recording lifecycle event
#[cfg(feature = "macos_15_0")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingEvent {
    /// Recording started successfully
    Started,
    /// Recording finished successfully
    Finished,
    /// Recording failed with an error
    Failed(String),
}

#[cfg(feature = "macos_15_0")]
struct AsyncRecordingState {
    events: std::collections::VecDeque<RecordingEvent>,
    waker: Option<Waker>,
    finished: bool,
}

#[cfg(feature = "macos_15_0")]
struct AsyncRecordingDelegate {
    state: Arc<Mutex<AsyncRecordingState>>,
}

#[cfg(feature = "macos_15_0")]
impl crate::recording_output::SCRecordingOutputDelegate for AsyncRecordingDelegate {
    fn recording_did_start(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.events.push_back(RecordingEvent::Started);
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }

    fn recording_did_fail(&self, error: String) {
        if let Ok(mut state) = self.state.lock() {
            state.events.push_back(RecordingEvent::Failed(error));
            state.finished = true;
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }

    fn recording_did_finish(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.events.push_back(RecordingEvent::Finished);
            state.finished = true;
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }
}

/// Future for getting the next recording event
#[cfg(feature = "macos_15_0")]
pub struct NextRecordingEvent<'a> {
    state: &'a Arc<Mutex<AsyncRecordingState>>,
}

#[cfg(feature = "macos_15_0")]
impl std::fmt::Debug for NextRecordingEvent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NextRecordingEvent").finish_non_exhaustive()
    }
}

#[cfg(feature = "macos_15_0")]
impl Future for NextRecordingEvent<'_> {
    type Output = Option<RecordingEvent>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Ok(mut state) = self.state.lock() else {
            return Poll::Ready(None);
        };

        if let Some(event) = state.events.pop_front() {
            return Poll::Ready(Some(event));
        }

        if state.finished {
            Poll::Ready(None)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Async wrapper for `SCRecordingOutput` with event stream (macOS 15.0+)
///
/// Provides async iteration over recording lifecycle events.
/// **Executor-agnostic** - works with any async runtime.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::async_api::{AsyncSCShareableContent, AsyncSCRecordingOutput, RecordingEvent};
/// use screencapturekit::recording_output::SCRecordingOutputConfiguration;
/// use screencapturekit::stream::{SCStream, configuration::SCStreamConfiguration, content_filter::SCContentFilter};
/// use std::path::Path;
///
/// async fn record_screen() -> Option<()> {
///     let content = AsyncSCShareableContent::get().await.ok()?;
///     let displays = content.displays();
///     let display = displays.first()?;
///     let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
///     let config = SCStreamConfiguration::new().with_width(1920).with_height(1080);
///
///     let rec_config = SCRecordingOutputConfiguration::new()
///         .with_output_url(Path::new("/tmp/recording.mp4"));
///
///     let (recording, events) = AsyncSCRecordingOutput::new(&rec_config)?;
///
///     let mut stream = SCStream::new(&filter, &config);
///     stream.add_recording_output(&recording).ok()?;
///     stream.start_capture().ok()?;
///
///     // Wait for recording events
///     while let Some(event) = events.next().await {
///         match event {
///             RecordingEvent::Started => println!("Recording started!"),
///             RecordingEvent::Finished => {
///                 println!("Recording finished!");
///                 break;
///             }
///             RecordingEvent::Failed(e) => {
///                 eprintln!("Recording failed: {}", e);
///                 break;
///             }
///         }
///     }
///
///     Some(())
/// }
/// ```
#[cfg(feature = "macos_15_0")]
pub struct AsyncSCRecordingOutput {
    state: Arc<Mutex<AsyncRecordingState>>,
}

#[cfg(feature = "macos_15_0")]
impl std::fmt::Debug for AsyncSCRecordingOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncSCRecordingOutput")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "macos_15_0")]
impl AsyncSCRecordingOutput {
    /// Create a new async recording output
    ///
    /// Returns a tuple of (`SCRecordingOutput`, `AsyncSCRecordingOutput`).
    /// The `SCRecordingOutput` should be added to an `SCStream`, while
    /// the `AsyncSCRecordingOutput` provides async event iteration.
    ///
    /// # Errors
    ///
    /// Returns `None` if the recording output cannot be created (requires macOS 15.0+).
    #[must_use]
    pub fn new(
        config: &crate::recording_output::SCRecordingOutputConfiguration,
    ) -> Option<(crate::recording_output::SCRecordingOutput, Self)> {
        let state = Arc::new(Mutex::new(AsyncRecordingState {
            events: std::collections::VecDeque::new(),
            waker: None,
            finished: false,
        }));

        let delegate = AsyncRecordingDelegate {
            state: Arc::clone(&state),
        };

        let recording =
            crate::recording_output::SCRecordingOutput::new_with_delegate(config, delegate)?;

        Some((recording, Self { state }))
    }

    /// Get the next recording event asynchronously
    ///
    /// Returns `None` when the recording has finished or failed.
    pub fn next(&self) -> NextRecordingEvent<'_> {
        NextRecordingEvent { state: &self.state }
    }

    /// Check if the recording has finished
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.state.lock().map(|s| s.finished).unwrap_or(true)
    }

    /// Get any pending events without waiting
    #[must_use]
    pub fn try_next(&self) -> Option<RecordingEvent> {
        self.state.lock().ok()?.events.pop_front()
    }
}
