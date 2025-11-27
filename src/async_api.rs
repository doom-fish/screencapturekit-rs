//! Async API for `ScreenCaptureKit`
//!
//! This module provides async versions of operations when the `async` feature is enabled.
//! The async API is **executor-agnostic** and works with any async runtime (Tokio, async-std, smol, etc.).
//!
//! # Runtime Agnostic Design
//!
//! This async API uses only `std` types and works with **any** async runtime:
//! - Uses callback-based Swift FFI for true async operations
//! - Uses `std::sync::{Arc, Mutex}` for synchronization
//! - Uses `std::task::{Poll, Waker}` for async primitives
//! - Uses `std::future::Future` trait
//!
//! # Examples
//!
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

use crate::error::SCError;
use crate::shareable_content::SCShareableContent;
use crate::stream::configuration::SCStreamConfiguration;
use crate::stream::content_filter::SCContentFilter;
use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

// ============================================================================
// AsyncSCShareableContent - True async with callback-based FFI
// ============================================================================

/// Shared state for async shareable content retrieval
struct ShareableContentState {
    result: Option<Result<SCShareableContent, SCError>>,
    waker: Option<Waker>,
}

/// Future for async shareable content retrieval
pub struct AsyncShareableContentFuture {
    state: Arc<Mutex<ShareableContentState>>,
}

impl Future for AsyncShareableContentFuture {
    type Output = Result<SCShareableContent, SCError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        
        state.result.take().map_or_else(|| {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }, Poll::Ready)
    }
}

/// Callback from Swift FFI for shareable content
extern "C" fn shareable_content_callback(content: *const c_void, error: *const i8, user_data: *mut c_void) {
    let state = unsafe { Arc::from_raw(user_data.cast::<Mutex<ShareableContentState>>()) };
    
    let result = if !error.is_null() {
        let error_msg = unsafe {
            std::ffi::CStr::from_ptr(error)
                .to_string_lossy()
                .into_owned()
        };
        Err(crate::utils::error::create_sc_error(&error_msg))
    } else if !content.is_null() {
        Ok(unsafe { SCShareableContent::from_ptr(content) })
    } else {
        Err(crate::utils::error::create_sc_error("Unknown error"))
    };
    
    {
        let mut guard = state.lock().unwrap();
        guard.result = Some(result);
        if let Some(waker) = guard.waker.take() {
            waker.wake();
        }
    }
    
    // Keep state alive - the Arc will be dropped when the future is dropped
    std::mem::forget(state);
}

/// Async wrapper for `SCShareableContent`
///
/// Provides async methods to retrieve displays, windows, and applications
/// without blocking. **Executor-agnostic** - works with any async runtime.
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
        Self::with_options().get_async()
    }

    /// Create options builder for customizing shareable content retrieval
    #[must_use]
    pub fn with_options() -> AsyncSCShareableContentOptions {
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
    pub fn exclude_desktop_windows(mut self, exclude: bool) -> Self {
        self.exclude_desktop_windows = exclude;
        self
    }

    /// Include only on-screen windows in the shareable content
    #[must_use]
    pub fn on_screen_windows_only(mut self, on_screen_only: bool) -> Self {
        self.on_screen_windows_only = on_screen_only;
        self
    }

    /// Asynchronously get the shareable content with these options
    pub fn get_async(self) -> AsyncShareableContentFuture {
        let state = Arc::new(Mutex::new(ShareableContentState {
            result: None,
            waker: None,
        }));
        
        let state_ptr = Arc::into_raw(state.clone()).cast_mut().cast::<c_void>();
        
        unsafe {
            crate::ffi::sc_shareable_content_get_async(
                self.exclude_desktop_windows,
                self.on_screen_windows_only,
                shareable_content_callback,
                state_ptr,
            );
        }
        
        AsyncShareableContentFuture { state }
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
/// let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
/// let config = SCStreamConfiguration::builder()
///     .width(1920)
///     .height(1080)
///     .build();
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

        Self { stream, iterator_state: state }
    }

    /// Get the next sample buffer asynchronously
    ///
    /// Returns `None` when the stream is closed.
    pub fn next(&self) -> NextSample<'_> {
        NextSample { state: &self.iterator_state }
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
        self.iterator_state.lock().map(|s| s.buffer.len()).unwrap_or(0)
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
