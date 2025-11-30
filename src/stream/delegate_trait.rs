//! Delegate trait for stream lifecycle events
//!
//! Defines the interface for receiving stream state change notifications.

use crate::error::SCError;

/// Trait for handling stream lifecycle events
///
/// Implement this trait to receive notifications about stream state changes,
/// errors, and video effects.
///
/// # Examples
///
/// ## Using a struct
///
/// ```
/// use screencapturekit::stream::delegate_trait::SCStreamDelegateTrait;
/// use screencapturekit::error::SCError;
///
/// struct MyDelegate;
///
/// impl SCStreamDelegateTrait for MyDelegate {
///     fn stream_did_stop(&self, error: Option<String>) {
///         if let Some(err) = error {
///             eprintln!("Stream stopped with error: {}", err);
///         } else {
///             println!("Stream stopped normally");
///         }
///     }
///
///     fn did_stop_with_error(&self, error: SCError) {
///         eprintln!("Stream error: {}", error);
///     }
/// }
/// ```
///
/// ## Using closures
///
/// Use [`StreamCallbacks`] to create a delegate from closures:
///
/// ```rust,no_run
/// use screencapturekit::prelude::*;
/// use screencapturekit::stream::delegate_trait::StreamCallbacks;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let content = SCShareableContent::get()?;
/// # let display = &content.displays()[0];
/// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
/// # let config = SCStreamConfiguration::default();
///
/// let delegate = StreamCallbacks::new()
///     .on_stop(|error| {
///         if let Some(e) = error {
///             eprintln!("Stream stopped with error: {}", e);
///         }
///     })
///     .on_error(|error| eprintln!("Error: {}", error));
///
/// let stream = SCStream::new_with_delegate(&filter, &config, delegate);
/// # Ok(())
/// # }
/// ```
pub trait SCStreamDelegateTrait: Send {
    /// Called when video effects start (macOS 14.0+)
    ///
    /// Notifies when the stream's overlay video effect (presenter overlay) has started.
    fn output_video_effect_did_start_for_stream(&self) {}

    /// Called when video effects stop (macOS 14.0+)
    ///
    /// Notifies when the stream's overlay video effect (presenter overlay) has stopped.
    fn output_video_effect_did_stop_for_stream(&self) {}

    /// Called when the stream becomes active (macOS 15.2+)
    ///
    /// Notifies the first time any window that was being shared in the stream
    /// is re-opened after all the windows being shared were closed.
    /// When all the windows being shared are closed, the client will receive
    /// `stream_did_become_inactive`.
    fn stream_did_become_active(&self) {}

    /// Called when the stream becomes inactive (macOS 15.2+)
    ///
    /// Notifies when all the windows that are currently being shared are exited.
    /// This callback occurs for all content filter types.
    fn stream_did_become_inactive(&self) {}

    /// Called when stream stops with an error
    fn did_stop_with_error(&self, _error: SCError) {}

    /// Called when stream stops
    ///
    /// # Parameters
    ///
    /// - `error`: Optional error message if the stream stopped due to an error
    fn stream_did_stop(&self, _error: Option<String>) {}
}

/// A simple error handler wrapper for closures
///
/// Allows using a closure as a stream delegate that only handles errors.
///
/// # Examples
///
/// ```rust,no_run
/// use screencapturekit::prelude::*;
/// use screencapturekit::stream::delegate_trait::ErrorHandler;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let content = SCShareableContent::get()?;
/// # let display = &content.displays()[0];
/// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
/// # let config = SCStreamConfiguration::default();
///
/// let error_handler = ErrorHandler::new(|error| {
///     eprintln!("Stream error: {}", error);
/// });
///
/// let stream = SCStream::new_with_delegate(&filter, &config, error_handler);
/// # Ok(())
/// # }
/// ```
pub struct ErrorHandler<F>
where
    F: Fn(SCError) + Send + 'static,
{
    handler: F,
}

