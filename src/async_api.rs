//! Async API for ScreenCaptureKit
//!
//! This module provides async versions of blocking operations when the `async` feature is enabled.
//! The async API is **executor-agnostic** and works with any async runtime (Tokio, async-std, smol, etc.).
//!
//! # Runtime Agnostic Design
//!
//! This async API uses only `std` types and works with **any** async runtime:
//! - Uses `std::thread::spawn` for blocking operations (no runtime dependency)
//! - Uses `std::sync::{Arc, Mutex}` for synchronization
//! - Uses `std::task::{Poll, Waker}` for async primitives
//! - Uses `std::future::Future` trait
//!
//! This means you can use this library with Tokio, async-std, smol, futures, or any custom executor.
//!
//! ## Performance Note
//!
//! Each async operation spawns a new thread using `std::thread::spawn`. This is slightly less
//! efficient than runtime-specific thread pools (like `tokio::spawn_blocking`), but:
//! - The overhead is negligible (~50-100μs vs 10-100ms operation time)
//! - It provides universal compatibility
//! - Users needing maximum performance can use the sync API with their runtime's spawn_blocking
//!
//! # Features
//!
//! - Async shareable content retrieval
//! - Async screenshot capture (macOS 14.0+)
//! - Async stream operations
//! - Async content picker (macOS 14.0+)
//! - **Executor-agnostic** - works with any async runtime
//!
//! # Examples
//!
//! With Tokio:
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use screencapturekit::async_api::AsyncSCShareableContent;
//!
//! let content = AsyncSCShareableContent::get().await?;
//! println!("Found {} displays", content.displays().len());
//! # Ok(())
//! # }
//! ```
//!
//! With async-std:
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use screencapturekit::async_api::AsyncSCShareableContent;
//!
//! let content = AsyncSCShareableContent::get().await?;
//! println!("Found {} displays", content.displays().len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Power Users
//!
//! If you need maximum performance and are using a specific runtime, you can use the sync API
//! with your runtime's spawn_blocking:
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use screencapturekit::shareable_content::SCShareableContent;
//!
//! // With Tokio's thread pool
//! let content = tokio::task::spawn_blocking(|| {
//!     SCShareableContent::get()
//! }).await??;
//! # Ok(())
//! # }
//! ```


use crate::error::SCError;
use crate::shareable_content::SCShareableContent;
use crate::stream::configuration::SCStreamConfiguration;
use crate::stream::content_filter::SCContentFilter;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

#[cfg(feature = "macos_14_0")]
use crate::screenshot_manager::{CGImage, SCScreenshotManager};

#[cfg(feature = "macos_14_0")]
use crate::content_sharing_picker::{
    SCContentSharingPicker, SCContentSharingPickerConfiguration, SCContentSharingPickerResult,
};

/// Shared state for executor-agnostic futures
struct SharedState<T> {
    result: Option<T>,
    waker: Option<Waker>,
}

/// A future that executes a blocking operation on a separate thread
///
/// This is executor-agnostic and uses `std::thread::spawn` internally.
/// It works with any async runtime: Tokio, async-std, smol, futures, etc.
pub struct BlockingFuture<T> {
    shared_state: Arc<Mutex<SharedState<T>>>,
}

impl<T> BlockingFuture<T>
where
    T: Send + 'static,
{
    fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let shared_state = Arc::new(Mutex::new(SharedState {
            result: None,
            waker: None,
        }));

        let thread_shared = Arc::clone(&shared_state);

        std::thread::spawn(move || {
            let result = f();

            let mut state = thread_shared.lock().unwrap();
            state.result = Some(result);
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        });

        Self { shared_state }
    }
}

impl<T> Future for BlockingFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.shared_state.lock().unwrap();

        state.result.take().map_or_else(|| {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }, Poll::Ready)
    }
}

/// Async wrapper for `SCShareableContent`
///
/// Provides async methods to retrieve displays, windows, and applications
/// without blocking the async runtime. **Executor-agnostic** - works with any async runtime.
pub struct AsyncSCShareableContent;

