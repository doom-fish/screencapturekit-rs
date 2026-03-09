//! Swift FFI based `SCStream` implementation
//!
//! This is the primary (and only) implementation in v1.0+.
//! All `ScreenCaptureKit` operations use direct Swift FFI bindings.
//!
//! Each stream owns a heap-allocated `StreamContext` that holds its output
//! handlers and delegate. The context pointer is passed through FFI so that
//! callbacks route directly to the owning stream — no global registries.

use std::ffi::{c_void, CStr};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::error::SCError;
use crate::stream::delegate_trait::SCStreamDelegateTrait;
use crate::utils::completion::UnitCompletion;
use crate::{
    dispatch_queue::DispatchQueue,
    ffi,
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType,
    },
};

/// Per-stream handler entry.
struct HandlerEntry {
    id: usize,
    of_type: SCStreamOutputType,
    handler: Box<dyn SCStreamOutputTrait>,
}

/// Per-stream context holding output handlers and an optional delegate.
///
/// Allocated on the heap via `Box::into_raw` and passed through FFI as an
/// opaque context pointer. Callbacks cast it back to `&StreamContext` for
/// direct, O(1) access to the owning stream's state.
struct StreamContext {
    handlers: Mutex<Vec<HandlerEntry>>,
    delegate: Mutex<Option<Box<dyn SCStreamDelegateTrait>>>,
    ref_count: AtomicUsize,
}

impl StreamContext {
    fn new() -> *mut Self {
        let ctx = Box::new(Self {
            handlers: Mutex::new(Vec::new()),
            delegate: Mutex::new(None),
            ref_count: AtomicUsize::new(1),
        });
        Box::into_raw(ctx)
    }

    fn new_with_delegate(delegate: Box<dyn SCStreamDelegateTrait>) -> *mut Self {
        let ctx = Box::new(Self {
            handlers: Mutex::new(Vec::new()),
            delegate: Mutex::new(Some(delegate)),
            ref_count: AtomicUsize::new(1),
        });
        Box::into_raw(ctx)
    }

