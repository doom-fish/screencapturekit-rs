//! Swift FFI based `SCStream` implementation
//!
//! This is the primary (and only) implementation in v1.0+.
//! All `ScreenCaptureKit` operations use direct Swift FFI bindings.

use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::fmt;
use std::sync::Mutex;

use crate::error::SCError;
use crate::utils::sync_completion::UnitCompletion;
use crate::{
    dispatch_queue::DispatchQueue,
    ffi,
    stream::{
        configuration::SCStreamConfiguration, content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait, output_type::SCStreamOutputType,
    },
};

// Global registry for output handlers
static HANDLER_REGISTRY: Mutex<Option<HashMap<usize, Box<dyn SCStreamOutputTrait>>>> =
    Mutex::new(None);
static NEXT_HANDLER_ID: Mutex<usize> = Mutex::new(1);

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
            return;
        }

        let output_type_enum = if output_type == 1 {
            SCStreamOutputType::Audio
        } else {
            SCStreamOutputType::Screen
        };

        let handler_count = handlers.len();

        // Call all registered handlers
        for (idx, (_id, handler)) in handlers.iter().enumerate() {
            // Convert raw pointer to CMSampleBuffer
            let buffer = unsafe { crate::cm::CMSampleBuffer::from_ptr(sample_buffer.cast_mut()) };

            // For all handlers except the last, we need to retain the buffer
            if idx < handler_count - 1 {
                // Retain the buffer so it's not released when this handler's buffer is dropped
                unsafe { crate::cm::ffi::cm_sample_buffer_retain(sample_buffer.cast_mut()) };
            }
            // The last handler will release the original retained reference from Swift

            handler.did_output_sample_buffer(buffer, output_type_enum);
        }
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
/// let mut config = SCStreamConfiguration::default();
/// config.set_width(1920);
/// config.set_height(1080);
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
}

unsafe impl Send for SCStream {}
unsafe impl Sync for SCStream {}

impl SCStream {
    /// Create a new stream with a content filter and configuration
    ///
    /// # Panics
    ///
    /// Panics if the Swift bridge returns a null stream pointer.
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
    /// let config = SCStreamConfiguration::default();
    ///
    /// let stream = SCStream::new(&filter, &config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(filter: &SCContentFilter, configuration: &SCStreamConfiguration) -> Self {
        extern "C" fn error_callback(_stream: *const c_void, msg: *const i8) {
            if !msg.is_null() {
                if let Ok(s) = unsafe { CStr::from_ptr(msg) }.to_str() {
                    eprintln!("SCStream error: {s}");
                }
            }
        }
        let ptr = unsafe {
            ffi::sc_stream_create(filter.as_ptr(), configuration.as_ptr(), error_callback)
        };
        assert!(!ptr.is_null(), "Swift bridge returned null stream");
        Self { ptr }
    }

    pub fn new_with_delegate(
        filter: &SCContentFilter,
        configuration: &SCStreamConfiguration,
        _delegate: impl crate::stream::delegate_trait::SCStreamDelegateTrait,
    ) -> Self {
        // Delegate callbacks not yet mapped in bridge version; stored for API parity.
        Self::new(filter, configuration)
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
            registry
                .as_mut()
                .unwrap()
                .insert(handler_id, Box::new(handler));
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
    /// Panics if the internal handler registry mutex is poisoned.
    ///
    /// # Returns
    ///
    /// Returns `true` if the handler was found and removed, `false` otherwise.
    pub fn remove_output_handler(&mut self, id: usize, _of_type: SCStreamOutputType) -> bool {
        // Mutex poisoning is unrecoverable; unwrap is appropriate
        let mut registry = HANDLER_REGISTRY.lock().unwrap();
        registry
            .as_mut()
            .and_then(|handlers| handlers.remove(&id))
            .is_some()
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
}

impl Drop for SCStream {
    fn drop(&mut self) {
        unsafe { ffi::sc_stream_release(self.ptr) };
    }
}

impl Clone for SCStream {
    fn clone(&self) -> Self {
        unsafe {
            Self {
                ptr: crate::ffi::sc_stream_retain(self.ptr),
            }
        }
    }
}

impl fmt::Debug for SCStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SCStream").field("ptr", &self.ptr).finish()
    }
}

impl fmt::Display for SCStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SCStream")
    }
}