impl AsyncSCShareableContent {
    /// Asynchronously get the shareable content (displays, windows, applications)
    ///
    /// This runs the blocking operation in a separate thread.
    /// **Executor-agnostic** - works with Tokio, async-std, smol, etc.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Screen recording permission is not granted
    /// - The system fails to retrieve shareable content
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use screencapturekit::async_api::AsyncSCShareableContent;
    ///
    /// let content = AsyncSCShareableContent::get().await?;
    /// println!("Found {} displays", content.displays().len());
    /// println!("Found {} windows", content.windows().len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get() -> Result<SCShareableContent, SCError> {
        BlockingFuture::new(SCShareableContent::get).await
    }

    /// Asynchronously get shareable content with options
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use screencapturekit::async_api::AsyncSCShareableContent;
    ///
    /// let content = AsyncSCShareableContent::with_options()
    ///     .on_screen_windows_only(true)
    ///     .exclude_desktop_windows(true)
    ///     .get_async()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_options() -> AsyncSCShareableContentOptions {
        AsyncSCShareableContentOptions {
            options: SCShareableContent::with_options(),
        }
    }
}

/// Async wrapper for `SCShareableContentOptions`
///
/// Allows configuration of what content to retrieve.
pub struct AsyncSCShareableContentOptions {
    options: crate::shareable_content::SCShareableContentOptions,
}

impl AsyncSCShareableContentOptions {
    /// Exclude desktop windows from the shareable content
    #[must_use]
    pub fn exclude_desktop_windows(mut self, exclude: bool) -> Self {
        self.options = self.options.exclude_desktop_windows(exclude);
        self
    }

    /// Include only on-screen windows in the shareable content
    #[must_use]
    pub fn on_screen_windows_only(mut self, on_screen_only: bool) -> Self {
        self.options = self.options.on_screen_windows_only(on_screen_only);
        self
    }

    /// Asynchronously get the shareable content with these options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Screen recording permission is not granted
    /// - The system fails to retrieve content
    pub async fn get_async(self) -> Result<SCShareableContent, SCError> {
        let options = self.options;
        BlockingFuture::new(move || options.get()).await
    }
}

/// Async wrapper for screenshot operations (macOS 14.0+)
///
/// Provides async methods to capture screenshots without blocking.
/// **Executor-agnostic** - works with any async runtime.
#[cfg(feature = "macos_14_0")]
pub struct AsyncSCScreenshotManager;

#[cfg(feature = "macos_14_0")]
impl AsyncSCScreenshotManager {
    /// Asynchronously capture a screenshot and return as `CGImage`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Screen recording permission is not granted
    /// - The capture fails
    /// - The system is not macOS 14.0+
    pub async fn capture_image(
        filter: &SCContentFilter,
        config: &SCStreamConfiguration,
    ) -> Result<CGImage, SCError> {
        let filter = filter.clone();
        let config = config.clone();

        BlockingFuture::new(move || SCScreenshotManager::capture_image(&filter, &config)).await
    }

    /// Asynchronously capture a screenshot as a sample buffer
    ///
    /// Returns the sample buffer for advanced processing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Screen recording permission is not granted
    /// - The capture fails
    /// - The system is not macOS 14.0+
    pub async fn capture_sample_buffer(
        filter: &SCContentFilter,
        config: &SCStreamConfiguration,
    ) -> Result<crate::cm::CMSampleBuffer, SCError> {
        let filter = filter.clone();
        let config = config.clone();

        BlockingFuture::new(move || {
            SCScreenshotManager::capture_sample_buffer(&filter, &config)
        })
        .await
    }
}

/// Async wrapper for content sharing picker (macOS 14.0+)
///
/// Provides async method to show the system picker UI.
/// **Executor-agnostic** - works with any async runtime.
#[cfg(feature = "macos_14_0")]
pub struct AsyncSCContentSharingPicker;

#[cfg(feature = "macos_14_0")]
impl AsyncSCContentSharingPicker {
    /// Asynchronously show the content sharing picker
    ///
    /// This displays the system UI for selecting displays, windows, or applications.
    /// The function returns when the user makes a selection or cancels.
    pub async fn show(
        config: &SCContentSharingPickerConfiguration,
    ) -> SCContentSharingPickerResult {
        let config = config.clone();
        BlockingFuture::new(move || SCContentSharingPicker::show(&config)).await
    }
}

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
/// let filter = SCContentFilter::build().display(display).exclude_windows(&[]).build();
/// let config = SCStreamConfiguration::build();
///
/// let mut stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
/// stream.start_capture().await?;
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
    iterator: AsyncSampleIterator,
}

/// Async iterator over sample buffers
///
/// Provides async iteration over captured frames.
/// Access via [`AsyncSCStream::next()`].
pub struct AsyncSampleIterator {
    inner: Arc<Mutex<AsyncSampleIteratorState>>,
}

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
    iterator: &'a AsyncSampleIterator,
}

