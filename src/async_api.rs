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
//! stream.start_capture().await?;
//!
//! // Process frames asynchronously
//! for _ in 0..100 {
//!     if let Some(frame) = stream.next().await {
//!         println!("Got frame at {:?}", frame.presentation_timestamp());
//!     }
//! }
//!
//! stream.stop_capture().await?;
//! # Ok(())
//! # }
//! ```

use crate::error::SCError;
use crate::shareable_content::SCShareableContent;
use crate::stream::configuration::SCStreamConfiguration;
use crate::stream::content_filter::SCContentFilter;
use crate::stream::output_type::SCStreamOutputType;
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
    crate::utils::panic_safe::catch_user_panic("shareable_content_callback", move || {
        if !error.is_null() {
            // SAFETY: `error` is non-null (checked above) and points to a valid null-terminated C string provided by the Swift completion handler.
            let error_msg = unsafe { error_from_cstr(error) };
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe { AsyncCompletion::<SCShareableContent>::complete_err(user_data, error_msg) };
        } else if !content.is_null() {
            // SAFETY: `content` is non-null (checked above) and is a valid `SCShareableContent` pointer retained for us by the Swift completion handler.
            let sc = unsafe { SCShareableContent::from_ptr(content) };
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe { AsyncCompletion::complete_ok(user_data, sc) };
        } else {
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe {
                AsyncCompletion::<SCShareableContent>::complete_err(
                    user_data,
                    "Unknown error".to_string(),
                );
            };
        }
    });
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

        // SAFETY: `context` is a valid one-shot completion pointer created by `AsyncCompletion::create()`; the Swift layer invokes the callback exactly once, after which the pointer is consumed.
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

        // SAFETY: `context` is a valid one-shot completion pointer created by `AsyncCompletion::create()`; the Swift layer invokes the callback exactly once, after which the pointer is consumed.
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

        // SAFETY: `context` is a valid one-shot completion pointer created by `AsyncCompletion::create()`; the Swift layer invokes the callback exactly once, after which the pointer is consumed.
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

        // SAFETY: `context` is a valid one-shot completion pointer created by `AsyncCompletion::create()`; the Swift layer invokes the callback exactly once, after which the pointer is consumed.
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

/// Async iterator over sample buffers.
///
/// # Delivery semantics
///
/// This is a **lossy, bounded** buffer. When the queue is full (`capacity`
/// reached) and a new sample arrives faster than the consumer polls it,
/// the **oldest** buffered sample is dropped to make room for the newest
/// (drop-oldest policy). This keeps latency low and favors fresh frames over
/// stale ones, but means consumers that fall behind will miss intermediate
/// frames rather than blocking the capture callback.
struct AsyncSampleIteratorState {
    buffer: std::collections::VecDeque<(crate::cm::CMSampleBuffer, SCStreamOutputType)>,
    waker: Option<Waker>,
    closed: bool,
    capacity: usize,
    stop_error: Option<SCError>,
}

/// Internal sender for async sample iterator
struct AsyncSampleSender {
    inner: Arc<Mutex<AsyncSampleIteratorState>>,
}