    /// Increment the reference count.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid, live `StreamContext`.
    unsafe fn retain(ptr: *mut Self) {
        unsafe { &*ptr }.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the reference count, freeing the context if it reaches zero.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid, live `StreamContext`. After this call,
    /// `ptr` must not be used if the context was freed.
    unsafe fn release(ptr: *mut Self) {
        if ptr.is_null() {
            return;
        }
        let prev = unsafe { &*ptr }.ref_count.fetch_sub(1, Ordering::Release);
        if prev == 1 {
            std::sync::atomic::fence(Ordering::Acquire);
            drop(unsafe { Box::from_raw(ptr) });
        }
    }
}

/// Monotonically increasing handler ID generator (process-wide).
static NEXT_HANDLER_ID: AtomicUsize = AtomicUsize::new(1);

// C callback for stream errors — dispatches to per-stream delegate via context pointer.
extern "C" fn delegate_error_callback(context: *mut c_void, error_code: i32, msg: *const i8) {
    if context.is_null() {
        return;
    }
    let ctx = unsafe { &*(context.cast::<StreamContext>()) };

    let message = if msg.is_null() {
        "Unknown error".to_string()
    } else {
        unsafe { CStr::from_ptr(msg) }
            .to_str()
            .unwrap_or("Unknown error")
            .to_string()
    };

    let error = if error_code != 0 {
        crate::error::SCStreamErrorCode::from_raw(error_code).map_or_else(
            || SCError::StreamError(format!("{message} (code: {error_code})")),
            |code| SCError::SCStreamError {
                code,
                message: Some(message.clone()),
            },
        )
    } else {
        SCError::StreamError(message.clone())
    };

    if let Ok(delegate_guard) = ctx.delegate.lock() {
        if let Some(ref delegate) = *delegate_guard {
            delegate.did_stop_with_error(error);
            delegate.stream_did_stop(Some(message));
            return;
        }
    }

    // Fallback to logging if no delegate registered
    eprintln!("SCStream error: {error}");
}

// C callback for sample buffers — dispatches to per-stream handlers via context pointer.
extern "C" fn sample_handler(context: *mut c_void, sample_buffer: *const c_void, output_type: i32) {
    if context.is_null() {
        unsafe { crate::cm::ffi::cm_sample_buffer_release(sample_buffer.cast_mut()) };
        return;
    }
    let ctx = unsafe { &*(context.cast::<StreamContext>()) };

    let output_type_enum = match output_type {
        0 => SCStreamOutputType::Screen,
        1 => SCStreamOutputType::Audio,
        2 => SCStreamOutputType::Microphone,
        _ => {
            eprintln!("Unknown output type: {output_type}");
            unsafe { crate::cm::ffi::cm_sample_buffer_release(sample_buffer.cast_mut()) };
            return;
        }
    };

    // Mutex poisoning is unrecoverable in C callback context; unwrap is appropriate
    let handlers = ctx.handlers.lock().unwrap();

    // Find handlers matching this output type
    let matching: Vec<&HandlerEntry> = handlers
        .iter()
        .filter(|e| e.of_type == output_type_enum)
        .collect();

    if matching.is_empty() {
        // Drop the lock before releasing buffer
        drop(handlers);
        unsafe { crate::cm::ffi::cm_sample_buffer_release(sample_buffer.cast_mut()) };
        return;
    }

    let count = matching.len();
    for (idx, entry) in matching.iter().enumerate() {
        let buffer = unsafe { crate::cm::CMSampleBuffer::from_ptr(sample_buffer.cast_mut()) };

        // Retain for all but the last handler; the last one consumes the
        // original reference that Swift passed via passRetained.
        if idx < count - 1 {
            unsafe { crate::cm::ffi::cm_sample_buffer_retain(sample_buffer.cast_mut()) };
        }

        entry
            .handler
            .did_output_sample_buffer(buffer, output_type_enum);
    }
}

/// `SCStream` is a lightweight wrapper around the Swift `SCStream` instance.
/// It provides direct FFI access to `ScreenCaptureKit` functionality.
///
/// This is the primary and only implementation of `SCStream` in v1.0+.
/// All `ScreenCaptureKit` operations go through Swift FFI bindings.
///
/// # Examples
///
/// ```no_run
/// use screencapturekit::prelude::*;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get shareable content
/// let content = SCShareableContent::get()?;
/// let display = &content.displays()[0];
///
/// // Create filter and configuration
/// let filter = SCContentFilter::create()
///     .with_display(display)
///     .with_excluding_windows(&[])
///     .build();
/// let config = SCStreamConfiguration::new()
///     .with_width(1920)
///     .with_height(1080);
///
/// // Create and start stream
/// let mut stream = SCStream::new(&filter, &config);
/// stream.start_capture()?;
///
/// // ... capture frames ...
///
/// stream.stop_capture()?;
/// # Ok(())
/// # }
/// ```
pub struct SCStream {
    ptr: *const c_void,
    /// Per-stream context holding handlers and delegate (ref-counted).
    context: *mut StreamContext,
}

unsafe impl Send for SCStream {}
unsafe impl Sync for SCStream {}

impl SCStream {
    /// Create a new stream with a content filter and configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::prelude::*;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// let display = &content.displays()[0];
    /// let filter = SCContentFilter::create()
    ///     .with_display(display)
    ///     .with_excluding_windows(&[])
    ///     .build();
    /// let config = SCStreamConfiguration::new()
    ///     .with_width(1920)
    ///     .with_height(1080);
    ///
    /// let stream = SCStream::new(&filter, &config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(filter: &SCContentFilter, configuration: &SCStreamConfiguration) -> Self {
        let context = StreamContext::new();
        let context_ptr = context.cast::<c_void>();