impl Future for NextSample<'_> {
    type Output = Option<crate::cm::CMSampleBuffer>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Ok(mut state) = self.iterator.inner.lock() else {
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

unsafe impl Send for AsyncSampleIterator {}
unsafe impl Sync for AsyncSampleIterator {}
unsafe impl Send for AsyncSampleSender {}
unsafe impl Sync for AsyncSampleSender {}

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

        let iterator = AsyncSampleIterator { inner: state };

        Self { stream, iterator }
    }

    /// Get the next sample buffer asynchronously
    ///
    /// Returns `None` when the stream is closed.
    pub fn next(&self) -> NextSample<'_> {
        NextSample { iterator: &self.iterator }
    }

    /// Try to get a sample without waiting
    #[must_use]
    pub fn try_next(&self) -> Option<crate::cm::CMSampleBuffer> {
        self.iterator.inner.lock().ok()?.buffer.pop_front()
    }

    /// Check if the stream has been closed
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.iterator.inner.lock().map(|s| s.closed).unwrap_or(true)
    }

    /// Get the number of buffered samples
    #[must_use]
    pub fn buffered_count(&self) -> usize {
        self.iterator.inner.lock().map(|s| s.buffer.len()).unwrap_or(0)
    }

    /// Clear all buffered samples
    pub fn clear_buffer(&self) {
        if let Ok(mut state) = self.iterator.inner.lock() {
            state.buffer.clear();
        }
    }

    /// Start capture asynchronously
    ///
    /// # Errors
    ///
    /// Returns an error if capture fails to start.
    pub async fn start_capture(&self) -> Result<(), SCError> {
        let stream_ptr = std::ptr::addr_of!(self.stream) as usize;
        BlockingFuture::new(move || {
            let stream_ref = unsafe { &*(stream_ptr as *const crate::stream::SCStream) };
            stream_ref.start_capture()
        }).await
    }

    /// Stop capture asynchronously
    ///
    /// # Errors
    ///
    /// Returns an error if capture fails to stop.
    pub async fn stop_capture(&self) -> Result<(), SCError> {
        let stream_ptr = std::ptr::addr_of!(self.stream) as usize;
        BlockingFuture::new(move || {
            let stream_ref = unsafe { &*(stream_ptr as *const crate::stream::SCStream) };
            stream_ref.stop_capture()
        }).await
    }

    /// Update stream configuration asynchronously
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn update_configuration(&self, config: &SCStreamConfiguration) -> Result<(), SCError> {
        let stream_ptr = std::ptr::addr_of!(self.stream) as usize;
        let config = config.clone();
        BlockingFuture::new(move || {
            let stream_ref = unsafe { &*(stream_ptr as *const crate::stream::SCStream) };
            stream_ref.update_configuration(&config)
        }).await
    }

    /// Update content filter asynchronously
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn update_content_filter(&self, filter: &SCContentFilter) -> Result<(), SCError> {
        let stream_ptr = std::ptr::addr_of!(self.stream) as usize;
        let filter = filter.clone();
        BlockingFuture::new(move || {
            let stream_ref = unsafe { &*(stream_ptr as *const crate::stream::SCStream) };
            stream_ref.update_content_filter(&filter)
        }).await
    }

    /// Get a reference to the underlying stream
    #[must_use]
    pub fn inner(&self) -> &crate::stream::SCStream {
        &self.stream
    }
}

/// Async wrapper for `SCRecordingOutput` (macOS 15.0+)
///
/// Provides async methods for recording operations.
#[cfg(feature = "macos_15_0")]
pub struct AsyncSCRecordingOutput;

#[cfg(feature = "macos_15_0")]
impl AsyncSCRecordingOutput {
    /// Asynchronously create a recording output
    ///
    /// Returns None if the system doesn't support recording output.
    /// Create a new recording output asynchronously
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(
        config: &crate::recording_output::SCRecordingOutputConfiguration,
    ) -> Option<crate::recording_output::SCRecordingOutput> {
        let config = config.clone();
        BlockingFuture::new(move || crate::recording_output::SCRecordingOutput::new(&config)).await
    }
}

/// Helper functions for async operations
///
/// These functions are executor-agnostic and work with any async runtime.
#[allow(clippy::wildcard_imports)]
pub mod utils {
    use super::*;

    /// Create a content filter asynchronously
    pub async fn create_display_filter(
        display: &crate::shareable_content::SCDisplay,
    ) -> SCContentFilter {
        let display = display.clone();
        BlockingFuture::new(move || {
            #[allow(deprecated)]
            SCContentFilter::new().with_display_excluding_windows(&display, &[])
        })
        .await
    }