impl crate::stream::output_trait::SCStreamOutputTrait for AsyncSampleSender {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: crate::cm::CMSampleBuffer,
        of_type: SCStreamOutputType,
    ) {
        let Ok(mut state) = self.inner.lock() else {
            return;
        };

        // Drop oldest if at capacity
        if state.buffer.len() >= state.capacity {
            state.buffer.pop_front();
        }

        state.buffer.push_back((sample_buffer, of_type));

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

/// Shared poll logic for the sample futures/streams: pop the next buffered
/// `(buffer, type)` pair, resolve to `None` when closed, or register the waker.
fn poll_next_sample(
    state: &Arc<Mutex<AsyncSampleIteratorState>>,
    cx: &Context<'_>,
) -> Poll<Option<(crate::cm::CMSampleBuffer, SCStreamOutputType)>> {
    let Ok(mut state) = state.lock() else {
        return Poll::Ready(None);
    };

    if let Some(sample) = state.buffer.pop_front() {
        return Poll::Ready(Some(sample));
    }

    if state.closed {
        Poll::Ready(None)
    } else {
        // Avoid the lost-wakeup race: when the same future/stream is polled by
        // a different task (e.g. moved between `tokio::select!` arms), the waker
        // changes. `will_wake` skips the clone when the executor reuses the same
        // waker; the explicit assignment guarantees the latest waker is the one
        // a future sample arrival will wake.
        let waker = cx.waker();
        match state.waker {
            Some(ref existing) if existing.will_wake(waker) => {}
            _ => state.waker = Some(waker.clone()),
        }
        Poll::Pending
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
        poll_next_sample(self.state, cx).map(|opt| opt.map(|(buffer, _of_type)| buffer))
    }
}

/// Future for getting the next sample buffer together with its output type.
///
/// Like [`NextSample`], but yields the [`SCStreamOutputType`] alongside the
/// buffer so consumers of a multi-output stream (e.g. screen + audio via
/// [`AsyncSCStream::add_output_type`]) can tell frames apart. Returned by
/// [`AsyncSCStream::next_typed`].
pub struct NextSampleTyped<'a> {
    state: &'a Arc<Mutex<AsyncSampleIteratorState>>,
}

impl std::fmt::Debug for NextSampleTyped<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NextSampleTyped").finish_non_exhaustive()
    }
}

impl Future for NextSampleTyped<'_> {
    type Output = Option<(crate::cm::CMSampleBuffer, SCStreamOutputType)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_next_sample(self.state, cx)
    }
}

/// A [`Stream`](futures_core::Stream) of captured sample buffers.
///
/// Yields `CMSampleBuffer`s and ends (`None`) when the stream closes. Returned
/// by [`AsyncSCStream::frames`]; it borrows the stream and is `Unpin`, so it
/// plugs straight into the `futures::StreamExt` combinators:
///
/// ```no_run
/// # async fn example(stream: screencapturekit::async_api::AsyncSCStream) {
/// use futures_util::StreamExt;
/// let first_ten: Vec<_> = stream.frames().take(10).collect().await;
/// # let _ = first_ten;
/// # }
/// ```
pub struct SampleStream<'a> {
    state: &'a Arc<Mutex<AsyncSampleIteratorState>>,
}

impl std::fmt::Debug for SampleStream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleStream").finish_non_exhaustive()
    }
}

impl futures_core::Stream for SampleStream<'_> {
    type Item = crate::cm::CMSampleBuffer;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        poll_next_sample(self.state, cx).map(|opt| opt.map(|(buffer, _of_type)| buffer))
    }
}

/// A [`Stream`](futures_core::Stream) of captured sample buffers tagged with
/// their [`SCStreamOutputType`].
///
/// Like [`SampleStream`] but yields `(CMSampleBuffer, SCStreamOutputType)` so a
/// multi-output stream's audio and video can be told apart. Returned by
/// [`AsyncSCStream::frames_typed`].
pub struct TypedSampleStream<'a> {
    state: &'a Arc<Mutex<AsyncSampleIteratorState>>,
}

impl std::fmt::Debug for TypedSampleStream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedSampleStream").finish_non_exhaustive()
    }
}

impl futures_core::Stream for TypedSampleStream<'_> {
    type Item = (crate::cm::CMSampleBuffer, SCStreamOutputType);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        poll_next_sample(self.state, cx)
    }
}

// SAFETY: `AsyncSampleSender` holds `Arc<Mutex<AsyncSampleIteratorState>>`.
// `AsyncSampleIteratorState` buffers `(CMSampleBuffer, SCStreamOutputType)`
// pairs plus an `Option<Waker>` and `Option<SCError>`; `CMSampleBuffer` has its
// own `unsafe impl Send` (it is an Apple-owned handle safe to transfer across
// threads) and the rest are `Send + Sync`, so the whole `Arc<Mutex<...>>` is
// safe to send and share across threads.
unsafe impl Send for AsyncSampleSender {}
unsafe impl Sync for AsyncSampleSender {}