impl<F> ErrorHandler<F>
where
    F: Fn(SCError) + Send + 'static,
{
    /// Create a new error handler from a closure
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> SCStreamDelegateTrait for ErrorHandler<F>
where
    F: Fn(SCError) + Send + 'static,
{
    fn did_stop_with_error(&self, error: SCError) {
        (self.handler)(error);
    }
}

/// Builder for closure-based stream delegate
///
/// Provides a convenient way to create a stream delegate using closures
/// instead of implementing the [`SCStreamDelegateTrait`] trait.
///
/// # Examples
///
/// ```rust,no_run
/// use screencapturekit::prelude::*;
/// use screencapturekit::stream::delegate_trait::StreamCallbacks;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let content = SCShareableContent::get()?;
/// # let display = &content.displays()[0];
/// # let filter = SCContentFilter::builder().display(display).exclude_windows(&[]).build();
/// # let config = SCStreamConfiguration::default();
///
/// // Create delegate with multiple callbacks
/// let delegate = StreamCallbacks::new()
///     .on_stop(|error| {
///         if let Some(e) = error {
///             eprintln!("Stream stopped with error: {}", e);
///         } else {
///             println!("Stream stopped normally");
///         }
///     })
///     .on_error(|error| eprintln!("Stream error: {}", error))
///     .on_active(|| println!("Stream became active"))
///     .on_inactive(|| println!("Stream became inactive"));
///
/// let stream = SCStream::new_with_delegate(&filter, &config, delegate);
/// # Ok(())
/// # }
/// ```
pub struct StreamCallbacks {
    on_stop: Option<Box<dyn Fn(Option<String>) + Send + 'static>>,
    on_error: Option<Box<dyn Fn(SCError) + Send + 'static>>,
    on_active: Option<Box<dyn Fn() + Send + 'static>>,
    on_inactive: Option<Box<dyn Fn() + Send + 'static>>,
    on_video_effect_start: Option<Box<dyn Fn() + Send + 'static>>,
    on_video_effect_stop: Option<Box<dyn Fn() + Send + 'static>>,
}

impl StreamCallbacks {
    /// Create a new empty callbacks builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            on_stop: None,
            on_error: None,
            on_active: None,
            on_inactive: None,
            on_video_effect_start: None,
            on_video_effect_stop: None,
        }
    }

    /// Set the callback for when the stream stops
    #[must_use]
    pub fn on_stop<F>(mut self, f: F) -> Self
    where
        F: Fn(Option<String>) + Send + 'static,
    {
        self.on_stop = Some(Box::new(f));
        self
    }

    /// Set the callback for when the stream encounters an error
    #[must_use]
    pub fn on_error<F>(mut self, f: F) -> Self
    where
        F: Fn(SCError) + Send + 'static,
    {
        self.on_error = Some(Box::new(f));
        self
    }

    /// Set the callback for when the stream becomes active (macOS 15.2+)
    #[must_use]
    pub fn on_active<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        self.on_active = Some(Box::new(f));
        self
    }

    /// Set the callback for when the stream becomes inactive (macOS 15.2+)
    #[must_use]
    pub fn on_inactive<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        self.on_inactive = Some(Box::new(f));
        self
    }

    /// Set the callback for when video effects start (macOS 14.0+)
    #[must_use]
    pub fn on_video_effect_start<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        self.on_video_effect_start = Some(Box::new(f));
        self
    }

    /// Set the callback for when video effects stop (macOS 14.0+)
    #[must_use]
    pub fn on_video_effect_stop<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + 'static,
    {
        self.on_video_effect_stop = Some(Box::new(f));
        self
    }
}

impl Default for StreamCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl SCStreamDelegateTrait for StreamCallbacks {
    fn stream_did_stop(&self, error: Option<String>) {
        if let Some(ref f) = self.on_stop {
            f(error);
        }
    }

    fn did_stop_with_error(&self, error: SCError) {
        if let Some(ref f) = self.on_error {
            f(error);
        }
    }

    fn stream_did_become_active(&self) {
        if let Some(ref f) = self.on_active {
            f();
        }
    }

    fn stream_did_become_inactive(&self) {
        if let Some(ref f) = self.on_inactive {
            f();
        }
    }

    fn output_video_effect_did_start_for_stream(&self) {
        if let Some(ref f) = self.on_video_effect_start {
            f();
        }
    }

    fn output_video_effect_did_stop_for_stream(&self) {
        if let Some(ref f) = self.on_video_effect_stop {
            f();
        }
    }
}