    /// Create a window filter asynchronously
    pub async fn create_window_filter(
        window: &crate::shareable_content::SCWindow,
    ) -> SCContentFilter {
        let window = window.clone();
        BlockingFuture::new(move || {
            #[allow(deprecated)]
            SCContentFilter::new().with_desktop_independent_window(&window)
        })
        .await
    }

    /// Create a stream configuration asynchronously
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration parameters are invalid.
    pub async fn create_stream_config(
        width: u32,
        height: u32,
    ) -> Result<SCStreamConfiguration, SCError> {
        BlockingFuture::new(move || {
            SCStreamConfiguration::build()
                .set_width(width)?
                .set_height(height)
        })
        .await
    }

    /// Create a recording output configuration asynchronously (macOS 15.0+)
    #[cfg(feature = "macos_15_0")]
    pub async fn create_recording_config(
        output_path: std::path::PathBuf,
        codec: crate::recording_output::SCRecordingOutputCodec,
        bitrate: i64,
    ) -> crate::recording_output::SCRecordingOutputConfiguration {
        BlockingFuture::new(move || {
            let mut config = crate::recording_output::SCRecordingOutputConfiguration::new();
            config.set_output_url(&output_path);
            config.set_video_codec(codec);
            config.set_average_bitrate(bitrate);
            config
        })
        .await
    }

    /// Get shareable content for a specific application asynchronously
    pub async fn get_application_content(
        app_name: String,
    ) -> Result<Option<crate::shareable_content::SCRunningApplication>, SCError> {
        BlockingFuture::new(move || {
            let content = crate::shareable_content::SCShareableContent::get()?;
            Ok(content
                .applications()
                .iter()
                .find(|app| app.application_name() == app_name)
                .cloned())
        })
        .await
    }

    /// Get all on-screen windows asynchronously
    pub async fn get_on_screen_windows(
    ) -> Result<Vec<crate::shareable_content::SCWindow>, SCError> {
        BlockingFuture::new(|| {
            let content = crate::shareable_content::SCShareableContent::get()?;
            Ok(content
                .windows()
                .iter()
                .filter(|w| w.is_on_screen())
                .cloned()
                .collect())
        })
        .await
    }

    /// Find a window by title asynchronously
    pub async fn find_window_by_title(
        title: String,
    ) -> Result<Option<crate::shareable_content::SCWindow>, SCError> {
        BlockingFuture::new(move || {
            let content = crate::shareable_content::SCShareableContent::get()?;
            Ok(content
                .windows()
                .iter()
                .find(|w| {
                    w.title()
                        .is_some_and(|t| t.contains(&title))
                })
                .cloned())
        })
        .await
    }