/// Stream delegate for [`AsyncSCStream`] that closes the sample iterator when
/// `ScreenCaptureKit` stops the stream with an error.
///
/// Without this, a stream that fails mid-capture (captured display
/// disconnected, permission revoked, …) would leave [`NextSample`] pending
/// forever. On an error stop this records the [`SCError`], marks the iterator
/// closed (so `next()` resolves to `None` once buffered frames drain), and
/// wakes any parked task. The error is retrievable via
/// [`AsyncSCStream::take_error`].
struct AsyncStreamDelegate {
    state: Arc<Mutex<AsyncSampleIteratorState>>,
}

impl crate::stream::delegate_trait::SCStreamDelegateTrait for AsyncStreamDelegate {
    fn did_stop_with_error(&self, error: SCError) {
        if let Ok(mut state) = self.state.lock() {
            state.stop_error = Some(error);
            state.closed = true;
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }
}

// SAFETY: mirrors `AsyncSampleSender` — `AsyncStreamDelegate` holds the same
// `Arc<Mutex<AsyncSampleIteratorState>>`, whose contents (`CMSampleBuffer`,
// `Waker`, `SCError`) are all safe to send and share across threads.
unsafe impl Send for AsyncStreamDelegate {}
unsafe impl Sync for AsyncStreamDelegate {}

// ----------------------------------------------------------------------------
// Stream lifecycle control futures (start / stop / update)
// ----------------------------------------------------------------------------

/// FFI completion callback for [`AsyncSCStream`] lifecycle operations.
///
/// Translates the Swift `(context, success, message)` completion into the
/// waker-based [`AsyncCompletion`] machinery, so awaiting a control future
/// resumes the task via its [`Waker`] instead of parking a thread. This is the
/// same primitive used by the content / screenshot / picker futures.
extern "C" fn stream_control_callback(context: *mut c_void, success: bool, msg: *const i8) {
    crate::utils::panic_safe::catch_user_panic("stream_control_callback", move || {
        if success {
            // SAFETY: `context` is the one-shot completion pointer from
            // `AsyncCompletion::<()>::create()`; Swift invokes this callback
            // exactly once, after which the pointer is consumed.
            unsafe { AsyncCompletion::<()>::complete_ok(context, ()) };
        } else {
            let error = unsafe { error_from_cstr(msg) };
            // SAFETY: see above — one-shot completion pointer, fired once.
            unsafe { AsyncCompletion::<()>::complete_err(context, error) };
        }
    });
}

/// Future for an [`AsyncSCStream`] lifecycle operation — `start_capture`,
/// `stop_capture`, `update_configuration`, or `update_content_filter`.
///
/// Resolves once `ScreenCaptureKit` acknowledges the operation. Awaiting it
/// **never blocks the executor thread**: the task is parked via its [`Waker`]
/// and resumed from the Swift completion callback. This mirrors the underlying
/// Swift `Task { try await … }` entry points, keeping the async surface
/// consistent end to end.
///
/// The operation is kicked off eagerly when the method is called (a "hot"
/// future), matching the rest of this module — e.g.
/// [`AsyncSCShareableContent::get`]. Dropping the future without awaiting is
/// safe; it simply means success/failure is not observed.
#[must_use = "the operation starts eagerly, but you must .await the future to observe success or failure"]
pub struct StreamControlFuture {
    inner: AsyncCompletionFuture<()>,
    map_err: fn(String) -> SCError,
}

impl std::fmt::Debug for StreamControlFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamControlFuture")
            .finish_non_exhaustive()
    }
}

