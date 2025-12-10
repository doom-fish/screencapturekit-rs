//! Swift FFI based `SCStream` implementation
//!
//! This is the primary (and only) implementation in v1.0+.
//! All `ScreenCaptureKit` operations use direct Swift FFI bindings.

use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::fmt;
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

// Handler entry with reference count
struct HandlerEntry {
    handler: Box<dyn SCStreamOutputTrait>,
    ref_count: usize,
}

// Global registry for output handlers with reference counting
static HANDLER_REGISTRY: Mutex<Option<HashMap<usize, HandlerEntry>>> = Mutex::new(None);
static NEXT_HANDLER_ID: Mutex<usize> = Mutex::new(1);

// Global registry for stream delegates (keyed by stream pointer) with reference counting
struct DelegateEntry {
    delegate: Box<dyn SCStreamDelegateTrait>,
    ref_count: usize,
}
static DELEGATE_REGISTRY: Mutex<Option<HashMap<usize, DelegateEntry>>> = Mutex::new(None);

/// Increment a handler's ref count
#[allow(clippy::significant_drop_tightening)]
fn increment_handler_ref_count(id: usize) {
    let mut registry = HANDLER_REGISTRY.lock().unwrap();
    let Some(handlers) = registry.as_mut() else {
        return;
    };
    if let Some(entry) = handlers.get_mut(&id) {
        entry.ref_count += 1;
    }
}

/// Decrement a handler's ref count, returning true if entry was removed (`ref_count` reached 0)
#[allow(clippy::significant_drop_tightening)]
fn decrement_handler_ref_count(id: usize) -> bool {
    let mut registry = HANDLER_REGISTRY.lock().unwrap();
    let Some(handlers) = registry.as_mut() else {
        return false;
    };
    let Some(entry) = handlers.get_mut(&id) else {
        return false;
    };

    entry.ref_count = entry.ref_count.saturating_sub(1);
    if entry.ref_count == 0 {
        handlers.remove(&id);
        true
    } else {
        false
    }
}

/// Increment a delegate's ref count
#[allow(clippy::significant_drop_tightening)]
fn increment_delegate_ref_count(stream_key: usize) {
    let Ok(mut registry) = DELEGATE_REGISTRY.lock() else {
        return;
    };
    let Some(delegates) = registry.as_mut() else {
        return;
    };
    if let Some(entry) = delegates.get_mut(&stream_key) {
        entry.ref_count += 1;
    }
}

/// Decrement a delegate's ref count, returning true if entry was removed (`ref_count` reached 0)
#[allow(clippy::significant_drop_tightening)]
fn decrement_delegate_ref_count(stream_key: usize) -> bool {
    let Ok(mut registry) = DELEGATE_REGISTRY.lock() else {
        return false;
    };
    let Some(delegates) = registry.as_mut() else {
        return false;
    };
    let Some(entry) = delegates.get_mut(&stream_key) else {
        return false;
    };

    entry.ref_count = entry.ref_count.saturating_sub(1);
    if entry.ref_count == 0 {
        delegates.remove(&stream_key);
        true
    } else {
        false
    }
}

// C callback for stream errors that dispatches to registered delegate
extern "C" fn delegate_error_callback(stream: *const c_void, error_code: i32, msg: *const i8) {
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

    // Look up delegate in registry and call it
    let stream_key = stream as usize;
    if let Ok(registry) = DELEGATE_REGISTRY.lock() {
        if let Some(ref delegates) = *registry {
            if let Some(entry) = delegates.get(&stream_key) {
                entry.delegate.did_stop_with_error(error);
                entry.delegate.stream_did_stop(Some(message));
                return;
            }
        }
    }

    // Fallback to logging if no delegate registered
    eprintln!("SCStream error: {error}");
}