        let ptr = unsafe {
            ffi::sc_stream_create(
                filter.as_ptr(),
                configuration.as_ptr(),
                context_ptr,
                delegate_error_callback,
                sample_handler,
            )
        };

        Self { ptr, context }
    }

    /// Create a new stream with a content filter, configuration, and delegate
    ///
    /// The delegate receives callbacks for stream lifecycle events:
    /// - `did_stop_with_error` - Called when the stream stops due to an error
    /// - `stream_did_stop` - Called when the stream stops (with optional error message)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use screencapturekit::prelude::*;
    /// use screencapturekit::stream::delegate_trait::StreamCallbacks;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = SCShareableContent::get()?;
    /// let display = &content.displays()[0];
    /// let filter = SCContentFilter::create()
    ///     .with_display(display)
    ///     .with_excluding_windows(&[])
    ///     .build();
    /// let config = SCStreamConfiguration::new()
    ///     .with_width(1920)
    ///     .with_height(1080);
    ///
    /// let delegate = StreamCallbacks::new()
    ///     .on_error(|e| eprintln!("Stream error: {}", e))
    ///     .on_stop(|err| {
    ///         if let Some(msg) = err {
    ///             eprintln!("Stream stopped with error: {}", msg);
    ///         }
    ///     });
    ///
    /// let stream = SCStream::new_with_delegate(&filter, &config, delegate);
    /// stream.start_capture()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        delegate: impl SCStreamDelegateTrait + 'static,
    ) -> Self {
        let context = StreamContext::new_with_delegate(Box::new(delegate));
        let context_ptr = context.cast::<c_void>();

        let ptr = unsafe {
            ffi::sc_stream_create(
                filter.as_ptr(),
                configuration.as_ptr(),
                context_ptr,
                delegate_error_callback,
                sample_handler,
            )
        };

        Self { ptr, context }
    }

    /// Add an output handler to receive captured frames
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler to receive callbacks. Can be:
    ///   - A struct implementing [`SCStreamOutputTrait`]
    ///   - A closure `|CMSampleBuffer, SCStreamOutputType| { ... }`
    /// * `of_type` - The type of output to receive (Screen, Audio, or Microphone)
    ///
    /// # Returns
    ///
    /// Returns `Some(handler_id)` on success, `None` on failure.
    /// The handler ID can be used with [`remove_output_handler`](Self::remove_output_handler).
    ///
    /// # Examples
    ///
    /// Using a struct:
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// struct MyHandler;
    /// impl SCStreamOutputTrait for MyHandler {
    ///     fn did_output_sample_buffer(&self, _sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
    ///         println!("Got frame!");
    ///     }
    /// }
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let display = &content.displays()[0];
    /// # let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    /// # let config = SCStreamConfiguration::default();
    /// let mut stream = SCStream::new(&filter, &config);
    /// stream.add_output_handler(MyHandler, SCStreamOutputType::Screen);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Using a closure:
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let display = &content.displays()[0];
    /// # let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    /// # let config = SCStreamConfiguration::default();
    /// let mut stream = SCStream::new(&filter, &config);
    /// stream.add_output_handler(
    ///     |_sample, _type| println!("Got frame!"),
    ///     SCStreamOutputType::Screen
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_output_handler(
        &mut self,
        handler: impl SCStreamOutputTrait + 'static,
        of_type: SCStreamOutputType,
    ) -> Option<usize> {
        self.add_output_handler_with_queue(handler, of_type, None)
    }

    /// Add an output handler with a custom dispatch queue
    ///
    /// This allows controlling which thread/queue the handler is called on.
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler to receive callbacks
    /// * `of_type` - The type of output to receive
    /// * `queue` - Optional custom dispatch queue for callbacks
    ///
    /// # Panics
    ///
    /// Panics if the internal handler mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    /// use screencapturekit::dispatch_queue::{DispatchQueue, DispatchQoS};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let display = &content.displays()[0];
    /// # let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    /// # let config = SCStreamConfiguration::default();
    /// let mut stream = SCStream::new(&filter, &config);
    /// let queue = DispatchQueue::new("com.myapp.capture", DispatchQoS::UserInteractive);
    ///
    /// stream.add_output_handler_with_queue(
    ///     |_sample, _type| println!("Got frame on custom queue!"),
    ///     SCStreamOutputType::Screen,
    ///     Some(&queue)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_output_handler_with_queue(
        &mut self,
        handler: impl SCStreamOutputTrait + 'static,
        of_type: SCStreamOutputType,
        queue: Option<&DispatchQueue>,
    ) -> Option<usize> {
        let handler_id = NEXT_HANDLER_ID.fetch_add(1, Ordering::Relaxed);

        // Convert output type to int for Swift
        let output_type_int = match of_type {
            SCStreamOutputType::Screen => 0,
            SCStreamOutputType::Audio => 1,
            SCStreamOutputType::Microphone => 2,
        };

        let ok = if let Some(q) = queue {
            unsafe {
                ffi::sc_stream_add_stream_output_with_queue(self.ptr, output_type_int, q.as_ptr())
            }
        } else {
            unsafe { ffi::sc_stream_add_stream_output(self.ptr, output_type_int) }
        };

        if ok {
            unsafe { &*self.context }
                .handlers
                .lock()
                .unwrap()
                .push(HandlerEntry {
                    id: handler_id,
                    of_type,
                    handler: Box::new(handler),
                });
            Some(handler_id)
        } else {
            None
        }
    }

    /// Remove an output handler
    ///
    /// # Arguments
    ///
    /// * `id` - The handler ID returned from [`add_output_handler`](Self::add_output_handler)
    /// * `of_type` - The type of output the handler was registered for
    ///
    /// # Panics
    ///
    /// Panics if the internal handler mutex is poisoned.
    ///
    /// # Returns
    ///
    /// Returns `true` if the handler was found and removed, `false` otherwise.
    pub fn remove_output_handler(&mut self, id: usize, of_type: SCStreamOutputType) -> bool {
        let mut handlers = unsafe { &*self.context }.handlers.lock().unwrap();
        let Some(pos) = handlers.iter().position(|e| e.id == id) else {
            return false;
        };
        handlers.remove(pos);

        // If no more handlers for this output type, tell Swift to remove the output
        let has_type = handlers.iter().any(|e| e.of_type == of_type);
        drop(handlers);

        if !has_type {
            let output_type_int = match of_type {
                SCStreamOutputType::Screen => 0,
                SCStreamOutputType::Audio => 1,
                SCStreamOutputType::Microphone => 2,
            };
            unsafe { ffi::sc_stream_remove_stream_output(self.ptr, output_type_int) };
        }

        true
    }

    /// Start capturing screen content
    ///
    /// This method blocks until the capture operation completes or fails.
    ///
    /// # Errors
    ///
    /// Returns `SCError::CaptureStartFailed` if the capture fails to start.
    pub fn start_capture(&self) -> Result<(), SCError> {
        let (completion, context) = UnitCompletion::new();
        unsafe { ffi::sc_stream_start_capture(self.ptr, context, UnitCompletion::callback) };
        completion.wait().map_err(SCError::CaptureStartFailed)
    }

    /// Stop capturing screen content
    ///
    /// This method blocks until the capture operation completes or fails.
    ///
    /// # Errors
    ///
    /// Returns `SCError::CaptureStopFailed` if the capture fails to stop.
    pub fn stop_capture(&self) -> Result<(), SCError> {
        let (completion, context) = UnitCompletion::new();
        unsafe { ffi::sc_stream_stop_capture(self.ptr, context, UnitCompletion::callback) };
        completion.wait().map_err(SCError::CaptureStopFailed)
    }

    /// Update the stream configuration
    ///
    /// This method blocks until the configuration update completes or fails.
    ///
    /// # Errors
    ///
    /// Returns `SCError::StreamError` if the configuration update fails.
    pub fn update_configuration(
        &self,
        configuration: &SCStreamConfiguration,
    ) -> Result<(), SCError> {
        let (completion, context) = UnitCompletion::new();
        unsafe {
            ffi::sc_stream_update_configuration(
                self.ptr,
                configuration.as_ptr(),
                context,
                UnitCompletion::callback,
            );
        }
        completion.wait().map_err(SCError::StreamError)
    }

    /// Update the content filter
    ///
    /// This method blocks until the filter update completes or fails.
    ///
    /// # Errors
    ///
    /// Returns `SCError::StreamError` if the filter update fails.
    pub fn update_content_filter(&self, filter: &SCContentFilter) -> Result<(), SCError> {
        let (completion, context) = UnitCompletion::new();
        unsafe {
            ffi::sc_stream_update_content_filter(
                self.ptr,
                filter.as_ptr(),
                context,
                UnitCompletion::callback,
            );
        }
        completion.wait().map_err(SCError::StreamError)
    }

    /// Get the synchronization clock for this stream (macOS 13.0+)
    ///
    /// Returns the `CMClock` used to synchronize the stream's output.
    /// This is useful for coordinating multiple streams or synchronizing
    /// with other media.
    ///
    /// Returns `None` if the clock is not available (e.g., stream not started
    /// or macOS version too old).
    #[cfg(feature = "macos_13_0")]
    pub fn synchronization_clock(&self) -> Option<crate::cm::CMClock> {
        let ptr = unsafe { ffi::sc_stream_get_synchronization_clock(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(crate::cm::CMClock::from_ptr(ptr))
        }
    }

    /// Add a recording output to the stream (macOS 15.0+)
    ///
    /// Starts recording if the stream is already capturing, otherwise recording
    /// will start when capture begins. The recording is written to the file URL
    /// specified in the `SCRecordingOutputConfiguration`.
    ///
    /// # Errors
    ///
    /// Returns `SCError::StreamError` if adding the recording output fails.
    #[cfg(feature = "macos_15_0")]
    pub fn add_recording_output(
        &self,
        recording_output: &crate::recording_output::SCRecordingOutput,
    ) -> Result<(), SCError> {
        let (completion, context) = UnitCompletion::new();
        unsafe {
            ffi::sc_stream_add_recording_output(
                self.ptr,
                recording_output.as_ptr(),
                UnitCompletion::callback,
                context,
            );
        }
        completion.wait().map_err(SCError::StreamError)
    }

    /// Remove a recording output from the stream (macOS 15.0+)
    ///
    /// Stops recording if the stream is currently recording.
    ///
    /// # Errors
    ///
    /// Returns `SCError::StreamError` if removing the recording output fails.
    #[cfg(feature = "macos_15_0")]
    pub fn remove_recording_output(
        &self,
        recording_output: &crate::recording_output::SCRecordingOutput,
    ) -> Result<(), SCError> {
        let (completion, context) = UnitCompletion::new();
        unsafe {
            ffi::sc_stream_remove_recording_output(
                self.ptr,
                recording_output.as_ptr(),
                UnitCompletion::callback,
                context,
            );
        }
        completion.wait().map_err(SCError::StreamError)
    }

    /// Returns the raw pointer to the underlying Swift `SCStream` instance.
    #[allow(dead_code)]
    pub(crate) fn as_ptr(&self) -> *const c_void {
        self.ptr
    }
}