impl Future for StreamControlFuture {
    type Output = Result<(), SCError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let map_err = self.map_err;
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(map_err))
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
/// let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
/// let config = SCStreamConfiguration::new()
///     .with_width(1920)
///     .with_height(1080);
///
/// let stream = AsyncSCStream::new(&filter, &config, 30, SCStreamOutputType::Screen);
/// stream.start_capture().await?;
///
/// // Process frames asynchronously
/// while let Some(frame) = stream.next().await {
///     println!("Got frame!");
/// }
/// # Ok(())
/// # }
/// ```
/// Async wrapper for `SCStream` with integrated frame iteration.
///
/// # Back-pressure and frame loss
///
/// `AsyncSCStream` buffers samples in a **bounded** internal queue sized
/// by the `buffer_capacity` argument to [`AsyncSCStream::new`]. When the
/// queue is full and a new sample arrives from `ScreenCaptureKit`, the
/// **oldest** queued sample is dropped to make room — the stream is
/// **lossy by design**.
///
/// This is the right policy for real-time UI rendering, screen-share
/// previews, and live encoding: a slow consumer would rather see the
/// most recent frame than a stale one. It is the *wrong* policy for
/// lossless capture (e.g. saving every frame to disk for later
/// editing) — for that, use the synchronous [`SCStream`](crate::stream::SCStream)
/// directly, where back-pressure is naturally enforced by Apple's
/// `queueDepth` setting and your handler's runtime.
///
/// To detect when frames are being dropped, watch `buffered_count()`
/// against `buffer_capacity` over time, or instrument your handler
/// with a per-frame timestamp delta and compare to your expected
/// frame interval.
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
            stop_error: None,
        }));

        let sender = AsyncSampleSender {
            inner: Arc::clone(&state),
        };

        let delegate = AsyncStreamDelegate {
            state: Arc::clone(&state),
        };

        let mut stream = crate::stream::SCStream::new_with_delegate(filter, config, delegate);
        if stream.add_output_handler(sender, output_type).is_none() {
            // Registration failed: close the iterator immediately so `next()`
            // resolves to `None` instead of pending forever, and record why.
            if let Ok(mut s) = state.lock() {
                s.closed = true;
                s.stop_error = Some(SCError::StreamError(
                    "failed to register stream output handler".to_string(),
                ));
            }
        }

        Self {
            stream,
            iterator_state: state,
        }
    }

    /// Get the next sample buffer asynchronously
    ///
    /// Returns `None` when the stream is closed. For a multi-output stream
    /// (see [`add_output_type`](Self::add_output_type)) use
    /// [`next_typed`](Self::next_typed) to also learn each sample's
    /// [`SCStreamOutputType`].
    pub fn next(&self) -> NextSample<'_> {
        NextSample {
            state: &self.iterator_state,
        }
    }

    /// Get the next sample buffer together with its output type.
    ///
    /// Use this when the stream carries more than one output type (e.g. screen
    /// and audio) and you need to tell the samples apart. Returns `None` when
    /// the stream is closed.
    pub fn next_typed(&self) -> NextSampleTyped<'_> {
        NextSampleTyped {
            state: &self.iterator_state,
        }
    }

    /// Borrow the captured frames as a [`Stream`](futures_core::Stream) of
    /// `CMSampleBuffer`s.
    ///
    /// This unlocks the `futures::StreamExt` combinator ecosystem
    /// (`map`, `filter`, `take`, `for_each`, `zip`, …) for processing frames:
    ///
    /// ```no_run
    /// # async fn example(stream: screencapturekit::async_api::AsyncSCStream) {
    /// use futures_util::StreamExt;
    ///
    /// let frames: Vec<_> = stream.frames().take(30).collect().await;
    /// # let _ = frames;
    /// # }
    /// ```
    ///
    /// The returned stream borrows `self`; for a multi-output stream use
    /// [`frames_typed`](Self::frames_typed) to keep each sample's output type.
    #[must_use]
    pub fn frames(&self) -> SampleStream<'_> {
        SampleStream {
            state: &self.iterator_state,
        }
    }

    /// Borrow the captured frames as a [`Stream`](futures_core::Stream) of
    /// `(CMSampleBuffer, SCStreamOutputType)` pairs.
    ///
    /// Like [`frames`](Self::frames) but keeps each sample's output type, so a
    /// stream carrying both audio and video (see
    /// [`add_output_type`](Self::add_output_type)) can route samples with
    /// `StreamExt` combinators:
    ///
    /// ```no_run
    /// # async fn example(stream: screencapturekit::async_api::AsyncSCStream) {
    /// use futures_util::StreamExt;
    /// use screencapturekit::stream::output_type::SCStreamOutputType;
    ///
    /// let audio: Vec<_> = stream
    ///     .frames_typed()
    ///     .filter(|(_, kind)| std::future::ready(*kind == SCStreamOutputType::Audio))
    ///     .take(10)
    ///     .collect()
    ///     .await;
    /// # let _ = audio;
    /// # }
    /// ```
    #[must_use]
    pub fn frames_typed(&self) -> TypedSampleStream<'_> {
        TypedSampleStream {
            state: &self.iterator_state,
        }
    }

    /// Also deliver samples of an additional output type.
    ///
    /// By default an [`AsyncSCStream`] carries the single output type passed to
    /// [`new`](Self::new). Call this to capture more than one type from one
    /// stream — for example add [`SCStreamOutputType::Audio`] to a stream
    /// created for [`SCStreamOutputType::Screen`] to capture audio and video
    /// together. Samples from every registered type share the same lossy
    /// buffer; use [`next_typed`](Self::next_typed) /
    /// [`try_next_typed`](Self::try_next_typed) to distinguish them.
    ///
    /// Returns `true` if the output type was registered. Registration can fail
    /// if the stream configuration does not enable that type (e.g. audio
    /// capture was not configured).
    pub fn add_output_type(&mut self, output_type: SCStreamOutputType) -> bool {
        let sender = AsyncSampleSender {
            inner: Arc::clone(&self.iterator_state),
        };
        self.stream
            .add_output_handler(sender, output_type)
            .is_some()
    }

    /// Try to get a sample without waiting
    #[must_use]
    pub fn try_next(&self) -> Option<crate::cm::CMSampleBuffer> {
        self.iterator_state
            .lock()
            .ok()?
            .buffer
            .pop_front()
            .map(|(buffer, _of_type)| buffer)
    }

    /// Try to get a sample together with its output type, without waiting.
    #[must_use]
    pub fn try_next_typed(&self) -> Option<(crate::cm::CMSampleBuffer, SCStreamOutputType)> {
        self.iterator_state.lock().ok()?.buffer.pop_front()
    }

    /// Check if the stream has been closed
    ///
    /// Returns `true` once the stream has stopped — either because this
    /// `AsyncSCStream` was dropped or because `ScreenCaptureKit` stopped it
    /// with an error (see [`take_error`](Self::take_error)).
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.iterator_state.lock().map_or(true, |s| s.closed)
    }

    /// Take the error that stopped the stream, if any.
    ///
    /// When `ScreenCaptureKit` stops the stream with an error (e.g. the
    /// captured display is disconnected or screen-recording permission is
    /// revoked), the sample iterator is closed — [`next`](Self::next) resolves
    /// to `None` after any buffered frames drain — and the [`SCError`] is stored
    /// here. Call this once the iteration loop ends to distinguish an error stop
    /// from a normal end of stream:
    ///
    /// ```no_run
    /// # async fn example(stream: screencapturekit::async_api::AsyncSCStream) {
    /// while let Some(_frame) = stream.next().await {
    ///     // process frames …
    /// }
    /// if let Some(err) = stream.take_error() {
    ///     eprintln!("capture stopped with error: {err}");
    /// }
    /// # }
    /// ```
    ///
    /// The stored error is cleared once taken.
    #[must_use]
    pub fn take_error(&self) -> Option<SCError> {
        self.iterator_state.lock().ok()?.stop_error.take()
    }

    /// Get the number of buffered samples
    #[must_use]
    pub fn buffered_count(&self) -> usize {
        self.iterator_state.lock().map_or(0, |s| s.buffer.len())
    }

    /// Clear all buffered samples
    pub fn clear_buffer(&self) {
        if let Ok(mut state) = self.iterator_state.lock() {
            state.buffer.clear();
        }
    }

    /// Start capture asynchronously.
    ///
    /// Resolves when `ScreenCaptureKit` confirms the stream has started.
    /// Unlike [`SCStream::start_capture`](crate::stream::SCStream::start_capture),
    /// awaiting this **does not block the executor thread** — the task is parked
    /// via its [`Waker`] and resumed from the Swift completion callback.
    ///
    /// The capture is initiated eagerly when this method is called; `.await`
    /// observes the completion (or error).
    ///
    /// # Errors
    ///
    /// The awaited result is `Err(SCError::CaptureStartFailed)` if the stream
    /// fails to start.
    pub fn start_capture(&self) -> StreamControlFuture {
        let (future, context) = AsyncCompletion::<()>::create();
        // SAFETY: `self.stream.as_ptr()` is a valid, live `SCStream` pointer for
        // the duration of this call; `context` is the one-shot completion
        // pointer from `AsyncCompletion::create()`, invoked exactly once.
        unsafe {
            crate::ffi::sc_stream_start_capture(
                self.stream.as_ptr(),
                context,
                stream_control_callback,
            );
        }
        StreamControlFuture {
            inner: future,
            map_err: SCError::CaptureStartFailed,
        }
    }

    /// Stop capture asynchronously.
    ///
    /// Resolves when `ScreenCaptureKit` confirms the stream has stopped. Awaiting
    /// this **does not block the executor thread**.
    ///
    /// # Errors
    ///
    /// The awaited result is `Err(SCError::CaptureStopFailed)` if the stream
    /// fails to stop.
    pub fn stop_capture(&self) -> StreamControlFuture {
        let (future, context) = AsyncCompletion::<()>::create();
        // SAFETY: see `start_capture` — live stream pointer, one-shot context.
        unsafe {
            crate::ffi::sc_stream_stop_capture(
                self.stream.as_ptr(),
                context,
                stream_control_callback,
            );
        }
        StreamControlFuture {
            inner: future,
            map_err: SCError::CaptureStopFailed,
        }
    }

    /// Update stream configuration asynchronously.
    ///
    /// Resolves when the reconfiguration completes. Awaiting this **does not
    /// block the executor thread**.
    ///
    /// # Errors
    ///
    /// The awaited result is `Err(SCError::StreamError)` if the update fails.
    pub fn update_configuration(&self, config: &SCStreamConfiguration) -> StreamControlFuture {
        let (future, context) = AsyncCompletion::<()>::create();
        // SAFETY: `self.stream.as_ptr()` and `config.as_ptr()` are valid for the
        // duration of this call; `context` is the one-shot completion pointer.
        unsafe {
            crate::ffi::sc_stream_update_configuration(
                self.stream.as_ptr(),
                config.as_ptr(),
                context,
                stream_control_callback,
            );
        }
        StreamControlFuture {
            inner: future,
            map_err: SCError::StreamError,
        }
    }

    /// Update content filter asynchronously.
    ///
    /// Resolves when the filter swap completes. Awaiting this **does not block
    /// the executor thread**.
    ///
    /// # Errors
    ///
    /// The awaited result is `Err(SCError::StreamError)` if the update fails.
    pub fn update_content_filter(&self, filter: &SCContentFilter) -> StreamControlFuture {
        let (future, context) = AsyncCompletion::<()>::create();
        // SAFETY: `self.stream.as_ptr()` and `filter.as_ptr()` are valid for the
        // duration of this call; `context` is the one-shot completion pointer.
        unsafe {
            crate::ffi::sc_stream_update_content_filter(
                self.stream.as_ptr(),
                filter.as_ptr(),
                context,
                stream_control_callback,
            );
        }
        StreamControlFuture {
            inner: future,
            map_err: SCError::StreamError,
        }
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
    crate::utils::panic_safe::catch_user_panic("screenshot_image_callback", move || {
        if !error_ptr.is_null() {
            // SAFETY: `error` is non-null (checked above) and points to a valid null-terminated C string provided by the Swift completion handler.
            let error = unsafe { error_from_cstr(error_ptr) };
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe {
                AsyncCompletion::<crate::screenshot_manager::CGImage>::complete_err(
                    user_data, error,
                );
            }
        } else if !image_ptr.is_null() {
            // SAFETY: the Swift bridge hands back a retained `CGImageRef` on success.
            let image = unsafe { crate::screenshot_manager::cgimage_from_retained_ptr(image_ptr) };
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe { AsyncCompletion::complete_ok(user_data, image) };
        } else {
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe {
                AsyncCompletion::<crate::screenshot_manager::CGImage>::complete_err(
                    user_data,
                    "Unknown error".to_string(),
                );
            };
        }
    });
}