// C callback that retrieves handler from registry
extern "C" fn sample_handler(
    _stream: *const c_void,
    sample_buffer: *const c_void,
    output_type: i32,
) {
    // Mutex poisoning is unrecoverable in C callback context; unwrap is appropriate
    let registry = HANDLER_REGISTRY.lock().unwrap();
    if let Some(handlers) = registry.as_ref() {
        if handlers.is_empty() {
            // No handlers registered - release the buffer that Swift passed us
            unsafe { crate::cm::ffi::cm_sample_buffer_release(sample_buffer.cast_mut()) };
            return;
        }

        let output_type_enum = match output_type {
            0 => SCStreamOutputType::Screen,
            1 => SCStreamOutputType::Audio,
            2 => SCStreamOutputType::Microphone,
            _ => {
                eprintln!("Unknown output type: {output_type}");
                // Unknown type - release the buffer
                unsafe { crate::cm::ffi::cm_sample_buffer_release(sample_buffer.cast_mut()) };
                return;
            }
        };

        let handler_count = handlers.len();

        // Call all registered handlers
        for (idx, (_id, entry)) in handlers.iter().enumerate() {
            // Convert raw pointer to CMSampleBuffer
            let buffer = unsafe { crate::cm::CMSampleBuffer::from_ptr(sample_buffer.cast_mut()) };

            // For all handlers except the last, we need to retain the buffer
            if idx < handler_count - 1 {
                // Retain the buffer so it's not released when this handler's buffer is dropped
                unsafe { crate::cm::ffi::cm_sample_buffer_retain(sample_buffer.cast_mut()) };
            }
            // The last handler will release the original retained reference from Swift

            entry
                .handler
                .did_output_sample_buffer(buffer, output_type_enum);
        }
    } else {
        // No registry - release the buffer
        unsafe { crate::cm::ffi::cm_sample_buffer_release(sample_buffer.cast_mut()) };
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
/// let filter = SCContentFilter::builder()
///     .display(display)
///     .exclude_windows(&[])
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
    /// Handler IDs registered by this stream instance, keyed by output type
    handler_ids: Vec<(usize, SCStreamOutputType)>,
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
    /// let filter = SCContentFilter::builder()
    ///     .display(display)
    ///     .exclude_windows(&[])
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
        extern "C" fn error_callback(_stream: *const c_void, error_code: i32, msg: *const i8) {
            let message = if msg.is_null() {
                "Unknown error"
            } else {
                unsafe { CStr::from_ptr(msg) }
                    .to_str()
                    .unwrap_or("Unknown error")
            };

            if error_code != 0 {
                if let Some(code) = crate::error::SCStreamErrorCode::from_raw(error_code) {
                    eprintln!("SCStream error ({code}): {message}");
                } else {
                    eprintln!("SCStream error (code {error_code}): {message}");
                }
            } else {
                eprintln!("SCStream error: {message}");
            }
        }
        let ptr = unsafe {
            ffi::sc_stream_create(filter.as_ptr(), configuration.as_ptr(), error_callback)
        };
        // Note: The Swift bridge should never return null for a valid filter/config,
        // but we handle it gracefully by creating an empty stream that will fail on use.
        // This maintains API compatibility while being more defensive.
        Self {
            ptr,
            handler_ids: Vec::new(),
        }
    }

    /// Create a new stream with a content filter, configuration, and delegate
    ///
    /// The delegate receives callbacks for stream lifecycle events:
    /// - `did_stop_with_error` - Called when the stream stops due to an error
    /// - `stream_did_stop` - Called when the stream stops (with optional error message)
    ///
    /// # Panics
    ///
    /// Panics if the internal delegate registry mutex is poisoned.
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
    /// let filter = SCContentFilter::builder()
    ///     .display(display)
    ///     .exclude_windows(&[])
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
        let ptr = unsafe {
            ffi::sc_stream_create(
                filter.as_ptr(),
                configuration.as_ptr(),
                delegate_error_callback,
            )
        };

        // Store delegate in registry keyed by stream pointer
        if !ptr.is_null() {
            let stream_key = ptr as usize;
            let mut registry = DELEGATE_REGISTRY.lock().unwrap();
            if registry.is_none() {
                *registry = Some(HashMap::new());
            }
            registry.as_mut().unwrap().insert(
                stream_key,
                DelegateEntry {
                    delegate: Box::new(delegate),
                    ref_count: 1,
                },
            );
        }

        Self {
            ptr,
            handler_ids: Vec::new(),
        }
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
    /// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
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
    /// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
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
    /// Panics if the internal handler registry mutex is poisoned.
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
    /// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
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
        // Get next handler ID
        let handler_id = {
            // Mutex poisoning is unrecoverable; unwrap is appropriate
            let mut id_lock = NEXT_HANDLER_ID.lock().unwrap();
            let id = *id_lock;
            *id_lock += 1;
            id
        };

        // Store handler in registry
        {
            // Mutex poisoning is unrecoverable; unwrap is appropriate
            let mut registry = HANDLER_REGISTRY.lock().unwrap();
            if registry.is_none() {
                *registry = Some(HashMap::new());
            }
            // We just ensured registry is Some above
            registry.as_mut().unwrap().insert(
                handler_id,
                HandlerEntry {
                    handler: Box::new(handler),
                    ref_count: 1,
                },
            );
        }

        // Convert output type to int for Swift
        let output_type_int = match of_type {
            SCStreamOutputType::Screen => 0,
            SCStreamOutputType::Audio => 1,
            SCStreamOutputType::Microphone => 2,
        };

        let ok = if let Some(q) = queue {
            unsafe {
                ffi::sc_stream_add_stream_output_with_queue(
                    self.ptr,
                    output_type_int,
                    sample_handler,
                    q.as_ptr(),
                )
            }
        } else {
            unsafe { ffi::sc_stream_add_stream_output(self.ptr, output_type_int, sample_handler) }
        };

        if ok {
            self.handler_ids.push((handler_id, of_type));
            Some(handler_id)
        } else {
            // Remove from registry since Swift rejected it
            HANDLER_REGISTRY
                .lock()
                .unwrap()
                .as_mut()
                .map(|handlers| handlers.remove(&handler_id));
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
    /// Panics if the internal handler registry mutex is poisoned.
    ///
    /// # Returns
    ///
    /// Returns `true` if the handler was found and removed, `false` otherwise.
    pub fn remove_output_handler(&mut self, id: usize, of_type: SCStreamOutputType) -> bool {
        // Remove from our tracking
        let Some(pos) = self.handler_ids.iter().position(|(hid, _)| *hid == id) else {
            return false;
        };
        self.handler_ids.remove(pos);

        // Decrement ref count in global registry, remove from Swift if this was the last reference
        if decrement_handler_ref_count(id) {
            let output_type_int = match of_type {
                SCStreamOutputType::Screen => 0,
                SCStreamOutputType::Audio => 1,
                SCStreamOutputType::Microphone => 2,
            };
            unsafe { ffi::sc_stream_remove_stream_output(self.ptr, output_type_int) }
        } else {
            true
        }
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
        // Clean up all registered handlers (decrement ref counts)
        for (id, of_type) in std::mem::take(&mut self.handler_ids) {
            if decrement_handler_ref_count(id) {
                // This was the last reference, tell Swift to remove the output
                let output_type_int = match of_type {
                    SCStreamOutputType::Screen => 0,
                    SCStreamOutputType::Audio => 1,
                    SCStreamOutputType::Microphone => 2,
                };
                unsafe { ffi::sc_stream_remove_stream_output(self.ptr, output_type_int) };
            }
        }

        // Clean up delegate from registry (decrement ref count)
        if !self.ptr.is_null() {
            decrement_delegate_ref_count(self.ptr as usize);
        }

        if !self.ptr.is_null() {
            unsafe { ffi::sc_stream_release(self.ptr) };
        }
    }
}

impl Clone for SCStream {
    /// Clone the stream reference.
    ///
    /// Cloning an `SCStream` creates a new reference to the same underlying
    /// Swift `SCStream` object. The cloned stream shares the same handlers
    /// as the original - they receive frames from the same capture session.
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
    /// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
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
        // Increment delegate ref count if one exists for this stream
        if !self.ptr.is_null() {
            increment_delegate_ref_count(self.ptr as usize);
        }

        // Increment handler ref counts for all handlers this stream references
        for (id, _) in &self.handler_ids {
            increment_handler_ref_count(*id);
        }

        unsafe {
            Self {
                ptr: crate::ffi::sc_stream_retain(self.ptr),
                handler_ids: self.handler_ids.clone(),
            }
        }
    }
}

impl fmt::Debug for SCStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCStream")
            .field("ptr", &self.ptr)
            .field("handler_ids", &self.handler_ids)
            .finish()
    }
}

impl fmt::Display for SCStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SCStream")
    }
}