impl Drop for SCStream {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::sc_stream_release(self.ptr) };
        }
        unsafe { StreamContext::release(self.context) };
    }
}

impl Clone for SCStream {
    /// Clone the stream reference.
    ///
    /// Cloning an `SCStream` creates a new reference to the same underlying
    /// Swift `SCStream` object. The cloned stream shares the same handlers
    /// as the original — they receive frames from the same capture session.
    ///
    /// Both the original and cloned stream share the same capture state, so:
    /// - Starting capture on one affects both
    /// - Stopping capture on one affects both
    /// - Configuration updates affect both
    /// - Handlers receive the same frames
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use screencapturekit::prelude::*;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let content = SCShareableContent::get()?;
    /// # let display = &content.displays()[0];
    /// # let filter = SCContentFilter::create().with_display(display).with_excluding_windows(&[]).build();
    /// # let config = SCStreamConfiguration::default();
    /// let mut stream = SCStream::new(&filter, &config);
    /// stream.add_output_handler(|_, _| println!("Handler 1"), SCStreamOutputType::Screen);
    ///
    /// // Clone shares the same handlers
    /// let stream2 = stream.clone();
    /// // Both stream and stream2 will receive frames via Handler 1
    /// # Ok(())
    /// # }
    /// ```
    fn clone(&self) -> Self {
        unsafe { StreamContext::retain(self.context) };

        Self {
            ptr: unsafe { crate::ffi::sc_stream_retain(self.ptr) },
            context: self.context,
        }
    }
}