/// Callback for async `CMSampleBuffer` capture
#[cfg(feature = "macos_14_0")]
extern "C" fn screenshot_buffer_callback(
    buffer_ptr: *const c_void,
    error_ptr: *const i8,
    user_data: *mut c_void,
) {
    crate::utils::panic_safe::catch_user_panic("screenshot_buffer_callback", move || {
        if !error_ptr.is_null() {
            // SAFETY: `error` is non-null (checked above) and points to a valid null-terminated C string provided by the Swift completion handler.
            let error = unsafe { error_from_cstr(error_ptr) };
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe { AsyncCompletion::<crate::cm::CMSampleBuffer>::complete_err(user_data, error) };
        } else if !buffer_ptr.is_null() {
            // SAFETY: `buffer_ptr` is non-null (checked above), is a valid `CMSampleBuffer` pointer, and `cast_mut()` is sound because the underlying object is uniquely owned at this point.
            let buffer = unsafe { crate::cm::CMSampleBuffer::from_ptr(buffer_ptr.cast_mut()) };
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe { AsyncCompletion::complete_ok(user_data, buffer) };
        } else {
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe {
                AsyncCompletion::<crate::cm::CMSampleBuffer>::complete_err(
                    user_data,
                    "Unknown error".to_string(),
                );
            };
        }
    });
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

        // SAFETY: `content_filter.as_ptr()` and `configuration.as_ptr()` return valid non-null pointers for the duration of this call (borrowed via `&`). `context` is a one-shot completion pointer from `AsyncCompletion::create()`.
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

        // SAFETY: `content_filter.as_ptr()` and `configuration.as_ptr()` return valid non-null pointers for the duration of this call (borrowed via `&`). `context` is a one-shot completion pointer from `AsyncCompletion::create()`.
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

        // SAFETY: The rectangle coordinates are plain values passed by copy. `context` is a one-shot completion pointer from `AsyncCompletion::create()`.
        unsafe {
            crate::ffi::sc_screenshot_manager_capture_image_in_rect(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
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

        // SAFETY: `content_filter.as_ptr()` and `configuration.as_ptr()` return valid non-null pointers for the duration of this call (borrowed via `&`). `context` is a one-shot completion pointer from `AsyncCompletion::create()`.
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

        // SAFETY: `configuration.as_ptr()` returns a valid non-null pointer for the duration of this call (borrowed via `&`). The rectangle coordinates are plain values passed by copy. `context` is a one-shot completion pointer from `AsyncCompletion::create()`.
        unsafe {
            crate::ffi::sc_screenshot_manager_capture_screenshot_in_rect(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
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
    crate::utils::panic_safe::catch_user_panic("screenshot_output_callback", move || {
        if !error_ptr.is_null() {
            // SAFETY: `error` is non-null (checked above) and points to a valid null-terminated C string provided by the Swift completion handler.
            let error = unsafe { error_from_cstr(error_ptr) };
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe {
                AsyncCompletion::<crate::screenshot_manager::SCScreenshotOutput>::complete_err(
                    user_data, error,
                );
            }
        } else if !output_ptr.is_null() {
            let output = crate::screenshot_manager::SCScreenshotOutput::from_ptr(output_ptr);
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`.
            unsafe { AsyncCompletion::complete_ok(user_data, output) };
        } else {
            // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`; Swift invokes this callback exactly once, so the pointer is still valid.
            unsafe {
                AsyncCompletion::<crate::screenshot_manager::SCScreenshotOutput>::complete_err(
                    user_data,
                    "Unknown error".to_string(),
                );
            };
        }
    });
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
// SAFETY: `AsyncPickerCallbackResult` stores a `*const c_void` that is an
// Apple Objective-C object reference (`SCPickerResult`). All ScreenCaptureKit
// objects are thread-safe to pass across threads (they follow ObjC ARC rules),
// so sending this pointer to another thread is sound.
unsafe impl Send for AsyncPickerCallbackResult {}

/// Callback for async picker
#[cfg(feature = "macos_14_0")]
extern "C" fn async_picker_callback(result_code: i32, ptr: *const c_void, user_data: *mut c_void) {
    crate::utils::panic_safe::catch_user_panic("async_picker_callback", move || {
        let result = AsyncPickerCallbackResult {
            code: result_code,
            ptr,
        };
        // SAFETY: `user_data` is the one-shot completion context from `AsyncCompletion::create()`.
        unsafe { AsyncCompletion::complete_ok(user_data, result) };
    });
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

        // SAFETY: `config.as_ptr()` returns a valid non-null pointer for the duration of this call. `context` is a one-shot completion pointer from `AsyncCompletion::create()`.
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

        // SAFETY: `config.as_ptr()` returns a valid non-null pointer for the duration of this call. `context` is a one-shot completion pointer from `AsyncCompletion::create()`.
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

        // SAFETY: `config.as_ptr()` and `stream.as_ptr()` return valid non-null pointers for the duration of this call. `context` is a one-shot completion pointer from `AsyncCompletion::create()`.
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
        poll_next_recording_event(self.state, cx)
    }
}

/// Shared poll logic for the recording-event future/stream.
#[cfg(feature = "macos_15_0")]
fn poll_next_recording_event(
    state: &Arc<Mutex<AsyncRecordingState>>,
    cx: &Context<'_>,
) -> Poll<Option<RecordingEvent>> {
    let Ok(mut state) = state.lock() else {
        return Poll::Ready(None);
    };

    if let Some(event) = state.events.pop_front() {
        return Poll::Ready(Some(event));
    }

    if state.finished {
        Poll::Ready(None)
    } else {
        // Avoid the lost-wakeup race — see `poll_next_sample` above.
        let waker = cx.waker();
        match state.waker {
            Some(ref existing) if existing.will_wake(waker) => {}
            _ => state.waker = Some(waker.clone()),
        }
        Poll::Pending
    }
}

/// A [`Stream`](futures_core::Stream) of recording lifecycle [`RecordingEvent`]s.
///
/// Yields `Started` / `Finished` / `Failed(_)` and ends (`None`) once the
/// recording finishes or fails. Returned by [`AsyncSCRecordingOutput::events`];
/// integrates with the `futures::StreamExt` combinators.
#[cfg(feature = "macos_15_0")]
pub struct RecordingEventStream<'a> {
    state: &'a Arc<Mutex<AsyncRecordingState>>,
}

#[cfg(feature = "macos_15_0")]
impl std::fmt::Debug for RecordingEventStream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordingEventStream")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "macos_15_0")]
impl futures_core::Stream for RecordingEventStream<'_> {
    type Item = RecordingEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        poll_next_recording_event(self.state, cx)
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

    /// Borrow the recording events as a [`Stream`](futures_core::Stream) of
    /// [`RecordingEvent`]s.
    ///
    /// Unlocks the `futures::StreamExt` combinators for the recording event
    /// flow (`for_each`, `take_while`, …):
    ///
    /// ```no_run
    /// # #[cfg(feature = "macos_15_0")]
    /// # async fn example(recording: screencapturekit::async_api::AsyncSCRecordingOutput) {
    /// use futures_util::StreamExt;
    ///
    /// recording
    ///     .events()
    ///     .for_each(|event| {
    ///         println!("recording event: {event:?}");
    ///         std::future::ready(())
    ///     })
    ///     .await;
    /// # }
    /// ```
    #[must_use]
    pub fn events(&self) -> RecordingEventStream<'_> {
        RecordingEventStream { state: &self.state }
    }

    /// Check if the recording has finished
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.state.lock().map_or(true, |s| s.finished)
    }

    /// Get any pending events without waiting
    #[must_use]
    pub fn try_next(&self) -> Option<RecordingEvent> {
        self.state.lock().ok()?.events.pop_front()
    }
}