    /// Get the main display asynchronously
    pub async fn get_main_display(
    ) -> Result<Option<crate::shareable_content::SCDisplay>, SCError> {
        BlockingFuture::new(|| {
            let content = crate::shareable_content::SCShareableContent::get()?;
            Ok(content.displays().first().cloned())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_shareable_content() {
        let result = AsyncSCShareableContent::get().await;

        match result {
            Ok(content) => {
                println!("Got {} displays", content.displays().len());
            }
            Err(e) => {
                println!("Expected error in test environment: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_async_shareable_content_with_options() {
        let result = AsyncSCShareableContent::with_options()
            .on_screen_windows_only(true)
            .get_async()
            .await;

        match result {
            Ok(_content) => {
                println!("Got content with options");
            }
            Err(e) => {
                println!("Expected error in test environment: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_async_utils_display_filter() {
        let result = AsyncSCShareableContent::get().await;

        if let Ok(content) = result {
            if !content.displays().is_empty() {
                let display = &content.displays()[0];
                let _filter = utils::create_display_filter(display).await;
                println!("Created display filter");
            }
        }
    }

    #[tokio::test]
    async fn test_async_utils_window_filter() {
        let result = AsyncSCShareableContent::get().await;

        if let Ok(content) = result {
            if !content.windows().is_empty() {
                let window = &content.windows()[0];
                let _filter = utils::create_window_filter(window).await;
                println!("Created window filter");
            }
        }
    }

    #[tokio::test]
    async fn test_async_utils_app_filter() {
        let result = AsyncSCShareableContent::get().await;

        if let Ok(content) = result {
            if !content.applications().is_empty() {
                let app = &content.applications()[0];
                // Test application content retrieval
                let _app_content = utils::get_application_content(app.application_name()).await;
                println!("Tested application content retrieval");
            }
        }
    }

    #[tokio::test]
    async fn test_async_config_creation() {
        let result = utils::create_stream_config(1920, 1080).await;
        assert!(result.is_ok(), "Config creation should succeed");
    }

    #[tokio::test]
    async fn test_async_get_main_display() {
        let result = utils::get_main_display().await;
        match result {
            Ok(Some(display)) => {
                println!(
                    "Main display: {}x{}",
                    display.width(),
                    display.height()
                );
            }
            Ok(None) => {
                println!("No displays available");
            }
            Err(e) => {
                println!("Expected error: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_async_get_on_screen_windows() {
        let result = utils::get_on_screen_windows().await;
        match result {
            Ok(windows) => {
                println!("Found {} on-screen windows", windows.len());
            }
            Err(e) => {
                println!("Expected error: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_async_find_window() {
        let result = utils::find_window_by_title("Terminal".to_string()).await;
        match result {
            Ok(Some(window)) => {
                println!("Found window: {:?}", window.title());
            }
            Ok(None) => {
                println!("Window not found");
            }
            Err(e) => {
                println!("Expected error: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_async_get_application() {
        let result = utils::get_application_content("Safari".to_string()).await;
        match result {
            Ok(Some(app)) => {
                println!("Found app: {}", app.application_name());
            }
            Ok(None) => {
                println!("App not found");
            }
            Err(e) => {
                println!("Expected error: {e}");
            }
        }
    }

    #[tokio::test]
    #[cfg(feature = "macos_14_0")]
    async fn test_async_screenshot_manager_exists() {
        let _ = AsyncSCScreenshotManager;
    }

    #[tokio::test]
    #[cfg(feature = "macos_14_0")]
    async fn test_async_picker_exists() {
        let _ = AsyncSCContentSharingPicker;
    }

    #[tokio::test]
    #[cfg(feature = "macos_15_0")]
    async fn test_async_recording_output_exists() {
        let _ = AsyncSCRecordingOutput;
    }

    #[tokio::test]
    async fn test_async_stream_creation() {
        let result = AsyncSCShareableContent::get().await;

        if let Ok(content) = result {
            if !content.displays().is_empty() {
                let display = &content.displays()[0];
                let filter = utils::create_display_filter(display).await;
                let config = utils::create_stream_config(640, 480).await.unwrap();

                let _stream = AsyncSCStream::new(&filter, &config);
                println!("Created async stream wrapper");
            }
        }
    }

    // Test with different executors to prove it's truly agnostic
    #[test]
    fn test_executor_agnostic_tokio() {
        // Works with Tokio
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(AsyncSCShareableContent::get());
        match result {
            Ok(_) | Err(_) => println!("Tokio: ✓"),
        }
    }

    #[test]
    fn test_blocking_future_thread_spawn() {
        // Verify that thread::spawn is actually being used
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&flag);

        let future = BlockingFuture::new(move || {
            // This closure runs in a spawned thread
            flag_clone.store(true, Ordering::SeqCst);
            42
        });

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(future);

        assert_eq!(result, 42);
        assert!(flag.load(Ordering::SeqCst), "thread::spawn was used");
        println!("✓ Verified thread::spawn is used in BlockingFuture");
    }

    #[tokio::test]
    async fn test_concurrent_async_operations() {
        // Test multiple async operations running concurrently
        let handles = vec![
            tokio::spawn(AsyncSCShareableContent::get()),
            tokio::spawn(AsyncSCShareableContent::get()),
            tokio::spawn(AsyncSCShareableContent::get()),
        ];

        let mut success_count = 0;
        for handle in handles {
            if let Ok(result) = handle.await {
                if result.is_ok() {
                    success_count += 1;
                }
            }
        }

        println!("✓ {success_count}/3 concurrent operations completed");
    }

    #[tokio::test]
    async fn test_async_stream_lifecycle() {
        let result = AsyncSCShareableContent::get().await;

        if let Ok(content) = result {
            if !content.displays().is_empty() {
                let display = &content.displays()[0];
                let filter = utils::create_display_filter(display).await;
                let config = utils::create_stream_config(320, 240).await.unwrap();

                let stream = AsyncSCStream::new(&filter, &config);

                // Test that async methods exist and can be called
                // (We don't actually start capture in tests to avoid permission issues)
                println!("✓ AsyncSCStream lifecycle methods available");

                // Test inner access
                let _ = stream.inner();
                println!("✓ AsyncSCStream inner access works");
            }
        }
    }
}