impl fmt::Debug for SCStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCStream")
            .field("ptr", &self.ptr)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for SCStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SCStream")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    /// Regression test for #135: multiple concurrent streams must not leak
    /// samples across each other.
    ///
    /// Creates two independent StreamContexts with separate handlers and
    /// directly invokes each context's handlers. Verifies that each handler
    /// only receives calls routed through its own context — not from the
    /// other context. With the old global HANDLER_REGISTRY, both handlers
    /// would have been called for every callback regardless of context.
    #[test]
    fn test_per_stream_callback_isolation() {
        let count_a = Arc::new(AtomicUsize::new(0));
        let count_b = Arc::new(AtomicUsize::new(0));

        // Create two independent contexts (simulates two SCStream instances)
        let ctx_a = StreamContext::new();
        let ctx_b = StreamContext::new();

        // Register an audio handler on context A
        {
            let counter = count_a.clone();
            let mut handlers = unsafe { &*ctx_a }.handlers.lock().unwrap();
            handlers.push(HandlerEntry {
                id: 1,
                of_type: SCStreamOutputType::Audio,
                handler: Box::new(
                    move |buf: crate::cm::CMSampleBuffer, _ty: SCStreamOutputType| {
                        counter.fetch_add(1, Ordering::Relaxed);
                        // Prevent Drop from calling cm_sample_buffer_release on our fake pointer
                        std::mem::forget(buf);
                    },
                ),
            });
        }

        // Register an audio handler on context B
        {
            let counter = count_b.clone();
            let mut handlers = unsafe { &*ctx_b }.handlers.lock().unwrap();
            handlers.push(HandlerEntry {
                id: 2,
                of_type: SCStreamOutputType::Audio,
                handler: Box::new(
                    move |buf: crate::cm::CMSampleBuffer, _ty: SCStreamOutputType| {
                        counter.fetch_add(1, Ordering::Relaxed);
                        std::mem::forget(buf);
                    },
                ),
            });
        }

        // Simulate 5 audio callbacks on context A by directly calling matching handlers
        for _ in 0..5 {
            let handlers = unsafe { &*ctx_a }.handlers.lock().unwrap();
            for entry in handlers
                .iter()
                .filter(|e| e.of_type == SCStreamOutputType::Audio)
            {
                let buf = unsafe { crate::cm::CMSampleBuffer::from_ptr(std::ptr::null_mut()) };
                entry
                    .handler
                    .did_output_sample_buffer(buf, SCStreamOutputType::Audio);
            }
        }

        // Simulate 3 audio callbacks on context B
        for _ in 0..3 {
            let handlers = unsafe { &*ctx_b }.handlers.lock().unwrap();
            for entry in handlers
                .iter()
                .filter(|e| e.of_type == SCStreamOutputType::Audio)
            {
                let buf = unsafe { crate::cm::CMSampleBuffer::from_ptr(std::ptr::null_mut()) };
                entry
                    .handler
                    .did_output_sample_buffer(buf, SCStreamOutputType::Audio);
            }
        }

        // Handler A must have received exactly 5 — not 8
        assert_eq!(
            count_a.load(Ordering::Relaxed),
            5,
            "handler A received callbacks meant for B (cross-stream leak)"
        );
        // Handler B must have received exactly 3 — not 8
        assert_eq!(
            count_b.load(Ordering::Relaxed),
            3,
            "handler B received callbacks meant for A (cross-stream leak)"
        );

        unsafe {
            StreamContext::release(ctx_a);
            StreamContext::release(ctx_b);
        }
    }

    /// Verify that handlers are filtered by output type within a single context.
    #[test]
    fn test_handler_output_type_filtering() {
        let screen_count = Arc::new(AtomicUsize::new(0));
        let audio_count = Arc::new(AtomicUsize::new(0));

        let ctx = StreamContext::new();

        {
            let counter = screen_count.clone();
            let mut handlers = unsafe { &*ctx }.handlers.lock().unwrap();
            handlers.push(HandlerEntry {
                id: 1,
                of_type: SCStreamOutputType::Screen,
                handler: Box::new(
                    move |buf: crate::cm::CMSampleBuffer, _ty: SCStreamOutputType| {
                        counter.fetch_add(1, Ordering::Relaxed);
                        std::mem::forget(buf);
                    },
                ),
            });
        }
        {
            let counter = audio_count.clone();
            let mut handlers = unsafe { &*ctx }.handlers.lock().unwrap();
            handlers.push(HandlerEntry {
                id: 2,
                of_type: SCStreamOutputType::Audio,
                handler: Box::new(
                    move |buf: crate::cm::CMSampleBuffer, _ty: SCStreamOutputType| {
                        counter.fetch_add(1, Ordering::Relaxed);
                        std::mem::forget(buf);
                    },
                ),
            });
        }

        // Send 4 screen callbacks
        for _ in 0..4 {
            let handlers = unsafe { &*ctx }.handlers.lock().unwrap();
            for entry in handlers
                .iter()
                .filter(|e| e.of_type == SCStreamOutputType::Screen)
            {
                let buf = unsafe { crate::cm::CMSampleBuffer::from_ptr(std::ptr::null_mut()) };
                entry
                    .handler
                    .did_output_sample_buffer(buf, SCStreamOutputType::Screen);
            }
        }

        // Send 2 audio callbacks
        for _ in 0..2 {
            let handlers = unsafe { &*ctx }.handlers.lock().unwrap();
            for entry in handlers
                .iter()
                .filter(|e| e.of_type == SCStreamOutputType::Audio)
            {
                let buf = unsafe { crate::cm::CMSampleBuffer::from_ptr(std::ptr::null_mut()) };
                entry
                    .handler
                    .did_output_sample_buffer(buf, SCStreamOutputType::Audio);
            }
        }

        assert_eq!(screen_count.load(Ordering::Relaxed), 4);
        assert_eq!(audio_count.load(Ordering::Relaxed), 2);

        unsafe { StreamContext::release(ctx) };
    }

    /// Verify that StreamContext ref counting works correctly.
    #[test]
    fn test_stream_context_ref_counting() {
        let ctx = StreamContext::new();

        // Initial ref count is 1
        assert_eq!(unsafe { &*ctx }.ref_count.load(Ordering::Relaxed), 1);

        // Retain bumps to 2
        unsafe { StreamContext::retain(ctx) };
        assert_eq!(unsafe { &*ctx }.ref_count.load(Ordering::Relaxed), 2);

        // First release drops to 1 — context still alive
        unsafe { StreamContext::release(ctx) };
        assert_eq!(unsafe { &*ctx }.ref_count.load(Ordering::Relaxed), 1);

        // Second release drops to 0 — context freed (no crash = success)
        unsafe { StreamContext::release(ctx) };
    }
}
